mod meta;

use super::name::is_valid_js_identifier;
use proc_macro2::TokenStream;
use syn::{spanned::Spanned, Ident, ImplItemFn, Type};

struct ClassItems {
    consts: Vec<syn::ImplItemConst>,
    fns: Vec<syn::ImplItemFn>,
    constructor: Option<syn::ImplItemFn>,
    has_finalizer: bool,
}

// Check if a method receiver is mutable (&mut self)
fn is_receiver_mutable(sig: &syn::Signature) -> bool {
    if let Some(syn::FnArg::Receiver(receiver)) = sig.inputs.first() {
        receiver.mutability.is_some()
    } else {
        false
    }
}

// Enum to track what kind of parameter we're dealing with
enum ParamKind {
    Value,
    Ref(Box<syn::Type>), // The inner type of the reference (e.g., Point for &Point)
    RefMut(Box<syn::Type>), // The inner type of the mutable reference (e.g., Point for &mut Point)
}

impl ParamKind {
    fn from_type(ty: &syn::Type) -> Self {
        match ty {
            syn::Type::Reference(type_ref) => {
                // Any reference to a path type (e.g., &Point, &mut Point, &Message)
                if let syn::Type::Path(_) = &*type_ref.elem {
                    if type_ref.mutability.is_some() {
                        ParamKind::RefMut(type_ref.elem.clone())
                    } else {
                        ParamKind::Ref(type_ref.elem.clone())
                    }
                } else {
                    ParamKind::Value
                }
            }
            _ => ParamKind::Value,
        }
    }
}

// Extract parameter types from signature, skipping self, context, and this
fn extract_param_types(sig: &syn::Signature, has_context: bool, has_this: bool) -> Vec<ParamKind> {
    let skip_count = 1 + (has_context as usize) + (has_this as usize);

    sig.inputs
        .iter()
        .skip(skip_count)
        .filter_map(|arg| match arg {
            syn::FnArg::Typed(pat_type) => Some(ParamKind::from_type(&pat_type.ty)),
            syn::FnArg::Receiver(_) => None,
        })
        .collect()
}

