//! Procedural macros supporting [Neon](https://docs.rs/neon/latest/neon/)

#[cfg(feature = "export")]
mod export;

#[cfg(feature = "export")]
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

#[cfg(not(feature = "export"))]
#[proc_macro_attribute]
pub fn main(
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
                env: *mut std::ffi::c_void,
                m: *mut std::ffi::c_void,
            ) -> *mut std::ffi::c_void {
                unsafe {
                    neon::macro_internal::initialize_module(env, m, #name);
                    m
                }
            }

            #block
        }
    )
    .into()
}

#[cfg(feature = "export")]
#[proc_macro_attribute]
pub fn export(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    export::export(attr, item)
}
