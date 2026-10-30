//! Probe-ladder tests for the `TypeScriptExt` boundary rung (rung 2).
//!
//! Rung 2 lets a foreign type used as the outermost type at a boundary resolve
//! through ts-rs instead of degrading to `"any"`. These tests exercise the bare
//! probe directly (the same call the macros emit: `(&probe).ts_type_of()`),
//! with `use neon_ts_rs::TypeScriptExt as _;` in scope so rung 2 is visible.

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::marker::PhantomData;

use neon_ts_rs::TypeScriptExt as _;
use neon_typescript::{TsProbe, TypeScript};

// A ts-rs-only type: implements `ts_rs::TS` but NOT `neon_typescript::TypeScript`.
// At a boundary it must resolve via rung 2 (through ts-rs), not fall to `"any"`.
#[derive(serde::Serialize, serde::Deserialize, ts_rs::TS)]
struct TsRsOnly {
    field: String,
}

// A type with a native `TypeScript` impl (rung 1). It also happens to impl
// `ts_rs::TS`, so this proves rung 1 (inherent) still outranks rung 2 (trait)
// at the same candidate.
struct NativeTs;

impl TypeScript for NativeTs {
    fn ts_type() -> Cow<'static, str> {
        "NativeTs".into()
    }
}

#[test]
fn rung2_resolves_ts_rs_type_through_ts_rs() {
    let probe = TsProbe::<TsRsOnly>(PhantomData);
    // ts-rs renders the type by its name.
    assert_eq!((&probe).ts_type_of(), "TsRsOnly");

    let mut decls: BTreeMap<String, String> = BTreeMap::new();
    (&probe).ts_collect_of(&mut decls);
    // The referenced declaration must not vanish.
    assert!(
        decls.contains_key("TsRsOnly"),
        "expected TsRsOnly decl, got: {decls:?}"
    );
}

#[test]
fn rung1_native_impl_still_wins() {
    let probe = TsProbe::<NativeTs>(PhantomData);
    // Rung 1 (inherent, native TypeScript impl) outranks rung 2.
    assert_eq!((&probe).ts_type_of(), "NativeTs");
}
