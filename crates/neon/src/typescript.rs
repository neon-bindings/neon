//! TypeScript type declaration generation for Neon modules.
//!
//! This module provides automatic generation of `.d.ts` type declaration files
//! for Neon module exports. It works by collecting type metadata from
//! `#[neon::export]` items at compile time and resolving TypeScript types at
//! runtime via the [`TypeScript`] trait.
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
//! Then call [`generate`] to produce a `.d.ts` string:
//!
//! ```ignore
//! let dts = neon::typescript::generate();
//! std::fs::write("index.d.ts", dts).unwrap();
//! ```

use std::borrow::Cow;
use std::cell::{Ref, RefCell, RefMut};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::hash::Hash;
use std::rc::Rc;
use std::sync::Arc;

#[cfg(feature = "serde")]
use crate::types::extract::Json;

use crate::types::extract::Boxed;

pub mod ast;
mod parser;

pub use ast::{Decl, TsType};

/// Parse a TypeScript type expression string into a structured AST node.
///
/// Used by macro-generated metadata to convert `#[neon(ts_type = "...")]`
/// and `#[neon::export(ts_returns = "...")]` override strings into AST.
/// Anything the parser can't structure falls through to [`TsType::Raw`].
#[doc(hidden)]
pub fn parse_type(s: &str) -> TsType {
    parser::parse(s)
}

// ——— The TypeScript trait ———

/// A Rust type that has a known TypeScript representation.
///
/// Neon provides built-in implementations for primitive types and common
/// wrappers. User-defined types (particularly those used with [`Json`])
/// can derive this trait with `#[derive(neon::TypeScript)]`.
///
/// # Manual implementation
///
/// For types from third-party crates (where you can't add `#[derive]`),
/// implement this trait directly:
///
/// ```ignore
/// use std::borrow::Cow;
///
/// // Simple type that maps to a TypeScript primitive
/// impl neon::typescript::TypeScript for my_crate::UserId {
///     fn ts_type() -> Cow<'static, str> {
///         "string".into()
///     }
/// }
///
/// // Complex type that needs a top-level declaration
/// impl neon::typescript::TypeScript for my_crate::GeoPoint {
///     fn ts_type() -> Cow<'static, str> {
///         "GeoPoint".into()
///     }
///
///     fn ts_decl() -> Option<Cow<'static, str>> {
///         Some("interface GeoPoint {\n  lat: number;\n  lng: number;\n}".into())
///     }
/// }
/// ```
pub trait TypeScript {
    /// The TypeScript type expression for this type.
    ///
    /// For example, `f64` returns `"number"`, `Vec<String>` returns `"string[]"`.
    fn ts_type() -> Cow<'static, str>;

    /// The structured TypeScript AST node for this type.
    ///
    /// The default implementation parses the string output of [`ts_type`](Self::ts_type)
    /// into an AST node. Types that want stable structured output (independent of
    /// parser evolution) should override this directly. All built-in impls in
    /// Neon override this for native structured output.
    fn ts_type_ast() -> TsType {
        parser::parse(&Self::ts_type())
    }

    /// An optional top-level TypeScript declaration needed to support this type.
    ///
    /// Returns `None` for types that need no declaration (e.g., primitives).
    /// Returns `Some(decl)` for types that need a top-level declaration such as
    /// an interface or type alias.
    fn ts_decl() -> Option<Cow<'static, str>> {
        None
    }

    /// Collect this type's declaration and all transitive type declarations
    /// into `decls`, keyed by type name for deduplication.
    ///
    /// The default implementation adds this type's own declaration if present.
    /// Types with fields or variants should override this to also collect
    /// declarations from their child types.
    fn ts_collect(decls: &mut BTreeMap<String, String>) {
        if let Some(decl) = Self::ts_decl() {
            let name = Self::ts_type().into_owned();
            decls.entry(name).or_insert_with(|| decl.into_owned());
        }
    }
}

// ——— Primitive impls ———

impl TypeScript for f64 {
    fn ts_type() -> Cow<'static, str> {
        "number".into()
    }
}

impl TypeScript for f32 {
    fn ts_type() -> Cow<'static, str> {
        "number".into()
    }
}

impl TypeScript for i64 {
    fn ts_type() -> Cow<'static, str> {
        "number".into()
    }
}

impl TypeScript for i32 {
    fn ts_type() -> Cow<'static, str> {
        "number".into()
    }
}

impl TypeScript for i16 {
    fn ts_type() -> Cow<'static, str> {
        "number".into()
    }
}

impl TypeScript for i8 {
    fn ts_type() -> Cow<'static, str> {
        "number".into()
    }
}

impl TypeScript for u64 {
    fn ts_type() -> Cow<'static, str> {
        "number".into()
    }
}

impl TypeScript for u32 {
    fn ts_type() -> Cow<'static, str> {
        "number".into()
    }
}

