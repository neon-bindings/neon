//! TypeScript type declaration generation for Neon modules.
//!
//! This module provides automatic generation of `.d.ts` type declaration files
//! for Neon module exports. It works by collecting type metadata from
//! `#[neon::export]` items at compile time and resolving TypeScript types at
//! runtime.
//!
//! A value crosses an export boundary through one of two conversions, and the
//! same Rust type can have a different TypeScript type in each:
//!
//! | Conversion | TypeScript trait | Example: `Option<u32>` |
//! |---|---|---|
//! | serde_json, via [`Json<T>`](crate::types::extract::Json) or `export(json)` | [`TsJson`] | `number \| null` |
//! | [`TryFromJs`](crate::types::extract::TryFromJs) (bare arguments) | [`TsFromJs`] | `number \| null \| undefined` |
//! | [`TryIntoJs`](crate::types::extract::TryIntoJs) (bare returns) | [`TsIntoJs`] | `number \| undefined` |
//!
//! The export macros pick the trait that matches the conversion actually used at
//! each position, so every declaration describes what happens at runtime. A type
//! with no known TypeScript type is declared `unknown`.
//!
//! # Usage
//!
//! Enable the `typescript` feature in your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! neon = { version = "...", features = ["typescript"] }
//! ```
//!
//! The generated declarations are attached to the addon's exports under
//! `Symbol.for("neon:types")`. They can also be produced with [`generate`]:
//!
//! ```ignore
//! let dts = neon::typescript::generate()?;
//! std::fs::write("index.d.ts", dts)?;
//! ```
//!
//! Generation fails if two different items would be declared under the same
//! name (see [`generate`]). Reading `Symbol.for("neon:types")` then throws the
//! error instead; loading the addon never fails.

use std::borrow::Cow;
use std::cell::{Ref, RefCell, RefMut};
use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::Arc;

use either::Either;
use neon_typescript::array_of;

use crate::handle::{Handle, Root};
use crate::macro_internal::ExportMeta;
use crate::object::Object;
use crate::types::buffer::Binary;
use crate::types::extract::{Array, ArrayBuffer, Boxed, Buffer};
use crate::types::{JsBox, JsTypedArray, Value};
use crate::types_impl::extract::private::Sealed;

#[cfg(feature = "serde")]
use crate::types::extract::Json;

/// The TypeScript type of a value as serialized to JSON by serde_json (used
/// for [`Json<T>`](crate::types::extract::Json) and `#[neon::export(json)]`).
///
/// The trait and its implementations for standard-library types live in the
/// [`neon-typescript`](neon_typescript) crate, so that generator adapters (e.g.
/// `neon-ts-rs`) can implement it without depending on `neon`.
pub use neon_typescript::TsJson;

// ——— Bare conversion traits ———

/// The TypeScript type of the JavaScript values accepted by a type's
/// [`TryFromJs`](crate::types::extract::TryFromJs) conversion (a bare argument).
///
/// Sealed: every bare conversion is defined by Neon, so Neon provides the
/// TypeScript type for each of them.
pub trait TsFromJs: Sealed {
    /// The TypeScript type expression for the accepted values.
    fn ts_from_js() -> Cow<'static, str>;

    /// Collect any declarations the type expression refers to.
    fn ts_from_js_collect(_decls: &mut BTreeMap<String, String>) {}
}

/// The TypeScript type of the JavaScript value produced by a type's
/// [`TryIntoJs`](crate::types::extract::TryIntoJs) conversion (a bare return).
///
/// Sealed: every bare conversion is defined by Neon, so Neon provides the
/// TypeScript type for each of them.
pub trait TsIntoJs: Sealed {
    /// The TypeScript type expression for the produced value.
    fn ts_into_js() -> Cow<'static, str>;

    /// Collect any declarations the type expression refers to.
    fn ts_into_js_collect(_decls: &mut BTreeMap<String, String>) {}
}

/// Implement both bare traits with the same TypeScript type.
macro_rules! ts_bare {
    ($([$($gen:tt)*] $ty:ty => $ts:expr),* $(,)?) => {$(
        impl<$($gen)*> TsFromJs for $ty {
            fn ts_from_js() -> Cow<'static, str> { $ts.into() }
        }
        impl<$($gen)*> TsIntoJs for $ty {
            fn ts_into_js() -> Cow<'static, str> { $ts.into() }
        }
    )*};
}

// ——— Primitives ———

impl TsFromJs for f64 {
    fn ts_from_js() -> Cow<'static, str> {
        "number".into()
    }
}

impl TsFromJs for i32 {
    fn ts_from_js() -> Cow<'static, str> {
        "number".into()
    }
}

impl TsFromJs for u32 {
    fn ts_from_js() -> Cow<'static, str> {
        "number".into()
    }
}

macro_rules! ts_into_number {
    ($($ty:ty),* $(,)?) => {$(
        impl TsIntoJs for $ty {
            fn ts_into_js() -> Cow<'static, str> { "number".into() }
        }
    )*};
}

ts_into_number!(u8, u16, u32, i8, i16, i32, f32, f64);

ts_bare! {
    [] bool => "boolean",
    [] String => "string",
}

impl TsIntoJs for &str {
    fn ts_into_js() -> Cow<'static, str> {
        "string".into()
    }
}

impl TsIntoJs for &String {
    fn ts_into_js() -> Cow<'static, str> {
        "string".into()
    }
}

// Extracting `()` is infallible and ignores its argument.
impl TsFromJs for () {
    fn ts_from_js() -> Cow<'static, str> {
        "unknown".into()
    }
}

impl TsIntoJs for () {
    fn ts_into_js() -> Cow<'static, str> {
        "undefined".into()
    }
}

#[cfg(feature = "napi-5")]
ts_bare! {
    [] crate::types::extract::Date => "Date",
}

// ——— Option and Result ———

// `TryFromJs` maps both `null` and `undefined` to `None`.
impl<T: TsFromJs> TsFromJs for Option<T> {
    fn ts_from_js() -> Cow<'static, str> {
        format!("{} | null | undefined", T::ts_from_js()).into()
    }

    fn ts_from_js_collect(decls: &mut BTreeMap<String, String>) {
        T::ts_from_js_collect(decls);
    }
}

// `TryIntoJs` produces `undefined` for `None` (never `null`).
impl<T: TsIntoJs> TsIntoJs for Option<T> {
    fn ts_into_js() -> Cow<'static, str> {
        format!("{} | undefined", T::ts_into_js()).into()
    }

    fn ts_into_js_collect(decls: &mut BTreeMap<String, String>) {
        T::ts_into_js_collect(decls);
    }
}

// The `Err` is thrown, so only the `Ok` value is observable.
impl<T: TsIntoJs, E> TsIntoJs for Result<T, E> {
    fn ts_into_js() -> Cow<'static, str> {
        T::ts_into_js()
    }

    fn ts_into_js_collect(decls: &mut BTreeMap<String, String>) {
        T::ts_into_js_collect(decls);
    }
}

impl<T: TsIntoJs + crate::types::extract::TryIntoJs<'static>> TsIntoJs for Box<T> {
    fn ts_into_js() -> Cow<'static, str> {
        T::ts_into_js()
    }

    fn ts_into_js_collect(decls: &mut BTreeMap<String, String>) {
        T::ts_into_js_collect(decls);
    }
}

