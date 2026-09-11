//! ts-rs adapter for Neon. Bridges `ts_rs::TS` to `neon_typescript::TypeScript`.
//!
//! Users derive both `ts_rs::TS` and `neon_ts_rs::TypeScript` on their types.
//! The bridge derive (re-exported here) generates an impl that delegates to the
//! runtime helpers below.

use std::borrow::Cow;
use std::collections::BTreeMap;

use ts_rs::{Config, TypeVisitor, TS};

pub use neon_ts_rs_macros::TypeScript;

/// Implementation detail for the `TypeScript` bridge derive. Re-exports the
/// paths the generated impl references, so a consumer only needs `neon-ts-rs`
/// (and `ts-rs`) in scope — never a direct dependency on `neon-typescript`.
/// Not public API; do not use directly.
#[doc(hidden)]
pub mod __private {
    pub use neon_typescript::TypeScript;
}

fn cfg() -> Config {
    // ts-rs defaults u64/i64 to `bigint`, but serde_json emits JSON numbers,
    // so map large ints to `number` to match the JSON boundary.
    Config::default().with_large_int("number")
}

/// Extends Neon's TypeScript output to cover **foreign types at an export
/// boundary** — types from other crates that you didn't define (e.g.
/// [`IndexMap`](https://docs.rs/indexmap)), used directly as a parameter or
/// return type.
///
/// Types you define are covered by `#[derive(neon_ts_rs::TypeScript)]`. A
/// foreign type can't carry a derive you didn't write, so without this it types
/// as `any`. Bringing this trait into scope lets ts-rs describe any foreign type
/// it understands, so the value is typed instead of silently becoming `any`:
///
/// ```ignore
/// use neon_ts_rs::TypeScriptExt as _;   // once per module that exports such types
/// ```
///
/// You never call its methods — the import alone turns the coverage on. Pair it
/// with `#[neon::export(ts_strict)]` to make an uncovered foreign type a compile
/// error instead of a silent `any`. Its reach is exactly what ts-rs supports:
/// a foreign type ts-rs doesn't implement still types as `any`.
pub trait TypeScriptExt {
    fn ts_type_of(&self) -> Cow<'static, str>;
    fn ts_collect_of(&self, decls: &mut BTreeMap<String, String>);
}

// Mechanism (maintainers): this is the ts-rs tier of the boundary type probe in
// `neon_typescript::TsProbe`. Implemented for the *bare* probe (`Self =
// TsProbe<T>`), it shares the same method-resolution candidate as the probe's
// inherent `TypeScript` method, so a native `TypeScript` impl still wins and this
// only catches genuinely-foreign payloads. Invisible unless imported into scope.
impl<T: TS + 'static> TypeScriptExt for neon_typescript::TsProbe<T> {
    fn ts_type_of(&self) -> Cow<'static, str> {
        ts_type::<T>()
    }

    fn ts_collect_of(&self, decls: &mut BTreeMap<String, String>) {
        ts_collect::<T>(decls)
    }
}

/// The TypeScript type expression for `T` (e.g. "SearchResult", "Array<string>").
pub fn ts_type<T: TS + 'static + ?Sized>() -> Cow<'static, str> {
    Cow::Owned(<T as TS>::name(&cfg()))
}

/// Collect `T`'s declaration plus its transitive declarations into `decls`,
/// keyed by name, as a FLAT set (Neon's model), using ts-rs's TypeVisitor.
pub fn ts_collect<T: TS + 'static + ?Sized>(decls: &mut BTreeMap<String, String>) {
    struct Collector<'a> {
        cfg: Config,
        decls: &'a mut BTreeMap<String, String>,
    }
    impl TypeVisitor for Collector<'_> {
        fn visit<U: TS + 'static + ?Sized>(&mut self) {
            // Inline types (primitives, Vec, Option, maps, …) have no
            // output_path; don't declare them, but recurse to reach named types
            // inside. For maps and similar containers, the value/key types are
            // reached only through `visit_generics` (ts-rs's `visit_dependencies`
            // for e.g. `HashMap<K, V>` recurses into `V`'s *dependencies* but
            // never visits `V` itself), so we must traverse both. This matters
            // when such a container is the *outermost* type at a boundary (via
            // the `TypeScriptExt` rung): there is no enclosing derived type to
            // register the value type for us.
            if <U as TS>::output_path().is_none() {
                <U as TS>::visit_dependencies(self);
                <U as TS>::visit_generics(self);
                return;
            }
            let key = <U as TS>::ident(&self.cfg);
            if self.decls.contains_key(&key) {
                return;
            }
            self.decls.insert(key, <U as TS>::decl(&self.cfg));
            <U as TS>::visit_dependencies(self);
        }
    }
    let mut c = Collector { cfg: cfg(), decls };
    TypeVisitor::visit::<T>(&mut c);
}
