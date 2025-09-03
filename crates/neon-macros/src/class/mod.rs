use proc_macro2::TokenStream;
use syn::{spanned::Spanned, Ident, ImplItemFn, Type};

struct ClassItems {
    consts: Vec<syn::ImplItemConst>,
    fns: Vec<syn::ImplItemFn>,
    constructor: Option<syn::ImplItemFn>,
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
    let impl_block = syn::parse_macro_input!(item as syn::ItemImpl);
    let impl_block_clone = impl_block.clone();

    // Parse the item as an implementation block
    let syn::ItemImpl {
        self_ty,
        items,
        ..
    } = impl_block;

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

    for f in fns {
        let fn_name = f.sig.ident.to_string();
        method_names.push_str(&format!(", {fn_name}Method"));
        // TODO: auto-JSify names, with attribute for optionally controlling
        prototype_patches.push_str(&format!("\n    prototype.{fn_name} = {fn_name}Method;"));
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

    // Generate the impl of `neon::object::Class` for the struct
    let impl_class: TokenStream = quote::quote! {
        impl neon::object::Class for #class_ident {
            fn name() -> String {
                stringify!(#class_ident).into()
            }

            fn current_instance<'cx>(cx: &mut neon::context::Cx<'cx>) -> neon::result::NeonResult<neon::object::ClassInstance<'cx>> {
                use neon::handle::{Handle, Root};
                use neon::object::{ClassInstance, RootClassInstance};
                use neon::thread::LocalKey;
                use neon::types::{JsFunction, JsObject};

                static CLASS_INSTANCE: LocalKey<RootClassInstance> = LocalKey::new();

                CLASS_INSTANCE
                    .get_or_try_init(cx, |cx| Self::generate_instance(cx).map(|v| RootClassInstance {
                        external_constructor: v.external_constructor.root(cx),
                        internal_constructor: v.internal_constructor.root(cx),
                    }))
                    .map(|v| v.to_inner(cx))
            }

            fn constructor<'cx>(cx: &mut neon::context::Cx<'cx>) -> neon::result::JsResult<'cx, neon::types::JsFunction> {
                let instance = Self::current_instance(cx)?;
                Ok(instance.external_constructor)
            }

            fn generate_instance<'cx>(cx: &mut neon::context::Cx<'cx>) -> neon::result::NeonResult<neon::object::ClassInstance<'cx>> {
                use neon::handle::Handle;
                use neon::object::ClassInstance;
                use neon::types::{JsFunction, JsObject};

                let wrap = JsFunction::new(cx, |mut cx| {
                    let (this, #(#ctor_locals,)*): (Handle<JsObject>, #(#ctor_infers),*) = cx.args()?;
                    neon::object::wrap(&mut cx, this, #make_new(#(#ctor_locals),*))?.or_throw(&mut cx)?;
                    Ok(cx.undefined())
                });

                // Create method functions
                #(let #method_ids = JsFunction::new(cx, |mut cx| {{
                    use neon::types::extract::TryIntoJs;
                    let this: Handle<JsObject> = cx.this()?;
                    let instance: &#class_ident = neon::object::unwrap(&mut cx, this)?.or_throw(&mut cx)?;
                    let (#(#method_locals_lists,)*) = cx.args()?;
                    instance.#method_ids(#(#method_locals_lists),*).try_into_js(&mut cx)
                }});)*

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
                Ok(ClassInstance {
                    external_constructor: external,
                    internal_constructor: internal,
                })
            }
        }
    };

    quote::quote! {
        #impl_block_clone
        #impl_class
    }.into()
}