impl<L: TsFromJs, R: TsFromJs> TsFromJs for Either<L, R> {
    fn ts_from_js() -> Cow<'static, str> {
        format!("{} | {}", L::ts_from_js(), R::ts_from_js()).into()
    }

    fn ts_from_js_collect(decls: &mut BTreeMap<String, String>) {
        L::ts_from_js_collect(decls);
        R::ts_from_js_collect(decls);
    }
}

impl<L: TsIntoJs, R: TsIntoJs> TsIntoJs for Either<L, R> {
    fn ts_into_js() -> Cow<'static, str> {
        format!("{} | {}", L::ts_into_js(), R::ts_into_js()).into()
    }

    fn ts_into_js_collect(decls: &mut BTreeMap<String, String>) {
        L::ts_into_js_collect(decls);
        R::ts_into_js_collect(decls);
    }
}

// ——— Arrays and binary data ———

// A native JS array, converted element by element.
impl<T> TsFromJs for Array<T>
where
    T: IntoIterator,
    T::Item: TsFromJs,
{
    fn ts_from_js() -> Cow<'static, str> {
        array_of(&T::Item::ts_from_js()).into()
    }

    fn ts_from_js_collect(decls: &mut BTreeMap<String, String>) {
        T::Item::ts_from_js_collect(decls);
    }
}

impl<T> TsIntoJs for Array<T>
where
    T: IntoIterator,
    T::Item: TsIntoJs,
{
    fn ts_into_js() -> Cow<'static, str> {
        array_of(&T::Item::ts_into_js()).into()
    }

    fn ts_into_js_collect(decls: &mut BTreeMap<String, String>) {
        T::Item::ts_into_js_collect(decls);
    }
}

/// The JavaScript typed array that holds elements of a [`Binary`] type.
#[doc(hidden)]
pub trait TsTypedArrayElem: Binary {
    const TYPED_ARRAY: &'static str;
}

macro_rules! ts_typed_array_elem {
    ($($ty:ty => $name:literal),* $(,)?) => {$(
        impl TsTypedArrayElem for $ty {
            const TYPED_ARRAY: &'static str = $name;
        }
    )*};
}

ts_typed_array_elem! {
    i8 => "Int8Array",
    u8 => "Uint8Array",
    i16 => "Int16Array",
    u16 => "Uint16Array",
    i32 => "Int32Array",
    u32 => "Uint32Array",
    f32 => "Float32Array",
    f64 => "Float64Array",
    i64 => "BigInt64Array",
    u64 => "BigUint64Array",
}

// A bare `Vec<T>` (and `[T; N]` / `&[T]`) of binary elements is a typed array,
// not a JS array.
impl<T> TsFromJs for Vec<T>
where
    JsTypedArray<T>: Value,
    T: TsTypedArrayElem,
{
    fn ts_from_js() -> Cow<'static, str> {
        T::TYPED_ARRAY.into()
    }
}

impl<T> TsIntoJs for Vec<T>
where
    JsTypedArray<T>: Value,
    T: TsTypedArrayElem,
{
    fn ts_into_js() -> Cow<'static, str> {
        T::TYPED_ARRAY.into()
    }
}

impl<T, const N: usize> TsIntoJs for [T; N]
where
    JsTypedArray<T>: Value,
    T: TsTypedArrayElem,
{
    fn ts_into_js() -> Cow<'static, str> {
        T::TYPED_ARRAY.into()
    }
}

impl<T> TsIntoJs for &[T]
where
    JsTypedArray<T>: Value,
    T: TsTypedArrayElem,
{
    fn ts_into_js() -> Cow<'static, str> {
        T::TYPED_ARRAY.into()
    }
}

ts_bare! {
    [B] ArrayBuffer<B> => "ArrayBuffer",
    [B] Buffer<B> => "Buffer",
    [T] crate::types::extract::Int8Array<T> => "Int8Array",
    [T] crate::types::extract::Uint8Array<T> => "Uint8Array",
    [T] crate::types::extract::Int16Array<T> => "Int16Array",
    [T] crate::types::extract::Uint16Array<T> => "Uint16Array",
    [T] crate::types::extract::Int32Array<T> => "Int32Array",
    [T] crate::types::extract::Uint32Array<T> => "Uint32Array",
    [T] crate::types::extract::Float32Array<T> => "Float32Array",
    [T] crate::types::extract::Float64Array<T> => "Float64Array",
    [T] crate::types::extract::BigInt64Array<T> => "BigInt64Array",
    [T] crate::types::extract::BigUint64Array<T> => "BigUint64Array",
}

// ——— Opaque boxed values ———
//
// Several conversions store a Rust value in a `JsBox<P>`. Two boxes are only
// interchangeable at runtime if they hold the same `P` (the downcast is by
// type), so the TypeScript is a *branded interface* keyed on `P`:
//
//   export interface BoxedArcDatabase { readonly [__neon_tag]: 'Arc<Database>' }
//
// The payload `P` differs by conversion: `Boxed<T>` boxes `T`, while a bare
// `Arc<T>` boxes `Arc<T>` itself. So `Boxed<Arc<T>>` and `Arc<T>` share a brand,
// but `Boxed<T>` and `Arc<T>` do not. The name comes from
// `std::any::type_name` with module paths stripped.

/// `std::any::type_name::<T>()` with module paths stripped from every path
/// segment, e.g. `alloc::sync::Arc<my_crate::Db>` becomes `Arc<Db>`.
fn short_type_name<T: ?Sized>() -> String {
    let full = std::any::type_name::<T>();
    let mut out = String::with_capacity(full.len());
    let mut segment = String::new();

    fn flush(segment: &mut String, out: &mut String) {
        let last = segment.rsplit("::").next().unwrap_or("");
        out.push_str(last);
        segment.clear();
    }

    for c in full.chars() {
        if c.is_alphanumeric() || c == '_' || c == ':' {
            segment.push(c);
        } else {
            flush(&mut segment, &mut out);
            out.push(c);
        }
    }
    flush(&mut segment, &mut out);
    out
}

/// Reduce a type name to an identifier-safe string (ASCII alphanumerics and
/// `_`), used to synthesize the `Boxed{...}` interface name.
fn sanitize_identifier(s: &str) -> String {
    let mut out: String = s
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_')
        .collect();
    if out.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        out.insert(0, '_');
    }
    out
}

/// The branded-interface name for a `JsBox<P>`.
fn boxed_name<P: ?Sized>() -> Cow<'static, str> {
    format!("Boxed{}", sanitize_identifier(&short_type_name::<P>())).into()
}

/// Collect the branded-interface declaration for a `JsBox<P>`.
fn boxed_collect<P: ?Sized>(decls: &mut BTreeMap<String, String>) {
    let short = short_type_name::<P>();
    let name = format!("Boxed{}", sanitize_identifier(&short));
    let decl = format!("interface {name} {{ readonly [__neon_tag]: '{short}' }}");
    neon_typescript::insert_decl(decls, name.clone(), decl);
    // The brand drops module paths, so `a::Database` and `b::Database` would
    // get identical declarations and silently become interchangeable. Record
    // which payload type owns the brand, so that's reported as a conflict.
    neon_typescript::insert_decl(
        decls,
        format!("{BRAND_OWNER_PREFIX}{name}"),
        std::any::type_name::<P>().to_string(),
    );
}

