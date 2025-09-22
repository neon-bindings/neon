mod meta;

use proc_macro2::TokenStream;
use syn::{spanned::Spanned, Ident, ImplItemFn, Type};

struct ClassItems {
    consts: Vec<syn::ImplItemConst>,
    fns: Vec<syn::ImplItemFn>,
    constructor: Option<syn::ImplItemFn>,
}

fn generate_method_wrapper(
    method_id: &syn::Ident,
    method_locals: &[syn::Ident],
    method_meta: &meta::Meta,
    class_ident: &syn::Ident,
) -> TokenStream {
    // Generate the tuple fields used to destructure `cx.args()`. Wrap in `Json` if necessary.
    let tuple_fields = method_locals.iter().map(|name| {
        if method_meta.json {
            quote::quote!(neon::types::extract::Json(#name))
        } else {
            quote::quote!(#name)
        }
    });

    // Tag whether we should JSON wrap results
    let return_tag = if method_meta.json {
        quote::format_ident!("NeonJsonTag")
    } else {
        quote::format_ident!("NeonValueTag")
    };

    // Generate result conversion based on JSON setting
    let result_extract = quote::quote!({
        use neon::macro_internal::{ToNeonMarker, #return_tag as NeonReturnTag};
        (&res).to_neon_marker::<NeonReturnTag>().neon_into_js(&mut cx, res)
    });

    match method_meta.kind {
        meta::Kind::Async => {
            quote::quote! {
                JsFunction::new(cx, |mut cx| {
                    let this: neon::handle::Handle<neon::types::JsObject> = cx.this()?;
                    let instance: &#class_ident = neon::object::unwrap(&mut cx, this)?.or_throw(&mut cx)?;

                    // Extract arguments with JSON wrapping if needed
                    let (#(#tuple_fields,)*) = cx.args()?;

                    // Call the method with &self - developer controls cloning in their impl
                    let fut = instance.#method_id(#(#method_locals),*);
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
                    let this: neon::handle::Handle<neon::types::JsObject> = cx.this()?;
                    let instance: &#class_ident = neon::object::unwrap(&mut cx, this)?.or_throw(&mut cx)?;

                    // Extract arguments with JSON wrapping if needed
                    let (#(#tuple_fields,)*) = cx.args()?;

                    // Clone the instance to move into async fn (takes self by value)
                    let instance_clone = instance.clone();

                    // Call the async fn method - it takes self by value to produce 'static Future
                    let fut = instance_clone.#method_id(#(#method_locals),*);

                    neon::macro_internal::spawn(&mut cx, fut, |mut cx, res| #result_extract)
                })
            }
        }
        meta::Kind::Task => {
            // For task methods, we need to move a clone into the closure
            // since tasks run on a different thread
            quote::quote! {
                JsFunction::new(cx, |mut cx| {
                    let this: neon::handle::Handle<neon::types::JsObject> = cx.this()?;
                    let instance: &#class_ident = neon::object::unwrap(&mut cx, this)?.or_throw(&mut cx)?;

                    // Clone the instance since we need to move it into the task
                    let instance_clone = instance.clone();

                    // Extract arguments with JSON wrapping if needed
                    let (#(#tuple_fields,)*) = cx.args()?;

                    let promise = neon::context::Context::task(&mut cx, move || {
                        instance_clone.#method_id(#(#method_locals),*)
                    })
                    .promise(|mut cx, res| #result_extract);
                    Ok(promise.upcast::<neon::types::JsValue>())
                })
            }
        }
        meta::Kind::Normal => {
            quote::quote! {
                JsFunction::new(cx, |mut cx| {
                    let this: neon::handle::Handle<neon::types::JsObject> = cx.this()?;
                    let instance: &#class_ident = neon::object::unwrap(&mut cx, this)?.or_throw(&mut cx)?;
                    // Extract arguments with JSON wrapping if needed
                    let (#(#tuple_fields,)*) = cx.args()?;
                    let res = instance.#method_id(#(#method_locals),*);
                    #result_extract
                })
            }
        }
    }
}

fn sort_class_items(
    items: Vec<syn::ImplItem>,
) -> Result<ClassItems, syn::Error> {
    let mut consts = Vec::new();
    let mut fns = Vec::new();
    let mut constructor = None;

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

    Ok(ClassItems { consts, fns, constructor })
}

pub(crate) fn class(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let mut impl_block = syn::parse_macro_input!(item as syn::ItemImpl);

    // Parse the item as an implementation block
    let syn::ItemImpl {
        self_ty,
        items,
        ..
    } = impl_block.clone();

    let class_ident = match *self_ty {
        syn::Type::Path(syn::TypePath { path: syn::Path { segments, .. }, .. }) => {
            let syn::PathSegment { ident, .. } = segments.last().unwrap();
            ident.clone()
        }
        _ => { panic!("class must be implemented for a type name"); }
    };
    let class_name = class_ident.to_string();

    // Sort the items into `const` and `fn` categories\
    // TODO: turn consts into static class properties
    let ClassItems { consts: _consts, fns, constructor } = match sort_class_items(items.clone()) {
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
                        return syn::Error::new_spanned(arg, "constructor cannot have a self receiver")
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
        Some(ImplItemFn { sig, .. }) => {
            sig.inputs.iter().enumerate().map(|(i, arg)| {
                Ident::new(&format!("neon_tmp_{i}"), arg.span().clone())
            }).collect::<Vec<_>>()
        }
        None => {
            vec![]
        }
    };

    let ctor_infers: Vec<Type> = match &constructor {
        Some(ImplItemFn { sig, .. }) => {
            sig.inputs.iter().map(|_| Type::Infer(syn::TypeInfer { underscore_token: Default::default() })).collect::<Vec<_>>()
        }
        None => {
            vec![]
        }
    };

    let ctor_param_list = ctor_params.join(", ");
    let ctor_arg_list = format!("{}{}", if ctor_params.is_empty() { "" } else { ", " }, ctor_param_list);

    fn starts_with_self_arg(sig: &syn::Signature) -> bool {
        if let Some(first_arg) = sig.inputs.first() {
            matches!(first_arg, syn::FnArg::Receiver(_))
        } else {
            false
        }
    }

    let method_locals_lists = fns.iter().map(|f| {
        if !starts_with_self_arg(&f.sig) {
            panic!("class methods must have a &self receiver");
        }

        f.sig.inputs.iter().skip(1).enumerate().map(|(i, arg)| {
            Ident::new(&format!("neon_tmp_{i}"), arg.span().clone())
        }).collect::<Vec<_>>()
    }).collect::<Vec<_>>();

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

        for syn::Attribute { meta, .. } in &f.attrs {
            match meta {
                syn::Meta::List(syn::MetaList {
                    path,
                    tokens,
                    ..
                }) if path.is_ident("neon") => {
                    // TODO: if parsed.is_some() error
                    let parser = meta::Parser;
                    let tokens = tokens.clone().into();
                    parsed = syn::parse_macro_input!(tokens with parser);
                }
                // syn::Meta::NameValue(syn::MetaNameValue {
                //     path,
                //     value: syn::Expr::Lit(syn::ExprLit {
                //         lit: syn::Lit::Str(value), ..
                //     }),
                //     ..
                // }) if path.is_ident("name") => {
                //     // TODO: if meta.is_some() error
                //     parsed.name = Some(value);
                // }
                _ => {
                    // TODO: error: unrecognized attribute
                }
            }
        }
        let js_name = match parsed.name.clone() {
            Some(name) => name.value(),
            None => crate::name::to_camel_case(&fn_name),
        };
        prototype_patches.push_str(&format!("\n    prototype.{js_name} = {fn_name}Method;"));
        method_metas.push(parsed);
    }

    let script = format!(r#"
(function makeClass(wrap{method_names}) {{
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
"#);

    let make_new: TokenStream = if constructor.is_some() {
        quote::quote! { #class_ident::new }
    } else {
        quote::quote! { <Self as ::std::default::Default>::default }
    };

    // Generate method wrappers based on their metadata
    let method_wrappers: Vec<TokenStream> = method_ids.iter()
        .zip(&method_locals_lists)
        .zip(&method_metas)
        .map(|((id, locals), meta)| {
            generate_method_wrapper(id, locals, meta, &class_ident)
        })
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
                    neon::object::wrap(&mut cx, this, #make_new(#(#ctor_locals),*))?.or_throw(&mut cx)?;
                    Ok(cx.undefined())
                });

                // Create method functions using the appropriate wrapper based on metadata
                #(let #method_ids = #method_wrappers;)*

                const CLASS_MAKER_SCRIPT: &str = #script;
                let src = cx.string(CLASS_MAKER_SCRIPT);
                let factory: Handle<JsFunction> = neon::reflect::eval(cx, src)?
                    .downcast(cx)
                    .or_throw(cx)?;
                let pair: Handle<JsObject> = factory
                    .bind(cx)
                    .arg(wrap)?
                    #( .arg(#method_ids)? )*
                    .call()?;
                let external: Handle<JsFunction> = pair.prop(cx, "external").get()?;
                let internal: Handle<JsFunction> = pair.prop(cx, "internal").get()?;
                Ok(neon::macro_internal::new_class_metadata(external, internal))
            }
        }
    };

    // Remove #[neon(...)] attributes from methods in the impl block
    for item in &mut impl_block.items {
        if let syn::ImplItem::Fn(f) = item {
            f.attrs.retain(|attr| {
                !matches!(&attr.meta, syn::Meta::List(list) if list.path.is_ident("neon"))
            });
        }
    }

    quote::quote! {
        #impl_block
        #impl_class
    }.into()
}
