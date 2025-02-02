use std::{
    cell::{Ref, RefCell, RefMut},
    rc::Rc,
    sync::Arc,
};

use crate::{
    context::Cx,
    handle::Handle,
    result::{JsResult, NeonResult},
    types::{
        extract::{TryFromJs, TryIntoJs},
        JsBox, JsValue,
    },
};

use super::error::{RefCellError, RustTypeExpected};

pub trait Container {
    fn container_name() -> &'static str;
}

impl<T> Container for RefCell<T> {
    fn container_name() -> &'static str {
        "std::cell::RefCell"
    }
}

impl<T> Container for Rc<T> {
    fn container_name() -> &'static str {
        "std::rc::Rc"
    }
}

impl<T> Container for Arc<T> {
    fn container_name() -> &'static str {
        "std::sync::Arc"
    }
}

impl<'cx, T: 'static> TryFromJs<'cx> for &'cx RefCell<T> {
    type Error = RustTypeExpected<RefCell<T>>;

    fn try_from_js(
        cx: &mut Cx<'cx>,
        v: Handle<'cx, JsValue>,
    ) -> NeonResult<Result<Self, Self::Error>> {
        match v.downcast::<JsBox<RefCell<T>>, _>(cx) {
            Ok(v) => Ok(Ok(JsBox::deref(&v))),
            Err(_) => Ok(Err(RustTypeExpected::new())),
        }
    }
}

impl<'cx, T: 'static> TryFromJs<'cx> for Ref<'cx, T> {
    type Error = RefCellError;

    fn try_from_js(
        cx: &mut Cx<'cx>,
        v: Handle<'cx, JsValue>,
    ) -> NeonResult<Result<Self, Self::Error>> {
        match v.downcast::<JsBox<RefCell<T>>, _>(cx) {
            Ok(v) => {
                let cell = JsBox::deref(&v);
                Ok(cell.try_borrow().map_err(|_| RefCellError::Borrowed))
            }
            Err(_) => Ok(Err(RefCellError::WrongType)),
        }
    }
}

impl<'cx, T: 'static> TryFromJs<'cx> for RefMut<'cx, T> {
    type Error = RefCellError;

    fn try_from_js(
        cx: &mut Cx<'cx>,
        v: Handle<'cx, JsValue>,
    ) -> NeonResult<Result<Self, Self::Error>> {
        match v.downcast::<JsBox<RefCell<T>>, _>(cx) {
            Ok(v) => {
                let cell = JsBox::deref(&v);
                Ok(cell
                    .try_borrow_mut()
                    .map_err(|_| RefCellError::MutablyBorrowed))
            }
            Err(_) => Ok(Err(RefCellError::WrongType)),
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
    type Error = RustTypeExpected<Rc<T>>;

    fn try_from_js(
        cx: &mut Cx<'cx>,
        v: Handle<'cx, JsValue>,
    ) -> NeonResult<Result<Self, Self::Error>> {
        match v.downcast::<JsBox<Rc<T>>, _>(cx) {
            Ok(v) => Ok(Ok(JsBox::deref(&v).clone())),
            Err(_) => Ok(Err(RustTypeExpected::new())),
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
    type Error = RustTypeExpected<Arc<T>>;

    fn try_from_js(
        cx: &mut Cx<'cx>,
        v: Handle<'cx, JsValue>,
    ) -> NeonResult<Result<Self, Self::Error>> {
        match v.downcast::<JsBox<Arc<T>>, _>(cx) {
            Ok(v) => Ok(Ok(JsBox::deref(&v).clone())),
            Err(_) => Ok(Err(RustTypeExpected::new())),
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
