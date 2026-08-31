use syn::spanned::Spanned;

use crate::export::function::meta::Kind;

pub(crate) mod meta;

pub(super) fn export(meta: meta::Meta, input: syn::ItemFn) -> proc_macro::TokenStream {
    let syn::ItemFn {
        attrs,
        vis,
        mut sig,
        block,
    } = input;

    let name = &sig.ident;

    // Generate the context or channel argument for the function
    let (context_extract, context_arg) = match context_parse(&meta, &sig) {
        Ok(arg) => arg,
        Err(err) => return err.into_compile_error().into(),
    };

    // Extract `this` if necessary
    let has_this = check_this(&meta, &sig, context_arg.is_some());
    let this_arg = has_this.then(|| quote::quote!(this,));
    let this_extract = has_this.then(|| {
        quote::quote!(
            let this = cx.this()?;
            let this = neon::types::extract::TryFromJs::from_js(&mut cx, this)?;
        )
    });

    // Generate an argument list used when calling the original function
    let num_args = count_args(&sig, context_arg.is_some(), has_this);
    let args = (0..num_args).map(|i| quote::format_ident!("a{i}"));

    // Generate the tuple fields used to destructure `cx.args()`. Wrap in `Json` if necessary.
    let tuple_fields = args.clone().map(|name| {
        if meta.json {
            quote::quote!(neon::types::extract::Json(#name))
        } else {
            quote::quote!(#name)
        }
    });

    // Tag whether we should JSON wrap results
    let return_tag = if meta.json {
        quote::format_ident!("NeonJsonTag")
    } else {
        quote::format_ident!("NeonValueTag")
    };

    // Convert the result
    // N.B.: Braces are intentionally included to avoid leaking trait to function body
    let result_extract = quote::quote!({
        use neon::macro_internal::{ToNeonMarker, #return_tag as NeonReturnTag};

        (&res).to_neon_marker::<NeonReturnTag>().neon_into_js(&mut cx, res)
    });

    // Generate the call to the original function
    let call_body = match meta.kind {
        Kind::Async => quote::quote!(
            #context_extract
            #this_extract
            let (#(#tuple_fields,)*) = cx.args()?;
            let fut = #name(#context_arg #this_arg #(#args),*);
            let fut = {
                use neon::macro_internal::{ToNeonMarker, NeonValueTag};

                (&fut).to_neon_marker::<NeonValueTag>().into_neon_result(&mut cx, fut)?
            };

            neon::macro_internal::spawn(&mut cx, fut, |mut cx, res| #result_extract)
        ),
        Kind::AsyncFn => quote::quote!(
            #context_extract
            #this_extract
            let (#(#tuple_fields,)*) = cx.args()?;
            let fut = #name(#context_arg #this_arg #(#args),*);

            neon::macro_internal::spawn(&mut cx, fut, |mut cx, res| #result_extract)
        ),
        Kind::Normal => quote::quote!(
            #context_extract
            #this_extract
            let (#(#tuple_fields,)*) = cx.args()?;
            let res = #name(#context_arg #this_arg #(#args),*);

            #result_extract
        ),
        Kind::Task => quote::quote!(
            #context_extract
            #this_extract
            let (#(#tuple_fields,)*) = cx.args()?;
            let promise = neon::context::Context::task(&mut cx, move || #name(#context_arg #this_arg #(#args),*))
                .promise(|mut cx, res| #result_extract);

            Ok(neon::handle::Handle::upcast(&promise))
        ),
    };

    // Generate the wrapper function
    let wrapper_name = quote::format_ident!("__NEON_EXPORT_WRAPPER__{name}");
    let wrapper_fn = quote::quote!(
        #[doc(hidden)]
        fn #wrapper_name(mut cx: neon::context::FunctionContext) -> neon::result::JsResult<neon::types::JsValue> {
            #call_body
        }
    );

    // Compute the export name string (used by the create function and TS metadata)
    let export_name_str = match &meta.name {
        Some(lit) => lit.value(),
        None => crate::name::to_camel_case(&name.to_string()),
    };
    let export_name = quote::quote!(#export_name_str);

    // Generate the function that is registered to create the function on addon initialization.
    // Braces are included to prevent names from polluting user code.
    let create_name = quote::format_ident!("__NEON_EXPORT_CREATE__{name}");
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

    // Generate TypeScript metadata, but only when the `typescript` feature is
    // enabled (via `neon/typescript` -> `neon-macros/typescript`). When off, we
    // emit no metadata statics, so `#[neon::export]` has zero TypeScript-related
    // cost. `cfg!` is evaluated when neon-macros is compiled; Cargo unifies the
    // feature across the build graph.
    let ts_meta = if cfg!(feature = "typescript") {
        generate_ts_metadata(
            &meta,
            &sig,
            &export_name_str,
            context_arg.is_some(),
            has_this,
        )
    } else {
        proc_macro2::TokenStream::new()
    };

    // Strip #[neon(...)] from parameter attributes (consumed by the macro,
    // not valid Rust if left on the emitted function signature). Must happen
    // after `generate_ts_metadata` so attribute extraction sees them.
    for arg in sig.inputs.iter_mut() {
        if let syn::FnArg::Typed(pat_type) = arg {
            pat_type.attrs.retain(|a| !a.path().is_ident("neon"));
        }
    }

    // Output the original function with the generated `create_fn` inside of it
    quote::quote!(
        #(#attrs) *
        #vis #sig {
            #create_fn
            #ts_meta
            #block
        }
    )
    .into()
}

// Determine the number of arguments to the function
fn count_args(sig: &syn::Signature, has_context: bool, has_this: bool) -> usize {
    let n = sig.inputs.len();

    match (has_context, has_this) {
        (true, true) => n - 2,
        (false, false) => n,
        _ => n - 1,
    }
}

// Generate the context extraction and argument for the function
fn context_parse(
    opts: &meta::Meta,
    sig: &syn::Signature,
) -> syn::Result<(
    Option<proc_macro2::TokenStream>,
    Option<proc_macro2::TokenStream>,
)> {
    match opts.kind {
        // Allow borrowing from context
        Kind::Async | Kind::Normal if check_context(opts, sig)? => {
            Ok((None, Some(quote::quote!(&mut cx,))))
        }

        // Require `'static` arguments
        Kind::AsyncFn | Kind::Task if check_channel(opts, sig)? => Ok((
            Some(quote::quote!(let ch = neon::context::Context::channel(&mut cx);)),
            Some(quote::quote!(ch,)),
        )),

        _ => Ok((None, None)),
    }
}

// Checks if a _sync_ function has a context argument and if it is valid
// * If the `context` attribute is included, must be at least one argument
// * Inferred to be context if named `FunctionContext` or `Cx`
// * Context argument must be a `&mut` reference
// * First argument must not be `Channel`
// * Must not be a `self` receiver
fn check_context(opts: &meta::Meta, sig: &syn::Signature) -> syn::Result<bool> {
    // Extract the first argument
    let ty = match first_arg(opts, sig)? {
        Some(arg) => arg,
        None => return Ok(false),
    };

    // Extract the reference type
    let ty = match &*ty.ty {
        // Tried to use a borrowed Channel
        syn::Type::Reference(ty) if !opts.context && is_channel_type(&ty.elem) => {
            return Err(syn::Error::new(
                ty.elem.span(),
                "Expected `&mut Cx` instead of a `Channel` reference.",
            ))
        }

        syn::Type::Reference(ty) => ty,

        // Context needs to be a reference
        _ if opts.context || is_context_type(&ty.ty) => {
            return Err(syn::Error::new(
                ty.ty.span(),
                "Context must be a `&mut` reference.",
            ))
        }

        // Hint that `Channel` should be swapped for `&mut Cx`
        _ if is_channel_type(&ty.ty) => {
            return Err(syn::Error::new(
                ty.ty.span(),
                "Expected `&mut Cx` instead of `Channel`.",
            ))
        }

        _ => return Ok(false),
    };

    // Not a forced or inferred context
    if !opts.context && !is_context_type(&ty.elem) {
        return Ok(false);
    }

    // Context argument must be mutable
    if ty.mutability.is_none() {
        return Err(syn::Error::new(ty.span(), "Must be a `&mut` reference."));
    }

    // All tests passed!
    Ok(true)
}

// Checks if a _async_ function has a Channel argument and if it is valid
// * If the `context` attribute is included, must be at least one argument
// * Inferred to be channel if named `Channel`
// * Channel argument must not be a reference
// * First argument must not be `FunctionContext` or `Cx`
// * Must not be a `self` receiver
fn check_channel(opts: &meta::Meta, sig: &syn::Signature) -> syn::Result<bool> {
    // Extract the first argument
    let ty = match first_arg(opts, sig)? {
        Some(arg) => arg,
        None => return Ok(false),
    };

    // Check the type
    match &*ty.ty {
        // Provided `&mut Channel` instead of `Channel`
        syn::Type::Reference(ty) if opts.context || is_channel_type(&ty.elem) => {
            Err(syn::Error::new(
                ty.span(),
                "Expected an owned `Channel` instead of a reference.",
            ))
        }

        // Provided a `&mut Cx` instead of a `Channel`
        syn::Type::Reference(ty) if is_context_type(&ty.elem) => Err(syn::Error::new(
            ty.elem.span(),
            "Expected an owned `Channel` instead of a context reference.",
        )),

        // Found a `Channel`
        _ if opts.context || is_channel_type(&ty.ty) => Ok(true),

        // Tried to use an owned `Cx`
        _ if is_context_type(&ty.ty) => Err(syn::Error::new(
            ty.ty.span(),
            "Context is not available in async functions. Try a `Channel` instead.",
        )),

        _ => Ok(false),
    }
}

// Extract the first argument, that may be a context, of a function
fn first_arg<'a>(
    opts: &meta::Meta,
    sig: &'a syn::Signature,
) -> syn::Result<Option<&'a syn::PatType>> {
    // Extract the first argument
    let arg = match sig.inputs.first() {
        Some(arg) => arg,

        // If context was forced, error to let the user know the mistake
        None if opts.context => {
            return Err(syn::Error::new(
                sig.inputs.span(),
                "Expected a context argument. Try removing the `context` attribute.",
            ))
        }

        None => return Ok(None),
    };

    // Expect a typed pattern; self receivers are not supported
    match arg {
        syn::FnArg::Typed(ty) => Ok(Some(ty)),
        syn::FnArg::Receiver(arg) => Err(syn::Error::new(
            arg.span(),
            "Exported functions cannot receive `self`.",
        )),
    }
}

fn is_context_type(ty: &syn::Type) -> bool {
    let ident = match type_path_ident(ty) {
        Some(ident) => ident,
        None => return false,
    };

    ident == "FunctionContext" || ident == "Cx"
}

fn is_channel_type(ty: &syn::Type) -> bool {
    let ident = match type_path_ident(ty) {
        Some(ident) => ident,
        None => return false,
    };

    ident == "Channel"
}

// Extract the identifier from the last segment of a type's path
fn type_path_ident(ty: &syn::Type) -> Option<&syn::Ident> {
    let segment = match ty {
        syn::Type::Path(ty) => ty.path.segments.last()?,
        _ => return None,
    };

    Some(&segment.ident)
}

// Determine if the function has a `this` argument. It will be either the `0th` element
// or, if a context argument is included, the `1st`.
fn check_this(opts: &meta::Meta, sig: &syn::Signature, has_context: bool) -> bool {
    static THIS: &str = "this";

    // Forced `this` argument
    if opts.this {
        return true;
    }

    // Get the first argument, skipping context
    let first = if has_context {
        sig.inputs.iter().nth(1)
    } else {
        sig.inputs.first()
    };

    // No other arguments; return early
    let first = match first {
        Some(first) => first,
        None => return false,
    };

    // Ignore `self` type receivers; those aren't used for `this`
    let ty = match first {
        syn::FnArg::Receiver(_) => return false,
        syn::FnArg::Typed(ty) => ty,
    };

    // Check for `this` ident or a tuple struct
    let pat = match &*ty.pat {
        syn::Pat::Ident(ident) if ident.ident == THIS => return true,
        syn::Pat::TupleStruct(pat) => pat,
        _ => return false,
    };

    // Expect exactly one element in the tuple struct
    let elem = match pat.elems.first() {
        Some(elem) if pat.elems.len() == 1 => elem,
        _ => return false,
    };

    // Must be an identifier named `this`
    match elem {
        syn::Pat::Ident(ident) => ident.ident == THIS,
        _ => false,
    }
}

// Extract the parameter name from a pattern
fn param_name(pat: &syn::Pat, index: usize) -> String {
    match pat {
        // Simple identifier: `name: String`
        syn::Pat::Ident(ident) => crate::name::to_camel_case(&ident.ident.to_string()),
        // Tuple struct destructuring: `Json(data): Json<T>`
        syn::Pat::TupleStruct(ts) => {
            if let Some(inner) = ts.elems.first() {
                param_name(inner, index)
            } else {
                format!("arg{index}")
            }
        }
        _ => format!("arg{index}"),
    }
}

use crate::name::type_needs_fallback;

// Generate TypeScript metadata for a function export.
//
// Emits a `static` registered into the `TYPE_METADATA` distributed slice, which
// `neon::typescript::generate()` walks at runtime. For
//
//   #[neon::export]
//   fn ts_add(a: f64, b: f64) -> f64 { a + b }
//
// the emitted metadata (roughly) is:
//
//   ExportMeta::Function(FunctionMeta {
//       name: "tsAdd",
//       params: &[ParamMeta { name: "a", ts_type: || ..f64.., .. }, /* b */],
//       ret_type: || ..f64.. ,
//       is_async: false,
//       ..
//   })
//
// where each `ts_type` closure resolves the TS type at runtime (via the probe,
// so a type without a `TypeScript` impl becomes `"any"` rather than a compile
// error — unless `ts_strict` is set). Returns empty tokens when `ts_skip` is set.
fn generate_ts_metadata(
    meta: &meta::Meta,
    sig: &syn::Signature,
    export_name: &str,
    has_context: bool,
    has_this: bool,
) -> proc_macro2::TokenStream {
    if meta.ts_skip {
        return proc_macro2::TokenStream::new();
    }

    let fn_name = &sig.ident;

    // Determine how many leading args to skip (context, this)
    let skip = match (has_context, has_this) {
        (true, true) => 2,
        (false, false) => 0,
        _ => 1,
    };

    // Collect parameter metadata
    let user_params: Vec<_> = sig
        .inputs
        .iter()
        .skip(skip)
        .enumerate()
        .filter_map(|(i, arg)| match arg {
            syn::FnArg::Typed(pat_type) => Some((i, pat_type)),
            _ => None,
        })
        .collect();

    let param_entries: Vec<proc_macro2::TokenStream> = user_params
        .iter()
        .map(|(i, pat_type)| {
            let name_str = param_name(&pat_type.pat, *i);

            // Per-param ts_type override
            if let Some(override_ty) = crate::name::extract_param_ts_type(&pat_type.attrs) {
                return quote::quote!(
                    neon::typescript::ParamMeta {
                        name: #name_str,
                        ts_type: || std::borrow::Cow::Borrowed(#override_ty),
                        ts_collect: |_| {},
                    }
                );
            }

            // If the type has impl Trait, Self, or unsized slices, fall back
            // to `any`. Non-`'static` lifetimes are handled by substitution below.
            if type_needs_fallback(&pat_type.ty) {
                return quote::quote!(
                    neon::typescript::ParamMeta {
                        name: #name_str,
                        ts_type: || std::borrow::Cow::Borrowed("any"),
                        ts_collect: |_| {},
                    }
                );
            }

            // Rewrite non-static lifetimes to 'static so the type can be used
            // as a TsProbe type parameter in static metadata.
            let ty = crate::name::substitute_lifetimes_with_static(&pat_type.ty);

            // If json mode, the actual extraction type is Json<T>, so TS type comes from
            // Json<T> which delegates to T. We wrap the type accordingly.
            let ts_ty = if meta.json {
                quote::quote!(neon::types::extract::Json<#ty>)
            } else {
                quote::quote!(#ty)
            };

            // Use the autoref specialization probe by default (types without
            // TypeScript impls silently fall back to "any"). In strict mode,
            // call the trait directly — missing impl is a compile error.
            if meta.ts_strict {
                quote::quote!(
                    neon::typescript::ParamMeta {
                        name: #name_str,
                        ts_type: || <#ts_ty as neon::typescript::TypeScript>::ts_type(),
                        ts_collect: |decls| <#ts_ty as neon::typescript::TypeScript>::ts_collect(decls),
                    }
                )
            } else {
                quote::quote!(
                    neon::typescript::ParamMeta {
                        name: #name_str,
                        ts_type: || {
                            use neon::macro_internal::TsFallback as _;
                            let __probe = neon::macro_internal::TsProbe::<#ts_ty>(std::marker::PhantomData);
                            (&__probe).ts_type_of()
                        },
                        ts_collect: |decls| {
                            use neon::macro_internal::TsFallback as _;
                            let __probe = neon::macro_internal::TsProbe::<#ts_ty>(std::marker::PhantomData);
                            (&__probe).ts_collect_of(decls)
                        },
                    }
                )
            }
        })
        .collect();

    let is_async = matches!(meta.kind, Kind::Task | Kind::AsyncFn | Kind::Async);
    let meta_name = quote::format_ident!("__NEON_TS_META__{fn_name}");

    // Extract return type, using the probe for graceful fallback
    let (ret_type_expr, ret_collect_expr) = if let Some(override_ty) = &meta.ts_returns {
        (
            quote::quote!(|| std::borrow::Cow::Borrowed(#override_ty)),
            quote::quote!(|_| {}),
        )
    } else {
        match &sig.output {
            syn::ReturnType::Default => (
                quote::quote!(|| std::borrow::Cow::Borrowed("undefined")),
                quote::quote!(|_| {}),
            ),
            syn::ReturnType::Type(_, ty) if type_needs_fallback(ty) => (
                quote::quote!(|| std::borrow::Cow::Borrowed("any")),
                quote::quote!(|_| {}),
            ),
            syn::ReturnType::Type(_, ty) => {
                let ty = crate::name::substitute_lifetimes_with_static(ty);
                let ret_ty = if meta.json {
                    quote::quote!(neon::types::extract::Json<#ty>)
                } else {
                    quote::quote!(#ty)
                };
                if meta.ts_strict {
                    (
                        quote::quote!(|| <#ret_ty as neon::typescript::TypeScript>::ts_type()),
                        quote::quote!(|decls| <#ret_ty as neon::typescript::TypeScript>::ts_collect(decls)),
                    )
                } else {
                    (
                        quote::quote!(|| {
                            use neon::macro_internal::TsFallback as _;
                            let __probe = neon::macro_internal::TsProbe::<#ret_ty>(std::marker::PhantomData);
                            (&__probe).ts_type_of()
                        }),
                        quote::quote!(|decls| {
                            use neon::macro_internal::TsFallback as _;
                            let __probe = neon::macro_internal::TsProbe::<#ret_ty>(std::marker::PhantomData);
                            (&__probe).ts_collect_of(decls)
                        }),
                    )
                }
            }
        }
    };

    quote::quote!({
        #[neon::macro_internal::linkme::distributed_slice(neon::macro_internal::TYPE_METADATA)]
        #[linkme(crate = neon::macro_internal::linkme)]
        static #meta_name: neon::typescript::ExportMeta =
            neon::typescript::ExportMeta::Function(neon::typescript::FunctionMeta {
                name: #export_name,
                params: &[#(#param_entries),*],
                ret_type: #ret_type_expr,
                ret_collect: #ret_collect_expr,
                is_async: #is_async,
            });
    })
}
