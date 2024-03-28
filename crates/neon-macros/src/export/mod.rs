use meta::ExportParser;

mod function;
mod global;
mod meta;

pub(crate) fn export(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    // Parse the macro attributes and item
    let meta = syn::parse_macro_input!(attr with ExportParser);
    let item = syn::parse_macro_input!(item as syn::Item);

    match item {
        // Export a function
        syn::Item::Fn(item) => function::export(meta, item),

        // Export a `const`
        syn::Item::Const(mut item) => {
            item.expr = global::export(meta, &item.ident, item.expr);
            quote::quote!(#item).into()
        }

        // Export a `static`
        syn::Item::Static(mut item) => {
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
    let msg = "`neon::export` can only be applied to functions, consts, and statics.";
    let err = syn::Error::new(span, msg);

    err.into_compile_error().into()
}
