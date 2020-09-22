//! `neon::sync` provides utilities for building multi-threaded Neon modules.
//!
//! ## `Root<T>`
//! In Neon, typically references to JavaScript values are bound by the lifetime
//! of a [`Context`](../context/trait.Context.html).
//! [`Root<T>`](struct.Root.html) allows Rust to maintain a reference to a
//! JavaScript object. The JavaScript object cannot be garbage collected
//! until the `Root<T>` is dropped.
//!
//! The following example demonstrates how `Root` can be used to create a
//! globally accesible callback.
//!
//! ```
//! use neon::prelude::*;
//! # struct OnceCell<T>(Option<T>);
//! # impl<T> OnceCell<T> {
//! #     const fn new() -> Self { Self(None) }
//! #     fn set(&self, v: T) -> Result<(), T> { todo!() }
//! #     fn get(&self) -> Option<T> { todo!() }
//! # }
//!
//! static CALLBACK: OnceCell<Root<JsFunction>> = OnceCell::new();
//!
//! fn init(mut cx: FunctionContext) -> JsResult<JsUndefined> {
//!     let callback = cx.argument::<JsFunction>(0)?.root(&mut cx);
//!
//!     CALLBACK.set(callback).unwrap();
//!
//!     Ok(cx.undefined())
//! }
//!
//! fn invoke(mut cx: FunctionContext) -> JsResult<JsValue> {
//!     let this = cx.undefined();
//!     let arg = cx.argument::<JsString>(0)?;
//!     let callback = CALLBACK.get().unwrap().to_inner(&mut cx);
//!
//!     callback.call(&mut cx, this, vec![arg])
//! }
//! ```
//!
//! ### Drop Safety
//!
//! `Root<T>` may only be dropped from the main JavaScript thread. To prevent
//! accidental leaks of JavaScript objects, `Root<T>` provides a `Drop`
//! implementation that panics if `drop` or `into_inner` is not called. Users
//! must be careful to ensure that `Root<T>` are properly disposed.
//!
//! The [`Finalize`](../prelude/trait.Finalize.html) trait provides an ergonomic
//! way to ensure `Root<T>` contained in a [`JsBox`](../types/struct.JsBox.html)
//! are dropped safely.
//!
//! In the following example, the callback will be safely dropped when the
//! client is garbage collected.
//!
//! ```
//! use neon::prelude::*;
//!
//! struct Client {
//!     callback: Root<JsFunction>,
//! }
//! 
//! impl Finalize for Client {
//!     fn finalize<'a, C: Context<'a>>(self, cx: &mut C) {
//!         self.callback.drop(cx);
//!     }
//! }
//! 
//! fn create_server(mut cx: FunctionContext) -> JsResult<JsBox<Client>> {
//!     let callback = cx.argument::<JsFunction>(0)?.root(&mut cx);
//!     let server = cx.boxed(Client { callback });
//! 
//!     Ok(server)
//! }
//! ```
//!
//! ## `EventQueue`
//!
//! Most Neon functions may only be called from the JavaScript main thread.
//! [`EventQueue`](struct.EventQueue.html) provides a method of
//! synchronization by allowing any thread to schedule a closure to execute
//! on the main thread.
//!
//! In the previous example using [`Root<T>`](#roott), a persistent reference to a
//! JavaScript callback was created. In this example, the reference is held
//! while work is performed on another thread and called when it has completed.
//!
//! ```
//! use neon::prelude::*;
//! # fn long_running_task() -> f64 { 42.0 }
//! 
//! fn perform_async(mut cx: FunctionContext) -> JsResult<JsUndefined> {
//!     let callback = cx.argument::<JsFunction>(0)?.root(&mut cx);
//!     let queue = cx.event_queue();
//! 
//!     // Spawn a background thread to perform our task
//!     std::thread::spawn(move || {
//!         // Perform any number of computations on a separate thread without
//!         // blocking the JavaScript queue.
//!         let result = long_running_task();
//! 
//!         // Once complete, an event can be scheduled back on the main
//!         // JavaScript thread.
//!         queue.send(move |mut cx| {
//!             // Neon functions can be called and the event queue is blocked
//!             // inside this closure.
//!             let callback = callback.into_inner(&mut cx);
//!             let this = cx.undefined();
//!             let arg = cx.number(result);
//! 
//!             callback.call(&mut cx, this, vec![arg])
//!         });
//!     });
//!
//!     // When this function returns, the event queue is no longer blocked
//!     // but, the thread may still be executing in the background.
//!     Ok(cx.undefined())
//! }
//! ```
//!
//! ### `Arc<EventQueue>`
//!
//! `EventQueue` are somewhat expensive to create. Ideally, code will re-use an
//! `EventQueue` for scheduling similar events instead of creating a new queue
//! for each event. `EventQueue` is `Sync` and can be called from any thread,
//! but, is not `Clone`. It can be useful to wrap an `EventQueue` in an
//! [`Arc`](https://doc.rust-lang.org/std/sync/struct.Arc.html) to ensure it
//! is not dropped while it might still be used.
//!
//! The following example uses `EventQueue` to callback to JavaScript after
//! performing a Rust async operation.
//!
//! ```edition2018
//! use neon::prelude::*;
//!
//! use std::sync::Arc;
//! # struct Runtime;
//! # impl Runtime { fn spawn<F>(&self, _: F) {} }
//! # impl Client { fn new<T>(_: T) -> Self { todo!() } }
//! # impl Finalize for Client {}
//!
//! struct Client {
//!     runtime: Runtime,
//!     queue: Arc<EventQueue>,
//! }
//!
//! async fn get_user_name(id: f64) -> String {
//!     String::from("Username")
//! }
//!
//! fn create_client(mut cx: FunctionContext) -> JsResult<JsBox<Client>> {
//!     let queue = cx.event_queue();
//!     let client = Client::new(queue);
//! 
//!     Ok(cx.boxed(client))
//! }
//!
//! fn get_user_name_js(mut cx: FunctionContext) -> JsResult<JsUndefined> {
//!     let client = cx.argument::<JsBox<Client>>(0)?;
//!     let user_id = cx.argument::<JsNumber>(1)?.value(&mut cx);
//!     let callback = cx.argument::<JsFunction>(2)?.root(&mut cx);
//!     let queue = Arc::clone(&client.queue);
//!
//!     client.runtime.spawn(async move {
//!         let username = get_user_name(user_id).await;
//! 
//!         queue.send(move |mut cx| {
//!             let this = cx.undefined();
//!             let callback = callback.into_inner(&mut cx);
//!             let args = vec![
//!                 cx.null().upcast::<JsValue>(),
//!                 cx.string(username).upcast(),
//!             ];
//! 
//!             callback.call(&mut cx, this, args)
//!         });
//!     });
//!
//!     Ok(cx.undefined())
//! }
//! ```

mod event_queue;
mod root;

pub use self::event_queue::EventQueue;
pub use self::root::Root;