fn generate_method_wrapper(
    meta: &meta::Meta,
    class_id: &syn::Ident,
    sig: &syn::Signature,
) -> TokenStream {
    // Validate method attributes
    if let Err(err) = validate_method_attributes(meta, sig) {
        return err.into_compile_error();
    }

    let name = &sig.ident;
    let is_mut = is_receiver_mutable(sig);

    // Check for context parameter and generate context extraction/argument
    let (context_extract, context_arg) = match context_parse(meta, sig) {
        Ok((extract, arg)) => (extract, arg),
        Err(err) => return err.into_compile_error(),
    };

    // Determine if method has context parameter
    let has_context = context_arg.is_some();

    // Check for this parameter
    let has_this = check_this(meta, sig, has_context);

    // Generate this extraction if needed
    let this_extract = if has_this {
        quote::quote!(
            let js_this: neon::handle::Handle<neon::types::JsObject> = cx.this()?;
            let this = neon::types::extract::TryFromJs::from_js(&mut cx, neon::handle::Handle::upcast(&js_this))?;
        )
    } else {
        quote::quote!()
    };

    // Generate this argument for method call
    let this_arg = if has_this {
        quote::quote!(this,)
    } else {
        quote::quote!()
    };

    // Analyze parameter types to determine which are references
    let param_types = extract_param_types(sig, has_context, has_this);
    let num_args = param_types.len();

    // Generate tuple fields for cx.args() extraction and type annotations
    let (tuple_fields, ref_type_annotations): (Vec<_>, Vec<_>) = param_types
        .iter()
        .enumerate()
        .map(|(i, param_kind)| {
            let name = quote::format_ident!("a{i}");
            match param_kind {
                ParamKind::Ref(_) | ParamKind::RefMut(_) => {
                    // For reference parameters, extract as a generic arg then type annotate
                    let obj_name = quote::format_ident!("a{i}_obj");
                    let annotation = quote::quote! {
                        let #obj_name: neon::handle::Handle<neon::types::JsObject> = #obj_name;
                    };
                    (quote::quote!(#obj_name), Some(annotation))
                }
                ParamKind::Value => {
                    // For value parameters, use existing logic with JSON wrapping if needed
                    let field = meta
                        .json
                        .then(|| quote::quote!(neon::types::extract::Json(#name)))
                        .unwrap_or_else(|| quote::quote!(#name));
                    (field, None)
                }
            }
        })
        .unzip();

    // Generate guard extraction code for reference parameters
    let ref_guards = param_types.iter().enumerate().filter_map(|(i, param_kind)| {
        match param_kind {
            ParamKind::Ref(inner_type) => {
                let obj_name = quote::format_ident!("a{i}_obj");
                let guard_name = quote::format_ident!("_guard_a{i}");
                let arg_name = quote::format_ident!("a{i}");

                Some(quote::quote! {
                    let #guard_name = <#inner_type as neon::types::extract::TryFromJsRef>::from_js_ref(
                        &mut cx,
                        neon::handle::Handle::upcast(&#obj_name)
                    )?;
                    let #arg_name: &#inner_type = &*#guard_name;
                })
            }
            ParamKind::RefMut(inner_type) => {
                let obj_name = quote::format_ident!("a{i}_obj");
                let guard_name = quote::format_ident!("_guard_a{i}");
                let arg_name = quote::format_ident!("a{i}");

                Some(quote::quote! {
                    let mut #guard_name = <#inner_type as neon::types::extract::TryFromJsRefMut>::from_js_ref_mut(
                        &mut cx,
                        neon::handle::Handle::upcast(&#obj_name)
                    )?;
                    let #arg_name: &mut #inner_type = &mut *#guard_name;
                })
            }
            ParamKind::Value => None,
        }
    });

    // Generate argument list for method call
    let args = (0..num_args).map(|i| quote::format_ident!("a{i}"));

    // Tag whether we should JSON wrap results
    let return_tag = if meta.json {
        quote::format_ident!("NeonJsonTag")
    } else {
        quote::format_ident!("NeonValueTag")
    };

    // Generate result conversion based on JSON setting
    let result_extract = quote::quote!({
        use neon::macro_internal::{ToNeonMarker, #return_tag as NeonReturnTag};
        (&res).to_neon_marker::<NeonReturnTag>().neon_into_js(&mut cx, res)
    });

    match meta.kind {
        meta::Kind::Async => {
            let borrow_call = if is_mut {
                quote::quote! {
                    let mut instance = instance_cell.borrow_mut();
                    let instance = &mut *instance;
                }
            } else {
                quote::quote! {
                    let instance = instance_cell.borrow();
                    let instance = &*instance;
                }
            };

            quote::quote! {
                JsFunction::new(cx, |mut cx| {
                    let js_this: neon::handle::Handle<neon::types::JsObject> = cx.this()?;
                    let instance_cell: &std::cell::RefCell<#class_id> = neon::object::unwrap(&mut cx, js_this)?.or_throw(&mut cx)?;

                    // Context extraction if needed
                    #context_extract

                    // This extraction if needed
                    #this_extract

                    // Extract non-context/this arguments with JSON wrapping if needed
                    let (#(#tuple_fields,)*) = cx.args()?;

                    // Type annotations for reference parameters
                    #(#ref_type_annotations)*

                    // Unwrap reference parameters and create guards
                    #(#ref_guards)*

                    // Borrow from RefCell
                    #borrow_call

                    // Call the method with &self or &mut self - developer controls cloning in their impl
                    let fut = instance.#name(#context_arg #this_arg #(#args),*);
                    // Always use NeonValueTag for Future conversion, JSON only applies to final result
                    let fut = {
                        use neon::macro_internal::{ToNeonMarker, NeonValueTag};
                        (&fut).to_neon_marker::<NeonValueTag>().into_neon_result(&mut cx, fut)?
                    };
                    neon::macro_internal::spawn(&mut cx, fut, |mut cx, res| #result_extract)
                })
            }
        }
        meta::Kind::AsyncFn => {
            quote::quote! {
                JsFunction::new(cx, |mut cx| {
                    let js_this: neon::handle::Handle<neon::types::JsObject> = cx.this()?;
                    let instance_cell: &std::cell::RefCell<#class_id> = neon::object::unwrap(&mut cx, js_this)?.or_throw(&mut cx)?;

                    // Context extraction if needed
                    #context_extract

                    // This extraction if needed
                    #this_extract

                    // Extract non-context/this arguments with JSON wrapping if needed
                    let (#(#tuple_fields,)*) = cx.args()?;

                    // Type annotations for reference parameters
                    #(#ref_type_annotations)*

                    // Unwrap reference parameters and create guards
                    #(#ref_guards)*

                    // Clone the instance to move into async fn (takes self by value)
                    let instance_clone = instance_cell.borrow().clone();

                    // Call the async fn method - it takes self by value to produce 'static Future
                    let fut = instance_clone.#name(#context_arg #this_arg #(#args),*);

                    neon::macro_internal::spawn(&mut cx, fut, |mut cx, res| #result_extract)
                })
            }
        }
        meta::Kind::Task => {
            // For task methods, we need to move a clone into the closure
            // since tasks run on a different thread
            let borrow_call = if is_mut {
                quote::quote! {
                    let mut instance = instance_clone;
                    let instance = &mut instance;
                }
            } else {
                quote::quote! {
                    let instance = &instance_clone;
                }
            };

            quote::quote! {
                JsFunction::new(cx, |mut cx| {
                    let js_this: neon::handle::Handle<neon::types::JsObject> = cx.this()?;
                    let instance_cell: &std::cell::RefCell<#class_id> = neon::object::unwrap(&mut cx, js_this)?.or_throw(&mut cx)?;

                    // Context extraction if needed
                    #context_extract

                    // This extraction if needed
                    #this_extract

                    // Clone the instance since we need to move it into the task
                    let instance_clone = instance_cell.borrow().clone();

                    // Extract non-context/this arguments with JSON wrapping if needed
                    let (#(#tuple_fields,)*) = cx.args()?;

                    // Type annotations for reference parameters
                    #(#ref_type_annotations)*

                    // Unwrap reference parameters and create guards
                    #(#ref_guards)*

                    let promise = neon::context::Context::task(&mut cx, move || {
                        #borrow_call
                        instance.#name(#context_arg #this_arg #(#args),*)
                    })
                    .promise(|mut cx, res| #result_extract);
                    Ok(promise.upcast::<neon::types::JsValue>())
                })
            }
        }
        meta::Kind::Normal => {
            let borrow_call = if is_mut {
                quote::quote! {
                    let mut instance = instance_cell.borrow_mut();
                    let instance = &mut *instance;
                }
            } else {
                quote::quote! {
                    let instance = instance_cell.borrow();
                    let instance = &*instance;
                }
            };

            quote::quote! {
                JsFunction::new(cx, |mut cx| {
                    let js_this: neon::handle::Handle<neon::types::JsObject> = cx.this()?;
                    let instance_cell: &std::cell::RefCell<#class_id> = neon::object::unwrap(&mut cx, js_this)?.or_throw(&mut cx)?;

                    // Context extraction if needed
                    #context_extract

                    // This extraction if needed
                    #this_extract

                    // Extract non-context/this arguments with JSON wrapping if needed
                    let (#(#tuple_fields,)*) = cx.args()?;

                    // Type annotations for reference parameters
                    #(#ref_type_annotations)*

                    // Unwrap reference parameters and create guards
                    #(#ref_guards)*

                    // Borrow from RefCell
                    #borrow_call

                    let res = instance.#name(#context_arg #this_arg #(#args),*);
                    #result_extract
                })
            }
        }
    }
}

// Generate context extraction and argument for class methods
fn context_parse(
    opts: &meta::Meta,
    sig: &syn::Signature,
) -> syn::Result<(
    Option<proc_macro2::TokenStream>,
    Option<proc_macro2::TokenStream>,
)> {
    match opts.kind {
        // Allow borrowing from context
        meta::Kind::Async | meta::Kind::Normal if check_context(opts, sig)? => {
            Ok((None, Some(quote::quote!(&mut cx,))))
        }

        // Require `'static` arguments
        meta::Kind::AsyncFn | meta::Kind::Task if check_channel(opts, sig)? => Ok((
            Some(quote::quote!(let ch = neon::context::Context::channel(&mut cx);)),
            Some(quote::quote!(ch,)),
        )),

        _ => Ok((None, None)),
    }
}

// Check if a sync method has a context argument (adapted from export function)
// Key difference from #[export]: methods have &self as first param, so context is second param
fn check_context(opts: &meta::Meta, sig: &syn::Signature) -> syn::Result<bool> {
    // Extract the first argument (after &self)
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
                "Context parameters must be a `&mut` reference. Try `&mut FunctionContext` or `&mut Cx`.",
            ))
        }

        // Hint that `Channel` should be swapped for `&mut Cx`
        _ if is_channel_type(&ty.ty) => {
            return Err(syn::Error::new(
                ty.ty.span(),
                "Unexpected `Channel` in sync method. Use `&mut FunctionContext` for sync methods, or `Channel` in async/task methods.",
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

// Check if an async method has a Channel argument (adapted from export function)
fn check_channel(opts: &meta::Meta, sig: &syn::Signature) -> syn::Result<bool> {
    // Extract the first argument (after &self)
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

// Extract the first argument (after &self) from a method signature
fn first_arg<'a>(
    opts: &meta::Meta,
    sig: &'a syn::Signature,
) -> syn::Result<Option<&'a syn::PatType>> {
    // Extract the second argument (skip &self)
    let arg = match sig.inputs.iter().nth(1) {
        Some(arg) => arg,

        // If context was forced, error to let the user know the mistake
        None if opts.context => {
            return Err(syn::Error::new(
                sig.inputs.span(),
                "Expected a context argument after `&self` when using `#[neon(context)]`. Add a parameter like `cx: &mut FunctionContext` or remove the `context` attribute.",
            ))
        }

        None => return Ok(None),
    };

    // Expect a typed pattern; self receivers are not supported (but shouldn't appear here)
    match arg {
        syn::FnArg::Typed(ty) => Ok(Some(ty)),
        syn::FnArg::Receiver(arg) => Err(syn::Error::new(
            arg.span(),
            "Unexpected second receiver argument.",
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

// Validate method attributes for common errors and conflicts
fn validate_method_attributes(meta: &meta::Meta, sig: &syn::Signature) -> syn::Result<()> {
    // Check for conflicting async attributes
    if matches!(meta.kind, meta::Kind::AsyncFn) && matches!(meta.kind, meta::Kind::Async) {
        return Err(syn::Error::new(
            sig.span(),
            "Cannot combine `async fn` with `#[neon(async)]` attribute",
        ));
    }

    // Check for async + task conflict
    if matches!(meta.kind, meta::Kind::AsyncFn | meta::Kind::Async)
        && matches!(meta.kind, meta::Kind::Task)
    {
        return Err(syn::Error::new(
            sig.span(),
            "Cannot combine async method with `#[neon(task)]` attribute",
        ));
    }

    // Validate that async fn methods take self by value if they're detected as AsyncFn
    if matches!(meta.kind, meta::Kind::AsyncFn) {
        if let Some(syn::FnArg::Receiver(receiver)) = sig.inputs.first() {
            if receiver.reference.is_some() {
                // This is &self or &mut self, but for AsyncFn we need self by value
                if sig.asyncness.is_some() {
                    return Err(syn::Error::new(
                        receiver.span(),
                        "Async functions in classes must take `self` by value, not `&self` or `&mut self`. This is required because async functions capture `self` in the Future, which must be `'static` for spawning."
                    ));
                }
            }
        } else {
            return Err(syn::Error::new(
                sig.span(),
                "Async functions in classes must take `self` as their first parameter.",
            ));
        }
    }

    // Check for self parameter in constructor
    if sig.ident == "new" {
        if let Some(syn::FnArg::Receiver(_)) = sig.inputs.first() {
            return Err(syn::Error::new(
                sig.ident.span(),
                "Constructor methods cannot have a `self` parameter",
            ));
        }
    } else {
        fn starts_with_self_arg(sig: &syn::Signature) -> bool {
            if let Some(first_arg) = sig.inputs.first() {
                matches!(first_arg, syn::FnArg::Receiver(_))
            } else {
                false
            }
        }

        // Check for self parameter in non-constructor methods
        if !starts_with_self_arg(sig) {
            return Err(syn::Error::new(
                sig.ident.span(),
                "Class methods must have a `self` receiver (`&self` or `&mut self`) as their first parameter",
            ));
        }
    }

    Ok(())
}

// Check if a method has a `this` parameter (adapted from export function)
// For methods: &self is 1st, context is 2nd (optional), this is 3rd (or 2nd if no context)
fn check_this(opts: &meta::Meta, sig: &syn::Signature, has_context: bool) -> bool {
    static THIS: &str = "this";

    // Forced `this` argument
    if opts.this {
        return true;
    }

    // Get the parameter after &self and optional context
    let param_index = if has_context { 2 } else { 1 }; // Skip &self and optional context
    let param = match sig.inputs.iter().nth(param_index) {
        Some(param) => param,
        None => return false,
    };

    // Ignore `self` type receivers (shouldn't happen at this index, but be safe)
    let ty = match param {
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

fn group_class_items(items: Vec<syn::ImplItem>) -> Result<ClassItems, syn::Error> {
    let mut consts = Vec::new();
    let mut fns = Vec::new();
    let mut constructor = None;
    let mut has_finalizer = false;

    for item in items {
        match item {
            syn::ImplItem::Const(c) => consts.push(c),
            syn::ImplItem::Fn(f) => {
                // Check if the function is a constructor
                if f.sig.ident == "new" {
                    if constructor.is_some() {
                        let span = syn::spanned::Spanned::span(&f);
                        let msg = "Only one `new` constructor is allowed in a class.";
                        return Err(syn::Error::new(span, msg));
                    }
                    constructor = Some(f);
                    continue; // Skip adding to fns
                } else if f.sig.ident == "finalize" {
                    if has_finalizer {
                        let span = syn::spanned::Spanned::span(&f);
                        let msg = "Only one `finalize` method is allowed in a class.";
                        return Err(syn::Error::new(span, msg));
                    }
                    has_finalizer = true;
                    continue; // Skip adding to fns
                }
                fns.push(f)
            }
            _ => {
                let span = syn::spanned::Spanned::span(&item);
                let msg = "`neon::class` can only contain `const` and `fn` items.";
                return Err(syn::Error::new(span, msg));
            }
        }
    }

    Ok(ClassItems {
        consts,
        fns,
        constructor,
        has_finalizer,
    })
}

pub(crate) fn class(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let mut impl_block = syn::parse_macro_input!(item as syn::ItemImpl);

    // Parse the item as an implementation block
    let syn::ItemImpl { self_ty, items, .. } = impl_block.clone();

    let class_ident = match *self_ty {
        syn::Type::Path(syn::TypePath {
            path: syn::Path { segments, .. },
            ..
        }) => {
            let syn::PathSegment { ident, .. } = segments.last().unwrap();
            ident.clone()
        }
        _ => {
            panic!("class must be implemented for a type name");
        }
    };
    let class_name = class_ident.to_string();

    // Group the items into `const` and `fn` categories
    let ClassItems {
        consts,
        fns,
        constructor,
        has_finalizer,
    } = match group_class_items(items.clone()) {
        Ok(items) => items,
        Err(err) => {
            // If sorting fails, return the error as a compile error
            return err.to_compile_error().into();
        }
    };

    let ctor_params = match &constructor {
        Some(ImplItemFn { sig, .. }) => {
            let mut params = Vec::new();
            for (i, arg) in sig.inputs.iter().enumerate() {
                match arg {
                    syn::FnArg::Typed(pat_type) => {
                        params.push(match pat_type.pat.as_ref() {
                            syn::Pat::Ident(ident) => ident.ident.to_string(),
                            // Rust identifiers can't begin with '$' so this can't conflict
                            // with any user-provided identifiers.
                            _ => format!("$arg{}", i),
                        });
                    }
                    syn::FnArg::Receiver(_) => {
                        return syn::Error::new_spanned(
                            arg,
                            "constructor cannot have a self receiver",
                        )
                        .to_compile_error()
                        .into();
                    }
                }
            }
            params
        }
        None => {
            vec![]
        }
    };

    let ctor_locals: Vec<Ident> = match &constructor {
        Some(ImplItemFn { sig, .. }) => sig
            .inputs
            .iter()
            .enumerate()
            .map(|(i, arg)| Ident::new(&format!("a{i}"), arg.span()))
            .collect::<Vec<_>>(),
        None => {
            vec![]
        }
    };

    let ctor_infers: Vec<Type> = match &constructor {
        Some(ImplItemFn { sig, .. }) => sig
            .inputs
            .iter()
            .map(|_| {
                Type::Infer(syn::TypeInfer {
                    underscore_token: Default::default(),
                })
            })
            .collect::<Vec<_>>(),
        None => {
            vec![]
        }
    };

    let ctor_param_list = ctor_params.join(", ");
    let ctor_arg_list = format!(
        "{}{}",
        if ctor_params.is_empty() { "" } else { ", " },
        ctor_param_list
    );

    let mut method_names = String::new();
    let mut prototype_patches = String::new();
    let method_ids = fns.iter().map(|f| f.sig.ident.clone()).collect::<Vec<_>>();
    let mut method_metas = Vec::new();

    for f in &fns {
        let fn_name = f.sig.ident.to_string();
        method_names.push_str(&format!(", {fn_name}Method"));
        // syn::parse_macro_input!(attr with parser);
        let mut parsed = meta::Meta::default();

        // Check if the function itself is async
        if f.sig.asyncness.is_some() {
            parsed.kind = meta::Kind::AsyncFn;
        }

        let mut found_neon_attr = false;
        for syn::Attribute { meta, .. } in &f.attrs {
            match meta {
                syn::Meta::List(syn::MetaList { path, tokens, .. }) if path.is_ident("neon") => {
                    if found_neon_attr {
                        return syn::Error::new_spanned(
                            meta,
                            "multiple #[neon(...)] attributes on class method are not allowed",
                        )
                        .to_compile_error()
                        .into();
                    }
                    found_neon_attr = true;
                    let parser = meta::Parser(parsed);
                    let tokens = tokens.clone().into();
                    parsed = syn::parse_macro_input!(tokens with parser);
                }
                _ => {}
            }
        }
        let js_name = match parsed.name.clone() {
            Some(name) => name.value(),
            None => crate::name::to_camel_case(&fn_name),
        };
        prototype_patches.push_str(&format!("\n    prototype.{js_name} = {fn_name}Method;"));
        method_metas.push(parsed);
    }

    // Process const items into static class properties
    let mut property_names = Vec::new();
    let mut property_assignments = Vec::new();
    let mut property_ids = Vec::new();
    let mut property_wrappers = Vec::new();
    let mut used_js_names = std::collections::HashSet::with_capacity(consts.len()); // Pre-allocate

    for const_item in &consts {
        let const_name = &const_item.ident;
        let property_id = quote::format_ident!("__{const_name}Property");

        // Parse property attributes
        let mut property_meta = meta::PropertyMeta::default();
        let mut found_neon_attr = false;
        for attr in &const_item.attrs {
            if let syn::Meta::List(syn::MetaList { path, tokens, .. }) = &attr.meta {
                if path.is_ident("neon") {
                    if found_neon_attr {
                        return syn::Error::new_spanned(
                            attr,
                            "multiple #[neon(...)] attributes on const property are not allowed",
                        )
                        .to_compile_error()
                        .into();
                    }
                    found_neon_attr = true;
                    let parser = meta::PropertyParser;
                    let tokens = tokens.clone().into();
                    property_meta = syn::parse_macro_input!(tokens with parser);
                }
            }
        }

        // Determine JavaScript property name (use custom name or default)
        let js_property_name = match &property_meta.name {
            Some(name) => {
                let name_value = name.value();
                // Validate JavaScript identifier
                if !is_valid_js_identifier(&name_value) {
                    return syn::Error::new_spanned(
                        name,
                        format!("'{}' is not a valid JavaScript identifier", name_value),
                    )
                    .to_compile_error()
                    .into();
                }
                name_value
            }
            None => const_name.to_string(),
        };

        // Check for name collisions
        if !used_js_names.insert(js_property_name.clone()) {
            return syn::Error::new_spanned(
                const_item,
                format!("duplicate property name '{}' - const property names must be unique in JavaScript", js_property_name)
            ).to_compile_error().into();
        }

        // Add to parameter list for JavaScript function
        property_names.push(property_id.to_string());

        // Add property assignment in JavaScript (immutable const properties)
        property_assignments.push(format!(
            "\n    Object.defineProperty({class_name}, '{js_property_name}', {{ value: {property_id}(), enumerable: true }});"
        ));

        // Create property getter function with JSON support
        let value_expr = if property_meta.json {
            quote::quote! {
                use neon::types::extract::{TryIntoJs, Json};
                let value = Json(#class_ident::#const_name);
                value.try_into_js(&mut cx).map(|v| v.upcast())
            }
        } else {
            quote::quote! {
                use neon::types::extract::TryIntoJs;
                let value = #class_ident::#const_name;
                value.try_into_js(&mut cx).map(|v| v.upcast())
            }
        };

        let property_wrapper = quote::quote! {
            neon::types::JsFunction::new(cx, |mut cx| -> neon::result::JsResult<neon::types::JsValue> {
                #value_expr
            })
        };

        property_ids.push(property_id);
        property_wrappers.push(property_wrapper);
    }

    let property_params = if property_names.is_empty() {
        String::new()
    } else {
        format!(", {}", property_names.join(", "))
    };

    let property_assignments = property_assignments.join("");

    let script = format!(
        r#"
(function makeClass(wrap{method_names}{property_params}) {{
  // Create the class exposed directly to JavaScript.
  //
  // The variables listed in method_names all come from
  // Rust identifiers, so they cannot start with '$'.
  function $makeExternal() {{
    class {class_name} {{
      constructor({ctor_param_list}) {{
        wrap(this{ctor_arg_list});
      }}
    }}
    const prototype = {class_name}.prototype;{prototype_patches}

    // Add static class properties{property_assignments}

    return {class_name};
  }}

  // Create the constructor used by Neon internally to construct
  // instances from a pre-existing Rust struct. Calling this
  // constructor directly will not create a valid instance.
  // Neon must wrap the resulting object with the Rust struct
  // before it can be handed back to JavaScript to be used.
  //
  // The variables listed in method_names all come from
  // Rust identifiers, so they cannot start with '$'.
  function $makeInternal(prototype) {{
    function {class_name}() {{ }}
    {class_name}.prototype = prototype;
    return {class_name};
  }}

  // The variables listed in method_names all come from
  // Rust identifiers, so they cannot start with '$'.
  const $external = $makeExternal();
  const $internal = $makeInternal($external.prototype);

  return {{
    external: $external,
    internal: $internal
  }};
}})
"#
    );

    let make_new: TokenStream = if constructor.is_some() {
        quote::quote! { #class_ident::new }
    } else {
        quote::quote! { <Self as ::std::default::Default>::default }
    };

    // Generate method wrappers based on their metadata
    let method_wrappers: Vec<TokenStream> = fns
        .iter()
        .zip(&method_metas)
        .map(|(f, meta)| generate_method_wrapper(meta, &class_ident, &f.sig))
        .collect();

    // Generate the impl of `neon::object::Class` for the struct
    let impl_class: TokenStream = quote::quote! {
        impl neon::object::Class for #class_ident {
            fn name() -> String {
                stringify!(#class_ident).into()
            }

            fn local<'cx>(cx: &mut neon::context::Cx<'cx>) -> neon::result::NeonResult<neon::object::ClassMetadata<'cx>> {
                use neon::handle::{Handle, Root};
                use neon::object::{ClassMetadata, RootClassMetadata};
                use neon::thread::LocalKey;
                use neon::types::{JsFunction, JsObject};

                static CLASS_METADATA: LocalKey<RootClassMetadata> = LocalKey::new();

                CLASS_METADATA
                    .get_or_try_init(cx, |cx| Self::create(cx).map(|v| v.root(cx)))
                    .map(|v| v.to_inner(cx))
            }

            fn constructor<'cx>(cx: &mut neon::context::Cx<'cx>) -> neon::result::JsResult<'cx, neon::types::JsFunction> {
                Ok(Self::local(cx)?.constructor())
            }

            fn create<'cx>(cx: &mut neon::context::Cx<'cx>) -> neon::result::NeonResult<neon::object::ClassMetadata<'cx>> {
                use neon::handle::Handle;
                use neon::types::{JsFunction, JsObject};

                let wrap = JsFunction::new(cx, |mut cx| {
                    let (this, #(#ctor_locals,)*): (Handle<JsObject>, #(#ctor_infers),*) = cx.args()?;
                    let instance = #make_new(#(#ctor_locals),*);
                    neon::object::wrap(&mut cx, this, std::cell::RefCell::new(instance))?.or_throw(&mut cx)?;
                    Ok(cx.undefined())
                });

                // Create method functions using the appropriate wrapper based on metadata
                #(let #method_ids = #method_wrappers;)*

                // Create property getter functions
                #(let #property_ids = #property_wrappers;)*

                const CLASS_MAKER_SCRIPT: &str = #script;
                let src = cx.string(CLASS_MAKER_SCRIPT);
                let factory: Handle<JsFunction> = neon::reflect::eval(cx, src)?
                    .downcast(cx)
                    .or_throw(cx)?;
                let pair: Handle<JsObject> = factory
                    .bind(cx)
                    .arg(wrap)?
                    #( .arg(#method_ids)? )*
                    #( .arg(#property_ids)? )*
                    .call()?;
                let external: Handle<JsFunction> = pair.prop(cx, "external").get()?;
                let internal: Handle<JsFunction> = pair.prop(cx, "internal").get()?;
                Ok(neon::macro_internal::new_class_metadata(external, internal))
            }
        }
    };

    let impl_finalize: TokenStream = if has_finalizer {
        quote::quote! {
            impl neon::types::Finalize for #class_ident {
                fn finalize<'a, C: neon::context::Context<'a>>(self, cx: &mut C) {
                    Self::finalize(self, cx)
                }
            }
        }
    } else {
        quote::quote! {
            impl neon::types::Finalize for #class_ident {
                fn finalize<'a, C: neon::context::Context<'a>>(self, _cx: &mut C) {}
            }
        }
    };

    let impl_sealed: TokenStream = quote::quote! {
        impl neon::macro_internal::Sealed for #class_ident {}
    };

    // This leverages the following hack: https://github.com/rust-lang/rust/issues/48214#issuecomment-2557829956
    // Adding an artificial `for<'a>` generalization circumvents the need for the unstable `trivial_bounds` feature.
    // This allows us to conditionally require `Clone` only for types that actually make use of `TryFromJs`.
    let impl_try_from_js: TokenStream = quote::quote! {
        impl<'cx> neon::types::extract::TryFromJs<'cx> for #class_ident where for<'a> Self: Clone {
            type Error = neon::types::extract::ObjectExpected;

            fn try_from_js(cx: &mut neon::context::Cx<'cx>, value: neon::handle::Handle<'cx, neon::types::JsValue>) -> neon::result::NeonResult<Result<Self, Self::Error>> {
                use neon::result::ResultExt;

                match neon::object::unwrap::<std::cell::RefCell<Self>, _>(cx, value) {
                    Ok(Ok(instance_cell)) => Ok(Ok(Self::clone(&*instance_cell.borrow()))),
                    _ => Ok(Err(neon::macro_internal::object_expected(<Self as neon::object::Class>::name()))),
                }
            }
        }
    };

    let impl_try_into_js: TokenStream = quote::quote! {
        impl<'cx> neon::types::extract::TryIntoJs<'cx> for #class_ident {
            type Value = neon::types::JsObject;

            fn try_into_js(self, cx: &mut neon::context::Cx<'cx>) -> neon::result::JsResult<'cx, Self::Value> {
                use neon::result::ResultExt;

                let object: neon::handle::Handle<neon::types::JsObject> =
                    neon::macro_internal::internal_constructor::<Self>(cx)?.bind(cx).construct()?;
                neon::macro_internal::object::wrap(cx, object, std::cell::RefCell::new(self))?.or_throw(cx)?;
                Ok(object)
            }
        }
    };

    let impl_try_from_js_ref: TokenStream = quote::quote! {
        impl<'cx> neon::types::extract::TryFromJsRef<'cx> for #class_ident {
            type Guard = std::cell::Ref<'cx, Self>;
            type Error = neon::types::extract::ObjectExpected;

            fn try_from_js_ref(
                cx: &mut neon::context::Cx<'cx>,
                value: neon::handle::Handle<'cx, neon::types::JsValue>
            ) -> neon::result::NeonResult<Result<Self::Guard, Self::Error>> {
                use neon::result::ResultExt;

                match neon::object::unwrap::<std::cell::RefCell<Self>, _>(cx, value) {
                    Ok(Ok(instance_cell)) => Ok(Ok(instance_cell.borrow())),
                    _ => Ok(Err(neon::macro_internal::object_expected(<Self as neon::object::Class>::name()))),
                }
            }
        }
    };

    let impl_try_from_js_ref_mut: TokenStream = quote::quote! {
        impl<'cx> neon::types::extract::TryFromJsRefMut<'cx> for #class_ident {
            type Guard = std::cell::RefMut<'cx, Self>;
            type Error = neon::types::extract::ObjectExpected;

            fn try_from_js_ref_mut(
                cx: &mut neon::context::Cx<'cx>,
                value: neon::handle::Handle<'cx, neon::types::JsValue>
            ) -> neon::result::NeonResult<Result<Self::Guard, Self::Error>> {
                use neon::result::ResultExt;

                match neon::object::unwrap::<std::cell::RefCell<Self>, _>(cx, value) {
                    Ok(Ok(instance_cell)) => Ok(Ok(instance_cell.borrow_mut())),
                    _ => Ok(Err(neon::macro_internal::object_expected(<Self as neon::object::Class>::name()))),
                }
            }
        }
    };

    // Remove #[neon(...)] attributes from methods and const items in the impl block
    for item in &mut impl_block.items {
        match item {
            syn::ImplItem::Fn(f) => {
                f.attrs.retain(
                    |attr| !matches!(&attr.meta, syn::Meta::List(list) if list.path.is_ident("neon")),
                );
            }
            syn::ImplItem::Const(c) => {
                c.attrs.retain(
                    |attr| !matches!(&attr.meta, syn::Meta::List(list) if list.path.is_ident("neon")),
                );
            }
            _ => {}
        }
    }

    quote::quote! {
        #impl_block
        #impl_class
        #impl_finalize
        #impl_sealed
        #impl_try_from_js
        #impl_try_into_js
        #impl_try_from_js_ref
        #impl_try_from_js_ref_mut
    }
    .into()
}