impl TypeScript for u16 {
    fn ts_type() -> Cow<'static, str> {
        "number".into()
    }
}

impl TypeScript for u8 {
    fn ts_type() -> Cow<'static, str> {
        "number".into()
    }
}

impl TypeScript for usize {
    fn ts_type() -> Cow<'static, str> {
        "number".into()
    }
}

impl TypeScript for isize {
    fn ts_type() -> Cow<'static, str> {
        "number".into()
    }
}

impl TypeScript for String {
    fn ts_type() -> Cow<'static, str> {
        "string".into()
    }
}

impl TypeScript for &str {
    fn ts_type() -> Cow<'static, str> {
        "string".into()
    }
}

impl TypeScript for bool {
    fn ts_type() -> Cow<'static, str> {
        "boolean".into()
    }
}

impl TypeScript for () {
    fn ts_type() -> Cow<'static, str> {
        "undefined".into()
    }
}

// ——— Wrapper impls ———

impl<T: TypeScript> TypeScript for Option<T> {
    fn ts_type() -> Cow<'static, str> {
        let inner = T::ts_type();
        format!("{inner} | undefined | null").into()
    }

    fn ts_collect(decls: &mut BTreeMap<String, String>) {
        T::ts_collect(decls);
    }
}

impl<T: TypeScript> TypeScript for Vec<T> {
    fn ts_type() -> Cow<'static, str> {
        let inner = T::ts_type();
        // Wrap union types in parens: (Foo | Bar)[] not Foo | Bar[]
        if inner.contains(" | ") {
            format!("({inner})[]").into()
        } else {
            format!("{inner}[]").into()
        }
    }

    fn ts_collect(decls: &mut BTreeMap<String, String>) {
        T::ts_collect(decls);
    }
}

impl<T: TypeScript + Eq + Hash> TypeScript for HashSet<T> {
    fn ts_type() -> Cow<'static, str> {
        Vec::<T>::ts_type()
    }

    fn ts_collect(decls: &mut BTreeMap<String, String>) {
        T::ts_collect(decls);
    }
}

impl<T: TypeScript + Ord> TypeScript for BTreeSet<T> {
    fn ts_type() -> Cow<'static, str> {
        Vec::<T>::ts_type()
    }

    fn ts_collect(decls: &mut BTreeMap<String, String>) {
        T::ts_collect(decls);
    }
}

// Maps serialize to JSON objects whose keys are *always* strings, regardless of
// the Rust key type `K` (serde stringifies keys, and JSON has no other kind of
// object key). So the observable TypeScript shape is always `Record<string, V>`.
// We deliberately ignore `K` here: describing a `HashMap<u32, V>` as
// `Record<number, V>` would be misleading (the runtime keys are strings), and
// non-primitive `K` would produce invalid TypeScript (`Record` keys must be
// `string | number | symbol`).
impl<K: TypeScript, V: TypeScript> TypeScript for HashMap<K, V> {
    fn ts_type() -> Cow<'static, str> {
        let v = V::ts_type();
        format!("Record<string, {v}>").into()
    }

    fn ts_collect(decls: &mut BTreeMap<String, String>) {
        V::ts_collect(decls);
    }
}

impl<K: TypeScript + Ord, V: TypeScript> TypeScript for BTreeMap<K, V> {
    fn ts_type() -> Cow<'static, str> {
        let v = V::ts_type();
        format!("Record<string, {v}>").into()
    }

    fn ts_collect(decls: &mut BTreeMap<String, String>) {
        V::ts_collect(decls);
    }
}

impl<T: TypeScript, E> TypeScript for Result<T, E> {
    fn ts_type() -> Cow<'static, str> {
        T::ts_type()
    }

    fn ts_collect(decls: &mut BTreeMap<String, String>) {
        T::ts_collect(decls);
    }
}

impl<T: TypeScript> TypeScript for Box<T> {
    fn ts_type() -> Cow<'static, str> {
        T::ts_type()
    }

    fn ts_collect(decls: &mut BTreeMap<String, String>) {
        T::ts_collect(decls);
    }
}

// ——— References ———

impl<'a, T: TypeScript> TypeScript for &'a T {
    fn ts_type() -> Cow<'static, str> {
        T::ts_type()
    }

    fn ts_collect(decls: &mut BTreeMap<String, String>) {
        T::ts_collect(decls);
    }
}

impl<'a, T: TypeScript> TypeScript for &'a mut T {
    fn ts_type() -> Cow<'static, str> {
        T::ts_type()
    }

    fn ts_collect(decls: &mut BTreeMap<String, String>) {
        T::ts_collect(decls);
    }
}

// ——— Handle and Root ———
//
// Raw Neon JS value types. These map to TypeScript's built-in types.