/// Records which boxed payload type a brand belongs to (see `boxed_collect`).
/// Like every key starting with `!`, it's bookkeeping, not a declaration.
const BRAND_OWNER_PREFIX: &str = "!brand:";

/// Implement both bare traits as the brand for a `JsBox<$payload>`.
macro_rules! ts_boxed {
    ($([$($gen:tt)*] $ty:ty => $payload:ty),* $(,)?) => {$(
        impl<$($gen)*> TsFromJs for $ty {
            fn ts_from_js() -> Cow<'static, str> { boxed_name::<$payload>() }
            fn ts_from_js_collect(decls: &mut BTreeMap<String, String>) {
                boxed_collect::<$payload>(decls)
            }
        }
        impl<$($gen)*> TsIntoJs for $ty {
            fn ts_into_js() -> Cow<'static, str> { boxed_name::<$payload>() }
            fn ts_into_js_collect(decls: &mut BTreeMap<String, String>) {
                boxed_collect::<$payload>(decls)
            }
        }
    )*};
}

ts_boxed! {
    [T] Boxed<T> => T,
    [T] Arc<T> => Arc<T>,
    [T] Rc<T> => Rc<T>,
    // `&RefCell<T>`, `Ref<T>`, and `RefMut<T>` arguments all borrow from a
    // `JsBox<RefCell<T>>`, which is what returning a `RefCell<T>` produces.
    [T] RefCell<T> => RefCell<T>,
    [T] &RefCell<T> => RefCell<T>,
    ['a, T] Ref<'a, T> => RefCell<T>,
    ['a, T] RefMut<'a, T> => RefCell<T>,
}

// ——— Handles ———

/// The TypeScript type of a JavaScript value type, for `Handle<V>` and
/// `Root<V>`.
#[doc(hidden)]
pub trait TsValue: Value {
    fn ts_value() -> Cow<'static, str>;

    fn ts_value_collect(_decls: &mut BTreeMap<String, String>) {}
}

macro_rules! ts_value {
    ($($(#[$attr:meta])* $ty:ty => $ts:literal),* $(,)?) => {$(
        $(#[$attr])*
        impl TsValue for $ty {
            fn ts_value() -> Cow<'static, str> { $ts.into() }
        }
    )*};
}

ts_value! {
    crate::types::JsValue => "unknown",
    crate::types::JsString => "string",
    crate::types::JsNumber => "number",
    crate::types::JsBoolean => "boolean",
    crate::types::JsNull => "null",
    crate::types::JsUndefined => "undefined",
    crate::types::JsArray => "unknown[]",
    crate::types::JsObject => "object",
    crate::types::JsFunction => "Function",
    crate::types::JsPromise => "Promise<unknown>",
    crate::types::JsArrayBuffer => "ArrayBuffer",
    crate::types::JsBuffer => "Buffer",
    crate::types::JsError => "Error",
    #[cfg(feature = "napi-5")]
    crate::types::JsDate => "Date",
    #[cfg(feature = "napi-6")]
    crate::types::JsBigInt => "bigint",
}

impl<T> TsValue for JsTypedArray<T>
where
    JsTypedArray<T>: Value,
    T: TsTypedArrayElem,
{
    fn ts_value() -> Cow<'static, str> {
        T::TYPED_ARRAY.into()
    }
}

impl<T: 'static> TsValue for JsBox<T> {
    fn ts_value() -> Cow<'static, str> {
        boxed_name::<T>()
    }

    fn ts_value_collect(decls: &mut BTreeMap<String, String>) {
        boxed_collect::<T>(decls);
    }
}

impl<'cx, V: TsValue> TsFromJs for Handle<'cx, V> {
    fn ts_from_js() -> Cow<'static, str> {
        V::ts_value()
    }

    fn ts_from_js_collect(decls: &mut BTreeMap<String, String>) {
        V::ts_value_collect(decls);
    }
}

impl<'cx, V: TsValue> TsIntoJs for Handle<'cx, V> {
    fn ts_into_js() -> Cow<'static, str> {
        V::ts_value()
    }

    fn ts_into_js_collect(decls: &mut BTreeMap<String, String>) {
        V::ts_value_collect(decls);
    }
}

impl<O: Object + TsValue> TsFromJs for Root<O> {
    fn ts_from_js() -> Cow<'static, str> {
        O::ts_value()
    }

    fn ts_from_js_collect(decls: &mut BTreeMap<String, String>) {
        O::ts_value_collect(decls);
    }
}

impl<O: Object + TsValue> TsIntoJs for Root<O> {
    fn ts_into_js() -> Cow<'static, str> {
        O::ts_value()
    }

    fn ts_into_js_collect(decls: &mut BTreeMap<String, String>) {
        O::ts_value_collect(decls);
    }
}

// ——— JSON ———
//
// A `Json<T>` crosses the boundary through serde_json, so its TypeScript type
// is the payload's JSON type. (The macros look through a top-level `Json<T>`
// themselves, so an adapter's `TsJsonExt` can also describe the payload; these
// impls cover a `Json<T>` nested inside another bare type.)

#[cfg(feature = "serde")]
impl<T: TsJson> TsFromJs for Json<T> {
    fn ts_from_js() -> Cow<'static, str> {
        T::ts_json()
    }

    fn ts_from_js_collect(decls: &mut BTreeMap<String, String>) {
        T::ts_json_collect(decls);
    }
}

#[cfg(feature = "serde")]
impl<T: TsJson> TsIntoJs for Json<T> {
    fn ts_into_js() -> Cow<'static, str> {
        T::ts_json()
    }

    fn ts_into_js_collect(decls: &mut BTreeMap<String, String>) {
        T::ts_json_collect(decls);
    }
}

#[cfg(feature = "serde")]
impl<T: TsJson> TsIntoJs for &Json<T> {
    fn ts_into_js() -> Cow<'static, str> {
        T::ts_json()
    }

    fn ts_into_js_collect(decls: &mut BTreeMap<String, String>) {
        T::ts_json_collect(decls);
    }
}

// ——— Bare autoref specialization probes ———
//
// Like `neon_typescript::TsJsonProbe`, but for bare arguments (`TsFromJs`) and
// bare returns (`TsIntoJs`). There's no adapter rung: generators only describe
// serde shapes, which are never what a bare conversion produces. Every bare
// conversion should have a shape, so the `unknown` fallback is a safety net.

#[doc(hidden)]
pub struct TsFromJsProbe<T: ?Sized>(pub PhantomData<T>);

impl<T: TsFromJs + ?Sized> TsFromJsProbe<T> {
    pub fn ts_from_js_of(&self) -> Cow<'static, str> {
        T::ts_from_js()
    }

    pub fn ts_from_js_collect_of(&self, decls: &mut BTreeMap<String, String>) {
        T::ts_from_js_collect(decls);
    }
}

#[doc(hidden)]
pub trait TsFromJsFallback {
    fn ts_from_js_of(&self) -> Cow<'static, str>;
    fn ts_from_js_collect_of(&self, decls: &mut BTreeMap<String, String>);
}

