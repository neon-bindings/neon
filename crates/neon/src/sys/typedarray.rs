use std::{ffi::c_void, mem::MaybeUninit};

use super::{
    bindings::{self as napi, TypedArrayType},
    raw::{Env, Local},
};

#[derive(Debug)]
/// Information describing a JavaScript [`TypedArray`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/TypedArray)
pub struct TypedArrayInfo {
    pub typ: TypedArrayType,
    pub length: usize,
    pub data: *mut c_void,
    pub buf: Local,
    pub offset: usize,
}

/// Get [information](TypedArrayInfo) describing a JavaScript `TypedArray`
///
/// # Safety
/// * `env` must be valid `napi_env` for the current scope
/// * `value` must be a handle pointing to a `TypedArray`
pub unsafe fn info(env: Env, value: Local) -> TypedArrayInfo {
    let mut info = MaybeUninit::<TypedArrayInfo>::zeroed();
    let ptr = info.as_mut_ptr();

    assert_eq!(
        napi::get_typedarray_info(
            env,
            value,
            &mut (*ptr).typ,
            &mut (*ptr).length,
            &mut (*ptr).data,
            &mut (*ptr).buf,
            &mut (*ptr).offset,
        ),
        napi::Status::Ok,
    );

    info.assume_init()
}
