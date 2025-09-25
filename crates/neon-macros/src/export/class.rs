use proc_macro2::TokenStream;
use quote::quote;
use syn::spanned::Spanned;

pub(crate) mod meta;

pub(super) fn export(meta: meta::Meta, input: syn::ItemImpl) -> proc_macro::TokenStream {
    // Extract the class name from the impl block
    let class_ident = match extract_class_ident(&input) {
        Ok(ident) => ident,
        Err(err) => return err.into_compile_error().into(),
    };

    // Generate the class using the existing class implementation
    let class_input: proc_macro::TokenStream = quote!(#input).into();
    let class_output = crate::class::class(proc_macro::TokenStream::new(), class_input);
    let class_tokens: TokenStream = class_output.into();

    // Determine the export name
    let export_name = meta.name.map(|name| quote!(#name)).unwrap_or_else(|| {
        let class_name = class_ident.to_string();
        quote!(#class_name)
    });

    // Create the export registration function
    let create_name = quote::format_ident!("__NEON_EXPORT_CREATE__{}", class_ident);
    let create_fn = quote!(
        #[doc(hidden)]
        #[allow(non_snake_case)]
        #[neon::macro_internal::linkme::distributed_slice(neon::macro_internal::EXPORTS)]
        #[linkme(crate = neon::macro_internal::linkme)]
        fn #create_name<'cx>(
            cx: &mut neon::context::ModuleContext<'cx>,
        ) -> neon::result::NeonResult<(&'static str, neon::handle::Handle<'cx, neon::types::JsValue>)> {
            use neon::object::Class;
            static NAME: &str = #export_name;

            #class_ident::constructor(cx).map(|v| (
                NAME,
                neon::handle::Handle::upcast(&v),
            ))
        }
    );

    // Combine the class implementation with the export registration
    quote!(
        #class_tokens
        #create_fn
    )
    .into()
}

// Extract the class identifier from an impl block
fn extract_class_ident(input: &syn::ItemImpl) -> syn::Result<syn::Ident> {
    match &*input.self_ty {
        syn::Type::Path(syn::TypePath {
            path: syn::Path { segments, .. },
            ..
        }) => {
            let syn::PathSegment { ident, .. } = segments
                .last()
                .ok_or_else(|| syn::Error::new(input.self_ty.span(), "Expected type name"))?;
            Ok(ident.clone())
        }
        _ => Err(syn::Error::new(
            input.self_ty.span(),
            "Class export can only be applied to named types",
        )),
    }
}
