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
use std::collections::BTreeMap;

#[cfg(feature = "serde")]
use crate::types::extract::Json;

use crate::types::extract::Boxed;

/// The [`TypeScript`] trait: Neon's minimal, stable contract for mapping a Rust
/// type to its TypeScript representation.
///
/// The trait itself and the built-in implementations for standard-library types
/// (primitives, `Option`, `Vec`, `HashMap`, tuples, `Result`, `Box`, refs, …)
/// live in the [`neon-typescript`](neon_typescript) crate. This module adds the
/// Neon-specific boundary implementations (`Handle`, `Root`, the extractors,
/// `Json`, boxed smart pointers, …).
pub use neon_typescript::TypeScript;

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
// `Boxed<T>` becomes an opaque boxed value in JavaScript, rendered as a branded
// interface to prevent accidental interchange. It reuses the branded-box
// convention from `neon-typescript` (shared with the `Arc`/`Rc`/`RefCell`/`Ref`/
// `RefMut` impls there), so `Boxed<Database>` and a bare `Arc<Database>` produce
// the same `BoxedDatabase` type.

impl<T: TypeScript> TypeScript for Boxed<T> {
    fn ts_type() -> Cow<'static, str> {
        neon_typescript::boxed_ts_type::<T>()
    }

    fn ts_decl() -> Option<Cow<'static, str>> {
        neon_typescript::boxed_ts_decl::<T>()
    }

    fn ts_collect(decls: &mut BTreeMap<String, String>) {
        neon_typescript::boxed_ts_collect::<T>(decls);
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

/// Options for [`generate_with`].
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

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boxed_name_for_composed_type_is_valid_identifier() {
        // Boxed<Option<String>> would otherwise yield "Boxedstring | null".
        let name = <Boxed<Option<String>> as TypeScript>::ts_type();
        assert_eq!(name, "Boxedstringnull");
        // The brand value retains the real (unsanitized) type expression.
        let decl = <Boxed<Option<String>> as TypeScript>::ts_decl().unwrap();
        assert!(
            decl.contains("__neon_tag]: 'string | null'"),
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
