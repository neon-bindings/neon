pub(crate) mod meta;

// Create a new block expression for the RHS of an assignment
pub(super) fn export(
    meta: meta::Meta,
    name: &syn::Ident,
    ty: &syn::Type,
    expr: Box<syn::Expr>,
) -> Box<syn::Expr> {
    // Name for the registered create function
    let create_name = quote::format_ident!("__NEON_EXPORT_CREATE__{name}");

    // Default export name as identity unless a name is provided
    let export_name = meta
        .name
        .as_ref()
        .map(|name| quote::quote!(#name))
        .unwrap_or_else(|| quote::quote!(stringify!(#name)));

    // If `json` is enabled, wrap the value in `Json` before `TryIntoJs` is called
    let value = if meta.json {
        quote::quote!(neon::types::extract::Json(&#name))
    } else {
        quote::quote!(#name)
    };

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
            use neon::types::extract::TryIntoJs;

            // This leverages autoderef to support several different types:
            // * If value is `&T` and `&T: TryIntoJs`, it will deref `&&T` to `&T`
            // * If value is `T` and `&T: TryIntoJs`, it will call as is
            // * If value is `&T` and `T: TryIntoJs` and `Copy`, it will copy and call
            (&#value).try_into_js(cx).map(|v| (#export_name, neon::handle::Handle::upcast(&v)))
        }
    });

    // TypeScript metadata, emitted only with the `typescript` feature (see the
    // function macro).
    let ts_meta = if cfg!(feature = "typescript") {
        let export_name_str = meta
            .name
            .as_ref()
            .map(|name| name.value())
            .unwrap_or_else(|| name.to_string());
        let meta_name = quote::format_ident!("__NEON_TS_GLOBAL_META__{name}");
        let (ts_type, ts_collect) = crate::ts::global(ty, meta.json);
        quote::quote!({
            #[neon::macro_internal::linkme::distributed_slice(neon::macro_internal::TYPE_METADATA)]
            #[linkme(crate = neon::macro_internal::linkme)]
            static #meta_name: neon::macro_internal::ExportMeta =
                neon::macro_internal::ExportMeta::Global(neon::macro_internal::GlobalMeta {
                    name: #export_name_str,
                    ts_type: #ts_type,
                    ts_collect: #ts_collect,
                });
        })
    } else {
        proc_macro2::TokenStream::new()
    };

    // Create a block to hold the original expression and the registered crate function
    let expr = quote::quote!({
        #create_fn
        #ts_meta
        #expr
    });

    // Create an expression from the token stream
    Box::new(syn::Expr::Verbatim(expr))
}
