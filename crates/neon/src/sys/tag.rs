use super::{
    bindings as napi,
    raw::{Env, Local},
};

/// Return true if an `napi_value` `val` has the expected value type.
unsafe fn is_type(env: Env, val: Local, expect: napi::ValueType) -> bool {
    let mut actual = napi::ValueType::Undefined;
    unsafe { napi::typeof_value(env, val, &mut actual as *mut _) }.unwrap();
    actual == expect
}

pub unsafe fn is_undefined(env: Env, val: Local) -> bool {
    unsafe { is_type(env, val, napi::ValueType::Undefined) }
}

pub unsafe fn is_null(env: Env, val: Local) -> bool {
    unsafe { is_type(env, val, napi::ValueType::Null) }
}

/// Is `val` a JavaScript number?
pub unsafe fn is_number(env: Env, val: Local) -> bool {
    unsafe { is_type(env, val, napi::ValueType::Number) }
}

/// Is `val` a JavaScript boolean?
pub unsafe fn is_boolean(env: Env, val: Local) -> bool {
    unsafe { is_type(env, val, napi::ValueType::Boolean) }
}

/// Is `val` a JavaScript string?
pub unsafe fn is_string(env: Env, val: Local) -> bool {
    unsafe { is_type(env, val, napi::ValueType::String) }
}

pub unsafe fn is_object(env: Env, val: Local) -> bool {
    unsafe { is_type(env, val, napi::ValueType::Object) }
}

pub unsafe fn is_array(env: Env, val: Local) -> bool {
    let mut result = false;
    unsafe { napi::is_array(env, val, &mut result as *mut _) }.unwrap();
    result
}

pub unsafe fn is_function(env: Env, val: Local) -> bool {
    unsafe { is_type(env, val, napi::ValueType::Function) }
}

pub unsafe fn is_error(env: Env, val: Local) -> bool {
    let mut result = false;
    unsafe { napi::is_error(env, val, &mut result as *mut _) }.unwrap();
    result
}

/// Is `val` a Node.js Buffer instance?
pub unsafe fn is_buffer(env: Env, val: Local) -> bool {
    let mut result = false;
    unsafe { napi::is_buffer(env, val, &mut result as *mut _) }.unwrap();
    result
}

/// Is `val` an ArrayBuffer instance?
pub unsafe fn is_arraybuffer(env: Env, val: Local) -> bool {
    let mut result = false;
    unsafe { napi::is_arraybuffer(env, val, &mut result as *mut _) }.unwrap();
    result
}

/// Is `val` a TypedArray instance?
pub unsafe fn is_typedarray(env: Env, val: Local) -> bool {
    let mut result = false;
    unsafe { napi::is_typedarray(env, val, &mut result as *mut _) }.unwrap();
    result
}

#[cfg(feature = "napi-5")]
pub unsafe fn is_date(env: Env, val: Local) -> bool {
    let mut result = false;
    unsafe { napi::is_date(env, val, &mut result as *mut _) }.unwrap();
    result
}

/// Is `val` a Promise?
///
/// # Safety
/// * `env` is a valid `napi_env` for the current thread
pub unsafe fn is_promise(env: Env, val: Local) -> bool {
    let mut result = false;
    unsafe { napi::is_promise(env, val, &mut result as *mut _) }.unwrap();
    result
}

#[cfg(feature = "napi-8")]
pub unsafe fn type_tag_object(env: Env, object: Local, tag: &super::TypeTag) {
    unsafe { napi::type_tag_object(env, object, tag as *const _) }.unwrap();
}

#[cfg(feature = "napi-8")]
pub unsafe fn check_object_type_tag(env: Env, object: Local, tag: &super::TypeTag) -> bool {
    let mut result = false;

    unsafe { napi::check_object_type_tag(env, object, tag as *const _, &mut result as *mut _) }
        .unwrap();
    result
}

#[cfg(feature = "napi-6")]
pub unsafe fn is_bigint(env: Env, val: Local) -> bool {
    unsafe { is_type(env, val, napi::ValueType::BigInt) }
}
