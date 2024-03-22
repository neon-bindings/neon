//! Procedural macros supporting [Neon](https://docs.rs/neon/latest/neon/)

#[proc_macro_attribute]
/// Marks a function as the main entry point for initialization in
/// a Neon module.
///
/// This attribute should only be used _once_ in a module and will
/// be called each time the module is initialized in a context.
///
/// ```ignore
/// #[neon::main]
/// fn main(mut cx: ModuleContext) -> NeonResult<()> {
///     let version = cx.string("1.0.0");
///
///     cx.export_value("version", version)?;
///
///     Ok(())
/// }
/// ```
///
/// If multiple functions are marked with `#[neon::main]`, there may be a compile error:
///
/// ```sh
/// error: symbol `napi_register_module_v1` is already defined
/// ```
pub fn main(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(item as syn_mid::ItemFn);

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
                neon::macro_internal::initialize_module(env, m, #name);
                m
            }

            #block
        }
    )
    .into()
}

#[proc_macro_attribute]
/// Export a function
pub fn export(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(item as syn_mid::ItemFn);

    let attrs = &input.attrs;
    let vis = &input.vis;
    let sig = &input.sig;
    let block = &input.block;
    let name = &sig.ident;
    let exported_name = quote::format_ident!("__EXPORTED__{name}");

    quote::quote!(
        #(#attrs)*
        #vis #sig {
            #[neon::macro_internal::linkme::distributed_slice(neon::macro_internal::EXPORTS)]
            #[linkme(crate = neon::macro_internal::linkme)]
            fn #exported_name<'cx>(
                cx: &mut neon::context::ModuleContext<'cx>,
            ) -> neon::result::NeonResult<(&'cx str, neon::handle::Handle<'cx, neon::types::JsValue>)> {
                neon::types::JsFunction::new(cx, #name).map(|v| (
                    stringify!(#name),
                    neon::handle::Handle::upcast(&v),
                ))
            }

            #block
        }
    )
    .into()
}
