//! ts-rs adapter for Neon. Bridges `ts_rs::TS` to `neon_typescript::TsJson`,
//! the TypeScript type of a value serialized to JSON.
//!
//! Import the [`prelude`] in each module with exports, and derive ts-rs's
//! `TS` on your types:
//!
//! ```ignore
//! use neon_ts_rs::prelude::*;
//!
//! #[derive(serde::Serialize, serde::Deserialize, TS)]
//! struct Options { limit: u32 }
//!
//! #[neon::export]
//! fn search(query: String, options: Option<Json<Options>>) -> Json<Vec<String>> { /* … */ }
//! ```
//!
//! That covers every JSON position: `Json<T>`, `Option<Json<T>>`, and
//! `#[neon::export(json)]`. The [`TsJson`](macro@TsJson) bridge derive, which
//! generates a `neon_typescript::TsJson` impl that delegates to the runtime
//! helpers below, is only needed for a `Json<T>` nested inside some other bare
//! type (say, `Either<Json<A>, Json<B>>`).
//!
//! # Pitfall: `skip_serializing_if` without `default`
//!
//! ts-rs leaves a field with only `#[serde(skip_serializing_if = "...")]`
//! *required* in its output, but serde_json omits the key, so a returned
//! object can lack a field its declaration says is present. Add
//! `#[serde(default)]` (declared `field?: T | null`) or `#[ts(optional)]`
//! (declared `field?: T`) alongside it:
//!
//! ```ignore
//! #[derive(serde::Serialize, ts_rs::TS, neon_ts_rs::TsJson)]
//! struct Hit {
//!     #[serde(skip_serializing_if = "Option::is_none", default)]
//!     score: Option<f64>,
//! }
//! ```

use std::any::TypeId;
use std::borrow::Cow;
use std::collections::{BTreeMap, HashSet};

use ts_rs::{Config, TypeVisitor, TS};

pub use neon_ts_rs_macros::TsJson;

/// Everything a module needs to give its exports TypeScript types through
/// ts-rs, in one import:
///
/// ```ignore
/// use neon_ts_rs::prelude::*;
///
/// #[derive(serde::Serialize, TS)]
/// struct SearchHit {
///     id: u32,
///     score: f64,
/// }
///
/// #[neon::export(json)]
/// fn search(query: String) -> Vec<SearchHit> { /* … */ }
/// ```
///
/// It brings in:
///
/// - ts-rs's `TS` derive, which describes your types.
/// - [`TsJsonExt`], anonymously, which lets Neon describe any ts-rs type in a
///   JSON position (`Json<T>`, `Option<Json<T>>`, or `#[neon::export(json)]`),
///   including types from other crates.
/// - The [`TsJson`](macro@TsJson) derive, for the rare position the above
///   doesn't reach: a `Json<T>` nested inside some other bare type.
pub mod prelude {
    pub use crate::TsJson;
    pub use crate::TsJsonExt as _;
    pub use ts_rs::TS;
}

/// Implementation detail for the `TsJson` bridge derive. Re-exports the paths
/// the generated impl references, so a consumer only needs `neon-ts-rs` (and
/// `ts-rs`) in scope — never a direct dependency on `neon-typescript`.
/// Not public API; do not use directly.
#[doc(hidden)]
pub mod __private {
    pub use neon_typescript::TsJson;
}

fn cfg() -> Config {
    // ts-rs defaults u64/i64 to `bigint`, but serde_json emits JSON numbers,
    // so map large ints to `number` to match the JSON boundary.
    Config::default().with_large_int("number")
}

