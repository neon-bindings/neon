use std::cell::RefCell;

use crate::{
    context::Cx, handle::Handle, result::{JsResult, NeonResult}, types::{extract::{TryFromJs, TryIntoJs}, JsBox, JsValue}
};

use super::error::RustTypeExpected;

pub trait Container {
    fn container_name() -> &'static str;
}

impl<T> Container for RefCell<T> {
    fn container_name() -> &'static str {
        "std::cell::RefCell"
    }
}

impl<'cx, T: Container + 'static> TryFromJs<'cx> for &'cx RefCell<T> {
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

impl<'cx, T> TryIntoJs<'cx> for RefCell<T>
where
    T: 'static,
{
    type Value = JsBox<RefCell<T>>;

    fn try_into_js(self, cx: &mut Cx<'cx>) -> JsResult<'cx, Self::Value> {
        Ok(JsBox::manually_finalize(cx, self))
    }
}