// Specific JS value types map to their TS equivalents. Handles to less-common
// JS types (or user-defined `Value` impls) fall back to "any" via the macro
// probe — the previous blanket `Handle<V: Value> -> "any"` impl is intentionally
// gone so that more specific impls can override.

impl<'cx> TypeScript for crate::handle::Handle<'cx, crate::types::JsValue> {
    fn ts_type() -> Cow<'static, str> {
        "any".into()
    }
}

#[cfg(feature = "napi-6")]
impl<'cx> TypeScript for crate::handle::Handle<'cx, crate::types::JsBigInt> {
    fn ts_type() -> Cow<'static, str> {
        "bigint".into()
    }
}

impl<'cx> TypeScript for crate::handle::Handle<'cx, crate::types::JsString> {
    fn ts_type() -> Cow<'static, str> {
        "string".into()
    }
}

impl<'cx> TypeScript for crate::handle::Handle<'cx, crate::types::JsNumber> {
    fn ts_type() -> Cow<'static, str> {
        "number".into()
    }
}

impl<'cx> TypeScript for crate::handle::Handle<'cx, crate::types::JsBoolean> {
    fn ts_type() -> Cow<'static, str> {
        "boolean".into()
    }
}

impl<'cx> TypeScript for crate::handle::Handle<'cx, crate::types::JsNull> {
    fn ts_type() -> Cow<'static, str> {
        "null".into()
    }
}

impl<'cx> TypeScript for crate::handle::Handle<'cx, crate::types::JsUndefined> {
    fn ts_type() -> Cow<'static, str> {
        "undefined".into()
    }
}

impl<'cx> TypeScript for crate::handle::Handle<'cx, crate::types::JsArray> {
    fn ts_type() -> Cow<'static, str> {
        "any[]".into()
    }
}

impl<'cx> TypeScript for crate::handle::Handle<'cx, crate::types::JsObject> {
    fn ts_type() -> Cow<'static, str> {
        "object".into()
    }
}

impl<'cx> TypeScript for crate::handle::Handle<'cx, crate::types::JsFunction> {
    fn ts_type() -> Cow<'static, str> {
        "Function".into()
    }
}

#[cfg(feature = "napi-5")]
impl<'cx> TypeScript for crate::handle::Handle<'cx, crate::types::JsDate> {
    fn ts_type() -> Cow<'static, str> {
        "Date".into()
    }
}

impl<'cx> TypeScript for crate::handle::Handle<'cx, crate::types::JsPromise> {
    fn ts_type() -> Cow<'static, str> {
        "Promise<any>".into()
    }
}

impl<'cx> TypeScript for crate::handle::Handle<'cx, crate::types::JsArrayBuffer> {
    fn ts_type() -> Cow<'static, str> {
        "ArrayBuffer".into()
    }
}

impl<'cx> TypeScript for crate::handle::Handle<'cx, crate::types::JsBuffer> {
    fn ts_type() -> Cow<'static, str> {
        "Buffer".into()
    }
}

impl<'cx> TypeScript for crate::handle::Handle<'cx, crate::types::JsError> {
    fn ts_type() -> Cow<'static, str> {
        "Error".into()
    }
}

impl<O: crate::object::Object> TypeScript for crate::handle::Root<O> {
    fn ts_type() -> Cow<'static, str> {
        "any".into()
    }
}

// ——— Tuples ———

impl<A: TypeScript, B: TypeScript> TypeScript for (A, B) {
    fn ts_type() -> Cow<'static, str> {
        let a = A::ts_type();
        let b = B::ts_type();
        format!("[{a}, {b}]").into()
    }

    fn ts_collect(decls: &mut BTreeMap<String, String>) {
        A::ts_collect(decls);
        B::ts_collect(decls);
    }
}

impl<A: TypeScript, B: TypeScript, C: TypeScript> TypeScript for (A, B, C) {
    fn ts_type() -> Cow<'static, str> {
        let a = A::ts_type();
        let b = B::ts_type();
        let c = C::ts_type();
        format!("[{a}, {b}, {c}]").into()
    }

    fn ts_collect(decls: &mut BTreeMap<String, String>) {
        A::ts_collect(decls);
        B::ts_collect(decls);
        C::ts_collect(decls);
    }
}

// ——— Extractors ———

impl<T: TypeScript> TypeScript for crate::types::extract::Array<T> {
    fn ts_type() -> Cow<'static, str> {
        T::ts_type()
    }

    fn ts_collect(decls: &mut BTreeMap<String, String>) {
        T::ts_collect(decls);
    }
}

impl<T> TypeScript for crate::types::extract::Uint8Array<T> {
    fn ts_type() -> Cow<'static, str> {
        "Uint8Array".into()
    }
}

impl<T> TypeScript for crate::types::extract::ArrayBuffer<T> {
    fn ts_type() -> Cow<'static, str> {
        "ArrayBuffer".into()
    }
}

