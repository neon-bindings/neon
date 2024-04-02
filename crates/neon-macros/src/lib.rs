//! Procedural macros supporting [Neon](https://docs.rs/neon/latest/neon/)

mod export;

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
    let syn::ItemFn {
        attrs,
        vis,
        sig,
        block,
    } = syn::parse_macro_input!(item as syn::ItemFn);

    let name = &sig.ident;
    let export_name = quote::format_ident!("__NEON_MAIN__{name}");
    let export_fn = quote::quote!({
        #[neon::macro_internal::linkme::distributed_slice(neon::macro_internal::MAIN)]
        #[linkme(crate = neon::macro_internal::linkme)]
        fn #export_name(cx: neon::context::ModuleContext) -> neon::result::NeonResult<()> {
            #name(cx)
        }
    });

    quote::quote!(
        #(#attrs) *
        #vis #sig {
            #export_fn
            #block
        }
    )
    .into()
}

#[proc_macro_attribute]
/// Register an item to be exported by the Neon addon
///
/// ## Exporting constants and statics
///
/// ```ignore
/// #[neon::export]
/// static GREETING: &str = "Hello, Neon!";
///
/// #[neon::export]
/// const ANSWER: u8 = 42;
/// ```
///
/// ### Renaming an export
///
/// By default, items will be exported with their Rust name. Exports may
/// be renamed by providing the `name` attribute.
///
/// ```ignore
/// #[neon::export(name = "myGreeting")]
/// static GREETING: &str = "Hello, Neon!";
/// ```
///
/// ### JSON exports
///
/// Complex values may be exported by automatically serializing to JSON and
/// parsing in JavaScript. Any type that implements `serde::Serialize` may be used.
///
/// ```ignore
/// #[neon::export]
/// static MESSAGES: &[&str] = &["hello", "goodbye"];
/// ```
///
/// ## Exporting functions
///
/// Functions may take any type that implements [`neon::types::extract::TryFromJs`] as
/// an argument and return any type that implements [`neon::types::extract::TryIntoJs`].
///
/// ```ignore
/// #[neon::export]
/// fn add(a: f64, b: f64) -> f64 {
///     a + b
/// }
/// ```
///
/// ### Interact with the JavaScript runtime
///
/// More complex functions may need to interact directly with the JavaScript runtime,
/// for example with [`neon::context::Context`] or handles to JavaScript values.
///
/// Functions may optionally include a [`neon::context::FunctionContext`] argument. Note
/// that unlike functions created with [`neon::types::JsFunction::new`], exported function
/// receive a borrowed context and may require explicit lifetimes.
///
/// ```ignore
/// #[neon::export]
/// fn add<'cx>(
///     cx: &mut FunctionContext<'cx>,
///     a: Handle<JsNumber>,
///     b: Handle<JsNumber>,
/// ) -> JsResult<'cx, JsNumber> {
///     let a = a.value(cx);
///     let b = b.value(cx);
///
///     Ok(cx.number(a + b))
/// }
/// ```
///
/// ### Exporting a function that uses JSON
///
/// The [`neon::types::extract::Json`] wrapper allows ergonomically handling complex
/// types that implement `serde::Deserialize` and `serde::Serialize`.
///
/// ```ignore
/// #[neon::export]
/// fn sort(Json(mut items): Json<Vec<String>>) -> Json<Vec<String>> {
///     items.sort();
///     Json(items)
/// }
/// ```
///
/// As a convenience, macro uses may add the `json` attribute to automatically
/// wrap arguments and return values with `Json`.
///
/// ```ignore
/// #[neon::export]
/// fn sort(mut items: Vec<String>) -> Vec<String> {
///     items.sort();
///     items
/// }
/// ```
///
/// ### Tasks
///
/// Neon provides an API for spawning tasks to execute asynchronously on Node's worker
/// pool. JavaScript may await a promise for completion of the task.
///
/// ```ignore
/// #[neon::export]
/// fn add<'cx>(cx: FunctionContext<'cx>, a: f64, b: f64) -> JsResult<'cx, JsPromise> {
///     let promise = cx
///         .task(move || a + b)
///         .promise(|mut cx, res| Ok(cx.number(res)));
///
///     Ok(promise)
/// }
/// ```
///
/// As a convenience, macro users may indicate that a function should be executed
/// asynchronously on the worker pool by adding the `task` attribute.
///
/// ```ignore
/// #[neon::export(task)]
/// fn add(a: f64, b: f64) -> f64 {
///     a + b
/// }
/// ```
pub fn export(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    export::export(attr, item)
}