impl<T: ?Sized> TsFromJsFallback for &TsFromJsProbe<T> {
    fn ts_from_js_of(&self) -> Cow<'static, str> {
        "unknown".into()
    }

    fn ts_from_js_collect_of(&self, _: &mut BTreeMap<String, String>) {}
}

#[doc(hidden)]
pub struct TsIntoJsProbe<T: ?Sized>(pub PhantomData<T>);

impl<T: TsIntoJs + ?Sized> TsIntoJsProbe<T> {
    pub fn ts_into_js_of(&self) -> Cow<'static, str> {
        T::ts_into_js()
    }

    pub fn ts_into_js_collect_of(&self, decls: &mut BTreeMap<String, String>) {
        T::ts_into_js_collect(decls);
    }
}

#[doc(hidden)]
pub trait TsIntoJsFallback {
    fn ts_into_js_of(&self) -> Cow<'static, str>;
    fn ts_into_js_collect_of(&self, decls: &mut BTreeMap<String, String>);
}

impl<T: ?Sized> TsIntoJsFallback for &TsIntoJsProbe<T> {
    fn ts_into_js_of(&self) -> Cow<'static, str> {
        "unknown".into()
    }

    fn ts_into_js_collect_of(&self, _: &mut BTreeMap<String, String>) {}
}

// ——— Generation ———

/// Generate TypeScript declarations for all `#[neon::export]`-ed items.
///
/// Returns a complete `.d.ts` file as a string. The output is CommonJS-style
/// type declarations suitable for placing beside a `.node` binary module.
///
/// # Errors
///
/// Fails if two different items would be declared under the same name, such
/// as two data types both named `Config`, a data type and a class with the
/// same name, or two exports with the same name. Any one declaration would be
/// wrong (or the file invalid), so the error lists every collision instead.
///
/// # Example
///
/// ```ignore
/// fn main() {
///     let dts = neon::typescript::generate().unwrap();
///     std::fs::write("index.d.ts", dts).unwrap();
/// }
/// ```
pub fn generate() -> Result<String, GenerateError> {
    render(crate::macro_internal::TYPE_METADATA.iter())
}

/// Two or more items that would be declared under the same name; see
/// [`generate`].
#[derive(Debug, Clone)]
pub struct GenerateError {
    collisions: Vec<String>,
}

impl std::fmt::Display for GenerateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "Neon can't generate TypeScript declarations, because some names are declared more than once:"
        )?;
        for collision in &self.collisions {
            writeln!(f, "  - {collision}")?;
        }
        write!(
            f,
            "Rename one item in each collision: a function, method, or export with \
             `name = \"...\"`, a class with `class(name = \"...\")`, or a JSON data type in \
             its generator (for ts-rs, `#[ts(rename = \"...\")]`)."
        )
    }
}

impl std::error::Error for GenerateError {}

