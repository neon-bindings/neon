//! The `neon_ts_rs::TypeScript` bridge derive. Generates a
//! `neon_typescript::TypeScript` impl for a user type by delegating to the
//! type's `ts_rs::TS` impl (via the runtime helpers in `neon-ts-rs`). Reads
//! nothing about the type's fields or serde attributes.

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, parse_quote, DeriveInput, GenericParam};

#[proc_macro_derive(TypeScript)]
pub fn derive_typescript(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    // Every type parameter must itself be a ts-rs type and `'static`, so the
    // delegated `ts_rs::TS` bound holds for the whole type.
    let mut generics = input.generics.clone();
    for param in &mut generics.params {
        if let GenericParam::Type(type_param) = param {
            type_param.bounds.push(parse_quote!(::ts_rs::TS));
            type_param.bounds.push(parse_quote!('static));
        }
    }
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics ::neon_typescript::TypeScript for #name #ty_generics #where_clause {
            fn ts_type() -> ::std::borrow::Cow<'static, str> {
                ::neon_ts_rs::ts_type::<Self>()
            }
            fn ts_collect(decls: &mut ::std::collections::BTreeMap<::std::string::String, ::std::string::String>) {
                ::neon_ts_rs::ts_collect::<Self>(decls)
            }
        }
    };

    expanded.into()
}
