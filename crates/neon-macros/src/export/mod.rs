mod class;
mod function;
mod global;

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

            item.expr = global::export(meta, "const", &item.ident, item.expr);

            quote::quote!(#item).into()
        }

        // Export a `static`
        syn::Item::Static(mut item) => {
            let meta = syn::parse_macro_input!(attr with global::meta::Parser);

            item.expr = global::export(meta, "static", &item.ident, item.expr);

            quote::quote!(#item).into()
        }

        // Export a class (impl block)
        syn::Item::Impl(item) => {
            let meta = syn::parse_macro_input!(attr with class::meta::Parser);

            class::export(meta, item)
        }

        // Return an error span for all other types
        _ => unsupported(item),
    }
}

// Register a static description of an export in `EXPORT_INFO`, so duplicate
// export names can be reported with the Rust items that export them. Emitted
// as a statement, so it must be placed inside a block or at item level.
fn export_info(
    kind: &str,
    ident: &syn::Ident,
    export_name: &proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let info_name = quote::format_ident!("__NEON_EXPORT_INFO__{ident}");

    quote::quote!(
        #[doc(hidden)]
        #[allow(non_upper_case_globals)]
        #[neon::macro_internal::linkme::distributed_slice(neon::macro_internal::EXPORT_INFO)]
        #[linkme(crate = neon::macro_internal::linkme)]
        static #info_name: neon::macro_internal::ExportInfo = neon::macro_internal::ExportInfo {
            name: #export_name,
            kind: #kind,
            rust_path: concat!(module_path!(), "::", stringify!(#ident)),
        };
    )
}

// Generate an error for unsupported item types
fn unsupported(item: syn::Item) -> proc_macro::TokenStream {
    let span = syn::spanned::Spanned::span(&item);
    let msg = "`neon::export` can only be applied to functions, consts, statics, and classes (impl blocks).";
    let err = syn::Error::new(span, msg);

    err.into_compile_error().into()
}
