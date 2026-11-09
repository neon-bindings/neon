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
//! let dts = neon::typescript::generate();
//! std::fs::write("index.d.ts", dts).unwrap();
//! ```

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
    decls.entry(name).or_insert(decl);
}

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
pub struct TsFromJsProbe<T>(pub PhantomData<T>);

impl<T: TsFromJs> TsFromJsProbe<T> {
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

impl<T> TsFromJsFallback for &TsFromJsProbe<T> {
    fn ts_from_js_of(&self) -> Cow<'static, str> {
        "unknown".into()
    }

    fn ts_from_js_collect_of(&self, _: &mut BTreeMap<String, String>) {}
}

#[doc(hidden)]
pub struct TsIntoJsProbe<T>(pub PhantomData<T>);

impl<T: TsIntoJs> TsIntoJsProbe<T> {
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

impl<T> TsIntoJsFallback for &TsIntoJsProbe<T> {
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
/// # Example
///
/// ```ignore
/// fn main() {
///     let dts = neon::typescript::generate();
///     std::fs::write("index.d.ts", dts).unwrap();
/// }
/// ```
pub fn generate() -> String {
    let mut decls: BTreeMap<String, String> = BTreeMap::new();
    let mut functions: Vec<String> = Vec::new();
    let mut classes: Vec<String> = Vec::new();

    for meta in crate::macro_internal::TYPE_METADATA.iter() {
        match meta {
            ExportMeta::Function(func) => {
                // Collect type declarations from params and return type
                for param in func.params.iter() {
                    (param.ts_collect)(&mut decls);
                }
                (func.ret_collect)(&mut decls);

                // Build function signature
                let params: Vec<String> = func
                    .params
                    .iter()
                    .map(|p| {
                        let ts = (p.ts_type)();
                        format!("{}: {ts}", p.name)
                    })
                    .collect();

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

                functions.push(format!(
                    "export declare function {}({}): {ret};",
                    func.name,
                    params.join(", "),
                ));
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

                let mut s = format!("export declare class {} {{\n", class.name);

                // Constructor
                if let Some(ctor) = &class.constructor {
                    let params: Vec<String> = ctor
                        .params
                        .iter()
                        .map(|p| {
                            let ts = (p.ts_type)();
                            format!("{}: {ts}", p.name)
                        })
                        .collect();
                    s.push_str(&format!("  constructor({});\n", params.join(", ")));
                }

                // Static properties
                for prop in class.static_properties.iter() {
                    let ts = (prop.ts_type)();
                    s.push_str(&format!("  static readonly {}: {};\n", prop.name, ts));
                }

                // Methods
                for method in class.methods.iter() {
                    let params: Vec<String> = method
                        .params
                        .iter()
                        .map(|p| {
                            let ts = (p.ts_type)();
                            format!("{}: {ts}", p.name)
                        })
                        .collect();
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
                        "  {}({}): {};\n",
                        method.name,
                        params.join(", "),
                        ret_str
                    ));
                }

                s.push('}');
                classes.push(s);
            }
        }
    }

    let mut output = String::new();
    output.push_str("// Auto-generated by Neon. Do not edit.\n");

    // Emit the branding symbol if we have any opaque types
    let has_opaque = decls.keys().any(|k| k.starts_with("Boxed"));
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

    // Emit function declarations
    if !functions.is_empty() {
        output.push('\n');
        for func in &functions {
            output.push_str(func);
            output.push('\n');
        }
    }

    output
}

/// Attach the generated TypeScript declarations to the module exports under
/// `Symbol.for("neon:types")`. Called automatically during module init when
/// the `typescript` feature is enabled.
///
/// This lets tools (e.g. a `neon types` CLI, or a small Node script) extract
/// the `.d.ts` text by loading the addon and reading the well-known symbol.
#[cfg(feature = "typescript")]
pub(crate) fn attach_to_module<'cx>(
    cx: &mut crate::context::ModuleContext<'cx>,
) -> crate::result::NeonResult<()> {
    use crate::context::Context;
    use crate::types::{JsFunction, JsValue};

    let symbol_fn: Handle<JsFunction> = cx.global("Symbol")?;
    let symbol_for: Handle<JsFunction> = symbol_fn.get(cx, "for")?;

    let exports = cx.exports_object()?;

    // Attach .d.ts string under Symbol.for("neon:types")
    let types_key = cx.string("neon:types");
    let types_key_arg = Handle::upcast::<JsValue>(&types_key);
    let types_symbol: Handle<JsValue> =
        symbol_for.call(cx, Handle::upcast::<JsValue>(&symbol_fn), [types_key_arg])?;
    let dts_str = cx.string(generate());
    exports.set(cx, types_symbol, dts_str)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Database;

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
