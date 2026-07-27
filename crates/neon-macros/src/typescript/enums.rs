//! Generates `impl TypeScript` for enums, producing a `type X = ...` union whose
//! shape depends on the serde tagging mode. Using this enum as the running example:
//!
//! ```ignore
//! enum E { Unit, New(String), Struct { x: f64 } }
//! ```
//!
//! | serde attribute on `E`         | mode               | generated `type E = ...`                                            |
//! |--------------------------------|--------------------|--------------------------------------------------------------------|
//! | (none)                         | externally tagged  | `"Unit" \| { New: string } \| { Struct: { x: number; } }`          |
//! | `tag = "t"`                    | internally tagged  | `{ t: "Unit" } \| { t: "Struct"; x: number; }` (tuple vars: error) |
//! | `tag = "t", content = "c"`     | adjacently tagged  | `{ t: "Unit" } \| { t: "New", c: string } \| { t: "Struct", c: {...} }` |
//! | `untagged`                     | untagged           | `null \| string \| { x: number; }`                                 |

use super::attrs::{ContainerAttrs, FieldAttrs, VariantAttrs};

pub(crate) fn generate(
    name: &syn::Ident,
    generics: &syn::Generics,
    container: &ContainerAttrs,
    data: &syn::DataEnum,
) -> syn::Result<proc_macro2::TokenStream> {
    // Type-level ts_type override: `#[neon(ts_type = "...")]` on the enum replaces
    // the whole generated type with the given literal string.
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

    match (&container.tag, &container.content, container.untagged) {
        // Untagged: plain union of variant data types
        (_, _, true) => generate_untagged(name, generics, container, data),
        // Internally tagged: { tag: "Variant", ...fields }
        (Some(tag), None, false) => {
            generate_internally_tagged(name, generics, container, data, tag)
        }
        // Adjacently tagged: { tag: "Variant", content: Data }
        (Some(tag), Some(content), false) => {
            generate_adjacently_tagged(name, generics, container, data, tag, content)
        }
        // Externally tagged (serde default): "Variant" | { Variant: Data }
        (None, None, false) => generate_externally_tagged(name, generics, container, data),
        // content without tag is invalid
        (None, Some(_), false) => Err(syn::Error::new_spanned(
            name,
            "`#[serde(content = \"...\")]` requires `#[serde(tag = \"...\")]`",
        )),
    }
}

/// Helper: resolve the serialized variant name from attrs and rename_all.
fn variant_tag_value(
    variant: &syn::Variant,
    variant_attrs: &VariantAttrs,
    container: &ContainerAttrs,
) -> String {
    if let Some(rename) = &variant_attrs.rename {
        rename.clone()
    } else if let Some(rule) = container.rename_all {
        rule.apply(&variant.ident.to_string())
    } else {
        variant.ident.to_string()
    }
}

