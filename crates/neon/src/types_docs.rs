#[cfg_attr(doc, aquamarine::aquamarine)]
/// Representations of JavaScript's core builtin types.
///
/// ## Modeling JavaScript Types
///
/// All JavaScript values in Neon implement the abstract [`Value`](Value) trait, which
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
/// The top of the JavaScript type hierarchy is modeled with the Neon type
/// [`JsValue`](JsValue). A [handle](crate::handle) to a `JsValue` can refer
/// to any JavaScript value. (For TypeScript programmers, this can be thought
/// of as similar to TypeScript's [`unknown`][unknown] type.)
///
/// From there, the type hierarchy divides into _object types_ and _primitive
/// types_:
///
/// ```mermaid
/// flowchart LR
/// JsValue(JsValue)
/// JsValue-->JsObject(JsObject)
/// click JsValue "./struct.JsValue.html" "JsValue"
/// click JsObject "./struct.JsObject.html" "JsObject"
/// subgraph primitives [Primitive Types]
///     JsBoolean(JsBoolean)
///     JsNumber(JsNumber)
///     JsString(JsString)
///     JsNull(JsNull)
///     JsUndefined(JsUndefined)
///     click JsBoolean "./struct.JsBoolean.html" "JsBoolean"
///     click JsNumber "./struct.JsNumber.html" "JsNumber"
///     click JsString "./struct.JsString.html" "JsString"
///     click JsNull "./struct.JsNull.html" "JsNull"
///     click JsUndefined "./struct.JsUndefined.html" "JsUndefined"
/// end
/// JsValue-->primitives
/// ```
///
/// The top of the object type hierarchy is [`JsObject`](JsObject). A handle to a
/// `JsObject` can refer to any JavaScript object.
///
/// The primitive types are the built-in JavaScript datatypes that are not object
/// types: [`JsBoolean`](JsBoolean), [`JsNumber`](JsNumber), [`JsString`](JsString),
/// [`JsNull`](JsNull), and [`JsUndefined`](JsUndefined).
///
/// #### Object Types
///
/// The object type hierarchy further divides into a variety of different subtypes:
///
/// ```mermaid
/// flowchart LR
/// JsObject(JsObject)
/// click JsObject "./struct.JsObject.html" "JsObject"
/// subgraph objects [Standard Object Types]
///     JsFunction(JsFunction)
///     JsArray(JsArray)
///     JsDate(JsDate)
///     JsError(JsError)
///     click JsFunction "./struct.JsFunction.html" "JsFunction"
///     click JsArray "./struct.JsArray.html" "JsArray"
///     click JsDate "./struct.JsDate.html" "JsDate"
///     click JsError "./struct.JsError.html" "JsError"
/// end
/// subgraph typedarrays [Typed Arrays]
///     JsBuffer(JsBuffer)
///     JsArrayBuffer(JsArrayBuffer)
///     JsTypedArray("JsTypedArray&lt;T&gt;")
///     click JsBuffer "./struct.JsBuffer.html" "JsBuffer"
///     click JsArrayBuffer "./struct.JsArrayBuffer.html" "JsArrayBuffer"
///     click JsTypedArray "./struct.JsTypedArray.html" "JsTypedArray"
/// end
/// subgraph custom [Custom Types]
///     JsBox(JsBox)
///     click JsBox "./struct.JsBox.html" "JsBox"
/// end
/// JsObject-->objects
/// JsObject-->typedarrays
/// JsObject-->custom
/// ```
///
/// These include several categories of object types:
/// - **Standard object types:** [`JsFunction`](JsFunction), [`JsArray`](JsArray),
///   [`JsDate`](JsDate), and [`JsError`](JsError).
/// - **Typed arrays:** [`JsBuffer`](JsBuffer), [`JsArrayBuffer`](JsArrayBuffer),
///   and [`JsTypedArray<T>`](JsTypedArray).
/// - **Custom types:** [`JsBox`](JsBox), a special Neon type that allows the creation
///   of custom objects that own Rust data structures.
///
/// All object types implement the [`Object`](crate::object::Object) trait, which
/// allows getting and setting properties of an object.
///
/// [unknown]: https://mariusschulz.com/blog/the-unknown-type-in-typescript#the-unknown-type
pub mod exports {
    pub use crate::types_impl::*;
}
