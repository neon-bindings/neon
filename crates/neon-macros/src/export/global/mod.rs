pub(crate) mod meta;

// Create a new block expression for the RHS of an assignment
pub(super) fn export(meta: meta::Meta, name: &syn::Ident, expr: Box<syn::Expr>) -> Box<syn::Expr> {
    // Name for the registered create function
    let create_name = quote::format_ident!("__NEON_EXPORT_CREATE__{name}");

    // Default export name as identity unless a name is provided
    let export_name = meta
        .name
        .map(|name| quote::quote!(#name))
        .unwrap_or_else(|| quote::quote!(stringify!(#name)));

    // If `json` is enabled, wrap the value in `Json` before `TryIntoJs` is called
    let value = meta
        .json
        .then(|| quote::quote!(neon::types::extract::Json(&#name)))
        .unwrap_or_else(|| quote::quote!(#name));

    // Generate the function that is registered to create the global on addon initialization.
    // Braces are included to prevent names from polluting user code.
    //
    // N.B.: The `linkme(..)` attribute informs the `distributed_slice(..)` macro where
    // to find the `linkme` crate. It is re-exported from neon to avoid dependents from
    // needing to adding a direct dependency on `linkme`. It is an undocumented feature.
    // https://github.com/dtolnay/linkme/issues/54
    let create_fn = quote::quote!({
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
    });

    // Create a block to hold the original expression and the registered crate function
    let expr = quote::quote!({
        #create_fn
        #expr
    });

    // Create an expression from the token stream
    Box::new(syn::Expr::Verbatim(expr))
}