/// Helper: generate codegen for named fields as `{ field: type; ... }` inside
/// a variant body. Returns (variant_stmts, collect_stmts).
fn named_fields_stmts(
    fields: &syn::FieldsNamed,
    variant_attrs: &VariantAttrs,
    container: &ContainerAttrs,
) -> syn::Result<(Vec<proc_macro2::TokenStream>, Vec<proc_macro2::TokenStream>)> {
    let field_rename = variant_attrs.rename_all.or(container.rename_all);
    let mut stmts = Vec::new();
    let mut collect = Vec::new();

    for field in &fields.named {
        let field_attrs = FieldAttrs::parse(&field.attrs)?;
        if field_attrs.skip {
            continue;
        }

        let field_ident = field.ident.as_ref().unwrap();
        let field_ty = &field.ty;

        let ts_field_name = if let Some(rename) = &field_attrs.rename {
            rename.clone()
        } else if let Some(rule) = field_rename {
            rule.apply(&field_ident.to_string())
        } else {
            field_ident.to_string()
        };

        let optional = if field_attrs.default { "?" } else { "" };

        if let Some(ts_override) = &field_attrs.ts_type {
            stmts.push(quote::quote! {
                s.push_str(&format!("  {}{}: {};\n", #ts_field_name, #optional, #ts_override));
            });
        } else {
            stmts.push(quote::quote! {
                s.push_str(&format!(
                    "  {}{}: {};\n",
                    #ts_field_name,
                    #optional,
                    <#field_ty as neon::typescript::TypeScript>::ts_type(),
                ));
            });
        }

        if field_attrs.ts_type.is_none() {
            collect.push(quote::quote! {
                <#field_ty as neon::typescript::TypeScript>::ts_collect(decls);
            });
        }
    }

    Ok((stmts, collect))
}

// ——— Internally tagged ———
//
// `#[serde(tag = "kind")]` — the discriminant lives on a `kind` field alongside
// the variant's own fields. Tuple variants are rejected (serde can't flatten a
// tuple's fields next to the tag).
//
//   enum Shape { Circle { radius: f64 }, Dot }
//   => type Shape = { kind: "Circle"; radius: number; } | { kind: "Dot" };

fn generate_internally_tagged(
    name: &syn::Ident,
    generics: &syn::Generics,
    container: &ContainerAttrs,
    data: &syn::DataEnum,
    tag: &str,
) -> syn::Result<proc_macro2::TokenStream> {
    let type_name = name.to_string();

    let mut variant_stmts = Vec::new();
    let mut collect_stmts = Vec::new();
    let mut first_variant = true;

    for variant in &data.variants {
        let variant_attrs = VariantAttrs::parse(&variant.attrs)?;

        if variant_attrs.skip {
            continue;
        }

        let tag_value = variant_tag_value(variant, &variant_attrs, container);

        let separator = if first_variant { "" } else { " | " };
        first_variant = false;

        match &variant.fields {
            syn::Fields::Unit => {
                variant_stmts.push(quote::quote! {
                    s.push_str(#separator);
                    s.push_str(&format!("{{ {}: \"{}\" }}", #tag, #tag_value));
                });
            }
            syn::Fields::Named(fields) => {
                let (field_stmts, field_collect) =
                    named_fields_stmts(fields, &variant_attrs, container)?;
                collect_stmts.extend(field_collect);

                variant_stmts.push(quote::quote! {
                    s.push_str(#separator);
                    s.push_str(&format!("{{\n  {}: \"{}\";\n", #tag, #tag_value));
                });
                variant_stmts.extend(field_stmts);
                variant_stmts.push(quote::quote! {
                    s.push('}');
                });
            }
            syn::Fields::Unnamed(_) => {
                return Err(syn::Error::new_spanned(
                    &variant.ident,
                    "Internally tagged enums do not support tuple variants. \
                     Use named fields instead.",
                ));
            }
        }
    }

    Ok(emit_enum_impl(
        &type_name,
        name,
        generics,
        &variant_stmts,
        &collect_stmts,
    ))
}

// ——— Externally tagged ———
//
// The serde default: the variant name is the key (or, for unit variants, a bare
// string literal).
//
//   enum Msg { Quit, Echo(String), Move { x: f64, y: f64 } }
//   => type Msg = "Quit"
//               | { Echo: string }
//               | { Move: { x: number; y: number; } };

fn generate_externally_tagged(
    name: &syn::Ident,
    generics: &syn::Generics,
    container: &ContainerAttrs,
    data: &syn::DataEnum,
) -> syn::Result<proc_macro2::TokenStream> {
    let type_name = name.to_string();

    let mut variant_stmts = Vec::new();
    let mut collect_stmts = Vec::new();
    let mut first_variant = true;

    for variant in &data.variants {
        let variant_attrs = VariantAttrs::parse(&variant.attrs)?;

        if variant_attrs.skip {
            continue;
        }

        let tag_value = variant_tag_value(variant, &variant_attrs, container);

        let separator = if first_variant { "" } else { " | " };
        first_variant = false;

        match &variant.fields {
            // Unit → "VariantName"
            syn::Fields::Unit => {
                variant_stmts.push(quote::quote! {
                    s.push_str(#separator);
                    s.push_str(&format!("\"{}\"", #tag_value));
                });
            }
            // Newtype / tuple → { VariantName: InnerType } or { VariantName: [T1, T2] }
            syn::Fields::Unnamed(fields) => {
                if fields.unnamed.len() == 1 {
                    // Newtype variant
                    let field_ty = &fields.unnamed[0].ty;
                    variant_stmts.push(quote::quote! {
                        s.push_str(#separator);
                        s.push_str(&format!(
                            "{{ {}: {} }}",
                            #tag_value,
                            <#field_ty as neon::typescript::TypeScript>::ts_type(),
                        ));
                    });
                    collect_stmts.push(quote::quote! {
                        <#field_ty as neon::typescript::TypeScript>::ts_collect(decls);
                    });
                } else {
                    // Tuple variant → { VariantName: [T1, T2, ...] }
                    let types: Vec<_> = fields.unnamed.iter().map(|f| &f.ty).collect();
                    let len = types.len();
                    variant_stmts.push(quote::quote! {
                        s.push_str(#separator);
                        s.push_str(&format!("{{ {}: [", #tag_value));
                    });
                    for (i, ty) in types.iter().enumerate() {
                        let comma = if i + 1 < len { ", " } else { "" };
                        variant_stmts.push(quote::quote! {
                            s.push_str(&format!(
                                "{}{}",
                                <#ty as neon::typescript::TypeScript>::ts_type(),
                                #comma,
                            ));
                        });
                        collect_stmts.push(quote::quote! {
                            <#ty as neon::typescript::TypeScript>::ts_collect(decls);
                        });
                    }
                    variant_stmts.push(quote::quote! {
                        s.push_str("] }");
                    });
                }
            }
            // Struct → { VariantName: { field: type; ... } }
            syn::Fields::Named(fields) => {
                let (field_stmts, field_collect) =
                    named_fields_stmts(fields, &variant_attrs, container)?;
                collect_stmts.extend(field_collect);

                variant_stmts.push(quote::quote! {
                    s.push_str(#separator);
                    s.push_str(&format!("{{ {}: {{\n", #tag_value));
                });
                variant_stmts.extend(field_stmts);
                variant_stmts.push(quote::quote! {
                    s.push_str("} }");
                });
            }
        }
    }

    Ok(emit_enum_impl(
        &type_name,
        name,
        generics,
        &variant_stmts,
        &collect_stmts,
    ))
}

// ——— Adjacently tagged ———
//
// `#[serde(tag = "type", content = "data")]` — the discriminant and the payload
// live in two sibling fields. Unit variants omit the content field.
//
//   enum Resp { Loading, Success(String), Error { code: u32 } }
//   => type Resp = { type: "Loading" }
//                | { type: "Success", data: string }
//                | { type: "Error", data: { code: number; } };

fn generate_adjacently_tagged(
    name: &syn::Ident,
    generics: &syn::Generics,
    container: &ContainerAttrs,
    data: &syn::DataEnum,
    tag: &str,
    content: &str,
) -> syn::Result<proc_macro2::TokenStream> {
    let type_name = name.to_string();

    let mut variant_stmts = Vec::new();
    let mut collect_stmts = Vec::new();
    let mut first_variant = true;

    for variant in &data.variants {
        let variant_attrs = VariantAttrs::parse(&variant.attrs)?;

        if variant_attrs.skip {
            continue;
        }

        let tag_value = variant_tag_value(variant, &variant_attrs, container);

        let separator = if first_variant { "" } else { " | " };
        first_variant = false;

        match &variant.fields {
            // Unit → { tag: "Variant" } (no content field)
            syn::Fields::Unit => {
                variant_stmts.push(quote::quote! {
                    s.push_str(#separator);
                    s.push_str(&format!("{{ {}: \"{}\" }}", #tag, #tag_value));
                });
            }
            // Newtype → { tag: "Variant", content: InnerType }
            syn::Fields::Unnamed(fields) => {
                if fields.unnamed.len() == 1 {
                    let field_ty = &fields.unnamed[0].ty;
                    variant_stmts.push(quote::quote! {
                        s.push_str(#separator);
                        s.push_str(&format!(
                            "{{ {}: \"{}\", {}: {} }}",
                            #tag,
                            #tag_value,
                            #content,
                            <#field_ty as neon::typescript::TypeScript>::ts_type(),
                        ));
                    });
                    collect_stmts.push(quote::quote! {
                        <#field_ty as neon::typescript::TypeScript>::ts_collect(decls);
                    });
                } else {
                    // Tuple → { tag: "Variant", content: [T1, T2] }
                    let types: Vec<_> = fields.unnamed.iter().map(|f| &f.ty).collect();
                    let len = types.len();
                    variant_stmts.push(quote::quote! {
                        s.push_str(#separator);
                        s.push_str(&format!("{{ {}: \"{}\", {}: [", #tag, #tag_value, #content));
                    });
                    for (i, ty) in types.iter().enumerate() {
                        let comma = if i + 1 < len { ", " } else { "" };
                        variant_stmts.push(quote::quote! {
                            s.push_str(&format!(
                                "{}{}",
                                <#ty as neon::typescript::TypeScript>::ts_type(),
                                #comma,
                            ));
                        });
                        collect_stmts.push(quote::quote! {
                            <#ty as neon::typescript::TypeScript>::ts_collect(decls);
                        });
                    }
                    variant_stmts.push(quote::quote! {
                        s.push_str("] }");
                    });
                }
            }
            // Struct → { tag: "Variant", content: { field: type; ... } }
            syn::Fields::Named(fields) => {
                let (field_stmts, field_collect) =
                    named_fields_stmts(fields, &variant_attrs, container)?;
                collect_stmts.extend(field_collect);

                variant_stmts.push(quote::quote! {
                    s.push_str(#separator);
                    s.push_str(&format!("{{ {}: \"{}\", {}: {{\n", #tag, #tag_value, #content));
                });
                variant_stmts.extend(field_stmts);
                variant_stmts.push(quote::quote! {
                    s.push_str("} }");
                });
            }
        }
    }

    Ok(emit_enum_impl(
        &type_name,
        name,
        generics,
        &variant_stmts,
        &collect_stmts,
    ))
}

// ——— Untagged ———
//
// `#[serde(untagged)]` — no discriminant; each variant serializes as just its
// data, and the union is matched structurally at runtime.
//
//   enum Value { Nothing, Text(String), Point { x: f64 } }
//   => type Value = null | string | { x: number; };

fn generate_untagged(
    name: &syn::Ident,
    generics: &syn::Generics,
    container: &ContainerAttrs,
    data: &syn::DataEnum,
) -> syn::Result<proc_macro2::TokenStream> {
    let type_name = name.to_string();

    let mut variant_stmts = Vec::new();
    let mut collect_stmts = Vec::new();
    let mut first_variant = true;

    for variant in &data.variants {
        let variant_attrs = VariantAttrs::parse(&variant.attrs)?;

        if variant_attrs.skip {
            continue;
        }

        let separator = if first_variant { "" } else { " | " };
        first_variant = false;

        match &variant.fields {
            // Unit → null
            syn::Fields::Unit => {
                variant_stmts.push(quote::quote! {
                    s.push_str(#separator);
                    s.push_str("null");
                });
            }
            // Newtype → InnerType directly
            syn::Fields::Unnamed(fields) => {
                if fields.unnamed.len() == 1 {
                    let field_ty = &fields.unnamed[0].ty;
                    variant_stmts.push(quote::quote! {
                        s.push_str(#separator);
                        s.push_str(&<#field_ty as neon::typescript::TypeScript>::ts_type());
                    });
                    collect_stmts.push(quote::quote! {
                        <#field_ty as neon::typescript::TypeScript>::ts_collect(decls);
                    });
                } else {
                    // Tuple → [T1, T2]
                    let types: Vec<_> = fields.unnamed.iter().map(|f| &f.ty).collect();
                    let len = types.len();
                    variant_stmts.push(quote::quote! {
                        s.push_str(#separator);
                        s.push_str("[");
                    });
                    for (i, ty) in types.iter().enumerate() {
                        let comma = if i + 1 < len { ", " } else { "" };
                        variant_stmts.push(quote::quote! {
                            s.push_str(&format!(
                                "{}{}",
                                <#ty as neon::typescript::TypeScript>::ts_type(),
                                #comma,
                            ));
                        });
                        collect_stmts.push(quote::quote! {
                            <#ty as neon::typescript::TypeScript>::ts_collect(decls);
                        });
                    }
                    variant_stmts.push(quote::quote! {
                        s.push_str("]");
                    });
                }
            }
            // Struct → { field: type; ... }
            syn::Fields::Named(fields) => {
                let (field_stmts, field_collect) =
                    named_fields_stmts(fields, &variant_attrs, container)?;
                collect_stmts.extend(field_collect);

                variant_stmts.push(quote::quote! {
                    s.push_str(#separator);
                    s.push_str("{\n");
                });
                variant_stmts.extend(field_stmts);
                variant_stmts.push(quote::quote! {
                    s.push('}');
                });
            }
        }
    }

    Ok(emit_enum_impl(
        &type_name,
        name,
        generics,
        &variant_stmts,
        &collect_stmts,
    ))
}

// ——— Shared codegen helper ———
//
// Wraps the per-variant statements (each of which pushes one union member onto
// a `String s`) into the final `impl TypeScript`. `ts_type()` returns the enum
// name (e.g. `"Shape"`); `ts_decl()` returns `type Shape = <members joined by
// " | ">;`; `ts_collect()` records that decl plus any referenced field types.

fn emit_enum_impl(
    type_name: &str,
    name: &syn::Ident,
    generics: &syn::Generics,
    variant_stmts: &[proc_macro2::TokenStream],
    collect_stmts: &[proc_macro2::TokenStream],
) -> proc_macro2::TokenStream {
    let bounded = generics_with_ts_bounds(generics);
    let (impl_generics, ty_generics, where_clause) = bounded.split_for_impl();

    quote::quote! {
        impl #impl_generics neon::typescript::TypeScript for #name #ty_generics #where_clause {
            fn ts_type() -> std::borrow::Cow<'static, str> {
                std::borrow::Cow::Borrowed(#type_name)
            }

            fn ts_decl() -> Option<std::borrow::Cow<'static, str>> {
                let mut s = String::new();
                s.push_str(&format!("type {} = ", #type_name));
                #(#variant_stmts)*
                s.push(';');
                Some(std::borrow::Cow::Owned(s))
            }

            fn ts_collect(decls: &mut std::collections::BTreeMap<String, String>) {
                if let Some(decl) = Self::ts_decl() {
                    let name = Self::ts_type().into_owned();
                    decls.entry(name).or_insert_with(|| decl.into_owned());
                }
                #(#collect_stmts)*
            }
        }
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
