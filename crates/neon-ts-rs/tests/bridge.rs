//! Integration test for the ts-rs -> neon_typescript bridge derive.

use std::collections::BTreeMap;

use neon_typescript::TsJson;

#[derive(serde::Serialize, serde::Deserialize, ts_rs::TS, neon_ts_rs::TsJson)]
#[serde(rename_all = "camelCase")]
struct SearchResult {
    doc_id: u32,
    tags: Vec<String>,
    page: Option<u32>,
}

#[test]
fn ts_json_is_type_name() {
    assert_eq!(<SearchResult as TsJson>::ts_json(), "SearchResult");
}

#[test]
fn ts_json_collect_produces_camel_case_decl() {
    let mut decls: BTreeMap<String, String> = BTreeMap::new();
    <SearchResult as TsJson>::ts_json_collect(&mut decls);

    let decl = decls
        .get("SearchResult")
        .expect("expected a decl for SearchResult");
    // ts-rs applies serde's rename_all = "camelCase".
    assert!(
        decl.contains("docId"),
        "expected camelCase field `docId` in decl, got: {decl}"
    );
}

// A generic type to exercise the derive's generics handling.
#[derive(serde::Serialize, serde::Deserialize, ts_rs::TS, neon_ts_rs::TsJson)]
struct Wrapper<T: ts_rs::TS> {
    inner: T,
}

#[test]
fn generic_type_compiles_and_collects() {
    let mut decls: BTreeMap<String, String> = BTreeMap::new();
    <Wrapper<SearchResult> as TsJson>::ts_json_collect(&mut decls);
    assert!(decls.contains_key("Wrapper"));
    // The transitive dependency should be collected too.
    assert!(decls.contains_key("SearchResult"));
}

// Recursive types must not send the collector into infinite recursion (it
// would overflow the stack at addon load, since generation runs at module init).
#[derive(serde::Serialize, serde::Deserialize, ts_rs::TS, neon_ts_rs::TsJson)]
struct TreeNode {
    label: String,
    children: Vec<TreeNode>,
}

#[derive(serde::Serialize, serde::Deserialize, ts_rs::TS, neon_ts_rs::TsJson)]
struct Ping {
    pong: Option<Box<Pong>>,
}

#[derive(serde::Serialize, serde::Deserialize, ts_rs::TS, neon_ts_rs::TsJson)]
struct Pong {
    ping: Option<Box<Ping>>,
}

#[test]
fn recursive_types_terminate() {
    let mut decls: BTreeMap<String, String> = BTreeMap::new();
    <TreeNode as TsJson>::ts_json_collect(&mut decls);
    assert!(decls.contains_key("TreeNode"));

    let mut decls: BTreeMap<String, String> = BTreeMap::new();
    <Ping as TsJson>::ts_json_collect(&mut decls);
    assert!(decls.contains_key("Ping"));
    assert!(decls.contains_key("Pong"));
}

// Each instantiation of a generic type must contribute its own arguments, even
// once the generic declaration itself has been collected.
#[derive(serde::Serialize, serde::Deserialize, ts_rs::TS, neon_ts_rs::TsJson)]
struct Pair<A: ts_rs::TS, B: ts_rs::TS> {
    a: A,
    b: B,
}

#[derive(serde::Serialize, serde::Deserialize, ts_rs::TS, neon_ts_rs::TsJson)]
struct Inner1 {
    x: u32,
}
#[derive(serde::Serialize, serde::Deserialize, ts_rs::TS, neon_ts_rs::TsJson)]
struct Inner2 {
    x: u32,
}
#[derive(serde::Serialize, serde::Deserialize, ts_rs::TS, neon_ts_rs::TsJson)]
struct Inner3 {
    x: u32,
}
#[derive(serde::Serialize, serde::Deserialize, ts_rs::TS, neon_ts_rs::TsJson)]
struct Inner4 {
    x: u32,
}

#[test]
fn every_generic_instantiation_collects_its_arguments() {
    let mut decls: BTreeMap<String, String> = BTreeMap::new();
    <Pair<Inner1, Inner2> as TsJson>::ts_json_collect(&mut decls);
    <Pair<Inner3, Inner4> as TsJson>::ts_json_collect(&mut decls);
    for name in ["Pair", "Inner1", "Inner2", "Inner3", "Inner4"] {
        assert!(decls.contains_key(name), "missing {name}: {decls:?}");
    }
}

// A lifetime-parameterized type (e.g. one borrowing a `&'a str`). The bridge
// applies to its `'static` instantiation, which is what the export macros
// probe (they rewrite lifetimes to `'static`).
#[derive(serde::Serialize, ts_rs::TS, neon_ts_rs::TsJson)]
struct Borrowed<'a> {
    name: &'a str,
}

#[test]
fn lifetime_parameterized_type_bridges() {
    assert_eq!(<Borrowed<'static> as TsJson>::ts_json(), "Borrowed");
    let mut decls: BTreeMap<String, String> = BTreeMap::new();
    <Borrowed<'static> as TsJson>::ts_json_collect(&mut decls);
    assert!(decls["Borrowed"].contains("name: string"), "{decls:?}");
}

// Two different types with the same TypeScript name, reached in one walk.
mod first {
    #[derive(serde::Serialize, ts_rs::TS)]
    pub struct Config {
        pub a: u32,
    }
}

mod second {
    #[derive(serde::Serialize, ts_rs::TS)]
    pub struct Config {
        pub b: String,
    }
}

#[derive(serde::Serialize, ts_rs::TS, neon_ts_rs::TsJson)]
struct BothConfigs {
    first: first::Config,
    second: second::Config,
}

#[test]
fn same_named_types_are_recorded_as_a_conflict() {
    let mut decls: BTreeMap<String, String> = BTreeMap::new();
    <BothConfigs as TsJson>::ts_json_collect(&mut decls);

    let conflict = decls
        .get(&format!("{}Config", neon_typescript::CONFLICT_PREFIX))
        .unwrap_or_else(|| panic!("expected a conflict for Config, got: {decls:#?}"));
    let versions: Vec<&str> = conflict
        .split(neon_typescript::CONFLICT_SEPARATOR)
        .collect();
    assert_eq!(versions.len(), 2, "{versions:#?}");
    assert!(versions.iter().any(|d| d.contains("a: number")));
    assert!(versions.iter().any(|d| d.contains("b: string")));
}
