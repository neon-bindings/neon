use std::{any::Any, error, ffi::c_void, fmt, mem::MaybeUninit, ptr};

use crate::{
    context::{
        internal::{ContextInternal, Env},
        Context, Cx,
    },
    handle::Handle,
    object::Object,
    result::{JsResult, NeonResult, ResultExt, Throw},
    sys,
    types::{extract::TryIntoJs, Finalize, Value},
};

type BoxAny = Box<dyn Any + 'static>;

#[derive(Debug)]
pub struct WrapError(WrapErrorType);

impl WrapError {
    fn object_expected() -> Self {
        Self(WrapErrorType::ObjectExpected)
    }

    fn already_wrapped() -> Self {
        Self(WrapErrorType::AlreadyWrapped)
    }

    fn not_wrapped() -> Self {
        Self(WrapErrorType::NotWrapped)
    }

    fn wrong_type(expected: &'static str) -> Self {
        Self(WrapErrorType::WrongType(expected))
    }

    #[cfg(feature = "napi-8")]
    fn foreign_type() -> Self {
        Self(WrapErrorType::ForeignType)
    }
}

impl fmt::Display for WrapError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl error::Error for WrapError {}

impl crate::types::extract::private::Sealed for WrapError {}

impl<'cx> TryIntoJs<'cx> for WrapError {
    type Value = crate::types::JsError;

    fn try_into_js(self, cx: &mut Cx<'cx>) -> JsResult<'cx, Self::Value> {
        match self.0 {
            WrapErrorType::ObjectExpected => cx.type_error("object expected"),
            _ => cx.type_error(self.to_string()),
        }
    }
}

impl<T> ResultExt<T> for Result<T, WrapError> {
    fn or_throw<'cx, C>(self, cx: &mut C) -> NeonResult<T>
    where
        C: Context<'cx>,
    {
        match self {
            Ok(v) => Ok(v),
            Err(WrapError(WrapErrorType::ObjectExpected)) => cx.throw_type_error("object expected"),
            Err(err) => cx.throw_error(err.to_string()),
        }
    }
}

#[derive(Debug)]
enum WrapErrorType {
    ObjectExpected,
    AlreadyWrapped,
    NotWrapped,
    WrongType(&'static str),
    #[cfg(feature = "napi-8")]
    ForeignType,
}

fn ref_cell_target_type_name(s: &str) -> Option<String> {
    if let Some(start) = s.find('<') {
        let s = &s[start + 1..];
        if let Some(end) = s.find('>') {
            return Some(s[0..end].to_string());
        }
    }
    None
}

impl fmt::Display for WrapErrorType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::ObjectExpected => write!(f, "object expected"),
            Self::AlreadyWrapped => write!(f, "non-class instance expected"),
            Self::NotWrapped => write!(f, "class instance expected"),
            Self::WrongType(expected) => {
                let target_type_name =
                    ref_cell_target_type_name(*expected).unwrap_or(expected.to_string());
                write!(f, "expected instance of {}", target_type_name)
            }
            #[cfg(feature = "napi-8")]
            Self::ForeignType => write!(f, "Neon object expected"),
        }
    }
}

pub fn wrap<T, V>(cx: &mut Cx, o: Handle<V>, v: T) -> NeonResult<Result<(), WrapError>>
where
    T: Finalize + 'static,
    V: Object,
{
    let env = cx.env().to_raw();
    let o = o.to_local();
    let v = Box::into_raw(Box::new(Box::new(v) as BoxAny));

    // # Safety
    // The `finalize` function will be called when the JavaScript object is garbage
    // collected. The `data` pointer is guaranteed to be the same pointer passed when
    // wrapping.
    unsafe extern "C" fn finalize<T>(env: sys::Env, data: *mut c_void, _hint: *mut c_void)
    where
        T: Finalize + 'static,
    {
        let data = Box::from_raw(data.cast::<BoxAny>());
        let data = *data.downcast::<T>().unwrap();
        let env = Env::from(env);

        Cx::with_context(env, move |mut cx| data.finalize(&mut cx));
    }

    // # Safety
    // The `env` value was obtained from a valid `Cx` and the `o` handle has
    // already been verified to be an object.
    unsafe {
        match sys::wrap(
            env,
            o,
            v.cast(),
            Some(finalize::<T>),
            ptr::null_mut(),
            ptr::null_mut(),
        ) {
            Err(sys::Status::InvalidArg) => {
                // Wrap failed, we can safely free the value
                let _ = Box::from_raw(v);

                return Ok(Err(WrapError::already_wrapped()));
            }
            Err(sys::Status::PendingException) => {
                // Wrap failed, we can safely free the value
                let _ = Box::from_raw(v);

                return Err(Throw::new());
            }
            // If an unexpected error occurs, we cannot safely free the value
            // because `finalize` may be called later.
            res => res.unwrap(),
        }

        #[cfg(feature = "napi-8")]
        match sys::type_tag_object(env, o, &*crate::MODULE_TAG) {
            Err(sys::Status::InvalidArg) => {
                sys::remove_wrap(env, o, ptr::null_mut()).unwrap();

                // Unwrap succeeded, we can safely free the value
                let _ = Box::from_raw(v);

                return Ok(Err(WrapError::foreign_type()));
            }
            res => res.unwrap(),
        }
    }

    Ok(Ok(()))
}

pub fn unwrap<'cx, T, V>(cx: &mut Cx, o: Handle<'cx, V>) -> NeonResult<Result<&'cx T, WrapError>>
where
    T: Finalize + 'static,
    V: Value,
{
    let env = cx.env().to_raw();
    let o = o.to_local();

    #[cfg(feature = "napi-8")]
    // # Safety
    // The `env` value was obtained from a valid `Cx`.
    unsafe {
        let mut is_tagged = false;

        match sys::check_object_type_tag(env, o, &*crate::MODULE_TAG, &mut is_tagged) {
            Err(sys::Status::PendingException) => return Err(Throw::new()),
            Err(sys::Status::ObjectExpected) => return Ok(Err(WrapError::object_expected())),
            res => res.unwrap(),
        }

        if !is_tagged {
            return Ok(Err(WrapError::not_wrapped()));
        }
    }

    // # Safety
    // The `env` value was obtained from a valid `Cx`.
    let data = unsafe {
        let mut data = MaybeUninit::<*mut BoxAny>::uninit();

        match sys::unwrap(env, o, data.as_mut_ptr().cast()) {
            Err(sys::Status::PendingException) => return Err(Throw::new()),
            Err(sys::Status::ObjectExpected) => return Ok(Err(WrapError::object_expected())),
            res => res.unwrap(),
        }

        // # Safety
        // Since `unwrap` was successful, we know this is a valid pointer. On Node-API
        // versions 8 and higher, we are also guaranteed it is a `BoxAny`.
        &*data.assume_init()
    };

    match data.downcast_ref() {
        Some(result) => Ok(Ok(result)),
        None => Ok(Err(WrapError::wrong_type(std::any::type_name::<T>()))),
    }
}