// ——— Either ———

impl<A: TypeScript, B: TypeScript> TypeScript for either::Either<A, B> {
    fn ts_type() -> Cow<'static, str> {
        let a = A::ts_type();
        let b = B::ts_type();
        format!("{a} | {b}").into()
    }

    fn ts_collect(decls: &mut BTreeMap<String, String>) {
        A::ts_collect(decls);
        B::ts_collect(decls);
    }
}

// ——— serde_json::Value ———

#[cfg(feature = "serde")]
impl TypeScript for serde_json::Value {
    fn ts_type() -> Cow<'static, str> {
        "any".into()
    }
}

// ——— Chrono date/time types ———

#[cfg(feature = "typescript-chrono")]
impl<Tz: chrono::TimeZone> TypeScript for chrono::DateTime<Tz> {
    fn ts_type() -> Cow<'static, str> {
        "string".into()
    }
}

#[cfg(feature = "typescript-chrono")]
impl TypeScript for chrono::NaiveDate {
    fn ts_type() -> Cow<'static, str> {
        "string".into()
    }
}

#[cfg(feature = "typescript-chrono")]
impl TypeScript for chrono::NaiveDateTime {
    fn ts_type() -> Cow<'static, str> {
        "string".into()
    }
}

#[cfg(feature = "typescript-chrono")]
impl TypeScript for chrono::NaiveTime {
    fn ts_type() -> Cow<'static, str> {
        "string".into()
    }
}

// ——— UUID ———

#[cfg(feature = "typescript-uuid")]
impl TypeScript for uuid::Uuid {
    fn ts_type() -> Cow<'static, str> {
        "string".into()
    }
}

// ——— OrderMap ———

// As with `HashMap`/`BTreeMap`, JSON object keys are always strings, so the
// observable shape is `Record<string, V>` regardless of `K`.
#[cfg(feature = "typescript-ordermap")]
impl<K: TypeScript, V: TypeScript> TypeScript for ordermap::OrderMap<K, V> {
    fn ts_type() -> Cow<'static, str> {
        let v = V::ts_type();
        format!("Record<string, {v}>").into()
    }

    fn ts_collect(decls: &mut BTreeMap<String, String>) {
        V::ts_collect(decls);
    }
}

// ——— Serde JSON wrapper ———

#[cfg(feature = "serde")]
impl<T: TypeScript> TypeScript for Json<T> {
    fn ts_type() -> Cow<'static, str> {
        T::ts_type()
    }

    fn ts_collect(decls: &mut BTreeMap<String, String>) {
        T::ts_collect(decls);
    }
}

// ——— Opaque boxed types ———
//
// Smart pointer wrappers that become opaque boxed values in JavaScript.
// Each generates a branded interface to prevent accidental interchange.
//
// `Boxed<T>` (and `Arc`/`Rc`/`RefCell`/...) are intended for *named* types
// (a database handle, an index, etc.) whose `ts_type()` is a plain identifier.
// The boxed interface name is synthesized as `Boxed{inner}`. If `inner` is a
// composed type expression (e.g. `string | undefined | null` from
// `Arc<Option<String>>`), the raw string would not be a valid identifier, so we
// sanitize it. The brand value retains the original `ts_type()` string, which is
// what actually distinguishes one boxed type from another.

/// Reduce an arbitrary TypeScript type expression to an identifier-safe string
/// by keeping ASCII alphanumerics and `_`, dropping everything else. Used to
/// synthesize the `Boxed{...}` interface name.
fn sanitize_identifier(s: &str) -> String {
    let mut out: String = s
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_')
        .collect();
    // An identifier can't start with a digit; prefix an underscore if needed.
    if out.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        out.insert(0, '_');
    }
    out
}

fn boxed_ts_type<T: TypeScript>() -> Cow<'static, str> {
    let inner = T::ts_type();
    format!("Boxed{}", sanitize_identifier(&inner)).into()
}

fn boxed_ts_decl<T: TypeScript>() -> Option<Cow<'static, str>> {
    let inner = T::ts_type();
    let boxed = format!("Boxed{}", sanitize_identifier(&inner));
    Some(format!("interface {boxed} {{ readonly [__neon_tag]: '{inner}' }}").into())
}

fn boxed_ts_collect<T: TypeScript>(decls: &mut BTreeMap<String, String>) {
    if let Some(d) = boxed_ts_decl::<T>() {
        decls
            .entry(boxed_ts_type::<T>().into_owned())
            .or_insert_with(|| d.into_owned());
    }
}

impl<T: TypeScript + 'static> TypeScript for Arc<T> {
    fn ts_type() -> Cow<'static, str> {
        boxed_ts_type::<T>()
    }

    fn ts_decl() -> Option<Cow<'static, str>> {
        boxed_ts_decl::<T>()
    }

    fn ts_collect(decls: &mut BTreeMap<String, String>) {
        boxed_ts_collect::<T>(decls);
    }
}

