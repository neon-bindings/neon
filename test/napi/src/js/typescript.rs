// Test module for TypeScript declaration generation.
// These exports exist solely to verify .d.ts output.

use indexmap::IndexMap;
use neon::types::extract::Json;
use neon::types::extract::{Array, Boxed};
use neon_ts_rs::TsJsonExt as _;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// --- Stage 2: Derived types ---

#[derive(Serialize, Deserialize, ts_rs::TS, neon_ts_rs::TsJson)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub doc_id: u32,
    pub score: f64,
    #[serde(skip)]
    pub internal: Vec<u8>,
    #[serde(default)]
    pub highlights: Vec<String>,
}

#[derive(Serialize, Deserialize, ts_rs::TS, neon_ts_rs::TsJson)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Shape {
    Circle {
        radius: f64,
    },
    Rectangle {
        width: f64,
        height: f64,
    },
    #[serde(rename = "dot")]
    Point,
}

#[derive(Serialize, Deserialize, ts_rs::TS, neon_ts_rs::TsJson)]
pub struct Config {
    pub name: String,
    #[ts(type = "Record<string, unknown>")]
    pub metadata: serde_json::Value,
}

// A simple opaque type. Boxed values are branded by the Rust type inside the
// `JsBox`, so no TypeScript impl is needed.
pub struct Database;

impl neon::types::Finalize for Database {}

#[neon::export]
fn ts_add(a: f64, b: f64) -> f64 {
    a + b
}

#[neon::export]
fn ts_greet(name: String) -> String {
    format!("Hello, {name}!")
}

#[neon::export]
fn ts_is_even(n: f64) -> bool {
    (n as i64) % 2 == 0
}

// Bare `Option`: `TryIntoJs` produces `undefined` for `None`, and `TryFromJs`
// accepts `null` or `undefined`.
#[neon::export]
fn ts_maybe_number(flag: bool) -> Option<f64> {
    flag.then_some(42.0)
}

#[neon::export]
fn ts_optional_arg(x: Option<f64>) -> f64 {
    x.unwrap_or(-1.0)
}

// The same `Option` through JSON: serde_json produces `null` for `None`.
#[neon::export(json)]
fn ts_json_maybe_number(flag: bool) -> Option<f64> {
    flag.then_some(42.0)
}

// A bare `Vec<f64>` is a `Float64Array` in both directions.
#[neon::export]
fn ts_typed_array(xs: Vec<f64>) -> Vec<f64> {
    xs.into_iter().map(|x| x * 2.0).collect()
}

// The JSON path serializes the implicit `()` to `null`.
#[neon::export(json)]
fn ts_json_unit() {}

#[neon::export(task)]
fn ts_async_add(a: f64, b: f64) -> f64 {
    a + b
}

// Async function returning unit should render as Promise<void>, not
// Promise<undefined>.
#[neon::export(task)]
fn ts_async_noop() {}

#[neon::export(name = "tsRenamedFunc")]
fn ts_renamed(x: f64) -> f64 {
    x
}

#[neon::export]
fn ts_create_db() -> Arc<Database> {
    Arc::new(Database)
}

#[neon::export]
fn ts_query_db(_db: Arc<Database>, _query: String) -> String {
    String::new()
}

// `Boxed<T>` boxes `T` itself, so it gets a different brand than `Arc<T>`.
#[neon::export]
fn ts_create_boxed_db() -> Boxed<Database> {
    Boxed(Database)
}

#[neon::export]
fn ts_no_args_or_return() {}

#[neon::export(json)]
fn ts_search(query: String) -> SearchResult {
    SearchResult {
        doc_id: 1,
        score: 0.95,
        internal: vec![],
        highlights: vec![query],
    }
}

#[neon::export(json)]
fn ts_create_shape(kind: String) -> Shape {
    match kind.as_str() {
        "circle" => Shape::Circle { radius: 1.0 },
        _ => Shape::Rectangle {
            width: 1.0,
            height: 2.0,
        },
    }
}

#[neon::export(json)]
fn ts_get_config() -> Config {
    Config {
        name: "default".into(),
        metadata: serde_json::json!({}),
    }
}

// --- Stage 4: Extended enum representations ---

// Externally tagged enum (serde default)
#[derive(Serialize, Deserialize, ts_rs::TS, neon_ts_rs::TsJson)]
pub enum ExternalMsg {
    Quit,
    Echo(String),
    Move { x: f64, y: f64 },
}

