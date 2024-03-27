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
    match syn::parse_macro_input!(item as syn::Item) {
        syn::Item::Fn(item) => export_fn(attr, item),
        syn::Item::Const(item) => export_global(attr, item.ident.clone(), quote::quote!(#item)),
        syn::Item::Static(item) => export_global(attr, item.ident.clone(), quote::quote!(#item)),
        _ => syn::parse::Error::new(proc_macro::Span::call_site().into(), "Invalid item")
            .to_compile_error()
            .into(),
    }
}

fn export_global(
    attr: proc_macro::TokenStream,
    name: syn::Ident,
    item: proc_macro2::TokenStream,
) -> proc_macro::TokenStream {
    let mut export_name = quote::quote!(stringify!(#name));
    let mut use_json = false;
    let create_name = quote::format_ident!("__EXPORT_CREATE__{name}");
    let attr_parser = syn::meta::parser(|meta| {
        if meta.path.is_ident("name") {
            let name = meta.value()?.parse::<syn::LitStr>()?;

            export_name = quote::quote!(#name);

            return Ok(());
        }

        if meta.path.is_ident("json") {
            use_json = true;

            return Ok(());
        }

        Err(meta.error("unsupported property"))
    });

    syn::parse_macro_input!(attr with attr_parser);

    let value = if use_json {
        quote::quote!(neon::types::extract::Json(&#name))
    } else {
        quote::quote!(#name)
    };

    quote::quote!(
        #item

        #[doc(hidden)]
        #[neon::macro_internal::linkme::distributed_slice(neon::macro_internal::EXPORTS)]
        #[linkme(crate = neon::macro_internal::linkme)]
        fn #create_name<'cx>(
            cx: &mut neon::context::ModuleContext<'cx>,
        ) -> neon::result::NeonResult<(&'static str, neon::handle::Handle<'cx, neon::types::JsValue>)> {
            neon::types::extract::TryIntoJs::try_into_js(#value, cx).map(|v| (
                #export_name,
                neon::handle::Handle::upcast(&v),
            ))
        }

    )
    .into()
}

fn is_context_arg(arg: &syn::FnArg) -> bool {
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
}

fn is_result_output(ret: &syn::ReturnType) -> bool {
    let ty = match ret {
        syn::ReturnType::Default => return false,
        syn::ReturnType::Type(_, ty) => &**ty,
    };

    let path = match ty {
        syn::Type::Path(path) => path,
        _ => return false,
    };

    let path = match path.path.segments.last() {
        Some(path) => path,
        None => return false,
    };

    path.ident == "Result" || path.ident == "NeonResult" || path.ident == "JsResult"
}

fn export_fn(attr: proc_macro::TokenStream, input: syn::ItemFn) -> proc_macro::TokenStream {
    let name = &input.sig.ident;
    let create_name = quote::format_ident!("__EXPORT_CREATE__{name}");
    let wrapper_name = quote::format_ident!("__EXPORT_WRAPPER__{name}");
    let mut export_name = quote::quote!(stringify!(#name));
    let mut use_json = false;
    let mut force_context = false;
    let attr_parser = syn::meta::parser(|meta| {
        if meta.path.is_ident("name") {
            let name = meta.value()?.parse::<syn::LitStr>()?;

            export_name = quote::quote!(#name);

            return Ok(());
        }

        if meta.path.is_ident("json") {
            use_json = true;

            return Ok(());
        }

        if meta.path.is_ident("context") {
            force_context = true;

            return Ok(());
        }

        Err(meta.error("unsupported property"))
    });

    syn::parse_macro_input!(attr with attr_parser);

    let has_context = input
        .sig
        .inputs
        .first()
        .map(is_context_arg)
        .unwrap_or(false);

    let (context_arg, start) = if force_context || has_context {
        (quote::quote!(&mut cx,), 1)
    } else {
        (quote::quote!(), 0)
    };

    let arg_names = (start..input.sig.inputs.len()).map(|i| quote::format_ident!("a{i}"));
    let tuple_fields = arg_names.clone().map(|name| {
        if use_json {
            quote::quote!(neon::types::extract::Json(#name))
        } else {
            quote::quote!(#name)
        }
    });

    let map_res = if use_json {
        if is_result_output(&input.sig.output) {
            quote::quote!(let res = res.map(neon::types::extract::Json);)
        } else {
            quote::quote!(let res = neon::types::extract::Json(res);)
        }
    } else {
        quote::quote!()
    };

    quote::quote!(
        #input

        #[neon::macro_internal::linkme::distributed_slice(neon::macro_internal::EXPORTS)]
        #[linkme(crate = neon::macro_internal::linkme)]
        #[doc(hidden)]
        fn #create_name<'cx>(
            cx: &mut neon::context::ModuleContext<'cx>,
        ) -> neon::result::NeonResult<(&'static str, neon::handle::Handle<'cx, neon::types::JsValue>)> {
            #[doc(hidden)]
            fn #wrapper_name(mut cx: neon::context::FunctionContext) -> neon::result::JsResult<neon::types::JsValue> {
                let (#(#tuple_fields,)*) = cx.args()?;
                let res = #name(#context_arg #(#arg_names),*);
                #map_res

                neon::types::extract::TryIntoJs::try_into_js(res, &mut cx).map(|v| neon::handle::Handle::upcast(&v))
            }

            static NAME: &str = #export_name;

            neon::types::JsFunction::with_name(cx, NAME, #wrapper_name).map(|v| (
                NAME,
                neon::handle::Handle::upcast(&v),
            ))
        }
    )
    .into()
}