impl<T: TypeScript + 'static> TypeScript for Rc<T> {
    fn ts_type() -> Cow<'static, str> {
        boxed_ts_type::<T>()
    }

    fn ts_decl() -> Option<Cow<'static, str>> {
        boxed_ts_decl::<T>()
    }

    fn ts_collect(decls: &mut BTreeMap<String, String>) {
        boxed_ts_collect::<T>(decls);
    }
}

impl<T: TypeScript + 'static> TypeScript for RefCell<T> {
    fn ts_type() -> Cow<'static, str> {
        boxed_ts_type::<T>()
    }

    fn ts_decl() -> Option<Cow<'static, str>> {
        boxed_ts_decl::<T>()
    }

    fn ts_collect(decls: &mut BTreeMap<String, String>) {
        boxed_ts_collect::<T>(decls);
    }
}

impl<'a, T: TypeScript + 'static> TypeScript for Ref<'a, T> {
    fn ts_type() -> Cow<'static, str> {
        boxed_ts_type::<T>()
    }

    fn ts_decl() -> Option<Cow<'static, str>> {
        boxed_ts_decl::<T>()
    }

    fn ts_collect(decls: &mut BTreeMap<String, String>) {
        boxed_ts_collect::<T>(decls);
    }
}

impl<'a, T: TypeScript + 'static> TypeScript for RefMut<'a, T> {
    fn ts_type() -> Cow<'static, str> {
        boxed_ts_type::<T>()
    }

    fn ts_decl() -> Option<Cow<'static, str>> {
        boxed_ts_decl::<T>()
    }

    fn ts_collect(decls: &mut BTreeMap<String, String>) {
        boxed_ts_collect::<T>(decls);
    }
}

impl<T: TypeScript> TypeScript for Boxed<T> {
    fn ts_type() -> Cow<'static, str> {
        boxed_ts_type::<T>()
    }

    fn ts_decl() -> Option<Cow<'static, str>> {
        boxed_ts_decl::<T>()
    }

    fn ts_collect(decls: &mut BTreeMap<String, String>) {
        boxed_ts_collect::<T>(decls);
    }
}

// ——— Metadata types ———
//
// These are used by the `#[neon::export]` macro to collect type information
// at compile time and resolve it at runtime.

/// Metadata for a single function parameter.
pub struct ParamMeta {
    /// The parameter name (used in the TypeScript declaration).
    pub name: &'static str,
    /// Returns the TypeScript type expression for this parameter.
    pub ts_type: fn() -> Cow<'static, str>,
    /// Returns the structured AST node for this parameter's type.
    pub ts_type_ast: fn() -> TsType,
    /// Collects type declarations needed by this parameter.
    pub ts_collect: fn(&mut BTreeMap<String, String>),
}

/// Metadata for an exported function.
pub struct FunctionMeta {
    /// The JavaScript export name.
    pub name: &'static str,
    /// Parameter metadata.
    pub params: &'static [ParamMeta],
    /// Returns the TypeScript type expression for the return type.
    pub ret_type: fn() -> Cow<'static, str>,
    /// Returns the structured AST node for the return type.
    pub ret_type_ast: fn() -> TsType,
    /// Collects type declarations needed by the return type.
    pub ret_collect: fn(&mut BTreeMap<String, String>),
    /// Whether the function returns a Promise.
    pub is_async: bool,
}

/// Metadata for a class method.
pub struct MethodMeta {
    /// The JavaScript method name.
    pub name: &'static str,
    /// Parameter metadata.
    pub params: &'static [ParamMeta],
    /// Returns the TypeScript type expression for the return type.
    pub ret_type: fn() -> Cow<'static, str>,
    /// Returns the structured AST node for the return type.
    pub ret_type_ast: fn() -> TsType,
    /// Collects type declarations needed by the return type.
    pub ret_collect: fn(&mut BTreeMap<String, String>),
    /// Whether the method returns a Promise.
    pub is_async: bool,
}

/// Metadata for a class constructor.
pub struct ConstructorMeta {
    /// Constructor parameter metadata.
    pub params: &'static [ParamMeta],
}

/// Metadata for a static class property.
pub struct PropertyMeta {
    /// The JavaScript property name.
    pub name: &'static str,
    /// Returns the TypeScript type expression for this property.
    pub ts_type: fn() -> Cow<'static, str>,
    /// Returns the structured AST node for this property's type.
    pub ts_type_ast: fn() -> TsType,
    /// Collects type declarations needed by this property.
    pub ts_collect: fn(&mut BTreeMap<String, String>),
}

