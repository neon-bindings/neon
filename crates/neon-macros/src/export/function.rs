use crate::export::meta::ExportMeta;

pub(super) fn export(meta: ExportMeta, input: syn::ItemFn) -> proc_macro::TokenStream {
    let syn::ItemFn {
        attrs,
        vis,
        sig,
        block,
    } = input;

    let name = &sig.ident;

    // Name for the registered create function
    let create_name = quote::format_ident!("__NEON_EXPORT_CREATE__{name}");

    // Name for the function that is wrapped by `JsFunction`. Delegates to the original.
    let wrapper_name = quote::format_ident!("__NEON_EXPORT_WRAPPER__{name}");

    // Default export name as identity unless a name is provided
    let export_name = meta
        .name
        .map(|name| quote::quote!(#name))
        .unwrap_or_else(|| quote::quote!(stringify!(#name)));

    // Determine if the first argument is `FunctionContext` and retain it if necessary
    let has_context = meta.context || has_context_arg(&sig);
    let context_arg = has_context.then(|| quote::quote!(&mut cx,));

    // Generate an argument list used when calling the original function
    let start = if has_context { 1 } else { 0 };
    let args = (start..sig.inputs.len()).map(|i| quote::format_ident!("a{i}"));

    // Generate the tuple fields used to destructure `cx.args()`. Wrap in `Json` if necessary.
    let tuple_fields = args.clone().map(|name| {
        meta.json
            .then(|| quote::quote!(neon::types::extract::Json(#name)))
            .unwrap_or_else(|| quote::quote!(#name))
    });

    // If necessary, wrap the return value in `Json` before calling `TryIntoJs`
    let json_return = meta.json.then(|| {
        is_result_output(&sig.output)
            // Use `.map(Json)` on a `Result`
            .then(|| quote::quote!(let res = res.map(neon::types::extract::Json);))
            // Wrap other values with `Json(res)`
            .unwrap_or_else(|| quote::quote!(let res = neon::types::extract::Json(res);))
    });

    // Generate the wrapper function that delegates to the original
    let wrapper_fn = quote::quote!(
        #[doc(hidden)]
        fn #wrapper_name(mut cx: neon::context::FunctionContext) -> neon::result::JsResult<neon::types::JsValue> {
            let (#(#tuple_fields,)*) = cx.args()?;
            let res = #name(#context_arg #(#args),*);
            #json_return

            neon::types::extract::TryIntoJs::try_into_js(res, &mut cx).map(|v| neon::handle::Handle::upcast(&v))
        }
    );

    // Generate the function that is registered to create the function on addon initialization.
    // Braces are included to prevent names from polluting user code.
    let create_fn = quote::quote!({
        #[doc(hidden)]
        #[neon::macro_internal::linkme::distributed_slice(neon::macro_internal::EXPORTS)]
        #[linkme(crate = neon::macro_internal::linkme)]
        fn #create_name<'cx>(
            cx: &mut neon::context::ModuleContext<'cx>,
        ) -> neon::result::NeonResult<(&'static str, neon::handle::Handle<'cx, neon::types::JsValue>)> {
            static NAME: &str = #export_name;

            #wrapper_fn

            neon::types::JsFunction::with_name(cx, NAME, #wrapper_name).map(|v| (
                NAME,
                neon::handle::Handle::upcast(&v),
            ))
        }
    });

    // Output the original function with the generated `create_fn` inside of it
    quote::quote!(
        #(#attrs) *
        #vis #sig {
            #create_fn
            #block
        }
    )
    .into()
}

// Determine if a function signature's first argument is `FunctionContext`
fn has_context_arg(sig: &syn::Signature) -> bool {
    sig.inputs.first().map(is_context_arg).unwrap_or(false)
}

// Determine if an argument is named `FunctionContext`
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

// Determine if a return type is a `Result`
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