fn render<'a>(metas: impl IntoIterator<Item = &'a ExportMeta>) -> Result<String, GenerateError> {
    let mut decls: BTreeMap<String, String> = BTreeMap::new();
    let mut functions: Vec<String> = Vec::new();
    let mut classes: Vec<String> = Vec::new();
    let mut globals: Vec<String> = Vec::new();

    // The names each item declares, inside the file (`locals`) and as module
    // exports (`exports`), to detect collisions.
    let mut locals = Scope::default();
    let mut exports = Scope::default();

    for meta in metas {
        match meta {
            ExportMeta::Function(func) => {
                // Collect type declarations from params and return type
                for param in func.params.iter() {
                    (param.ts_collect)(&mut decls);
                }
                (func.ret_collect)(&mut decls);

                // Build function signature
                let params = render_params(func.params);

                // Normalize unit returns to `void` first, then wrap in Promise
                // so async unit returns become `Promise<void>`, not
                // `Promise<undefined>`.
                let ret = (func.ret_type)();
                let ret = if ret == "undefined" { "void" } else { &ret };
                let ret = if func.is_async {
                    format!("Promise<{ret}>")
                } else {
                    ret.to_string()
                };

                let signature = format!("({params}): {ret};");
                functions.push(export_function(func.name, &signature));
                declare_export(
                    &mut locals,
                    &mut exports,
                    func.name,
                    format!("function `{}`", func.name),
                );
            }
            ExportMeta::Global(global) => {
                (global.ts_collect)(&mut decls);
                let ts = (global.ts_type)();
                globals.push(export_declaration(
                    "const",
                    global.name,
                    &format!(": {ts};"),
                ));
                declare_export(
                    &mut locals,
                    &mut exports,
                    global.name,
                    format!("const `{}`", global.name),
                );
            }
            ExportMeta::Class(class) => {
                // Collect type declarations from all parts of the class
                if let Some(ctor) = &class.constructor {
                    for param in ctor.params.iter() {
                        (param.ts_collect)(&mut decls);
                    }
                }
                for method in class.methods.iter() {
                    for param in method.params.iter() {
                        (param.ts_collect)(&mut decls);
                    }
                    (method.ret_collect)(&mut decls);
                }
                for prop in class.static_properties.iter() {
                    (prop.ts_collect)(&mut decls);
                }

                // Declare the class under its class name, and export it under
                // its actual module binding, which can differ (e.g.
                // `#[neon::export(class(name = "X"), name = "Y")]`). A bare
                // `#[neon::class]` isn't exported by Neon (if user code exports
                // it, Neon can't see under what name), so it's declared but not
                // exported: other declarations can still refer to it.
                let what = match class.export_name {
                    Some(binding) if binding != class.name => {
                        format!("class `{}` (exported as `{binding}`)", class.name)
                    }
                    _ => format!("class `{}`", class.name),
                };
                locals.declare(class.name, Space::Both, what.clone());
                if let Some(binding) = class.export_name {
                    if is_identifier_name(binding) {
                        exports.declare(binding, Space::Both, what);
                    }
                }

                let mut s = if class.export_name == Some(class.name) {
                    format!("export declare class {} {{\n", class.name)
                } else {
                    format!("declare class {} {{\n", class.name)
                };

                // Constructor. A class with no declared constructor gets an
                // implicit public `constructor()` in TypeScript, so a hidden
                // constructor must be declared private.
                if class.private_constructor {
                    s.push_str("  private constructor();\n");
                } else if let Some(ctor) = &class.constructor {
                    let params = render_params(ctor.params);
                    s.push_str(&format!("  constructor({params});\n"));
                }

                // Static properties
                for prop in class.static_properties.iter() {
                    let ts = (prop.ts_type)();
                    s.push_str(&format!(
                        "  static readonly {}: {};\n",
                        property_name(prop.name),
                        ts
                    ));
                }

                // Methods
                for method in class.methods.iter() {
                    let params = render_params(method.params);
                    // Normalize unit returns to `void` before wrapping in
                    // Promise (async unit → `Promise<void>`, not
                    // `Promise<undefined>`).
                    let ret = (method.ret_type)();
                    let ret = if ret == "undefined" { "void" } else { &ret };
                    let ret_str = if method.is_async {
                        format!("Promise<{ret}>")
                    } else {
                        ret.to_string()
                    };
                    s.push_str(&format!(
                        "  {}({params}): {ret_str};\n",
                        property_name(method.name)
                    ));
                }

                s.push('}');
                if let Some(binding) = class.export_name.filter(|b| *b != class.name) {
                    if is_identifier_name(binding) {
                        s.push_str(&format!("\nexport {{ {} as {binding} }};", class.name));
                    } else {
                        s.push_str(&format!(
                            "\n// neon: omitted export {binding:?}, which is not a valid TypeScript identifier"
                        ));
                    }
                }
                classes.push(s);
            }
        }
    }

    // Keys starting with `!` are bookkeeping from `insert_decl` and
    // `boxed_collect`, not declarations. Conflicting declarations under one
    // name were recorded there.
    let mut collisions: Vec<String> = Vec::new();
    for (key, value) in &decls {
        let Some(name) = key.strip_prefix(neon_typescript::CONFLICT_PREFIX) else {
            continue;
        };
        let mut versions: Vec<&str> = value.split(neon_typescript::CONFLICT_SEPARATOR).collect();
        versions.sort_unstable();
        collisions.push(match name.strip_prefix(BRAND_OWNER_PREFIX) {
            Some(brand) => format!(
                "`{brand}` is the boxed type of more than one Rust type: {}",
                list(versions.iter().map(|ty| format!("`{ty}`")))
            ),
            None => format!(
                "two different types are both declared as `{name}`: {}",
                list(
                    versions
                        .iter()
                        .map(|decl| format!("`{}`", abbreviate(decl)))
                )
            ),
        });
    }
    decls.retain(|key, _| !key.starts_with('!'));

    let has_opaque = decls.keys().any(|k| k.starts_with("Boxed"));
    for (name, decl) in &decls {
        let what = format!("type `{name}`");
        let space = decl_space(decl);
        locals.declare(name, space, what.clone());
        exports.declare(name, space, what);
    }
    if has_opaque {
        let what = "Neon's `__neon_tag` brand symbol".to_string();
        locals.declare("__neon_tag", Space::Value, what.clone());
        exports.declare("__neon_tag", Space::Value, what);
    }

    // Report each set of colliding items once, under its export name if it
    // has one (an exported function shows up in both scopes).
    let mut reported = std::collections::BTreeSet::new();
    for (name, mut items) in exports.collisions().chain(locals.collisions()) {
        items.sort();
        if reported.insert(items.clone()) {
            collisions.push(format!("`{name}` is declared by {}", list(items)));
        }
    }

    if !collisions.is_empty() {
        collisions.sort();
        return Err(GenerateError { collisions });
    }

    // `TYPE_METADATA` is in link order, which can vary between builds; sort so
    // the output is deterministic. (Declarations are already ordered by name.)
    functions.sort();
    classes.sort();
    globals.sort();

    let mut output = String::new();
    output.push_str("// Auto-generated by Neon. Do not edit.\n");

    // `Buffer` is declared by Node's types, which a project may not load by
    // default; reference them so the file stands on its own.
    let uses_buffer = decls
        .values()
        .chain(&classes)
        .chain(&functions)
        .chain(&globals)
        .any(|s| contains_word(s, "Buffer"));
    if uses_buffer {
        output.push_str("/// <reference types=\"node\" />\n");
    }

    // Emit the branding symbol if we have any opaque types
    if has_opaque {
        output.push('\n');
        output.push_str("export declare const __neon_tag: unique symbol;\n");
    }

    // Emit type declarations
    if !decls.is_empty() {
        output.push('\n');
        for decl in decls.values() {
            if !decl.starts_with("export ") {
                output.push_str("export ");
            }
            output.push_str(decl);
            output.push('\n');
        }
    }

    // Emit class declarations
    if !classes.is_empty() {
        output.push('\n');
        for class in &classes {
            output.push_str(class);
            output.push('\n');
        }
    }

    // Emit global declarations
    if !globals.is_empty() {
        output.push('\n');
        for global in &globals {
            output.push_str(global);
            output.push('\n');
        }
    }

    // Emit function declarations
    if !functions.is_empty() {
        output.push('\n');
        for func in &functions {
            output.push_str(func);
            output.push('\n');
        }
    }

    // In a `.d.ts` module, every top-level declaration is implicitly exported
    // unless the file has an export statement like this one. Without it, a
    // class that Neon doesn't export (and the internal names behind aliased
    // exports) would still be importable.
    output.push_str("\nexport {};\n");

    Ok(output)
}

// ——— Collisions ———
//
// TypeScript has two declaration spaces: a type alias or interface declares a
// type, a function or `const` declares a value, and a class declares both. A
// type and a value can share a name (`type Config` and `function Config` are
// both fine); two types or two values can't.

#[derive(Clone, Copy)]
enum Space {
    Type,
    Value,
    Both,
}

/// The items declaring each name in one scope, by declaration space.
#[derive(Default)]
struct Scope {
    types: BTreeMap<String, Vec<String>>,
    values: BTreeMap<String, Vec<String>>,
}

impl Scope {
    fn declare(&mut self, name: &str, space: Space, what: String) {
        if matches!(space, Space::Type | Space::Both) {
            self.types
                .entry(name.into())
                .or_default()
                .push(what.clone());
        }
        if matches!(space, Space::Value | Space::Both) {
            self.values.entry(name.into()).or_default().push(what);
        }
    }

    /// Each name declared by more than one item in the same space.
    fn collisions(&self) -> impl Iterator<Item = (String, Vec<String>)> + '_ {
        self.types
            .iter()
            .chain(&self.values)
            .filter(|(_, items)| items.len() > 1)
            .map(|(name, items)| (name.clone(), items.clone()))
    }
}

/// Declare an exported function or `const`: its export name, and its name in
/// the file (an internal name for a reserved word; see `export_declaration`).
fn declare_export(locals: &mut Scope, exports: &mut Scope, name: &str, what: String) {
    if is_binding_name(name) {
        locals.declare(name, Space::Value, what.clone());
    } else if is_identifier_name(name) {
        locals.declare(&format!("__neon_export_{name}"), Space::Value, what.clone());
    } else {
        // Omitted from the output, so it can't collide there.
        return;
    }
    exports.declare(name, Space::Value, what);
}

/// The declaration space of a collected declaration. Anything other than a
/// type alias or interface (e.g. an `enum`) is assumed to declare both.
fn decl_space(decl: &str) -> Space {
    let decl = decl.trim_start();
    let decl = decl.strip_prefix("export ").unwrap_or(decl);
    let decl = decl.strip_prefix("declare ").unwrap_or(decl);
    if decl.starts_with("type ") || decl.starts_with("interface ") {
        Space::Type
    } else {
        Space::Both
    }
}

/// `a and b`, or `a, b, and c`.
fn list(items: impl IntoIterator<Item = String>) -> String {
    let items: Vec<String> = items.into_iter().collect();
    match items.as_slice() {
        [] => String::new(),
        [one] => one.clone(),
        [a, b] => format!("{a} and {b}"),
        [rest @ .., last] => format!("{}, and {last}", rest.join(", ")),
    }
}

