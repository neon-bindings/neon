//! Generates `impl TypeScript` for structs. The output shape depends on the
//! struct's fields and attributes:
//!
//! ```ignore
//! struct Point { x: f64, y: f64 }        // => interface Point { x: number; y: number; }
//! struct Id(String);                     // newtype => string (the inner type)
//! #[serde(transparent)] struct W(Inner); // => Inner's type, passed through
//! #[neon(ts_type = "...")] struct X(..); // => the literal override
//! struct Env<T> { data: T }              // => interface Env<T> { data: T; }
//! ```

use super::attrs::{ContainerAttrs, FieldAttrs};

pub(crate) fn generate(
    name: &syn::Ident,
    generics: &syn::Generics,
    container: &ContainerAttrs,
    data: &syn::DataStruct,
) -> syn::Result<proc_macro2::TokenStream> {
    // `#[neon(ts_type = "...")]` on the struct replaces everything with the literal.
    if let Some(ts_override) = &container.ts_type {
        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
        return Ok(quote::quote! {
            impl #impl_generics neon::typescript::TypeScript for #name #ty_generics #where_clause {
                fn ts_type() -> std::borrow::Cow<'static, str> {
                    std::borrow::Cow::Borrowed(#ts_override)
                }
            }
        });
    }

    // Transparent: delegate to the single field
    if container.transparent {
        return generate_transparent(name, generics, data);
    }

    match &data.fields {
        // `struct Point { x: f64 }` → `interface Point { x: number; }`
        syn::Fields::Named(fields) => generate_named(name, generics, container, fields),
        // `struct Id(String)` → the inner type (`string`); no interface of its own
        syn::Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
            generate_newtype(name, generics, &fields.unnamed[0])
        }
        syn::Fields::Unnamed(_) => Err(syn::Error::new_spanned(
            name,
            "TypeScript derive does not support tuple structs with multiple fields. \
             Use #[serde(transparent)] for newtypes or #[neon(ts_type = \"...\")] as a workaround.",
        )),
        syn::Fields::Unit => Err(syn::Error::new_spanned(
            name,
            "TypeScript derive does not support unit structs. \
             Use #[neon(ts_type = \"...\")] as a workaround.",
        )),
    }
}

/// `#[serde(transparent)]`: the struct serializes as its single field, so its
/// TypeScript type is exactly that field's type.
///
///   #[serde(transparent)] struct Wrapper { inner: Vec<u8> }
///   => number[]   (Wrapper produces no interface of its own)
fn generate_transparent(
    name: &syn::Ident,
    generics: &syn::Generics,
    data: &syn::DataStruct,
) -> syn::Result<proc_macro2::TokenStream> {
    // Find the single non-skipped field (serde transparent requires exactly one)
    let field = match &data.fields {
        syn::Fields::Named(fields) => fields.named.first(),
        syn::Fields::Unnamed(fields) => fields.unnamed.first(),
        syn::Fields::Unit => None,
    };

    let field = field.ok_or_else(|| {
        syn::Error::new_spanned(name, "transparent struct must have at least one field")
    })?;

    let ty = &field.ty;
    let bounded = generics_with_ts_bounds(generics);
    let (impl_generics, ty_generics, where_clause) = bounded.split_for_impl();

    Ok(quote::quote! {
        impl #impl_generics neon::typescript::TypeScript for #name #ty_generics #where_clause {
            fn ts_type() -> std::borrow::Cow<'static, str> {
                <#ty as neon::typescript::TypeScript>::ts_type()
            }

            fn ts_decl() -> Option<std::borrow::Cow<'static, str>> {
                <#ty as neon::typescript::TypeScript>::ts_decl()
            }

            fn ts_collect(decls: &mut std::collections::BTreeMap<String, String>) {
                <#ty as neon::typescript::TypeScript>::ts_collect(decls);
            }
        }
    })
}

