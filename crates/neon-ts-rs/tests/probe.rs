//! Probe-ladder tests for the `TsJsonExt` rung (rung 2 of the JSON probe).
//!
//! Rung 2 lets a foreign JSON payload type resolve through ts-rs instead of
//! degrading to `"unknown"`. These tests exercise the probe directly (the same
//! call the macros emit: `(&probe).ts_json_of()`), with
//! `use neon_ts_rs::TsJsonExt as _;` in scope so rung 2 is visible.

// The `(&probe)` borrow mirrors the macro-emitted call shape; it's what makes
// the fallback rung reachable, even where it's redundant for a given type.
#![allow(clippy::needless_borrow)]

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::marker::PhantomData;

use neon_ts_rs::TsJsonExt as _;
use neon_typescript::{TsJson, TsJsonFallback as _, TsJsonProbe};

// A ts-rs-only type: implements `ts_rs::TS` but NOT `neon_typescript::TsJson`.
// It must resolve via rung 2 (through ts-rs), not fall to `"unknown"`.
#[derive(serde::Serialize, serde::Deserialize, ts_rs::TS)]
struct TsRsOnly {
    field: String,
}

// A type implementing *both* `TsJson` and `ts_rs::TS`, rendered differently by
// each, so the result shows which rung won.
#[derive(serde::Serialize, serde::Deserialize, ts_rs::TS)]
struct Both {
    field: String,
}

impl TsJson for Both {
    fn ts_json() -> Cow<'static, str> {
        "BothFromRung1".into()
    }
}

// Neither trait: falls through to rung 3.
struct Neither;

#[test]
fn rung2_resolves_ts_rs_type_through_ts_rs() {
    let probe = TsJsonProbe::<TsRsOnly>(PhantomData);
    // ts-rs renders the type by its name.
    assert_eq!((&probe).ts_json_of(), "TsRsOnly");

    let mut decls: BTreeMap<String, String> = BTreeMap::new();
    (&probe).ts_json_collect_of(&mut decls);
    // The referenced declaration must not vanish.
    assert!(
        decls.contains_key("TsRsOnly"),
        "expected TsRsOnly decl, got: {decls:?}"
    );
}

#[test]
fn rung1_outranks_rung2() {
    // `Both` implements both traits; rung 1 (inherent) must win over rung 2.
    let probe = TsJsonProbe::<Both>(PhantomData);
    assert_eq!((&probe).ts_json_of(), "BothFromRung1");

    // Same for a std type both crates describe: `TsJson` renders `string[]`,
    // while ts-rs would render `Array<string>`.
    let probe = TsJsonProbe::<Vec<String>>(PhantomData);
    assert_eq!((&probe).ts_json_of(), "string[]");
}

#[test]
fn rung3_falls_back_to_unknown() {
    let probe = TsJsonProbe::<Neither>(PhantomData);
    assert_eq!((&probe).ts_json_of(), "unknown");
}
