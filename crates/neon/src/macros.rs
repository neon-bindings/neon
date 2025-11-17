//! Helper module to add documentation to macros prior to re-exporting.

#[cfg(feature = "napi-6")]
/// Create a Neon class from a Rust datatype
///
/// The `#[neon::class]` attribute can be applied to an `impl` block to create a JavaScript
/// class that wraps a Rust struct (or enum). The `impl` block specifies a constructor method named
/// `new` to create instances of the struct, which Neon automatically attaches to instances of the
/// JavaScript class during object construction.
///
/// Typically, Neon classes are exported from their addon, which can be done with the
/// [`#[neon::export(class)]`](crate::export) attribute.
///
/// ## Example
///
/// ```
/// # use neon::prelude::*;
/// # use neon::{context::Context, types::Finalize};
/// #[derive(Clone)]
/// pub struct User {
///     username: String,
///     first_name: String,
///     last_name: String,
/// }
///
/// #[neon::export(class)]
/// impl User {
///     pub fn new(username: String, first_name: String, last_name: String) -> Self {
///         Self { username, first_name, last_name }
///     }
///
///     pub fn to_string(&self) -> String {
///         format!("[object User:{}]", self.username)
///     }
/// }
/// ```
///
/// ## Constructor
///
/// Classes must have exactly one constructor method named `new`. The constructor takes
/// the class parameters and returns `Self`. Constructor arguments can be any type that
/// implements [`TryFromJs`](crate::types::extract::TryFromJs).
///
/// ```
/// # use neon::prelude::*;
/// # use neon::types::Finalize;
/// pub struct Person {
///     name: String,
///     age: u32,
/// }
///
/// #[neon::class]
/// impl Person {
///     pub fn new(name: String, age: u32) -> Self {
///         Self { name, age }
///     }
/// }
/// ```
///
/// ## Methods
///
/// Class methods can have either `&self` or `&mut self` as their first parameter.
/// Methods can take any type that implements [`TryFromJs`](crate::types::extract::TryFromJs)
/// and return any type that implements [`TryIntoJs`](crate::types::extract::TryIntoJs).
///
/// ```
/// # use neon::prelude::*;
/// # use neon::types::Finalize;
/// pub struct Counter {
///     value: i32,
/// }
///
/// #[neon::class]
/// impl Counter {
///     pub fn new(initial: i32) -> Self {
///         Self { value: initial }
///     }
///
///     pub fn increment(&mut self) -> i32 {
///         self.value += 1;
///         self.value
///     }
///
///     pub fn get(&self) -> i32 {
///         self.value
///     }
/// }
/// ```
///
/// ### Reference Parameters
///
/// Methods can accept class instances by reference (`&T`) or mutable reference (`&mut T`)
/// to avoid cloning when passing instances between methods. The type must implement
/// [`TryFromJsRef`](crate::types::extract::TryFromJsRef) for immutable references or
/// [`TryFromJsRefMut`](crate::types::extract::TryFromJsRefMut) for mutable references.
/// The `#[neon::class]` macro automatically implements these traits for all class types.
///
/// ```
/// # use neon::prelude::*;
/// # use neon::types::Finalize;
/// #[derive(Clone)]
/// pub struct Point {
///     x: f64,
///     y: f64,
/// }
///
/// #[neon::class]
/// impl Point {
///     pub fn new(x: f64, y: f64) -> Self {
///         Self { x, y }
///     }
///
///     // Accept another Point by immutable reference (no clone)
///     pub fn distance(&self, other: &Self) -> f64 {
///         let dx = self.x - other.x;
///         let dy = self.y - other.y;
///         (dx * dx + dy * dy).sqrt()
///     }
///
///     // Accept another Point by mutable reference (no clone)
///     pub fn swap_coordinates(&mut self, other: &mut Self) {
///         std::mem::swap(&mut self.x, &mut other.x);
///         std::mem::swap(&mut self.y, &mut other.y);
///     }
///
///     // Accept another Point by value (clones the instance)
///     pub fn midpoint(&self, other: Self) -> Self {
///         Self {
///             x: (self.x + other.x) / 2.0,
///             y: (self.y + other.y) / 2.0,
///         }
///     }
/// }
/// ```
///
/// From JavaScript:
/// ```js
/// const p1 = new Point(0, 0);
/// const p2 = new Point(3, 4);
///
/// console.log(p1.distance(p2));    // 5 (no cloning)
///
/// p1.swapCoordinates(p2);          // Mutates both points
/// console.log(p1.x);               // 3
/// console.log(p2.x);               // 0
///
/// const mid = p1.midpoint(p2);     // Clones p2
/// ```
///
/// **When to use references:**
/// - Use `&T` when you only need to read from the instance
/// - Use `&mut T` when you need to mutate the instance
/// - Use `T` (by value) when the semantics require taking ownership
///
/// Note that reference parameters still use [`RefCell`](std::cell::RefCell) internally,
/// so runtime borrow checking applies. Attempting to borrow the same instance both mutably
/// and immutably (or multiple times mutably) will panic.
///
/// ## Finalizer
///
/// Classes can implement a `finalize` method to perform cleanup when the
/// JavaScript object is garbage collected. The `finalize` method takes
/// ownership of the class instance and is called when the object is no longer
/// reachable from JavaScript.
///
/// ```
/// # use neon::prelude::*;
/// pub struct Logger {
///     name: String,
/// }
///
/// #[neon::class]
/// impl Logger {
///     pub fn new(name: String) -> Self {
///         Self { name }
///     }
///
///     pub fn finalize<'cx, C: Context<'cx>>(self, _cx: &mut C) {
///         println!("Logger {} is being finalized", self.name);
///     }
/// }
/// ```
///
/// ## Mutability and Borrow Checking
///
/// Neon classes use [`RefCell`](std::cell::RefCell) internally to allow mutation through
/// `&mut self` methods while maintaining JavaScript's shared ownership semantics. This means
/// that borrow checking happens at runtime, not compile time, and violating Rust's borrowing
/// rules will cause a panic.
///
/// **Important:** You cannot call a method that requires `&mut self` while another method
/// is borrowing the instance (even with `&self`). This includes:
/// - Reentrancy from JavaScript callbacks
/// - Nested method calls on the same instance
///
/// For complex scenarios involving callbacks or shared mutable state across threads,
/// consider using additional interior mutability types like
/// [`Arc<Mutex<T>>`](std::sync::Arc) for the specific fields that need it.
///
/// ### Method Attributes
///
/// Methods support the same attributes as [`#[neon::export]`](crate::export) functions, including
/// `json`, `task`, `async`, `context`, `this`, and `name`.
///
/// #### JSON Methods
///
/// ```
/// # use neon::prelude::*;
/// # use neon::types::Finalize;
/// pub struct DataProcessor;
///
/// #[neon::class]
/// impl DataProcessor {
///     pub fn new() -> Self {
///         Self
///     }
///
///     #[neon(json)]
///     pub fn process_data(&self, items: Vec<String>) -> Vec<String> {
///         items.into_iter().map(|s| s.to_uppercase()).collect()
///     }
/// }
/// ```
///
/// #### Async Methods
///
/// Methods declared with `async fn` are automatically detected and exported as async. Because the
/// data is shared across threads, it is automatically cloned before the method is called, so the
/// receiver must be `self` by value instead of `&self` or `&mut self` and the struct must
/// implement `Clone`. Any shared mutable state should use types like
/// [`Arc<Mutex<T>>`](std::sync::Arc) for thread-safe interior mutability.
///
/// ```
/// # #[cfg(all(feature = "napi-6", feature = "futures"))]
/// # {
/// # use neon::prelude::*;
/// # use neon::types::Finalize;
/// # use std::sync::{Arc, Mutex};
/// # #[derive(Clone)]
/// # pub struct AsyncWorker {
/// #     counter: Arc<Mutex<i32>>,
/// # }
/// #[neon::class]
/// impl AsyncWorker {
///     pub fn new() -> Self {
///         Self {
///             counter: Arc::new(Mutex::new(0)),
///         }
///     }
///
///     // Takes `self` - the struct is cloned before calling
///     pub async fn fetch_data(self, url: String) -> String {
///         // Simulate async work
///         let mut count = self.counter.lock().unwrap();
///         *count += 1;
///         format!("Data from {} (request #{})", url, count)
///     }
/// }
/// # }
/// ```
///
/// ##### Synchronous Setup
///
/// For more control over async behavior, use `#[neon(async)]` with a method that
/// returns a [`Future`](std::future::Future). This allows synchronous setup on
/// the JavaScript main thread.
///
/// ```
/// # #[cfg(all(feature = "napi-6", feature = "futures"))]
/// # {
/// # use neon::prelude::*;
/// # use neon::types::Finalize;
/// # use std::future::Future;
/// # #[derive(Clone)]
/// # pub struct AsyncWorker;
/// #[neon::class]
/// impl AsyncWorker {
/// #   pub fn new() -> Self { Self }
///     #[neon(async)]
///     pub fn process_data(&self, data: String) -> impl Future<Output = String> + 'static {
///         println!("Setup on main thread");
///         let data_clone = data;
///         async move {
///             data_clone.to_uppercase()
///         }
///     }
/// }
/// # }
/// ```
///
/// #### Task Methods
///
/// Methods can be executed on Node's worker pool using the `task` attribute. The instance
/// is cloned to move into the worker thread, so the struct must implement `Clone`.
///
/// ```
/// # use neon::prelude::*;
/// # use neon::types::Finalize;
/// #[derive(Clone)]
/// pub struct CpuWorker;
///
/// #[neon::class]
/// impl CpuWorker {
///     pub fn new() -> Self {
///         Self
///     }
///
///     #[neon(task)]
///     pub fn heavy_computation(&self, iterations: u32) -> u32 {
///         (0..iterations).map(|i| i as u32).sum()
///     }
/// }
/// ```
///
/// #### Method Naming
///
/// Like [`#[neon::export]`](crate::export) functions, method names are converted from `snake_case`
/// to `camelCase`. Custom names can be specified with the `name` attribute:
///
/// ```
/// # use neon::prelude::*;
/// # use neon::types::Finalize;
/// pub struct Label {
///     data: String,
/// }
///
/// #[neon::class]
/// impl Label {
///     pub fn new() -> Self {
///         Self { data: String::new() }
///     }
///
///     #[neon(name = "trimStart")]
///     pub fn trim_leading(&self) -> String {
///         self.data.trim_start().to_string()
///     }
/// }
/// ```
///
/// ## Const Properties
///
/// Classes can expose Rust constants as static, immutable properties on the JavaScript class:
///
/// ```
/// # use neon::prelude::*;
/// # use neon::types::Finalize;
/// pub struct MathConstants;
///
/// #[neon::class]
/// impl MathConstants {
///     const PI: f64 = 3.14159;
///     const VERSION: u32 = 1;
///
///     #[neon(name = "maxValue")]
///     const MAX_VALUE: f64 = f64::MAX;
///
///     #[neon(json)]
///     const DEFAULT_SETTINGS: &'static [&'static str] = &["feature1", "feature2"];
///
///     pub fn new() -> Self {
///         Self
///     }
/// }
/// ```
///
/// From JavaScript:
/// ```js
/// console.log(MathConstants.PI);               // 3.14159
/// console.log(MathConstants.maxValue);         // 1.7976931348623157e+308
/// console.log(MathConstants.DEFAULT_SETTINGS); // ["feature1", "feature2"]
/// ```
///
/// Const properties support the same attributes as globals: `name` for custom naming
/// and `json` for automatic JSON serialization. Properties are immutable from JavaScript.
///
/// ## Context and This Parameters
///
/// Methods can access the JavaScript runtime context and the JavaScript object wrapper:
///
/// ```
/// # use neon::prelude::*;
/// # use neon::types::Finalize;
/// pub struct Interactive {
///     data: String,
/// }
///
/// #[neon::class]
/// impl Interactive {
///     pub fn new(data: String) -> Self {
///         Self { data }
///     }
///
///     // Method with context parameter
///     pub fn create_object<'cx>(
///         &self,
///         cx: &mut FunctionContext<'cx>,
///     ) -> JsResult<'cx, JsObject> {
///         let obj = cx.empty_object();
///         let value = cx.string(&self.data);
///         obj.set(cx, "data", value)?;
///         Ok(obj)
///     }
///
///     // Method with this parameter (access to JS object)
///     pub fn inspect_this(&self, this: Handle<JsObject>) -> String {
///         format!("JS object available: {}", self.data)
///     }
/// }
/// ```
///
/// ## Working with Class Instances
///
/// Methods can accept and return instances of the same class directly. When a class instance
/// is passed as a parameter or returned from a method, it is automatically cloned from (or into)
/// the internal [`RefCell`](std::cell::RefCell) storage, so the struct must implement `Clone`.
///
/// ```
/// # use neon::prelude::*;
/// # use neon::types::Finalize;
/// #[derive(Clone)]
/// pub struct Point {
///     x: f64,
///     y: f64,
/// }
///
/// #[neon::class]
/// impl Point {
///     pub fn new(x: f64, y: f64) -> Self {
///         Self { x, y }
///     }
///
///     pub fn distance(&self, other: Self) -> f64 {
///         let dx = self.x - other.x;
///         let dy = self.y - other.y;
///         (dx * dx + dy * dy).sqrt()
///     }
///
///     pub fn midpoint(&self, other: Self) -> Self {
///         Self {
///             x: (self.x + other.x) / 2.0,
///             y: (self.y + other.y) / 2.0,
///         }
///     }
/// }
/// ```
///
/// From JavaScript, you can call these methods with other instances of the same class:
/// ```js
/// const p1 = new Point(0, 0);
/// const p2 = new Point(3, 4);
/// console.log(p1.distance(p2)); // 5
/// const midpoint = p1.midpoint(p2); // Point { x: 1.5, y: 2 }
/// ```
///
/// ## Export Shorthand
///
/// Use [`#[neon::export(class)]`](crate::export) to combine class definition with
/// automatic module export:
///
/// ```
/// # use neon::prelude::*;
/// # use neon::types::Finalize;
/// pub struct AutoExported {
///     value: u32,
/// }
///
/// // Combines #[neon::class] with automatic export
/// #[neon::export(class)]
/// impl AutoExported {
///     pub fn new(value: u32) -> Self {
///         Self { value }
///     }
/// }
/// ```
///
/// Like other exports, classes can be exported with custom names:
///
/// ```
/// # use neon::prelude::*;
/// # use neon::types::Finalize;
/// pub struct InternalPoint {
///     x: f64,
///     y: f64,
/// }
///
/// // Export as "Point" instead of "InternalPoint"
/// #[neon::export(class, name = "Point")]
/// impl InternalPoint {
///     pub fn new(x: f64, y: f64) -> Self {
///         Self { x, y }
///     }
///
///     pub fn distance_from_origin(&self) -> f64 {
///         (self.x * self.x + self.y * self.y).sqrt()
///     }
/// }
/// ```
///
/// ## Error Handling
///
/// Methods can return [`Result`] types to throw JavaScript exceptions, just like
/// [`#[neon::export]`](crate::export) functions:
///
/// ```
/// # use neon::prelude::*;
/// # use neon::types::{Finalize, extract::Error};
/// pub struct FileReader;
///
/// #[neon::class]
/// impl FileReader {
///     pub fn new() -> Self {
///         Self
///     }
///
///     pub fn read_file(&self, path: String) -> Result<String, Error> {
///         std::fs::read_to_string(path).map_err(Error::from)
///     }
/// }
/// ```
///
/// ## `Class` Trait
///
/// The `#[neon::class]` macro automatically implements the [`Class`](crate::object::Class)
/// trait for the struct. This trait can be used to access the constructor function at runtime.
///
/// ```
/// # use neon::prelude::*;
/// # use neon::types::Finalize;
/// use neon::object::Class;
/// # #[derive(Clone)]
/// # pub struct Point {
/// #     x: f64,
/// #     y: f64,
/// # }
/// #
/// # #[neon::class]
/// # impl Point {
/// #     pub fn new(x: f64, y: f64) -> Self {
/// #         Self { x, y }
/// #     }
/// # }
///
/// # fn init_statics<'cx>(cx: &mut FunctionContext<'cx>) -> JsResult<'cx, JsUndefined> {
/// let constructor = Point::constructor(cx)?;
/// constructor
///     .prop(cx, "ORIGIN")
///     .set(Point::new(0.0, 0.0))?;
/// # Ok(cx.undefined())
/// # }
/// ```
pub use neon_macros::class;

