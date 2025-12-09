use std::mem::MaybeUninit;

use either::Either;

use crate::{
    context::{internal::ContextInternal, Context, Cx},
    handle::Handle,
    result::{JsResult, NeonResult, Throw},
    sys,
    types::{
        extract::{private, TryFromJs, TryIntoJs, TypeExpected},
        private::ValueInternal,
        JsArray, JsValue,
    },
};

pub struct Array<T>(pub T);

impl<T> private::Sealed for Array<T> {}

impl<'cx, T> TryFromJs<'cx> for Array<T>
where
    T: FromIterator<T::Item>,
    T: IntoIterator,
    T::Item: TryFromJs<'cx>,
{
    type Error = Either<TypeExpected<JsArray>, <T::Item as TryFromJs<'cx>>::Error>;

    fn try_from_js(
        cx: &mut Cx<'cx>,
        v: Handle<'cx, JsValue>,
    ) -> NeonResult<Result<Self, Self::Error>> {
        let env = cx.env().to_raw();
        let v = v.to_local();
        let len = unsafe {
            let mut len = 0;

            match sys::get_array_length(env, v, &mut len) {
                Err(sys::Status::PendingException) => return Err(Throw::new()),
                Err(sys::Status::ArrayExpected) => {
                    return Ok(Err(Either::Left(TypeExpected::new())))
                }
                res => res.unwrap(),
            }

            len
        };

        (0..len)
            .map(|i| {
                let item = unsafe {
                    let mut item = MaybeUninit::uninit();

                    match sys::get_element(env, v, i, item.as_mut_ptr()) {
                        Err(sys::Status::PendingException) => return Err(Throw::new()),
                        res => res.unwrap(),
                    }

                    Handle::new_internal(JsValue::from_local(cx.env(), item.assume_init()))
                };

                match T::Item::try_from_js(cx, item) {
                    Ok(Ok(item)) => Ok(Ok(item)),
                    Ok(Err(err)) => Ok(Err(Either::Right(err))),
                    Err(err) => Err(err),
                }
            })
            .collect::<Result<Result<_, _>, _>>()
            .map(|v| v.map(Array))
    }
}

impl<'cx, T> TryIntoJs<'cx> for Array<T>
where
    T: IntoIterator,
    T::Item: TryIntoJs<'cx>,
{
    type Value = JsArray;

    fn try_into_js(self, cx: &mut Cx<'cx>) -> JsResult<'cx, Self::Value> {
        let iter = self.0.into_iter();
        let env = cx.env().to_raw();
        let (len, _) = iter.size_hint();
        let arr = unsafe {
            let mut arr = MaybeUninit::uninit();

            match sys::create_array_with_length(env, len, arr.as_mut_ptr()) {
                Err(sys::Status::PendingException) => return Err(Throw::new()),
                res => res.unwrap(),
            }

            arr.assume_init()
        };

        for (i, item) in iter.enumerate() {
            let item = item.try_into_js(cx)?.to_local();
            let Ok(i) = u32::try_from(i) else {
                return cx.throw_error("Exceeded maximum length of an array");
            };

            unsafe {
                match sys::set_element(env, arr, i, item) {
                    Err(sys::Status::PendingException) => return Err(Throw::new()),
                    res => res.unwrap(),
                }
            }
        }

        unsafe { Ok(Handle::new_internal(JsArray::from_local(cx.env(), arr))) }
    }
}
