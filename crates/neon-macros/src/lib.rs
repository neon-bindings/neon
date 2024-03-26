//! Procedural macros supporting [Neon](https://docs.rs/neon/latest/neon/)

#[proc_macro_attribute]
/// Marks a function as the main entry point for initialization in
/// a Neon module.
///
/// This attribute should only be used _once_ in a module and will
/// be called each time the module is initialized in a context.
///
/// If a `main` function is not provided, all registered exports will be exported.
///
/// ```ignore
/// #[neon::main]
/// fn main(mut cx: ModuleContext) -> NeonResult<()> {
///     // Export all registered exports
///     neon::registered().export(&mut cx)?;
///
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
    let input = syn::parse_macro_input!(item as syn::ItemFn);

    quote::quote!(
        #[neon::macro_internal::linkme::distributed_slice(neon::macro_internal::MAIN)]
        #[linkme(crate = neon::macro_internal::linkme)]
        #input
    )
    .into()
}

#[proc_macro_attribute]
/// Register an export a Rust function or value from an addon
///
/// ```ignore
/// #[neon::export]
/// fn hello(mut cx: FunctionContext) -> JsResult<JsString> {
///     Ok(cx.string("Hello, Neon!"))
/// }
/// ```
pub fn export(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(item as syn::ItemFn);
    let name = &input.sig.ident;
    let create_name = quote::format_ident!("__EXPORT_CREATE__{name}");
    let wrapper_name = quote::format_ident!("__EXPORT_WRAPPER__{name}");
    let mut export_name = syn::LitStr::new(&name.to_string(), name.span());
    let attr_parser = syn::meta::parser(|meta| {
        if meta.path.is_ident("name") {
            export_name = meta.value()?.parse()?;
        }

        Ok(())
    });

    syn::parse_macro_input!(attr with attr_parser);

    let has_context = input
        .sig
        .inputs
        .first()
        .map(|arg| {
            let ty = match arg {
                syn::FnArg::Receiver(v) => &*v.ty,
                syn::FnArg::Typed(v) => &*v.ty,
            };

            let ty = match ty {
                syn::Type::Reference(ty) => &*ty.elem,
                _ => return false,
            };

            let path = match ty {
                syn::Type::Path(path) => path,
                _ => return false,
            };

            let path = match path.path.segments.last() {
                Some(path) => path,
                None => return false,
            };

            path.ident == "FunctionContext"
        })
        .unwrap_or(false);

    let (context_arg, start) = if has_context {
        (quote::quote!(&mut cx,), 1)
    } else {
        (quote::quote!(), 0)
    };

    let tuple_names = (start..input.sig.inputs.len()).map(|i| quote::format_ident!("a{i}"));
    let arg_names = tuple_names.clone();

    quote::quote!(
        #input

        {
            fn #wrapper_name(mut cx: neon::context::FunctionContext) -> neon::result::JsResult<neon::types::JsValue> {
                let (#(#tuple_names,)*) = cx.args()?;
                let res = #name(#context_arg #(#arg_names),*);

                neon::types::extract::TryIntoJs::try_into_js(res, &mut cx).map(|v| neon::handle::Handle::upcast(&v))
            }

            #[neon::macro_internal::linkme::distributed_slice(neon::macro_internal::EXPORTS)]
            #[linkme(crate = neon::macro_internal::linkme)]
            fn #create_name<'cx>(
                cx: &mut neon::context::ModuleContext<'cx>,
            ) -> neon::result::NeonResult<(&'static str, neon::handle::Handle<'cx, neon::types::JsValue>)> {
                static NAME: &str = #export_name;

                neon::types::JsFunction::with_name(cx, NAME, #wrapper_name).map(|v| (
                    NAME,
                    neon::handle::Handle::upcast(&v),
                ))
            }
        }
    )
    .into()
}
