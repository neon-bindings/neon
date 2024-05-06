use crate::export::function::meta::Kind;

pub(crate) mod meta;

static TASK_CX_ERROR: &str = "`FunctionContext` is not allowed with `task` attribute";

pub(super) fn export(meta: meta::Meta, input: syn::ItemFn) -> proc_macro::TokenStream {
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

    // Determine if the first argument is `FunctionContext`
    let has_context = match has_context_arg(&meta, &sig) {
        Ok(has_context) => has_context,
        Err(err) => return err.into_compile_error().into(),
    };

    // Retain the context argument, if necessary
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
        is_result_output(&meta, &sig.output)
            // Use `.map(Json)` on a `Result`
            .then(|| quote::quote!(let res = res.map(neon::types::extract::Json);))
            // Wrap other values with `Json(res)`
            .unwrap_or_else(|| quote::quote!(let res = neon::types::extract::Json(res);))
    });

    // Default export name as identity unless a name is provided
    let export_name = meta
        .name
        .map(|name| quote::quote!(#name))
        .unwrap_or_else(|| quote::quote!(stringify!(#name)));

    // Generate the call to the original function
    let call_body = match meta.kind {
        Kind::Normal => quote::quote!(
            let (#(#tuple_fields,)*) = cx.args()?;
            let res = #name(#context_arg #(#args),*);
            #json_return

            neon::types::extract::TryIntoJs::try_into_js(res, &mut cx)
                .map(|v| neon::handle::Handle::upcast(&v))
        ),
        Kind::Task => quote::quote!(
            let (#(#tuple_fields,)*) = cx.args()?;
            let promise = neon::context::Context::task(&mut cx, move || {
                let res = #name(#context_arg #(#args),*);
                #json_return
                res
            })
            .promise(|mut cx, res| neon::types::extract::TryIntoJs::try_into_js(res, &mut cx));

            Ok(neon::handle::Handle::upcast(&promise))
        ),
    };

    // Generate the wrapper function
    let wrapper_fn = quote::quote!(
        #[doc(hidden)]
        fn #wrapper_name(mut cx: neon::context::FunctionContext) -> neon::result::JsResult<neon::types::JsValue> {
            #call_body
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

// Get the ident for the first argument
fn first_arg_ty(sig: &syn::Signature) -> Option<&syn::Ident> {
    let arg = sig.inputs.first()?;
    let ty = match arg {
        syn::FnArg::Receiver(v) => &*v.ty,
        syn::FnArg::Typed(v) => &*v.ty,
    };

    let ty = match ty {
        syn::Type::Reference(ty) => &*ty.elem,
        _ => return None,
    };

    let path = match ty {
        syn::Type::Path(path) => path,
        _ => return None,
    };

    let path = path.path.segments.last()?;

    Some(&path.ident)
}

// Determine if the function has a context argument and if it is allowed
fn has_context_arg(meta: &meta::Meta, sig: &syn::Signature) -> syn::Result<bool> {
    // Forced context argument
    if meta.context {
        return Ok(true);
    }

    // Return early if no arguments
    let first = match first_arg_ty(sig) {
        Some(first) => first,
        None => return Ok(false),
    };

    // First argument isn't context
    if first != "FunctionContext" {
        return Ok(false);
    }

    // Context is only allowed for normal functions
    match meta.kind {
        Kind::Normal => {}
        Kind::Task => return Err(syn::Error::new(first.span(), TASK_CX_ERROR)),
    }

    Ok(true)
}

// Determine if a return type is a `Result`
fn is_result_output(meta: &meta::Meta, ret: &syn::ReturnType) -> bool {
    // Forced result output
    if meta.result {
        return true;
    }

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
