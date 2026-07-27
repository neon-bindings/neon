//! Implements `#[derive(neon::TypeScript)]`.
//!
//! Given a Rust struct or enum, this generates an `impl neon::typescript::TypeScript`
//! whose `ts_type()` / `ts_decl()` build the TypeScript text for that type. The
//! derive reads serde attributes (`rename`, `rename_all`, `tag`, `content`,
//! `untagged`, `flatten`, `skip`, `default`, `transparent`) so the emitted types
//! match what serde actually serializes.
//!
//! Example:
//!
//! ```ignore
//! #[derive(Serialize, Deserialize, neon::TypeScript)]
//! #[serde(rename_all = "camelCase")]
//! struct SearchResult { doc_id: u32, score: f64 }
//! ```
//!
//! generates an impl whose `ts_decl()` returns:
//!
//! ```text
//! interface SearchResult {
//!   docId: number;
//!   score: number;
//! }
//! ```

mod attrs;
mod enums;
mod rename;
mod structs;

/// Extract `#[neon(ts_type = "...")]` from attributes (e.g. on a function parameter).
///
/// Used by the export/class macros to let a caller override a single parameter's
/// TypeScript type. For example, given
///
/// ```ignore
/// #[neon::export]
/// fn f(#[neon(ts_type = "ReadonlyArray<number>")] xs: Vec<f64>) -> f64 { ... }
/// ```
///
/// this returns `Some("ReadonlyArray<number>")` for the `xs` parameter.
pub(crate) fn extract_param_ts_type(attrs: &[syn::Attribute]) -> Option<String> {
    let mut result = None;
    for attr in attrs {
        if !attr.path().is_ident("neon") {
            continue;
        }
        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("ts_type") {
                if let Ok(value) = meta.value().and_then(|v| v.parse::<syn::LitStr>()) {
                    result = Some(value.value());
                }
            }
            Ok(())
        });
    }
    result
}

pub(crate) fn derive_typescript(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    match derive_impl(&input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.into_compile_error().into(),
    }
}

fn derive_impl(input: &syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    // Container-level attributes apply to the whole type, e.g.
    // `#[serde(rename_all = "camelCase", tag = "kind")]`.
    let container = attrs::ContainerAttrs::parse(&input.attrs)?;

    match &input.data {
        // `struct S { ... }` → an `interface` (or a passthrough for newtype/transparent)
        syn::Data::Struct(data) => {
            structs::generate(&input.ident, &input.generics, &container, data)
        }
        // `enum E { ... }` → a `type` union, shaped by the serde tagging attributes
        syn::Data::Enum(data) => enums::generate(&input.ident, &input.generics, &container, data),
        // `union U { ... }` has no serde/JSON representation, so it can't be mapped
        syn::Data::Union(_) => Err(syn::Error::new_spanned(
            &input.ident,
            "TypeScript derive does not support unions",
        )),
    }
}
