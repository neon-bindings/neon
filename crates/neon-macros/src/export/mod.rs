mod function;
mod global;
use crate::error::{self, ErrorCode};

// N.B.: Meta attribute parsing happens in this function because `syn::parse_macro_input!`
// must be called from a function that returns `proc_macro::TokenStream`.
pub(crate) fn export(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    // Parse item to determine the type of export
    let item = syn::parse_macro_input!(item as syn::Item);

    match item {
        // Export a function
        syn::Item::Fn(item) => {
            let parser = function::meta::Parser::new(item);
            let (item, meta) = syn::parse_macro_input!(attr with parser);

            function::export(meta, item)
        }

        // Export a `const`
        syn::Item::Const(mut item) => {
            let meta = syn::parse_macro_input!(attr with global::meta::Parser);

            item.expr = global::export(meta, &item.ident, item.expr);

            quote::quote!(#item).into()
        }

        // Export a `static`
        syn::Item::Static(mut item) => {
            let meta = syn::parse_macro_input!(attr with global::meta::Parser);

            item.expr = global::export(meta, &item.ident, item.expr);

            quote::quote!(#item).into()
        }

        // Return an error span for all other types
        _ => unsupported(item),
    }
}

// Generate an error for unsupported item types
fn unsupported(item: syn::Item) -> proc_macro::TokenStream {
    let span = syn::spanned::Spanned::span(&item);
    let err = error::error(span, ErrorCode::UnsupportedExportItem);

    err.into_compile_error().into()
}