/// Metadata for an exported class.
pub struct ClassMeta {
    /// The JavaScript class name.
    pub name: &'static str,
    /// Constructor metadata (None if no public constructor).
    pub constructor: Option<ConstructorMeta>,
    /// Instance methods.
    pub methods: &'static [MethodMeta],
    /// Static readonly properties.
    pub static_properties: &'static [PropertyMeta],
}

/// Metadata for an exported item (function, class, or global).
pub enum ExportMeta {
    /// A function export.
    Function(FunctionMeta),
    /// A class export.
    Class(ClassMeta),
}

// ——— Generation ———

/// Options for [`generate_with`] and [`generate_ast_with`].
///
/// Defaults to flat output. Set `module` to wrap all declarations in
/// `declare module "X" { ... }`.
#[derive(Default, Clone, Debug)]
pub struct GenerateOptions {
    /// If `Some(name)`, wrap all declarations in `declare module "<name>" { ... }`.
    ///
    /// Useful when the addon is loaded indirectly (e.g. via a `load.cjs` shim) and
    /// types should be attached to that module's import path rather than emitted
    /// as top-level exports.
    pub module: Option<String>,
}

/// Generate TypeScript declarations for all `#[neon::export]`-ed items.
///
/// Returns a complete `.d.ts` file as a string. The output is CommonJS-style
/// type declarations suitable for placing beside a `.node` binary module.
///
/// For control over the rendered output (e.g. wrapping in `declare module
/// "X" { ... }`), see [`generate_with`].
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

/// Generate TypeScript declarations as a string, with rendering options.
///
/// When `options.module` is set, the output is wrapped in
/// `declare module "<name>" { ... }` with the body indented.
///
/// # Example
///
/// ```ignore
/// use neon::typescript::{generate_with, GenerateOptions};
///
/// let dts = generate_with(GenerateOptions {
///     module: Some("./load.cjs".into()),
/// });
/// std::fs::write("index.d.ts", dts)?;
/// ```
pub fn generate_with(options: GenerateOptions) -> String {
    let body = generate();
    match options.module {
        None => body,
        Some(name) => wrap_in_module(&body, &name),
    }
}

fn wrap_in_module(body: &str, name: &str) -> String {
    // `body` starts with "// Auto-generated by Neon. Do not edit.\n" — split
    // off the header so it stays at the top of the file.
    let (header, rest) = match body.split_once('\n') {
        Some((h, r)) => (h, r),
        None => ("", body),
    };

    let mut out = String::with_capacity(body.len() + name.len() + 64);
    if !header.is_empty() {
        out.push_str(header);
        out.push('\n');
    }
    out.push('\n');
    out.push_str(&format!("declare module \"{name}\" {{\n"));

    // Indent each non-empty line by two spaces.
    for line in rest.lines() {
        if line.is_empty() {
            out.push('\n');
        } else {
            out.push_str("  ");
            out.push_str(line);
            out.push('\n');
        }
    }

    out.push_str("}\n");
    out
}

/// Map a unit return type (`undefined` or `void`) to `void`, leaving all other
/// types unchanged. Applied before Promise-wrapping so async unit returns render
/// as `Promise<void>` rather than `Promise<undefined>`.
fn normalize_unit_return(ty: TsType) -> TsType {
    match ty {
        TsType::TSUndefinedKeyword | TsType::TSVoidKeyword => TsType::TSVoidKeyword,
        other => other,
    }
}

