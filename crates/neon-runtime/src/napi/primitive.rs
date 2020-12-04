use crate::raw::{Local, Env};
use crate::napi::bindings as napi;

/// Mutates the `out` argument provided to refer to the global `undefined` object.
pub unsafe extern "C" fn undefined(out: &mut Local, env: Env) {
    napi::get_undefined(env, out as *mut Local);
}

/// Mutates the `out` argument provided to refer to the global `null` object.
pub unsafe extern "C" fn null(out: &mut Local, env: Env) {
    napi::get_null(env, out as *mut Local);
}

/// Mutates the `out` argument provided to refer to one of the global `true` or `false` objects.
pub unsafe extern "C" fn boolean(out: &mut Local, env: Env, b: bool) {
    napi::get_boolean(env, b, out as *mut Local);
}

/// Get the boolean value out of a `Local` object. If the `Local` object does not contain a
/// boolean, this function panics.
pub unsafe extern "C" fn boolean_value(env: Env, p: Local) -> bool {
    let mut value = false;
    assert_eq!(napi::get_value_bool(env, p, &mut value as *mut bool), napi::Status::Ok);
    value
}

// DEPRECATE(0.2)
pub unsafe extern "C" fn integer(_out: &mut Local, _isolate: Env, _x: i32) { unimplemented!() }

pub unsafe extern "C" fn is_u32(_p: Local) -> bool { unimplemented!() }

pub unsafe extern "C" fn is_i32(_p: Local) -> bool { unimplemented!() }

// DEPRECATE(0.2)
pub unsafe extern "C" fn integer_value(_p: Local) -> i64 { unimplemented!() }

/// Mutates the `out` argument provided to refer to a newly created `Local` containing a
/// JavaScript number.
pub unsafe extern "C" fn number(out: &mut Local, env: Env, v: f64) {
    napi::create_double(env, v, out as *mut Local);
}

/// Gets the underlying value of an `Local` object containing a JavaScript number. Panics if
/// the given `Local` is not a number.
pub unsafe extern "C" fn number_value(env: Env, p: Local) -> f64 {
    let mut value = 0.0;
    assert_eq!(napi::get_value_double(env, p, &mut value as *mut f64), napi::Status::Ok);
    return value;
}
