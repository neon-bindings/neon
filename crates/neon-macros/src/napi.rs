pub(crate) fn main(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(item as syn::ItemFn);

    let attrs = &input.attrs;
    let vis = &input.vis;
    let sig = &input.sig;
    let block = &input.block;
    let name = &sig.ident;

    quote::quote!(
        #(#attrs) *
        #vis #sig {
            #[no_mangle]
            unsafe extern "C" fn napi_register_module_v1(
                env: ::neon::macro_internal::runtime::raw::Env,
                m: ::neon::macro_internal::runtime::raw::Local,
            ) -> ::neon::macro_internal::runtime::raw::Local {
                ::neon::macro_internal::initialize_module(
                    env,
                    ::std::mem::transmute(m),
                    #name,
                );
    
                m
            }

            #block
        }
    )
    .into()
}