/// Generate TypeScript declarations as a structured AST.
///
/// Returns a vector of top-level declarations (functions, classes, interfaces,
/// type aliases, ambient consts) suitable for serializing to JSON or applying
/// programmatic transformations. The AST node shapes mirror
/// [TSESTree](https://typescript-eslint.io/packages/typescript-estree/),
/// so output is compatible with TypeScript-ESLint tools.
///
/// For a complete `.d.ts` *string*, use [`generate`] instead.
///
/// # Example
///
/// ```ignore
/// let ast = neon::typescript::generate_ast();
/// let json = serde_json::to_string_pretty(&ast)?;
/// std::fs::write("types.json", json)?;
/// ```
pub fn generate_ast() -> Vec<Decl> {
    use ast::*;
    let mut decls: BTreeMap<String, String> = BTreeMap::new();
    let mut functions: Vec<Decl> = Vec::new();
    let mut classes: Vec<Decl> = Vec::new();

    for meta in crate::macro_internal::TYPE_METADATA.iter() {
        match meta {
            ExportMeta::Function(func) => {
                for param in func.params.iter() {
                    (param.ts_collect)(&mut decls);
                }
                (func.ret_collect)(&mut decls);

                let params: Vec<Param> = func
                    .params
                    .iter()
                    .map(|p| Param {
                        name: p.name.into(),
                        type_annotation: TSTypeAnnotation {
                            type_annotation: (p.ts_type_ast)(),
                        },
                        optional: false,
                    })
                    .collect();

                // Normalize unit (`undefined`/`void`) to `void` first, then wrap
                // in Promise for async, so async unit returns become
                // `Promise<void>`, not `Promise<undefined>`.
                let ret_inner = normalize_unit_return((func.ret_type_ast)());
                let return_type = if func.is_async {
                    Some(TSTypeAnnotation {
                        type_annotation: TsType::reference_with("Promise", vec![ret_inner]),
                    })
                } else {
                    Some(TSTypeAnnotation {
                        type_annotation: ret_inner,
                    })
                };

                functions.push(Decl::TSDeclareFunction(TSDeclareFunction {
                    id: Identifier::new(func.name),
                    params,
                    return_type,
                }));
            }
            ExportMeta::Class(class) => {
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

                let mut body: Vec<ClassMember> = Vec::new();

                if let Some(ctor) = &class.constructor {
                    let params: Vec<Param> = ctor
                        .params
                        .iter()
                        .map(|p| Param {
                            name: p.name.into(),
                            type_annotation: TSTypeAnnotation {
                                type_annotation: (p.ts_type_ast)(),
                            },
                            optional: false,
                        })
                        .collect();
                    body.push(ClassMember::MethodDefinition(MethodDefinition {
                        key: Identifier::new("constructor"),
                        kind: MethodKind::Constructor,
                        value: FunctionExpression {
                            params,
                            return_type: None,
                        },
                        is_static: false,
                    }));
                }

                for prop in class.static_properties.iter() {
                    body.push(ClassMember::PropertyDefinition(PropertyDefinition {
                        key: Identifier::new(prop.name),
                        type_annotation: Some(TSTypeAnnotation {
                            type_annotation: (prop.ts_type_ast)(),
                        }),
                        is_static: true,
                        readonly: true,
                    }));
                }

                for method in class.methods.iter() {
                    let params: Vec<Param> = method
                        .params
                        .iter()
                        .map(|p| Param {
                            name: p.name.into(),
                            type_annotation: TSTypeAnnotation {
                                type_annotation: (p.ts_type_ast)(),
                            },
                            optional: false,
                        })
                        .collect();
                    let ret_inner = normalize_unit_return((method.ret_type_ast)());
                    let return_type = if method.is_async {
                        TSTypeAnnotation {
                            type_annotation: TsType::reference_with("Promise", vec![ret_inner]),
                        }
                    } else {
                        TSTypeAnnotation {
                            type_annotation: ret_inner,
                        }
                    };
                    body.push(ClassMember::MethodDefinition(MethodDefinition {
                        key: Identifier::new(method.name),
                        kind: MethodKind::Method,
                        value: FunctionExpression {
                            params,
                            return_type: Some(return_type),
                        },
                        is_static: false,
                    }));
                }

                classes.push(Decl::ClassDeclaration(ClassDeclaration {
                    id: Identifier::new(class.name),
                    declare: true,
                    body: ClassBody { body },
                }));
            }
        }
    }

    let mut out: Vec<Decl> = Vec::new();

    // Branding symbol for opaque types
    let has_opaque = decls.keys().any(|k| k.starts_with("Boxed"));
    if has_opaque {
        out.push(Decl::VariableDeclaration(ast::VariableDeclaration {
            kind: ast::VariableKind::Const,
            declarations: vec![ast::VariableDeclarator {
                id: Identifier::new("__neon_tag"),
                type_annotation: Some(TSTypeAnnotation {
                    type_annotation: TsType::TSSymbolKeyword,
                }),
                unique_symbol: true,
            }],
            declare: true,
        }));
    }

    // Collected type declarations (interfaces, type aliases). Currently
    // emitted as `Raw` decls (the existing trait API produces strings);
    // a future change can promote these to structured `TSInterfaceDeclaration`
    // / `TSTypeAliasDeclaration` variants by adding a structured `ts_decl_ast`
    // method to the `TypeScript` trait.
    for decl_str in decls.values() {
        out.push(Decl::Raw {
            value: decl_str.clone(),
        });
    }

    out.extend(classes);
    out.extend(functions);
    out
}

