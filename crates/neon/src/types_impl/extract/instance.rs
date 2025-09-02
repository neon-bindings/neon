use crate::{
    context::Cx,
    handle::Handle,
    object::Class,
    result::{JsResult, NeonResult, ResultExt},
    types::{
        extract::{private, ObjectExpected, TryFromJs, TryIntoJs}, Finalize, JsObject, JsValue
    },
};

pub struct Instance<T>(pub T);

impl<T> std::ops::Deref for Instance<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'cx, T> TryFromJs<'cx> for Instance<T>
where
    T: Class + Clone + Finalize + 'static,
{
    type Error = ObjectExpected;

    fn try_from_js(
        cx: &mut Cx<'cx>,
        v: Handle<'cx, JsValue>,
    ) -> NeonResult<Result<Self, Self::Error>> {
        let object = v.downcast::<JsObject, _>(cx).or_throw(cx)?;
        match crate::object::unwrap(cx, object) {
            Ok(Ok(instance)) => Ok(Ok(Self(T::clone(instance)))),
            _ => Ok(Err(ObjectExpected::new(T::name()))),
        }
    }
}

impl<'cx, T> TryIntoJs<'cx> for Instance<T>
where
    T: Class + Finalize + 'static,
{
    type Value = JsObject;

    fn try_into_js(self, cx: &mut Cx<'cx>) -> JsResult<'cx, Self::Value> {
        let class_instance = T::current_instance(cx)?;
        let object: Handle<JsObject> = class_instance.internal_constructor
            .bind(cx)
            .construct()?;
        crate::object::wrap(cx, object, self.0)?.or_throw(cx)?;
        Ok(object)
    }
}

impl<T> private::Sealed for Instance<T> {}
