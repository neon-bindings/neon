//! Integration test for the ts-rs -> neon_typescript bridge derive.

use std::collections::BTreeMap;

use neon_typescript::TypeScript;

#[derive(serde::Serialize, serde::Deserialize, ts_rs::TS, neon_ts_rs::TypeScript)]
#[serde(rename_all = "camelCase")]
struct SearchResult {
    doc_id: u32,
    tags: Vec<String>,
    page: Option<u32>,
}

#[test]
fn ts_type_is_type_name() {
    assert_eq!(<SearchResult as TypeScript>::ts_type(), "SearchResult",);
}

#[test]
fn ts_collect_produces_camel_case_decl() {
    let mut decls: BTreeMap<String, String> = BTreeMap::new();
    <SearchResult as TypeScript>::ts_collect(&mut decls);

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
#[derive(serde::Serialize, serde::Deserialize, ts_rs::TS, neon_ts_rs::TypeScript)]
struct Wrapper<T: ts_rs::TS> {
    inner: T,
}

#[test]
fn generic_type_compiles_and_collects() {
    let mut decls: BTreeMap<String, String> = BTreeMap::new();
    <Wrapper<SearchResult> as TypeScript>::ts_collect(&mut decls);
    assert!(decls.contains_key("Wrapper"));
    // The transitive dependency should be collected too.
    assert!(decls.contains_key("SearchResult"));
}
