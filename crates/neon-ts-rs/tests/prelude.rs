//! `use neon_ts_rs::prelude::*;` alone is enough: it brings in the `TS` and
//! `TsJson` derives and makes the `TsJsonExt` rung of the JSON probe visible.

// The `(&probe)` borrow mirrors the macro-emitted call shape.
#![allow(clippy::needless_borrow)]

use std::marker::PhantomData;

use neon_ts_rs::prelude::*;
use neon_typescript::TsJsonProbe;

// Only `TS`, from the prelude.
#[derive(serde::Serialize, TS)]
struct OnlyTs {
    field: u32,
}

// Both derives, from the prelude.
#[derive(serde::Serialize, TS, TsJson)]
struct Bridged {
    field: u32,
}

#[test]
fn prelude_enables_the_ts_rs_rung() {
    let probe = TsJsonProbe::<OnlyTs>(PhantomData);
    assert_eq!((&probe).ts_json_of(), "OnlyTs");
    let probe = TsJsonProbe::<Vec<OnlyTs>>(PhantomData);
    assert_eq!((&probe).ts_json_of(), "Array<OnlyTs>");
}

#[test]
fn prelude_provides_the_bridge_derive() {
    assert_eq!(<Bridged as neon_typescript::TsJson>::ts_json(), "Bridged");
}