/// Generate TypeScript declarations as a structured AST, with rendering options.
///
/// When `options.module` is set, all declarations are wrapped in a single
/// [`TSModuleDeclaration`](ast::TSModuleDeclaration) node.
///
/// # Example
///
/// ```ignore
/// use neon::typescript::{generate_ast_with, GenerateOptions};
///
/// let ast = generate_ast_with(GenerateOptions {
///     module: Some("./load.cjs".into()),
/// });
/// ```
pub fn generate_ast_with(options: GenerateOptions) -> Vec<Decl> {
    let body = generate_ast();
    match options.module {
        None => body,
        Some(name) => vec![Decl::TSModuleDeclaration(ast::TSModuleDeclaration {
            id: ast::StringLiteral { value: name },
            body: ast::TSModuleBlock { body },
            declare: true,
        })],
    }
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
    use crate::object::Object;
    use crate::types::{JsFunction, JsValue};

    let symbol_fn: crate::handle::Handle<JsFunction> = cx.global("Symbol")?;
    let symbol_for: crate::handle::Handle<JsFunction> = symbol_fn.get(cx, "for")?;

    let exports = cx.exports_object()?;

    // Attach .d.ts string under Symbol.for("neon:types")
    let types_key = cx.string("neon:types");
    let types_key_arg = crate::handle::Handle::upcast::<JsValue>(&types_key);
    let types_symbol: crate::handle::Handle<JsValue> = symbol_for.call(
        cx,
        crate::handle::Handle::upcast::<JsValue>(&symbol_fn),
        [types_key_arg],
    )?;
    let dts_str = cx.string(generate());
    exports.set(cx, types_symbol, dts_str)?;

    // Attach AST JSON under Symbol.for("neon:types-ast") (requires serde)
    #[cfg(feature = "serde")]
    {
        let ast_key = cx.string("neon:types-ast");
        let ast_key_arg = crate::handle::Handle::upcast::<JsValue>(&ast_key);
        let ast_symbol: crate::handle::Handle<JsValue> = symbol_for.call(
            cx,
            crate::handle::Handle::upcast::<JsValue>(&symbol_fn),
            [ast_key_arg],
        )?;
        let ast_json = serde_json::to_string(&generate_ast()).unwrap_or_else(|_| "[]".to_string());
        let ast_str = cx.string(ast_json);
        exports.set(cx, ast_symbol, ast_str)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_identifier_keeps_valid_names() {
        assert_eq!(sanitize_identifier("Database"), "Database");
        assert_eq!(sanitize_identifier("Foo_Bar2"), "Foo_Bar2");
    }

    #[test]
    fn sanitize_identifier_strips_invalid_chars() {
        // Composed type expressions produce non-identifier strings; the boxed
        // name must still be a valid identifier.
        assert_eq!(
            sanitize_identifier("string | undefined | null"),
            "stringundefinednull"
        );
        assert_eq!(sanitize_identifier("number[]"), "number");
        assert_eq!(
            sanitize_identifier("Record<string, number>"),
            "Recordstringnumber"
        );
    }

    #[test]
    fn sanitize_identifier_prefixes_leading_digit() {
        assert_eq!(sanitize_identifier("3d"), "_3d");
    }

    #[test]
    fn boxed_name_for_composed_type_is_valid_identifier() {
        // Arc<Option<String>> would otherwise yield "Boxedstring | undefined | null".
        let name = <Arc<Option<String>> as TypeScript>::ts_type();
        assert_eq!(name, "Boxedstringundefinednull");
        // The brand value retains the real (unsanitized) type expression.
        let decl = <Arc<Option<String>> as TypeScript>::ts_decl().unwrap();
        assert!(
            decl.contains("__neon_tag]: 'string | undefined | null'"),
            "brand should retain original type expression: {decl}"
        );
    }

    #[test]
    fn wrap_in_module_indents_body_and_preserves_header() {
        let body = "// Auto-generated by Neon. Do not edit.\n\
                    \n\
                    export declare function foo(): number;\n";
        let wrapped = wrap_in_module(body, "./load.cjs");
        assert!(
            wrapped.starts_with("// Auto-generated by Neon. Do not edit.\n"),
            "header missing: {wrapped}"
        );
        assert!(
            wrapped.contains("declare module \"./load.cjs\" {\n"),
            "module header missing: {wrapped}"
        );
        // Body line should be indented by two spaces.
        assert!(
            wrapped.contains("\n  export declare function foo(): number;\n"),
            "body not indented: {wrapped}"
        );
        assert!(
            wrapped.trim_end().ends_with('}'),
            "no closing brace: {wrapped}"
        );
    }

    #[test]
    fn wrap_in_module_handles_body_without_newline() {
        // Edge case: body is a single line with no trailing newline. split_once
        // returns None and we should still emit a syntactically valid wrapper.
        let wrapped = wrap_in_module("just one line", "foo");
        // No header section to extract — the whole body becomes module content.
        assert!(wrapped.contains("declare module \"foo\" {\n"));
        assert!(wrapped.contains("  just one line\n"));
        assert!(wrapped.trim_end().ends_with('}'));
    }

    #[test]
    fn wrap_in_module_handles_empty_body_lines() {
        // Body with blank lines between decls — blank lines should pass through
        // un-indented (preserving readability).
        let body = "// header\n\nexport interface A {}\n\nexport interface B {}\n";
        let wrapped = wrap_in_module(body, "m");
        // Blank lines stay blank (no spurious indentation).
        assert!(wrapped.contains("\n\n  export interface A"));
        assert!(wrapped.contains("\n\n  export interface B"));
    }
}