// Adjacently tagged enum
#[derive(Serialize, Deserialize, ts_rs::TS, neon_ts_rs::TsJson)]
#[serde(tag = "type", content = "data")]
pub enum ApiResponse {
    Success(String),
    Error { code: u32, message: String },
    Loading,
}

// Untagged enum
#[derive(Serialize, Deserialize, ts_rs::TS, neon_ts_rs::TsJson)]
#[serde(untagged)]
pub enum StringOrNumber {
    Str(String),
    Num(f64),
}

#[neon::export(json)]
fn ts_send_message(kind: String) -> ExternalMsg {
    match kind.as_str() {
        "quit" => ExternalMsg::Quit,
        "echo" => ExternalMsg::Echo("hello".into()),
        _ => ExternalMsg::Move { x: 1.0, y: 2.0 },
    }
}

#[neon::export(json)]
fn ts_api_response(ok: bool) -> ApiResponse {
    if ok {
        ApiResponse::Success("ok".into())
    } else {
        ApiResponse::Error {
            code: 404,
            message: "not found".into(),
        }
    }
}

#[neon::export(json)]
fn ts_parse_value(input: String) -> StringOrNumber {
    match input.parse::<f64>() {
        Ok(n) => StringOrNumber::Num(n),
        Err(_) => StringOrNumber::Str(input),
    }
}

#[neon::export(json)]
fn ts_get_any_value() -> serde_json::Value {
    serde_json::json!({"key": "value"})
}

// --- Stage 4b: Flatten and generics ---

// Flatten: intersection types
#[derive(Serialize, Deserialize, ts_rs::TS, neon_ts_rs::TsJson)]
pub struct Pagination {
    pub page: u32,
    pub per_page: u32,
}

#[derive(Serialize, Deserialize, ts_rs::TS, neon_ts_rs::TsJson)]
pub struct UserList {
    pub users: Vec<String>,
    #[serde(flatten)]
    pub pagination: Pagination,
}

// Generic struct
#[derive(Serialize, Deserialize, ts_rs::TS, neon_ts_rs::TsJson)]
pub struct Envelope<T> {
    pub data: T,
    pub timestamp: f64,
}

#[neon::export(json)]
fn ts_get_user_list() -> UserList {
    UserList {
        users: vec!["alice".into()],
        pagination: Pagination {
            page: 1,
            per_page: 10,
        },
    }
}

#[neon::export(json)]
fn ts_get_string_envelope() -> Envelope<String> {
    Envelope {
        data: "hello".into(),
        timestamp: 1.0,
    }
}

#[neon::export(json)]
fn ts_get_number_envelope() -> Envelope<f64> {
    Envelope {
        data: 42.0,
        timestamp: 2.0,
    }
}

// --- Stage 4c: Exercise built-in TypeScript impls that the type-mapping tests
// missed (HashMap, tuples, nested generics, manual TypeScript impl with ts_decl). ---

#[neon::export(json)]
fn ts_get_scores() -> std::collections::HashMap<String, f64> {
    let mut m = std::collections::HashMap::new();
    m.insert("alice".into(), 0.9);
    m
}

// A map with a non-string key: JSON stringifies the keys, so the TS type must
// still be `Record<string, V>` (not `Record<number, V>`).
#[neon::export(json)]
fn ts_get_counts() -> std::collections::HashMap<u32, f64> {
    std::collections::HashMap::new()
}

#[neon::export(json)]
fn ts_get_pair() -> (f64, String) {
    (1.0, "one".into())
}

#[neon::export(json)]
fn ts_get_triple() -> (f64, String, bool) {
    (1.0, "one".into(), true)
}

#[neon::export(json)]
fn ts_get_nested() -> std::collections::HashMap<String, Vec<f64>> {
    std::collections::HashMap::new()
}

// Manual `TsJson` impl with a custom `ts_json_decl` (not just a name): exercises
// the trait's default `ts_json_collect` and the generator's interface-dedup path.
pub struct GeoPoint {
    pub lat: f64,
    pub lng: f64,
}

impl neon::typescript::TsJson for GeoPoint {
    fn ts_json() -> std::borrow::Cow<'static, str> {
        "GeoPoint".into()
    }
    fn ts_json_decl() -> Option<std::borrow::Cow<'static, str>> {
        Some("interface GeoPoint {\n  lat: number;\n  lng: number;\n}".into())
    }
}

