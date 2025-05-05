use crate::{
    context::{Context, Cx},
    handle::Handle,
    result::{JsResult, NeonResult},
    types::{
        Finalize, JsBox, JsValue,
        extract::{TryFromJs, TryIntoJs, TypeExpected, private},
    },
};

/// Wrapper to extract `T` from a [`JsBox<T>`](JsBox) or create a [`JsBox`]
/// from a `T`.
///
/// [`Boxed`] is especially useful for exporting async functions and tasks.
///
/// ```
/// # use std::sync::Arc;
/// # use neon::{prelude::*, types::extract::Boxed};
/// struct Greeter {
///     greeting: String,
/// }
///
/// impl Finalize for Greeter {}
///
/// impl Greeter {
///     fn new(greeting: String) -> Self {
///         Self { greeting }
///     }
///
///     fn greet(&self, name: &str) -> String {
///         format!("{}, {name}!", self.greeting)
///     }
/// }
///
/// #[neon::export]
/// fn create_greeter(greeting: String) -> Boxed<Arc<Greeter>> {
///     Boxed(Arc::new(Greeter::new(greeting)))
/// }
///
/// #[neon::export(task)]
/// fn greet(Boxed(greeter): Boxed<Arc<Greeter>>, name: String) -> String {
///     greeter.greet(&name)
/// }
/// ```
pub struct Boxed<T>(pub T);

impl<'cx, T> TryFromJs<'cx> for Boxed<T>
where
    T: Clone + 'static,
{
    type Error = TypeExpected<JsBox<T>>;

    fn try_from_js(
        cx: &mut Cx<'cx>,
        v: Handle<'cx, JsValue>,
    ) -> NeonResult<Result<Self, Self::Error>> {
        match v.downcast::<JsBox<T>, _>(cx) {
            Ok(v) => Ok(Ok(Self(T::clone(&v)))),
            Err(_) => Ok(Err(TypeExpected::new())),
        }
    }
}

impl<'cx, T> TryIntoJs<'cx> for Boxed<T>
where
    T: Finalize + 'static,
{
    type Value = JsBox<T>;

    fn try_into_js(self, cx: &mut Cx<'cx>) -> JsResult<'cx, Self::Value> {
        Ok(cx.boxed(self.0))
    }
}

impl<T> private::Sealed for Boxed<T> {}