/// Lets Neon describe **any ts-rs type in a JSON position**: the payload of a
/// `Json<T>` or `Option<Json<T>>` argument or return value, or of a
/// `#[neon::export(json)]` function. That includes your own types (with just
/// `#[derive(TS)]`) and types from other crates (e.g.
/// [`IndexMap`](https://docs.rs/indexmap)), which can't carry a derive you
/// didn't write.
///
/// It's included in the [`prelude`], which is the usual way to bring it in. On
/// its own:
///
/// ```ignore
/// use neon_ts_rs::TsJsonExt as _;   // once per module with exports
/// ```
///
/// You never call its methods: the import alone turns the coverage on. Without
/// it, a type that only implements `ts_rs::TS` is declared `unknown`. Its reach
/// is exactly what ts-rs supports: a foreign type ts-rs doesn't implement is
/// still `unknown`.
pub trait TsJsonExt {
    fn ts_json_of(&self) -> Cow<'static, str>;
    fn ts_json_collect_of(&self, decls: &mut BTreeMap<String, String>);
}

// Mechanism (maintainers): this is the ts-rs tier of the JSON type probe in
// `neon_typescript::TsJsonProbe`. Implemented for the *bare* probe (`Self =
// TsJsonProbe<T>`), it shares the same method-resolution candidate as the
// probe's inherent `TsJson` method, so a native `TsJson` impl still wins and
// this only catches genuinely-foreign payloads. Invisible unless imported.
impl<T: TS + 'static + ?Sized> TsJsonExt for neon_typescript::TsJsonProbe<T> {
    fn ts_json_of(&self) -> Cow<'static, str> {
        ts_json::<T>()
    }

    fn ts_json_collect_of(&self, decls: &mut BTreeMap<String, String>) {
        ts_json_collect::<T>(decls)
    }
}

/// The TypeScript type expression for `T` (e.g. `SearchResult`, `Array<string>`).
pub fn ts_json<T: TS + 'static + ?Sized>() -> Cow<'static, str> {
    Cow::Owned(<T as TS>::name(&cfg()))
}

/// Collect `T`'s declaration plus its transitive declarations into `decls`,
/// keyed by name, as a FLAT set (Neon's model), using ts-rs's TypeVisitor.
pub fn ts_json_collect<T: TS + 'static + ?Sized>(decls: &mut BTreeMap<String, String>) {
    struct Collector<'a> {
        cfg: Config,
        decls: &'a mut BTreeMap<String, String>,
        // Every type already walked, by `TypeId`. This must cover inline types
        // (`Vec<T>`, `Option<T>`, …) too, not just declared ones: ts-rs's
        // wrapper impls forward `visit_dependencies` straight to the inner type
        // without going through `visit`, so a recursive type like
        // `struct Tree { children: Vec<Tree> }` only passes back through
        // `visit` at the `Vec<Tree>` field. Keying on `TypeId` (rather than the
        // declaration name) also walks each generic instantiation separately,
        // so `Pair<A, B>` and `Pair<C, D>` both contribute their arguments.
        seen: HashSet<TypeId>,
    }
    impl TypeVisitor for Collector<'_> {
        fn visit<U: TS + 'static + ?Sized>(&mut self) {
            if !self.seen.insert(TypeId::of::<U>()) {
                return;
            }
            // Inline types (primitives, Vec, Option, maps, …) have no
            // output_path; don't declare them, but recurse to reach named types
            // inside. For maps and similar containers, the value/key types are
            // reached only through `visit_generics` (ts-rs's `visit_dependencies`
            // for e.g. `HashMap<K, V>` recurses into `V`'s *dependencies* but
            // never visits `V` itself), so we traverse both. This matters when
            // such a container is the *outermost* type of a JSON payload: there
            // is no enclosing derived type to register the value type for us.
            if <U as TS>::output_path().is_some() {
                neon_typescript::insert_decl(
                    self.decls,
                    <U as TS>::ident(&self.cfg),
                    <U as TS>::decl(&self.cfg),
                );
            }
            <U as TS>::visit_dependencies(self);
            <U as TS>::visit_generics(self);
        }
    }
    let mut c = Collector {
        cfg: cfg(),
        decls,
        seen: HashSet::new(),
    };
    TypeVisitor::visit::<T>(&mut c);
}
