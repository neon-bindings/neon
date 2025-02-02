use std::{
    cell::{Ref, RefCell, RefMut},
    rc::Rc,
    sync::Arc,
};

use crate::{
    context::{Context, Cx},
    handle::Handle,
    result::{JsResult, NeonResult},
    types::{
        extract::{TryFromJs, TryIntoJs},
        JsBox, JsValue,
    },
};

use super::error::TypeExpected;

impl<'cx, T: 'static> TryFromJs<'cx> for &'cx RefCell<T> {
    type Error = TypeExpected<JsBox<RefCell<T>>>;

    fn try_from_js(
        cx: &mut Cx<'cx>,
        v: Handle<'cx, JsValue>,
    ) -> NeonResult<Result<Self, Self::Error>> {
        match v.downcast::<JsBox<RefCell<T>>, _>(cx) {
            Ok(v) => Ok(Ok(JsBox::deref(&v))),
            Err(_) => Ok(Err(TypeExpected::new())),
        }
    }
}

impl<'cx, T: 'static> TryFromJs<'cx> for Ref<'cx, T> {
    type Error = TypeExpected<JsBox<RefCell<T>>>;

    fn try_from_js(
        cx: &mut Cx<'cx>,
        v: Handle<'cx, JsValue>,
    ) -> NeonResult<Result<Self, Self::Error>> {
        match v.downcast::<JsBox<RefCell<T>>, _>(cx) {
            Ok(v) => match JsBox::deref(&v).try_borrow() {
                Ok(r) => Ok(Ok(r)),
                Err(_) => cx.throw_error("RefCell is already mutably borrowed"),
            },
            Err(_) => Ok(Err(TypeExpected::new())),
        }
    }
}

impl<'cx, T: 'static> TryFromJs<'cx> for RefMut<'cx, T> {
    type Error = TypeExpected<JsBox<RefCell<T>>>;

    fn try_from_js(
        cx: &mut Cx<'cx>,
        v: Handle<'cx, JsValue>,
    ) -> NeonResult<Result<Self, Self::Error>> {
        match v.downcast::<JsBox<RefCell<T>>, _>(cx) {
            Ok(v) => match JsBox::deref(&v).try_borrow_mut() {
                Ok(r) => Ok(Ok(r)),
                Err(_) => cx.throw_error("RefCell is already borrowed"),
            },
            Err(_) => Ok(Err(TypeExpected::new())),
        }
    }
}

impl<'cx, T> TryIntoJs<'cx> for RefCell<T>
where
    T: 'static,
{
    type Value = JsBox<RefCell<T>>;

    fn try_into_js(self, cx: &mut Cx<'cx>) -> JsResult<'cx, Self::Value> {
        Ok(JsBox::manually_finalize(cx, self))
    }
}

impl<'cx, T: 'static> TryFromJs<'cx> for Rc<T> {
    type Error = TypeExpected<JsBox<Rc<T>>>;

    fn try_from_js(
        cx: &mut Cx<'cx>,
        v: Handle<'cx, JsValue>,
    ) -> NeonResult<Result<Self, Self::Error>> {
        match v.downcast::<JsBox<Rc<T>>, _>(cx) {
            Ok(v) => Ok(Ok(JsBox::deref(&v).clone())),
            Err(_) => Ok(Err(TypeExpected::new())),
        }
    }
}

impl<'cx, T> TryIntoJs<'cx> for Rc<T>
where
    T: 'static,
{
    type Value = JsBox<Rc<T>>;

    fn try_into_js(self, cx: &mut Cx<'cx>) -> JsResult<'cx, Self::Value> {
        Ok(JsBox::manually_finalize(cx, self))
    }
}

impl<'cx, T: 'static> TryFromJs<'cx> for Arc<T> {
    type Error = TypeExpected<JsBox<Arc<T>>>;

    fn try_from_js(
        cx: &mut Cx<'cx>,
        v: Handle<'cx, JsValue>,
    ) -> NeonResult<Result<Self, Self::Error>> {
        match v.downcast::<JsBox<Arc<T>>, _>(cx) {
            Ok(v) => Ok(Ok(JsBox::deref(&v).clone())),
            Err(_) => Ok(Err(TypeExpected::new())),
        }
    }
}

impl<'cx, T> TryIntoJs<'cx> for Arc<T>
where
    T: 'static,
{
    type Value = JsBox<Arc<T>>;

    fn try_into_js(self, cx: &mut Cx<'cx>) -> JsResult<'cx, Self::Value> {
        Ok(JsBox::manually_finalize(cx, self))
    }
}
