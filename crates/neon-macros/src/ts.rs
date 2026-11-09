//! Shared TypeScript metadata codegen for the export and class macros.
//!
//! Each argument or return type is described by the conversion that actually
//! runs at that position:
//!
//! - **JSON** (`#[neon::export(json)]`, or a top-level `Json<T>`): the payload's
//!   serde_json type, via `TsJson` (and an adapter's `TsJsonExt`, if imported).
//! - **Bare argument**: the values `TryFromJs` accepts, via `TsFromJs`.
//! - **Bare return**: the value `TryIntoJs` produces, via `TsIntoJs`.
//!
//! Each lookup is an autoref-specialization probe that falls back to `unknown`,
//! so a type without a known TypeScript type never causes a compile error.

use proc_macro2::TokenStream;

use crate::name::{substitute_lifetimes_with_static, type_needs_fallback};

/// Which side of the boundary a type sits on.
#[derive(Clone, Copy)]
pub(crate) enum Position {
    /// A function or method argument (converted with `TryFromJs`).
    Param,
    /// A return value or static property (converted with `TryIntoJs`).
    Return,
}

/// `(ts_type, ts_collect)` closure tokens for a metadata entry.
pub(crate) type TsTokens = (TokenStream, TokenStream);

/// Tokens for a literal TypeScript type (an override, or a fixed type).
pub(crate) fn literal(ts: &str) -> TsTokens {
    (
        quote::quote!(|| std::borrow::Cow::Borrowed(#ts)),
        quote::quote!(|_| {}),
    )
}

/// Tokens for a function or method with no declared return type.
///
/// The JSON path serializes the implicit `()` to `null`; the bare path
/// produces `undefined` (rendered as `void`).
pub(crate) fn unit_return(json: bool) -> TsTokens {
    literal(if json { "null" } else { "undefined" })
}

/// Tokens describing the type `ty` at a boundary position.
///
/// `json` is set when the item is marked `json`. For class members, `Self` must
/// already have been replaced with the class name.
pub(crate) fn boundary(ty: &syn::Type, json: bool, position: Position) -> TsTokens {
    // `impl Trait`, `Self`, and unsized slices can't name a probe type.
    if type_needs_fallback(ty) {
        return literal("unknown");
    }

    let ty = substitute_lifetimes_with_static(ty);

    // A returned `Result` is control flow, not data: the `Err` is thrown, so
    // only the `Ok` type is observable. Peel it here (rather than leaving it to
    // a trait impl) so the payload stays visible to the JSON adapter lookup.
    let ty = match position {
        Position::Return => peel_result(&ty).unwrap_or(ty),
        Position::Param => ty,
    };

    if json {
        return json_probe(&ty);
    }

    // Look through a top-level `Json<T>` so an adapter's `TsJsonExt` (which only
    // works at a concrete call site with the user's import in scope) can
    // describe a foreign payload.
    if let Some(payload) = outer_json(&ty) {
        return json_probe(&payload);
    }

    match position {
        // Reference arguments (`&Class`, `&RefCell<T>`) are extracted from the
        // referent's JS value, so describe the referent.
        Position::Param => from_js_probe(&peel_reference(&ty)),
        Position::Return => into_js_probe(&ty),
    }
}

/// The JSON probe: `TsJson`, then an adapter's `TsJsonExt`, then `unknown`.
fn json_probe(ty: &syn::Type) -> TsTokens {
    (
        quote::quote!(|| {
            use neon::macro_internal::TsJsonFallback as _;
            let __probe = neon::macro_internal::TsJsonProbe::<#ty>(std::marker::PhantomData);
            (&__probe).ts_json_of()
        }),
        quote::quote!(|decls| {
            use neon::macro_internal::TsJsonFallback as _;
            let __probe = neon::macro_internal::TsJsonProbe::<#ty>(std::marker::PhantomData);
            (&__probe).ts_json_collect_of(decls)
        }),
    )
}

/// The bare-argument probe: `TsFromJs`, then `unknown`.
fn from_js_probe(ty: &syn::Type) -> TsTokens {
    (
        quote::quote!(|| {
            use neon::macro_internal::TsFromJsFallback as _;
            let __probe = neon::macro_internal::TsFromJsProbe::<#ty>(std::marker::PhantomData);
            (&__probe).ts_from_js_of()
        }),
        quote::quote!(|decls| {
            use neon::macro_internal::TsFromJsFallback as _;
            let __probe = neon::macro_internal::TsFromJsProbe::<#ty>(std::marker::PhantomData);
            (&__probe).ts_from_js_collect_of(decls)
        }),
    )
}

/// The bare-return probe: `TsIntoJs`, then `unknown`.
fn into_js_probe(ty: &syn::Type) -> TsTokens {
    (
        quote::quote!(|| {
            use neon::macro_internal::TsIntoJsFallback as _;
            let __probe = neon::macro_internal::TsIntoJsProbe::<#ty>(std::marker::PhantomData);
            (&__probe).ts_into_js_of()
        }),
        quote::quote!(|decls| {
            use neon::macro_internal::TsIntoJsFallback as _;
            let __probe = neon::macro_internal::TsIntoJsProbe::<#ty>(std::marker::PhantomData);
            (&__probe).ts_into_js_collect_of(decls)
        }),
    )
}

/// The first generic type argument of a path type whose last segment is
/// `name`, e.g. `T` for `Json<T>` or `neon::types::extract::Json<T>`.
fn first_type_arg(ty: &syn::Type, names: &[&str]) -> Option<syn::Type> {
    let syn::Type::Path(type_path) = ty else {
        return None;
    };
    if type_path.qself.is_some() {
        return None;
    }
    let seg = type_path.path.segments.last()?;
    if !names.iter().any(|name| seg.ident == name) {
        return None;
    }
    let syn::PathArguments::AngleBracketed(args) = &seg.arguments else {
        return None;
    };
    args.args.iter().find_map(|arg| match arg {
        syn::GenericArgument::Type(inner) => Some(inner.clone()),
        _ => None,
    })
}

/// `T` for a top-level `Json<T>`.
fn outer_json(ty: &syn::Type) -> Option<syn::Type> {
    first_type_arg(ty, &["Json"])
}

/// The `Ok` type of a returned `Result<T, E>` (including one-parameter aliases
/// like `anyhow::Result<T>` and `NeonResult<T>`).
fn peel_result(ty: &syn::Type) -> Option<syn::Type> {
    first_type_arg(ty, &["Result", "NeonResult"])
}

/// The referent of a reference type, or the type itself.
fn peel_reference(ty: &syn::Type) -> syn::Type {
    match ty {
        syn::Type::Reference(r) => (*r.elem).clone(),
        other => other.clone(),
    }
}
