//! Helper module to add documentation to macros prior to re-exporting.

/// Marks a function as the main entry point for initialization in
/// a Neon module.
///
/// This attribute should only be used _once_ in a module and will
/// be called each time the module is initialized in a context.
///
/// If a `main` function is not provided, all registered exports will be exported.
///
/// ```
/// # use neon::prelude::*;
/// # fn main() {
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
/// # }
/// ```
pub use neon_macros::main;

/// Register an item to be exported by the Neon addon
///
/// ## Exporting constants and statics
///
/// ```
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
/// ```
/// #[neon::export(name = "myGreeting")]
/// static GREETING: &str = "Hello, Neon!";
/// ```
///
/// ### JSON exports
///
/// Complex values may be exported by automatically serializing to JSON and
/// parsing in JavaScript. Any type that implements `serde::Serialize` may be used.
///
/// ```
/// #[neon::export(json)]
/// static MESSAGES: &[&str] = &["hello", "goodbye"];
/// ```
///
/// ## Exporting functions
///
/// Functions may take any type that implements [`TryFromJs`](crate::types::extract::TryFromJs) as
/// an argument and return any type that implements [`TryIntoJs`](crate::types::extract::TryIntoJs).
///
/// ```
/// #[neon::export]
/// fn add(a: f64, b: f64) -> f64 {
///     a + b
/// }
/// ```
///
/// ### Exporting a function that uses JSON
///
/// The [`Json`](crate::types::extract::Json) wrapper allows ergonomically handling complex
/// types that implement `serde::Deserialize` and `serde::Serialize`.
///
/// ```
/// # use neon::types::extract::Json;
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
/// ```
/// #[neon::export(json)]
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
/// ```
/// # use neon::prelude::*;
/// #[neon::export]
/// fn add<'cx>(cx: &mut FunctionContext<'cx>, a: f64, b: f64) -> JsResult<'cx, JsPromise> {
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
/// ```
/// #[neon::export(task)]
/// fn add(a: f64, b: f64) -> f64 {
///     a + b
/// }
/// ```
///
/// ### Error Handling
///
/// If an exported function returns a [`Result`], a JavaScript exception will be thrown
/// with the [`Err`]. Any error type that implements [`TryIntoJs`](crate::types::extract::TryIntoJs)
/// may be used.
///
/// ```
/// #[neon::export]
/// fn throw(msg: String) -> Result<(), String> {
///     Err(msg)
/// }
/// ```
///
/// The [`Error`](crate::types::extract::Error) type is provided for ergonomic error conversions
/// from most error types using the `?` operator.
///
/// ```
/// use neon::types::extract::Error;
///
/// #[neon::export]
/// fn read_file(path: String) -> Result<String, Error> {
///     let contents = std::fs::read_to_string(path)?;
///     Ok(contents)
/// }
/// ```
///
/// ### Interact with the JavaScript runtime
///
/// More complex functions may need to interact directly with the JavaScript runtime,
/// for example with [`Context`](crate::context::Context) or handles to JavaScript values.
///
/// Functions may optionally include a [`FunctionContext`](crate::context::FunctionContext) argument. Note
/// that unlike functions created with [`JsFunction::new`](crate::types::JsFunction), exported function
/// receive a borrowed context and may require explicit lifetimes.
///
/// ```
/// # use neon::prelude::*;
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
/// ### Advanced
///
/// The following attributes are for advanced configuration and may not be
/// necessary for most users.
///
/// #### `context`
///
/// The `#[neon::export]` macro looks checks if the first argument has a type of
/// `&mut FunctionContext` to determine if the [`Context`](crate::context::Context)
/// should be passed to the function.
///
/// If the type has been renamed when importing, the `context` attribute can be
/// added to force it to be passed.
///
/// ```
/// use neon::context::{FunctionContext as FnCtx};
///
/// #[neon::export(context)]
/// fn add(_cx: &mut FnCtx, a: f64, b: f64) -> f64 {
///     a + b
/// }
/// ```
///
/// ### `result`
///
/// The `#[neon::export]` macro will infer an exported function returns a [`Result`]
/// if the type is named [`Result`], [`NeonResult`](crate::result::NeonResult) or
/// [`JsResult`](crate::result::JsResult).
///
/// If a type alias is used for [`Result`], the `result` attribute can be added to
/// inform the generated code.
///
/// ```
/// use neon::result::{NeonResult as Res};
///
/// fn add(a: f64, b: f64) -> Res<f64> {
///     Ok(a + b)
/// }
/// ```
pub use neon_macros::export;
