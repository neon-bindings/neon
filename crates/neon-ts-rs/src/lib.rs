//! ts-rs adapter for Neon. Bridges `ts_rs::TS` to `neon_typescript::TypeScript`.
//!
//! Users derive both `ts_rs::TS` and `neon_ts_rs::TypeScript` on their types.
//! The bridge derive (re-exported here) generates an impl that delegates to the
//! runtime helpers below.

use std::borrow::Cow;
use std::collections::BTreeMap;

use ts_rs::{Config, TypeVisitor, TS};

pub use neon_ts_rs_macros::TypeScript;

fn cfg() -> Config {
    // ts-rs defaults u64/i64 to `bigint`, but serde_json emits JSON numbers,
    // so map large ints to `number` to match the JSON boundary.
    Config::default().with_large_int("number")
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
            // Inline types (primitives, Vec, Option, …) have no output_path;
            // don't declare them, but recurse to reach named types inside.
            if <U as TS>::output_path().is_none() {
                <U as TS>::visit_dependencies(self);
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