/// The first line of a declaration, shortened, for an error message.
fn abbreviate(decl: &str) -> String {
    const MAX: usize = 80;
    let line = decl.lines().next().unwrap_or_default().trim();
    if line.chars().count() > MAX || decl.lines().nth(1).is_some() {
        let short: String = line.chars().take(MAX).collect();
        format!("{short} …")
    } else {
        line.to_string()
    }
}

// ——— Names ———
//
// Export, parameter, and member names come from Rust identifiers or from
// user-supplied strings (`name = "..."`), and one invalid name would make the
// whole file unparseable. These helpers make every emitted name valid.

/// Words that can't be used as a binding name (a function or parameter name)
/// in a TypeScript module, which is always strict mode.
const RESERVED_WORDS: &[&str] = &[
    "await",
    "break",
    "case",
    "catch",
    "class",
    "const",
    "continue",
    "debugger",
    "default",
    "delete",
    "do",
    "else",
    "enum",
    "export",
    "extends",
    "false",
    "finally",
    "for",
    "function",
    "if",
    "implements",
    "import",
    "in",
    "instanceof",
    "interface",
    "let",
    "new",
    "null",
    "package",
    "private",
    "protected",
    "public",
    "return",
    "static",
    "super",
    "switch",
    "this",
    "throw",
    "true",
    "try",
    "typeof",
    "var",
    "void",
    "while",
    "with",
    "yield",
];

/// Whether `name` is an *IdentifierName*: usable as a property name or in an
/// export specifier, though possibly a reserved word.
fn is_identifier_name(name: &str) -> bool {
    let mut chars = name.chars();
    chars
        .next()
        .is_some_and(|c| c.is_alphabetic() || c == '_' || c == '$')
        && chars.all(|c| c.is_alphanumeric() || c == '_' || c == '$')
}

/// Whether `name` can be used as a binding name (an identifier that isn't a
/// reserved word).
fn is_binding_name(name: &str) -> bool {
    is_identifier_name(name) && !RESERVED_WORDS.contains(&name)
}

/// A valid parameter name: a raw identifier's `r#` prefix is dropped, and a
/// reserved word gets a trailing `_` (parameter names have no runtime meaning).
fn param_name(name: &str, index: usize) -> String {
    let name = name.strip_prefix("r#").unwrap_or(name);
    if is_binding_name(name) {
        name.to_string()
    } else if is_identifier_name(name) {
        format!("{name}_")
    } else {
        format!("arg{index}")
    }
}

