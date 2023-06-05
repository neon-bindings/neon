use crate::{
    context::{Context, FunctionContext, TaskContext},
    handle::Handle,
    result::{JsResult, NeonResult},
    types::{JsNumber, JsUndefined, JsValue, Value},
};

pub trait Function<'cx, Args> {
    fn call(&self, cx: FunctionContext<'cx>) -> JsResult<'cx, JsValue>;
}

impl<'cx, F, O> Function<'cx, ()> for F
where
    F: Fn(FunctionContext<'cx>) -> O,
    O: TryIntoJs<'cx>,
{
    fn call(&self, mut cx: FunctionContext<'cx>) -> JsResult<'cx, JsValue> {
        let mut complete_cx = unsafe { TaskContext::new(&mut cx) };

        Ok((self)(cx).try_into_js(&mut complete_cx)?.upcast())
    }
}

impl<'cx, F, O, T0> Function<'cx, (T0,)> for F
where
    F: Fn(FunctionContext<'cx>, T0) -> O,
    O: TryIntoJs<'cx>,
    T0: TryFromJs<'cx>,
{
    fn call(&self, mut cx: FunctionContext<'cx>) -> JsResult<'cx, JsValue> {
        let t0 = cx.argument::<JsValue>(0)?;
        let t0 = T0::try_from(&mut cx, t0)?;
        let mut complete_cx = unsafe { TaskContext::new(&mut cx) };

        Ok((self)(cx, t0).try_into_js(&mut complete_cx)?.upcast())
    }
}

impl<'cx, F, O, T0, T1> Function<'cx, (FunctionContext<'static>, T0, T1)> for F
where
    F: Fn(FunctionContext<'cx>, T0, T1) -> O,
    O: TryIntoJs<'cx>,
    T0: TryFromJs<'cx>,
    T1: TryFromJs<'cx>,
{
    fn call(&self, mut cx: FunctionContext<'cx>) -> JsResult<'cx, JsValue> {
        let t0 = cx.argument::<JsValue>(0)?;
        let t1 = cx.argument::<JsValue>(1)?;
        let t0 = T0::try_from(&mut cx, t0)?;
        let t1 = T1::try_from(&mut cx, t1)?;
        let mut complete_cx = unsafe { TaskContext::new(&mut cx) };

        Ok((self)(cx, t0, t1).try_into_js(&mut complete_cx)?.upcast())
    }
}

impl<'cx, F, O, T0, T1> Function<'cx, (T0, T1)> for F
where
    F: Fn(T0, T1) -> O,
    O: TryIntoJs<'cx>,
    T0: TryFromJs<'cx>,
    T1: TryFromJs<'cx>,
{
    fn call(&self, mut cx: FunctionContext<'cx>) -> JsResult<'cx, JsValue> {
        let t0 = cx.argument::<JsValue>(0)?;
        let t1 = cx.argument::<JsValue>(1)?;
        let t0 = T0::try_from(&mut cx, t0)?;
        let t1 = T1::try_from(&mut cx, t1)?;
        let mut complete_cx = unsafe { TaskContext::new(&mut cx) };

        Ok((self)(t0, t1).try_into_js(&mut complete_cx)?.upcast())
    }
}

pub trait TryIntoJs<'cx> {
    type Output: Value;

    fn try_into_js<C>(self, cx: &mut C) -> JsResult<'cx, Self::Output>
    where
        C: Context<'cx>;
}

impl<'cx, V> TryIntoJs<'cx> for Handle<'cx, V>
where
    V: Value,
{
    type Output = V;

    fn try_into_js<C>(self, _cx: &mut C) -> JsResult<'cx, Self::Output>
    where
        C: Context<'cx>,
    {
        Ok(self)
    }
}

impl<'cx> TryIntoJs<'cx> for () {
    type Output = JsUndefined;

    fn try_into_js<C>(self, cx: &mut C) -> JsResult<'cx, Self::Output>
    where
        C: Context<'cx>,
    {
        Ok(cx.undefined())
    }
}

impl<'cx> TryIntoJs<'cx> for f64 {
    type Output = JsNumber;

    fn try_into_js<C>(self, cx: &mut C) -> JsResult<'cx, Self::Output>
    where
        C: Context<'cx>,
    {
        Ok(cx.number(self))
    }
}

trait TryFromJs<'cx>: Sized {
    fn try_from<C>(cx: &mut C, v: Handle<'cx, JsValue>) -> NeonResult<Self>
    where
        C: Context<'cx>;
}

impl<'cx> TryFromJs<'cx> for f64 {
    fn try_from<C>(cx: &mut C, v: Handle<'cx, JsValue>) -> NeonResult<Self>
    where
        C: Context<'cx>,
    {
        Ok(v.downcast_or_throw::<JsNumber, _>(cx)?.value(cx))
    }
}

impl<'cx, V> TryFromJs<'cx> for Handle<'cx, V>
where
    V: Value,
{
    fn try_from<C>(cx: &mut C, v: Handle<'cx, JsValue>) -> NeonResult<Self>
    where
        C: Context<'cx>,
    {
        v.downcast_or_throw(cx)
    }
}

pub fn arg0<V, F>(f: F) -> impl for<'cx> Function<'cx, ()>
where
    for<'cx> F: Fn(FunctionContext<'cx>) -> JsResult<'cx, V>,
    V: Value,
{
    f
}

impl<'cx, T> TryIntoJs<'cx> for NeonResult<T>
where
    T: TryIntoJs<'cx>,
{
    type Output = T::Output;

    fn try_into_js<C>(self, cx: &mut C) -> JsResult<'cx, Self::Output>
    where
        C: Context<'cx>,
    {
        self?.try_into_js(cx)
    }
}
