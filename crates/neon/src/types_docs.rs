#[cfg_attr(doc, aquamarine::aquamarine)]
/// Representations of JavaScript's core builtin types.
///
/// ## Modeling JavaScript Types
///
/// All JavaScript values in Neon implement the abstract [`Value`] trait, which
/// is the most generic way to work with JavaScript values. Neon provides a
/// number of types that implement this trait, each representing a particular
/// type of JavaScript value.
///
/// By convention, JavaScript types in Neon have the prefix `Js` in their name,
/// such as [`JsNumber`](crate::types::JsNumber) (for the JavaScript `number`
/// type) or [`JsFunction`](crate::types::JsFunction) (for the JavaScript
/// `function` type).
///
/// ### Handles and Casts
///
/// Access to JavaScript values in Neon works through [handles](crate::handle),
/// which ensure the safe interoperation between Rust and the JavaScript garbage
/// collector. This means, for example, a Rust variable that stores a JavaScript string
/// will have the type `Handle<JsString>` rather than [`JsString`](crate::types::JsString).
///
/// Neon types model the JavaScript type hierarchy through the use of *casts*.
/// The [`Handle::upcast()`](crate::handle::Handle::upcast) method safely converts
/// a handle to a JavaScript value of one type into a handle to a value of its
/// supertype. For example, it's safe to treat a [`JsArray`](crate::types::JsArray)
/// as a [`JsObject`](crate::types::JsObject), so you can do an "upcast" and it will
/// never fail:
///
/// ```
/// # use neon::prelude::*;
/// fn as_object(array: Handle<JsArray>) -> Handle<JsObject> {
///     let object: Handle<JsObject> = array.upcast();
///     object
/// }
/// ```
///
/// Unlike upcasts, the [`Handle::downcast()`](crate::handle::Handle::downcast) method
/// requires a runtime check to test a value's type at runtime, so it can fail with
/// a [`DowncastError`](crate::handle::DowncastError):
///
/// ```
/// # use neon::prelude::*;
/// fn as_array<'a>(
///     cx: &mut impl Context<'a>,
///     object: Handle<'a, JsObject>
/// ) -> JsResult<'a, JsArray> {
///     object.downcast(cx).or_throw(cx)
/// }
/// ```
///
/// ### The JavaScript Type Hierarchy
///
/// ```mermaid
/// flowchart TB
/// JsValue
/// JsValue-->JsObject
/// subgraph primitives [Primitive Types]
///     JsBoolean
///     JsNumber
///     JsString
///     JsNull
///     JsUndefined
/// end
/// subgraph objects [Standard Object Types]
///     JsFunction
///     JsArray
///     JsDate
///     JsError
/// end
/// subgraph typedarrays [Typed Arrays]
///     JsBuffer
///     JsArrayBuffer
///     JsTypedArray["JsTypedArray&lt;T&gt;"]
/// end
/// subgraph custom [Custom Types]
///     JsBox
/// end
/// JsValue-->primitives
/// JsObject-->objects
/// JsObject-->typedarrays
/// JsObject-->custom
/// ```
///
/// The JavaScript type hierarchy includes:
///
/// - [`JsValue`](JsValue): This is the top of the type hierarchy, and can refer to
///   any JavaScript value. (For TypeScript programmers, this can be thought of as
///   similar to TypeScript's [`unknown`][unknown] type.)
/// - [`JsObject`](JsObject): This is the top of the object type hierarchy. Object
///   types all implement the [`Object`](crate::object::Object) trait, which allows
///   getting and setting properties.
///   - **Standard object types:** [`JsFunction`](JsFunction), [`JsArray`](JsArray),
///     [`JsDate`](JsDate), and [`JsError`](JsError).
///   - **Typed arrays:** [`JsBuffer`](JsBuffer), [`JsArrayBuffer`](JsArrayBuffer),
///     and [`JsTypedArray<T>`](JsTypedArray).
///   - **Custom types:** [`JsBox`](JsBox), a special Neon type that allows the creation
///     of custom objects that own Rust data structures.
/// - **Primitive types:** These are the built-in JavaScript datatypes that are not
///   object types: [`JsNumber`](JsNumber), [`JsBoolean`](JsBoolean),
///   [`JsString`](JsString), [`JsNull`](JsNull), and [`JsUndefined`](JsUndefined).
///
/// [unknown]: https://mariusschulz.com/blog/the-unknown-type-in-typescript#the-unknown-type
pub mod exports {
    pub use crate::types_impl::*;
}