/// Single-field tuple struct (`struct Id(String);`): treated like a newtype, so
/// its TypeScript type is the inner field's type (`string`). Identical in spirit
/// to `#[serde(transparent)]`, but applies to the `Foo(T)` shape without the attr.
fn generate_newtype(
    name: &syn::Ident,
    generics: &syn::Generics,
    field: &syn::Field,
) -> syn::Result<proc_macro2::TokenStream> {
    let ty = &field.ty;
    let bounded = generics_with_ts_bounds(generics);
    let (impl_generics, ty_generics, where_clause) = bounded.split_for_impl();

    Ok(quote::quote! {
        impl #impl_generics neon::typescript::TypeScript for #name #ty_generics #where_clause {
            fn ts_type() -> std::borrow::Cow<'static, str> {
                <#ty as neon::typescript::TypeScript>::ts_type()
            }

            fn ts_decl() -> Option<std::borrow::Cow<'static, str>> {
                <#ty as neon::typescript::TypeScript>::ts_decl()
            }

            fn ts_collect(decls: &mut std::collections::BTreeMap<String, String>) {
                <#ty as neon::typescript::TypeScript>::ts_collect(decls);
            }
        }
    })
}

/// Named-field struct → `interface`. Each field becomes one `name: type;` line,
/// with serde `rename`/`rename_all` applied to the name, `#[serde(default)]`
/// making it optional (`name?:`), `#[serde(skip)]` dropping it, and
/// `#[serde(flatten)]` turning it into an ` & Other` intersection.
///
///   #[serde(rename_all = "camelCase")]
///   struct User { user_id: u32, #[serde(flatten)] page: Page }
///   => interface User { userId: number; } & Page
///
/// Generic structs (`struct Env<T>`) emit `interface Env<T> { ... }` in the
/// declaration while still keying the collected decl by the base name `Env`, so
/// `Env<string>` and `Env<number>` share a single interface.
fn generate_named(
    name: &syn::Ident,
    generics: &syn::Generics,
    container: &ContainerAttrs,
    fields: &syn::FieldsNamed,
) -> syn::Result<proc_macro2::TokenStream> {
    let type_name = name.to_string();
    let type_param_names: Vec<String> = generics
        .type_params()
        .map(|tp| tp.ident.to_string())
        .collect();
    let has_generics = !type_param_names.is_empty();

    // Collect field info for ts_decl() code generation
    let mut field_stmts = Vec::new();
    let mut collect_stmts = Vec::new();

    for field in &fields.named {
        let field_attrs = FieldAttrs::parse(&field.attrs)?;

        if field_attrs.skip {
            continue;
        }

        let field_ident = field.ident.as_ref().unwrap();
        let field_ty = &field.ty;

        // Determine the TS field name
        let ts_field_name = if let Some(rename) = &field_attrs.rename {
            rename.clone()
        } else if let Some(rule) = container.rename_all {
            rule.apply(&field_ident.to_string())
        } else {
            field_ident.to_string()
        };

        let optional = if field_attrs.default { "?" } else { "" };

        if field_attrs.flatten {
            // Flatten: append ` & Type` as an intersection
            if let Some(ts) = &field_attrs.ts_type {
                field_stmts.push(quote::quote! {
                    flatten_types.push(#ts.to_string());
                });
            } else {
                field_stmts.push(quote::quote! {
                    flatten_types.push(<#field_ty as neon::typescript::TypeScript>::ts_type().into_owned());
                });
            }
        } else if let Some(ts_override) = &field_attrs.ts_type {
            field_stmts.push(quote::quote! {
                s.push_str(&format!("  {}{}: {};\n", #ts_field_name, #optional, #ts_override));
            });
        } else if has_generics && is_type_param(&field.ty, &type_param_names) {
            // Field type is exactly a generic type parameter — use its name literally
            // in the declaration, but call ts_type() for the concrete instantiation key
            let param_name = type_param_name(&field.ty);
            field_stmts.push(quote::quote! {
                if generic_decl {
                    s.push_str(&format!("  {}{}: {};\n", #ts_field_name, #optional, #param_name));
                } else {
                    s.push_str(&format!(
                        "  {}{}: {};\n",
                        #ts_field_name,
                        #optional,
                        <#field_ty as neon::typescript::TypeScript>::ts_type(),
                    ));
                }
            });
        } else if has_generics && contains_type_param(&field.ty, &type_param_names) {
            // Field type contains a generic param in a nested position (e.g., Vec<T>).
            // In the generic declaration, we substitute the concrete param type back to
            // the param name for a best-effort generic representation.
            let params_for_sub: Vec<_> = type_params_in_type(&field.ty, &type_param_names);
            let param_idents: Vec<syn::Type> = params_for_sub
                .iter()
                .map(|p| syn::parse_str(p).unwrap())
                .collect();
            let param_names: Vec<String> = params_for_sub.clone();

            field_stmts.push(quote::quote! {
                if generic_decl {
                    let mut field_ts = <#field_ty as neon::typescript::TypeScript>::ts_type().into_owned();
                    #(
                        let concrete = <#param_idents as neon::typescript::TypeScript>::ts_type();
                        field_ts = field_ts.replace(&*concrete, #param_names);
                    )*
                    s.push_str(&format!("  {}{}: {};\n", #ts_field_name, #optional, field_ts));
                } else {
                    s.push_str(&format!(
                        "  {}{}: {};\n",
                        #ts_field_name,
                        #optional,
                        <#field_ty as neon::typescript::TypeScript>::ts_type(),
                    ));
                }
            });
        } else {
            field_stmts.push(quote::quote! {
                s.push_str(&format!(
                    "  {}{}: {};\n",
                    #ts_field_name,
                    #optional,
                    <#field_ty as neon::typescript::TypeScript>::ts_type(),
                ));
            });
        }

        // Collect declarations from field types (unless overridden)
        if field_attrs.ts_type.is_none() {
            collect_stmts.push(quote::quote! {
                <#field_ty as neon::typescript::TypeScript>::ts_collect(decls);
            });
        }
    }

    let bounded = generics_with_ts_bounds(generics);
    let (impl_generics, ty_generics, where_clause) = bounded.split_for_impl();

    // Build the ts_type() and ts_decl() bodies differently for generic vs non-generic
    let ts_type_body = if has_generics {
        let param_idents: Vec<&syn::Ident> = generics.type_params().map(|tp| &tp.ident).collect();
        quote::quote! {
            let mut s = #type_name.to_string();
            s.push('<');
            let params: Vec<String> = vec![
                #( <#param_idents as neon::typescript::TypeScript>::ts_type().into_owned() ),*
            ];
            s.push_str(&params.join(", "));
            s.push('>');
            std::borrow::Cow::Owned(s)
        }
    } else {
        quote::quote! {
            std::borrow::Cow::Borrowed(#type_name)
        }
    };

    let ts_decl_body = if has_generics {
        let param_names_str: Vec<String> = type_param_names.clone();
        let ts_generic_params = param_names_str.join(", ");
        let interface_header = format!("interface {}<{}> {{\n", type_name, ts_generic_params);

        quote::quote! {
            let generic_decl = true;
            let mut s = String::new();
            let mut flatten_types: Vec<String> = Vec::new();

            s.push_str(#interface_header);
            #(#field_stmts)*
            s.push('}');

            if !flatten_types.is_empty() {
                for ft in &flatten_types {
                    s = format!("{} & {}", s, ft);
                }
            }

            Some(std::borrow::Cow::Owned(s))
        }
    } else {
        quote::quote! {
            let generic_decl = false;
            let mut s = String::new();
            let mut flatten_types: Vec<String> = Vec::new();

            s.push_str(&format!("interface {} {{\n", #type_name));
            #(#field_stmts)*
            s.push('}');

            if !flatten_types.is_empty() {
                for ft in &flatten_types {
                    s = format!("{} & {}", s, ft);
                }
            }

            Some(std::borrow::Cow::Owned(s))
        }
    };

    let ts_collect_body = if has_generics {
        // For generic types, key by the base name to dedup across instantiations
        quote::quote! {
            // Use the base type name (without generic params) as the dedup key
            let key = #type_name.to_string();
            if let Some(decl) = Self::ts_decl() {
                decls.entry(key).or_insert_with(|| decl.into_owned());
            }
            #(#collect_stmts)*
        }
    } else {
        quote::quote! {
            if let Some(decl) = Self::ts_decl() {
                let name = Self::ts_type().into_owned();
                decls.entry(name).or_insert_with(|| decl.into_owned());
            }
            #(#collect_stmts)*
        }
    };

    Ok(quote::quote! {
        impl #impl_generics neon::typescript::TypeScript for #name #ty_generics #where_clause {
            fn ts_type() -> std::borrow::Cow<'static, str> {
                #ts_type_body
            }

            fn ts_decl() -> Option<std::borrow::Cow<'static, str>> {
                #ts_decl_body
            }

            fn ts_collect(decls: &mut std::collections::BTreeMap<String, String>) {
                #ts_collect_body
            }
        }
    })
}

/// Check if a type is *exactly* a generic type parameter — `T` given
/// `params = ["T"]` is true, but `Vec<T>` or `String` is false. A field typed
/// exactly `T` prints the param name (`T`) verbatim in the generic declaration.
fn is_type_param(ty: &syn::Type, params: &[String]) -> bool {
    if let syn::Type::Path(type_path) = ty {
        if type_path.qself.is_none() && type_path.path.segments.len() == 1 {
            let seg = &type_path.path.segments[0];
            if seg.arguments.is_none() {
                return params.contains(&seg.ident.to_string());
            }
        }
    }
    false
}

/// Extract the type parameter name from a type that is exactly a type param.
fn type_param_name(ty: &syn::Type) -> String {
    if let syn::Type::Path(type_path) = ty {
        type_path.path.segments[0].ident.to_string()
    } else {
        unreachable!()
    }
}

/// Check if a type contains a generic param *anywhere*, including nested — `Vec<T>`,
/// `Option<Box<T>>`, `(T, u8)` are all true given `params = ["T"]`. Such fields go
/// through the substitution branch, which best-effort replaces the concrete param
/// type back with the param name in the generic declaration.
fn contains_type_param(ty: &syn::Type, params: &[String]) -> bool {
    match ty {
        syn::Type::Path(type_path) => {
            for seg in &type_path.path.segments {
                if params.contains(&seg.ident.to_string()) {
                    return true;
                }
                if let syn::PathArguments::AngleBracketed(args) = &seg.arguments {
                    for arg in &args.args {
                        if let syn::GenericArgument::Type(inner) = arg {
                            if contains_type_param(inner, params) {
                                return true;
                            }
                        }
                    }
                }
            }
            false
        }
        syn::Type::Reference(r) => contains_type_param(&r.elem, params),
        syn::Type::Slice(s) => contains_type_param(&s.elem, params),
        syn::Type::Array(a) => contains_type_param(&a.elem, params),
        syn::Type::Tuple(t) => t.elems.iter().any(|e| contains_type_param(e, params)),
        _ => false,
    }
}

/// Collect all type parameter names referenced in a type.
fn type_params_in_type(ty: &syn::Type, params: &[String]) -> Vec<String> {
    let mut found = Vec::new();
    collect_params(ty, params, &mut found);
    found.dedup();
    found
}

fn collect_params(ty: &syn::Type, params: &[String], found: &mut Vec<String>) {
    match ty {
        syn::Type::Path(type_path) => {
            for seg in &type_path.path.segments {
                let name = seg.ident.to_string();
                if params.contains(&name) && !found.contains(&name) {
                    found.push(name);
                }
                if let syn::PathArguments::AngleBracketed(args) = &seg.arguments {
                    for arg in &args.args {
                        if let syn::GenericArgument::Type(inner) = arg {
                            collect_params(inner, params, found);
                        }
                    }
                }
            }
        }
        syn::Type::Reference(r) => collect_params(&r.elem, params, found),
        syn::Type::Slice(s) => collect_params(&s.elem, params, found),
        syn::Type::Array(a) => collect_params(&a.elem, params, found),
        syn::Type::Tuple(t) => {
            for e in &t.elems {
                collect_params(e, params, found);
            }
        }
        _ => {}
    }
}

/// Add `T: neon::typescript::TypeScript` bounds for all type params.
fn generics_with_ts_bounds(generics: &syn::Generics) -> syn::Generics {
    let mut generics = generics.clone();
    for param in &mut generics.params {
        if let syn::GenericParam::Type(tp) = param {
            tp.bounds
                .push(syn::parse_quote!(neon::typescript::TypeScript));
        }
    }
    generics
}
