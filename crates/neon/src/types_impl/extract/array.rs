use std::{error, fmt, mem::MaybeUninit};

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

/// Extracts a [JavaScript array](JsArray) into a Rust collection or converts a collection to a JS array.
///
/// Any collection that implements [`FromIterator`] and [`IntoIterator`] can be extracted. Extraction
/// fails with [`ArrayError`] if the value is not an array or an element fails to be extracted.
///
/// # Example
///
/// ```
/// # use std::collections::HashSet;
/// # use neon::types::extract::Array;
/// #[neon::export]
/// fn list_of_strings(Array(arr): Array<Vec<String>>) -> Array<Vec<String>> {
///     Array(arr)
/// }
///
/// #[neon::export]
/// fn double(Array(arr): Array<Vec<f64>>) -> Array<impl Iterator<Item = f64>> {
///     Array(arr.into_iter().map(|x| x * 2.0))
/// }
///
/// #[neon::export]
/// fn dedupe(set: Array<HashSet<String>>) -> Array<HashSet<String>> {
///     set
/// }
/// ```
///
/// **Note**: Only native JS arrays are accepted. For typed arrays use [`Uint8Array`](super::Uint8Array),
/// [`Float64Array`](super::Float64Array), etc.
pub struct Array<T>(pub T);

impl<T> private::Sealed for Array<T> {}

impl<'cx, T> TryFromJs<'cx> for Array<T>
where
    T: FromIterator<T::Item>,
    T: IntoIterator,
    T::Item: TryFromJs<'cx>,
{
    type Error = ArrayError<<T::Item as TryFromJs<'cx>>::Error>;

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
                Err(sys::Status::ArrayExpected) => return Ok(Err(ArrayError::array())),
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
                    Ok(Err(err)) => Ok(Err(ArrayError::item(err))),
                    Err(err) => Err(err),
                }
            })
            .collect::<Result<Result<T, Self::Error>, Throw>>()
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

/// Error when extracting an [`Array<T>`]
#[derive(Debug)]
pub enum ArrayError<E> {
    /// Value was not a JavaScript array.
    Array(TypeExpected<JsArray>),
    /// An element failed to convert to `T::Item`.
    Item(E),
}

impl<E> ArrayError<E> {
    fn array() -> Self {
        Self::Array(TypeExpected::<JsArray>::new())
    }

    fn item(err: E) -> Self {
        Self::Item(err)
    }
}

impl<E> fmt::Display for ArrayError<E>
where
    E: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ArrayError::Array(err) => write!(f, "{}", err),
            ArrayError::Item(err) => write!(f, "{}", err),
        }
    }
}

impl<E> error::Error for ArrayError<E> where E: error::Error {}

impl<'cx, E> TryIntoJs<'cx> for ArrayError<E>
where
    E: TryIntoJs<'cx>,
{
    type Value = JsValue;

    fn try_into_js(self, cx: &mut Cx<'cx>) -> JsResult<'cx, Self::Value> {
        match self {
            ArrayError::Array(err) => err.try_into_js(cx).map(|v| v.upcast()),
            ArrayError::Item(err) => err.try_into_js(cx).map(|v| v.upcast()),
        }
    }
}

impl<'cx, E> private::Sealed for ArrayError<E> where E: TryIntoJs<'cx> {}
