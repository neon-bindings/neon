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