/// Marks a function as the main entry point for initialization in
/// a Neon module.
///
/// This attribute should only be used _once_ in a module and will
/// be called each time the module is initialized in a context.
///
/// If a `main` function is not provided, all registered exports will be exported. If
/// the `tokio` feature flag is enabled, a multithreaded tokio runtime will also be
/// registered globally.
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
/// ### Naming exported functions
///
/// Conventionally, Rust uses `snake_case` for function identifiers and JavaScript uses `camelCase`.
/// By default, Neon will attempt to convert function names to camel case. For example:
///
/// ```rust
/// #[neon::export]
/// fn add_one(n: f64) -> f64 {
///     n + 1.0
/// }
/// ```
///
/// The `add_one` function will be exported as `addOne` in JavaScript.
///
/// ```js
/// import { addOne } from ".";
/// ```
///
/// [Similar to globals](#renaming-an-export), exported functions can be overridden with the `name`
/// attribute.
///
/// ```rust
/// #[neon::export(name = "addOneSync")]
/// fn add_one(n: f64) -> f64 {
///     n + 1.0
/// }
/// ```
/// Neon uses the following rules when converting `snake_case` to `camelCase`:
///
/// * All _leading_ and _trailing_ underscores (`_`) are preserved
/// * Characters _immediately_ following a _non-leading_ underscore are converted to uppercase
/// * If the identifier contains an _unexpected_ character, **no** conversion is performed and
///   the identifier is used _unchanged_. Unexpected characters include:
///   - Uppercase characters
///   - Duplicate _interior_ (non-leading, non-trailing underscores)
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
/// ### Async Functions
///
/// The [`export`] macro can export `async fn`, converting to a JavaScript `Promise`, if a global
/// future executor is registered. See [`neon::set_global_executor`](crate::set_global_executor) for
/// more details.
///
/// ```
/// # #[cfg(all(feature = "napi-6", feature = "futures"))]
/// # {
/// #[neon::export]
/// async fn add(a: f64, b: f64) -> f64 {
///     a + b
/// }
/// # }
/// ```
///
/// #### Synchronous Setup
///
/// To implement a function that appears asynchronous to JavaScript, but needs to perform
/// some synchronous setup on the JavaScript main thread, a normal (i.e., non-`async`) Rust
/// function that returns a [`Future`](std::future::Future) can be annotated with
/// `#[neon::export(async)]`.
///
/// ```
/// # #[cfg(all(feature = "napi-6", feature = "futures"))]
/// # {
/// # use std::future::Future;
/// # use neon::prelude::*;
/// #[neon::export(async)]
/// fn add(a: f64, b: f64) -> impl Future<Output = f64> {
///     println!("Hello from the JavaScript main thread!");
///
///     async move {
///         a + b
///     }
/// }
/// # }
/// ```
///
/// If work needs to be performed on the JavaScript main thread _after_ the asynchronous operation,
/// the [`With`](crate::types::extract::With) extractor can be used to execute a closure before returning.
///
/// ```
/// # #[cfg(all(feature = "napi-6", feature = "futures"))]
/// # {
/// # use neon::types::extract::{self, TryIntoJs};
/// #[neon::export]
/// async fn add(a: f64, b: f64) -> impl for<'cx> TryIntoJs<'cx> {
///     let sum = a + b;
///
///     extract::with(move |cx| {
///         println!("Hello from the JavaScript main thread!");
///
///         sum.try_into_js(cx)
///     })
/// }
/// # }
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
/// Functions may optionally include a [`Cx`](crate::context::Cx) or
/// [`FunctionContext`](crate::context::FunctionContext) argument. Note that unlike functions
/// created with [`JsFunction::new`](crate::types::JsFunction), exported function receive a borrowed
/// context and may require explicit lifetimes.
///
/// ```
/// # use neon::prelude::*;
/// #[neon::export]
/// fn add<'cx>(
///     cx: &mut Cx<'cx>,
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
/// ### Classes
///
/// The `#[neon::export(class)]` attribute may be used on an `impl` block to
/// combine class definition with automatic export. See the documentation for
/// [`#[neon::class]`](crate::class) for more details.
///
/// ### Advanced
///
/// The following attributes are for advanced configuration and may not be
/// necessary for most users.
///
/// #### `context`
///
/// The `#[neon::export]` uses a heuristic to determine if the first argument
/// to a function is a _context_ argument.
///
/// * In a function executed on the JavaScript main thread, it looks for `&mut Cx`
///   or `&mut FunctionContext` to determine if the [`Context`](crate::context::Context)
///   should be passed.
/// * In a function executed on another thread, it looks for [`Channel`](crate::event::Channel).
///
/// If the type has been renamed when importing, the `context` attribute can be
/// added to force it to be passed.
///
/// ```
/// use neon::event::Channel as Ch;
/// use neon::context::FunctionContext as FnCtx;
///
/// #[neon::export(context)]
/// fn add(_cx: &mut FnCtx, a: f64, b: f64) -> f64 {
///     a + b
/// }
///
/// #[neon::export(context)]
/// async fn div(_ch: Ch, a: f64, b: f64) -> f64 {
///     a / b
/// }
/// ```
///
/// #### `this`
///
/// The `#[neon::export]` uses a heuristic to determine if an argument to this function is
/// referring to [`this`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/this).
///
/// 1. If the first argument is a [context](#context), use the 0th argument, otherwise use the 1st.
/// 2. If the argument binding is named `this`
/// 3. Or if it is a tuple struct pattern with an element named `this`
///
/// ```
/// use neon::types::extract::Boxed;
///
/// #[neon::export]
/// fn buffer_clone(this: Vec<u8>) -> Vec<u8> {
///     this
/// }
///
/// #[neon::export]
/// fn box_to_string(Boxed(this): Boxed<String>) -> String {
///     this
/// }
/// ```
///
/// If the function uses a variable name other than `this`, the `this` attribute may
/// be added.
///
/// ```
/// #[neon::export(this)]
/// fn buffer_clone(me: Vec<u8>) -> Vec<u8> {
///     me
/// }
/// ```
pub use neon_macros::export;