impl serde::Serialize for GeoPoint {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut st = s.serialize_struct("GeoPoint", 2)?;
        st.serialize_field("lat", &self.lat)?;
        st.serialize_field("lng", &self.lng)?;
        st.end()
    }
}

impl<'de> serde::Deserialize<'de> for GeoPoint {
    fn deserialize<D: serde::Deserializer<'de>>(_d: D) -> Result<Self, D::Error> {
        // Test fixtures only construct, never deserialize.
        Ok(GeoPoint { lat: 0.0, lng: 0.0 })
    }
}

#[neon::export(json)]
fn ts_get_point() -> GeoPoint {
    GeoPoint {
        lat: 37.5,
        lng: -122.3,
    }
}

// --- Foreign type as a JSON payload ---
//
// `IndexMap` has no `TsJson` impl, only `ts_rs::TS`. As a JSON payload it would
// fall back to `unknown` (and drop `FieldDescriptor`) without the adapter's
// `TsJsonExt` (brought into scope by `use neon_ts_rs::TsJsonExt as _;` above).
#[derive(Serialize, Deserialize, ts_rs::TS, neon_ts_rs::TsJson)]
pub struct FieldDescriptor {
    pub name: String,
    pub required: bool,
}

#[neon::export]
fn ts_index_map() -> Json<IndexMap<String, FieldDescriptor>> {
    unimplemented!()
}

// A returned `Result` is control flow: the `Err` is thrown, so only the `Ok`
// type is declared (not ts-rs's serde shape `{ Ok: T } | { Err: E }`).
#[neon::export(json)]
fn ts_json_fallible() -> Result<IndexMap<String, FieldDescriptor>, String> {
    Err("nope".into())
}

#[neon::export]
fn ts_bare_fallible_json() -> Result<Json<IndexMap<String, FieldDescriptor>>, String> {
    Err("nope".into())
}

// --- New attributes: ts_skip, ts_name, ts_returns, per-param ts_type ---

#[neon::export(ts_skip)]
fn ts_skipped_function() -> String {
    "internal".into()
}

#[neon::export(ts_returns = "bigint")]
fn ts_returns_override() -> f64 {
    42.0
}

#[neon::export]
fn ts_bigint_return<'cx>(
    cx: &mut neon::context::FunctionContext<'cx>,
) -> neon::result::JsResult<'cx, neon::types::JsBigInt> {
    Ok(neon::types::JsBigInt::from_i64(cx, 42))
}

#[neon::export]
fn ts_handle_string_return<'cx>(
    cx: &mut neon::context::FunctionContext<'cx>,
) -> neon::result::JsResult<'cx, neon::types::JsString> {
    use neon::context::Context;
    Ok(cx.string("hi"))
}

// Class with ts_no_constructor: emits class declaration without constructor.
pub struct NoCtor;

#[neon::export(class, ts_no_constructor)]
impl NoCtor {
    pub fn new() -> Self {
        NoCtor
    }

    pub fn ping(&self) -> String {
        "pong".into()
    }
}

// Class where the constructor itself is marked ts_skip.
pub struct CtorSkipped;

#[neon::export(class)]
impl CtorSkipped {
    #[neon(ts_skip)]
    pub fn new() -> Self {
        CtorSkipped
    }

    pub fn ping(&self) -> String {
        "pong".into()
    }
}

#[neon::export]
fn ts_param_override(#[neon(ts_type = "ReadonlyArray<number>")] xs: Array<Vec<f64>>) -> f64 {
    xs.0.iter().sum()
}

pub struct Hidden;

#[neon::export(class, ts_skip)]
impl Hidden {
    pub fn new() -> Self {
        Hidden
    }
}

pub struct Renamed;

#[neon::export(class, ts_name = "PublicName")]
impl Renamed {
    pub fn new() -> Self {
        Renamed
    }

    #[neon(ts_skip)]
    pub fn ts_internal_method(&self) -> f64 {
        0.0
    }

    #[neon(ts_name = "publicName")]
    pub fn ts_renamed_method(&self) -> String {
        "ok".into()
    }

    #[neon(ts_returns = "bigint")]
    pub fn ts_method_returns(&self) -> f64 {
        0.0
    }
}
