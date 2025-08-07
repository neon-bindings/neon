//! Procedural macros supporting [Neon](https://docs.rs/neon/latest/neon/)

mod export;
mod error;

#[proc_macro_attribute]
pub fn main(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let syn::ItemFn {
        attrs,
        vis,
        sig,
        block,
    } = syn::parse_macro_input!(item as syn::ItemFn);

    let name = &sig.ident;
    let export_name = quote::format_ident!("__NEON_MAIN__{name}");
    let export_fn = quote::quote!({
        #[neon::macro_internal::linkme::distributed_slice(neon::macro_internal::MAIN)]
        #[linkme(crate = neon::macro_internal::linkme)]
        fn #export_name(cx: neon::context::ModuleContext) -> neon::result::NeonResult<()> {
            #name(cx)
        }
    });

    quote::quote!(
        #(#attrs) *
        #vis #sig {
            #export_fn
            #block
        }
    )
    .into()
}

#[proc_macro_attribute]
pub fn export(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    export::export(attr, item)
}