/// A parameter list. A trailing run of parameters whose types accept
/// `undefined` is declared optional (`options?: T`), since an omitted argument
/// reaches Neon as `undefined`: N-API fills missing arguments with it. Only a
/// trailing run, because TypeScript doesn't allow a required parameter after
/// an optional one.
fn render_params(params: &[crate::macro_internal::ParamMeta]) -> String {
    let types: Vec<Cow<'static, str>> = params.iter().map(|p| (p.ts_type)()).collect();
    let optional_from = types
        .iter()
        .rposition(|ty| without_undefined(ty).is_none())
        .map_or(0, |last_required| last_required + 1);

    params
        .iter()
        .zip(&types)
        .enumerate()
        .map(|(i, (p, ty))| {
            let name = param_name(p.name, i);
            match without_undefined(ty).filter(|_| i >= optional_from) {
                Some(rest) => format!("{name}?: {rest}"),
                None => format!("{name}: {ty}"),
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// If the type `ty` is a union with an `undefined` member, the union without
/// it (`undefined` alone stays `undefined`). `None` otherwise, including for
/// `unknown` and `any`: those admit `undefined` only implicitly, which doesn't
/// mean the runtime accepts it.
fn without_undefined(ty: &str) -> Option<String> {
    let members = union_members(ty);
    if !members.contains(&"undefined") {
        return None;
    }
    let rest: Vec<&str> = members.into_iter().filter(|m| *m != "undefined").collect();
    Some(if rest.is_empty() {
        "undefined".to_string()
    } else {
        rest.join(" | ")
    })
}

/// The top-level members of a union type, so `Array<A | B> | undefined` is
/// `["Array<A | B>", "undefined"]`. Brackets and string literals are skipped
/// over, and the `>` of a function type's `=>` doesn't close a bracket.
fn union_members(ty: &str) -> Vec<&str> {
    let mut members = Vec::new();
    let mut depth = 0usize;
    let mut quote: Option<char> = None;
    let mut start = 0;
    let mut prev = '\0';
    for (i, c) in ty.char_indices() {
        match quote {
            Some(q) => {
                if c == q && prev != '\\' {
                    quote = None;
                }
            }
            None => match c {
                '"' | '\'' | '`' => quote = Some(c),
                '(' | '[' | '{' | '<' => depth += 1,
                '>' if prev == '=' => {}
                ')' | ']' | '}' | '>' => depth = depth.saturating_sub(1),
                '|' if depth == 0 => {
                    members.push(ty[start..i].trim());
                    start = i + 1;
                }
                _ => {}
            },
        }
        prev = c;
    }
    members.push(ty[start..].trim());
    members
}

/// A method or property name, quoted if it isn't an identifier name.
fn property_name(name: &str) -> String {
    if is_identifier_name(name) {
        name.to_string()
    } else {
        format!("{name:?}")
    }
}

/// Declare an exported function.
fn export_function(name: &str, signature: &str) -> String {
    export_declaration("function", name, signature)
}

/// Declare an exported `function` or `const`. A reserved word can't be the
/// declaration's own name, but can be an export name, so declare it under an
/// internal name and re-export it. A name that isn't an identifier at all can't
/// be exported without string export names (TypeScript 5.6+), so it's left out.
fn export_declaration(kind: &str, name: &str, rest: &str) -> String {
    if is_binding_name(name) {
        format!("export declare {kind} {name}{rest}")
    } else if is_identifier_name(name) {
        let local = format!("__neon_export_{name}");
        format!("declare {kind} {local}{rest}\nexport {{ {local} as {name} }};")
    } else {
        format!("// neon: omitted export {name:?}, which is not a valid TypeScript identifier")
    }
}

/// Whether `haystack` contains `word` as a whole identifier (so `ArrayBuffer`
/// doesn't count as `Buffer`).
fn contains_word(haystack: &str, word: &str) -> bool {
    let is_ident = |c: char| c.is_alphanumeric() || c == '_' || c == '$';
    haystack.match_indices(word).any(|(i, _)| {
        let before = haystack[..i].chars().next_back();
        let after = haystack[i + word.len()..].chars().next();
        !before.is_some_and(is_ident) && !after.is_some_and(is_ident)
    })
}

/// The generated declarations, or the collision report if generation failed.
/// The output depends only on the binary, so it's generated once per process,
/// even when the addon is loaded repeatedly (e.g. by worker threads).
#[cfg(feature = "typescript")]
static DTS: std::sync::OnceLock<Result<String, String>> = std::sync::OnceLock::new();

/// Attach the generated TypeScript declarations to the module exports under
/// `Symbol.for("neon:types")`. Called automatically during module init when
/// the `typescript` feature is enabled.
///
/// This lets tools (e.g. a `neon types` CLI, or a small Node script) extract
/// the `.d.ts` text by loading the addon and reading the well-known symbol. If
/// generation fails, reading the symbol throws the error instead, so the
/// failure surfaces where the declarations are extracted, never at `require()`.
#[cfg(feature = "typescript")]
pub(crate) fn attach_to_module<'cx>(
    cx: &mut crate::context::ModuleContext<'cx>,
) -> crate::result::NeonResult<()> {
    use crate::context::Context;
    use crate::types::{JsFunction, JsValue};

    let symbol_fn: Handle<JsFunction> = cx.global("Symbol")?;
    let symbol_for: Handle<JsFunction> = symbol_fn.get(cx, "for")?;

    let exports = cx.exports_object()?;

    let types_key = cx.string("neon:types");
    let types_key_arg = Handle::upcast::<JsValue>(&types_key);
    let types_symbol: Handle<JsValue> =
        symbol_for.call(cx, Handle::upcast::<JsValue>(&symbol_fn), [types_key_arg])?;

    match DTS.get_or_init(|| generate().map_err(|err| err.to_string())) {
        Ok(dts) => {
            let dts = cx.string(dts);
            exports.set(cx, types_symbol, dts)?;
        }
        Err(_) => {
            // Object.defineProperty(exports, symbol, { get: throwGenerateError })
            // (`Object` is a function, so it doesn't downcast to `JsObject`.)
            let object: Handle<JsFunction> = cx.global("Object")?;
            let define_property: Handle<JsFunction> = object.get(cx, "defineProperty")?;
            let getter = JsFunction::new(cx, throw_generate_error)?;
            let descriptor = cx.empty_object();
            descriptor.set(cx, "get", getter)?;
            define_property.call(
                cx,
                object,
                [exports.upcast(), types_symbol, descriptor.upcast()],
            )?;
        }
    }

    Ok(())
}

/// The getter for `Symbol.for("neon:types")` when generation failed.
#[cfg(feature = "typescript")]
fn throw_generate_error(
    mut cx: crate::context::FunctionContext,
) -> crate::result::JsResult<crate::types::JsValue> {
    use crate::context::Context;

    let message = match DTS.get() {
        Some(Err(message)) => message.as_str(),
        _ => "Neon couldn't generate TypeScript declarations",
    };
    cx.throw_error(message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::macro_internal::{ClassMeta, FunctionMeta, ParamMeta};

    struct Database;

    mod a {
        pub struct Database;
    }

    mod b {
        pub struct Database;
    }

    // ——— Collision fixtures ———

    fn number() -> Cow<'static, str> {
        "number".into()
    }

    fn nothing(_: &mut BTreeMap<String, String>) {}

    fn config_a(decls: &mut BTreeMap<String, String>) {
        neon_typescript::insert_decl(
            decls,
            "Config".into(),
            "type Config = { a: number };".into(),
        );
    }

    fn config_b(decls: &mut BTreeMap<String, String>) {
        neon_typescript::insert_decl(
            decls,
            "Config".into(),
            "type Config = { b: string };".into(),
        );
    }

    fn boxed_a(decls: &mut BTreeMap<String, String>) {
        <Boxed<a::Database>>::ts_into_js_collect(decls);
    }

    fn boxed_b(decls: &mut BTreeMap<String, String>) {
        <Boxed<b::Database>>::ts_into_js_collect(decls);
    }

    const fn function(
        name: &'static str,
        ret_collect: fn(&mut BTreeMap<String, String>),
    ) -> ExportMeta {
        ExportMeta::Function(FunctionMeta {
            name,
            params: &[],
            ret_type: number,
            ret_collect,
            is_async: false,
        })
    }

    const fn class(name: &'static str, export_name: Option<&'static str>) -> ExportMeta {
        ExportMeta::Class(ClassMeta {
            name,
            export_name,
            constructor: None,
            private_constructor: false,
            methods: &[],
            static_properties: &[],
        })
    }

    fn collisions(metas: &[ExportMeta]) -> Vec<String> {
        match render(metas) {
            Ok(_) => Vec::new(),
            Err(err) => err.collisions,
        }
    }

    #[test]
    fn distinct_names_dont_collide() {
        let metas = [
            function("open", config_a),
            function("close", config_a),
            class("Point", Some("Point")),
        ];
        assert_eq!(collisions(&metas), Vec::<String>::new());
    }

    #[test]
    fn a_type_and_a_value_can_share_a_name() {
        // `type Config` and `function Config` are both valid TypeScript.
        let metas = [function("Config", config_a)];
        assert_eq!(collisions(&metas), Vec::<String>::new());
    }

    #[test]
    fn different_types_with_one_name_collide() {
        let metas = [function("first", config_a), function("second", config_b)];
        assert_eq!(
            collisions(&metas),
            ["two different types are both declared as `Config`: \
              `type Config = { a: number };` and `type Config = { b: string };`"]
        );
    }

    #[test]
    fn a_type_and_a_class_with_one_name_collide() {
        let metas = [function("load", config_a), class("Config", Some("Config"))];
        assert_eq!(
            collisions(&metas),
            ["`Config` is declared by class `Config` and type `Config`"]
        );
    }

    #[test]
    fn duplicate_exports_collide() {
        let metas = [function("parse", nothing), function("parse", nothing)];
        assert_eq!(
            collisions(&metas),
            ["`parse` is declared by function `parse` and function `parse`"]
        );
        // A reserved word is declared under an internal name; it's reported
        // once, under its export name.
        let metas = [function("delete", nothing), function("delete", nothing)];
        assert_eq!(
            collisions(&metas),
            ["`delete` is declared by function `delete` and function `delete`"]
        );
    }

    #[test]
    fn a_class_name_collides_with_an_export_of_that_name() {
        // `declare class Foo` + `export { Foo as Bar }` + `function Foo`: the
        // runtime exports (`Bar`, `Foo`) are distinct, but the file isn't valid.
        let metas = [class("Foo", Some("Bar")), function("Foo", nothing)];
        assert_eq!(
            collisions(&metas),
            ["`Foo` is declared by class `Foo` (exported as `Bar`) and function `Foo`"]
        );
        // So does a bare class, which is declared but not exported.
        let metas = [class("Foo", None), function("Foo", nothing)];
        assert_eq!(
            collisions(&metas),
            ["`Foo` is declared by class `Foo` and function `Foo`"]
        );
    }

    #[test]
    fn same_named_boxed_types_collide() {
        let metas = [function("openA", boxed_a), function("openB", boxed_b)];
        let a = std::any::type_name::<a::Database>();
        let b = std::any::type_name::<b::Database>();
        let (first, second) = if a < b { (a, b) } else { (b, a) };
        assert_eq!(
            collisions(&metas),
            [format!(
                "`BoxedDatabase` is the boxed type of more than one Rust type: \
                 `{first}` and `{second}`"
            )]
        );
    }

    #[test]
    fn the_error_lists_every_collision() {
        let metas = [
            function("parse", config_a),
            function("parse", config_b),
            class("Config", Some("Config")),
        ];
        let message = render(&metas).unwrap_err().to_string();
        assert!(message.starts_with("Neon can't generate TypeScript declarations"));
        assert_eq!(message.matches("\n  - ").count(), 3, "{message}");
        assert!(message.contains("#[ts(rename"), "{message}");
    }

    #[test]
    fn union_members_split_only_at_the_top_level() {
        assert_eq!(union_members("number"), ["number"]);
        assert_eq!(
            union_members("Array<A | B> | null | undefined"),
            ["Array<A | B>", "null", "undefined"]
        );
        assert_eq!(
            union_members("{ a: \"x | y\" } | undefined"),
            ["{ a: \"x | y\" }", "undefined"]
        );
        assert_eq!(
            union_members("((a: number) => A | B) | undefined"),
            ["((a: number) => A | B)", "undefined"]
        );
    }

    #[test]
    fn without_undefined_needs_an_explicit_member() {
        assert_eq!(
            without_undefined("T | null | undefined").as_deref(),
            Some("T | null")
        );
        assert_eq!(without_undefined("undefined").as_deref(), Some("undefined"));
        assert_eq!(without_undefined("T | null"), None);
        assert_eq!(without_undefined("unknown"), None);
        assert_eq!(without_undefined("Array<T | undefined>"), None);
    }

    fn optional_number() -> Cow<'static, str> {
        "number | null | undefined".into()
    }

    fn string() -> Cow<'static, str> {
        "string".into()
    }

    const fn param(name: &'static str, ts_type: fn() -> Cow<'static, str>) -> ParamMeta {
        ParamMeta {
            name,
            ts_type,
            ts_collect: nothing,
        }
    }

    #[test]
    fn trailing_undefined_params_are_optional() {
        let params = [
            param("a", optional_number),
            param("b", string),
            param("c", optional_number),
            param("d", optional_number),
        ];
        // `a` is followed by a required parameter, so it can't be optional.
        assert_eq!(
            render_params(&params),
            "a: number | null | undefined, b: string, c?: number | null, d?: number | null"
        );
        assert_eq!(render_params(&params[..1]), "a?: number | null");
        assert_eq!(
            render_params(&params[..2]),
            "a: number | null | undefined, b: string"
        );
        assert_eq!(render_params(&[]), "");
    }

    #[test]
    fn output_ends_with_an_export_statement() {
        let output = render(&[class("Point", None)]).unwrap();
        assert!(output.contains("\ndeclare class Point {\n"), "{output}");
        assert!(output.ends_with("\nexport {};\n"), "{output}");
    }

    #[test]
    fn names_are_made_valid() {
        assert_eq!(param_name("query", 0), "query");
        assert_eq!(param_name("r#type", 0), "type");
        assert_eq!(param_name("r#in", 0), "in_");
        assert_eq!(property_name("delete"), "delete");
        assert_eq!(property_name("two words"), "\"two words\"");
        assert_eq!(
            export_function("tsAdd", "(): void;"),
            "export declare function tsAdd(): void;"
        );
        assert_eq!(
            export_function("delete", "(): void;"),
            "declare function __neon_export_delete(): void;\nexport { __neon_export_delete as delete };"
        );
        assert!(export_function("my-func", "(): void;").starts_with("// neon: omitted"));
    }

    #[test]
    fn contains_word_matches_whole_identifiers() {
        assert!(contains_word("x: Buffer", "Buffer"));
        assert!(contains_word("Buffer[]", "Buffer"));
        assert!(!contains_word("x: ArrayBuffer", "Buffer"));
        assert!(!contains_word("BufferLike", "Buffer"));
    }

    #[test]
    fn short_type_name_strips_paths() {
        assert_eq!(short_type_name::<Option<String>>(), "Option<String>");
        assert_eq!(short_type_name::<Arc<Database>>(), "Arc<Database>");
        assert_eq!(short_type_name::<(i32, f64)>(), "(i32, f64)");
    }

    #[test]
    fn sanitize_identifier_strips_invalid_chars() {
        assert_eq!(sanitize_identifier("Database"), "Database");
        assert_eq!(sanitize_identifier("Arc<Database>"), "ArcDatabase");
        assert_eq!(sanitize_identifier("3d"), "_3d");
    }

    #[test]
    fn brands_key_on_the_boxed_payload() {
        // `Boxed<T>` boxes `T`; a bare `Arc<T>` boxes `Arc<T>` itself.
        assert_eq!(<Boxed<Database>>::ts_into_js(), "BoxedDatabase");
        assert_eq!(<Arc<Database>>::ts_into_js(), "BoxedArcDatabase");
        // `Boxed<Arc<T>>` and `Arc<T>` produce the same `JsBox<Arc<T>>`.
        assert_eq!(
            <Boxed<Arc<Database>>>::ts_into_js(),
            <Arc<Database>>::ts_into_js()
        );
        // `Ref<T>` arguments borrow from the box a returned `RefCell<T>` makes.
        assert_eq!(
            <Ref<'static, Database>>::ts_from_js(),
            <RefCell<Database>>::ts_into_js()
        );

        let mut decls = BTreeMap::new();
        <Arc<Database>>::ts_into_js_collect(&mut decls);
        assert_eq!(
            decls.get("BoxedArcDatabase").map(String::as_str),
            Some("interface BoxedArcDatabase { readonly [__neon_tag]: 'Arc<Database>' }")
        );
    }

    #[test]
    fn option_differs_by_direction() {
        assert_eq!(<Option<f64>>::ts_from_js(), "number | null | undefined");
        assert_eq!(<Option<f64>>::ts_into_js(), "number | undefined");
    }

    #[test]
    fn bare_vec_is_a_typed_array() {
        assert_eq!(<Vec<f64>>::ts_from_js(), "Float64Array");
        assert_eq!(<Vec<u8>>::ts_into_js(), "Uint8Array");
        assert_eq!(<Vec<i64>>::ts_into_js(), "BigInt64Array");
        assert_eq!(<[u8; 4]>::ts_into_js(), "Uint8Array");
    }

    #[test]
    fn array_extractor_is_a_js_array() {
        assert_eq!(<Array<Vec<String>>>::ts_from_js(), "string[]");
        assert_eq!(
            <Array<Vec<Option<f64>>>>::ts_into_js(),
            "(number | undefined)[]"
        );
    }

    #[test]
    fn result_return_is_its_ok_type() {
        assert_eq!(<Result<String, String>>::ts_into_js(), "string");
    }

    #[test]
    fn unit() {
        assert_eq!(<()>::ts_into_js(), "undefined");
        assert_eq!(<()>::ts_from_js(), "unknown");
    }

    #[test]
    fn handles() {
        assert_eq!(
            <Handle<'static, crate::types::JsString>>::ts_from_js(),
            "string"
        );
        assert_eq!(
            <Handle<'static, crate::types::JsValue>>::ts_into_js(),
            "unknown"
        );
    }

    #[cfg(feature = "serde")]
    #[test]
    fn json_uses_the_json_type() {
        assert_eq!(<Json<Option<f64>>>::ts_into_js(), "number | null");
    }

    // The `(&probe)` borrow mirrors the macro-emitted call shape.
    #[allow(clippy::needless_borrow)]
    #[test]
    fn bare_probe_falls_back_to_unknown() {
        struct NoImpl;
        let probe = TsIntoJsProbe::<NoImpl>(PhantomData);
        assert_eq!((&probe).ts_into_js_of(), "unknown");
        let probe = TsIntoJsProbe::<Option<f64>>(PhantomData);
        assert_eq!((&probe).ts_into_js_of(), "number | undefined");
    }
}
