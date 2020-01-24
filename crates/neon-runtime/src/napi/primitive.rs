use raw::{Local, Env};

use nodejs_sys as napi;

/// Mutates the `out` argument provided to refer to the global `undefined` object.
pub unsafe extern "C" fn undefined(out: &mut Local, env: Env) {
    napi::napi_get_undefined(env, out as *mut Local);
}

/// Mutates the `out` argument provided to refer to the global `null` object.
pub unsafe extern "C" fn null(out: &mut Local, env: Env) {
    napi::napi_get_null(env, out as *mut Local);
}

pub unsafe extern "C" fn boolean(_out: &mut Local, _b: bool) { unimplemented!() }

pub unsafe extern "C" fn boolean_value(_p: Local) -> bool { unimplemented!() }

// DEPRECATE(0.2)
pub unsafe extern "C" fn integer(_out: &mut Local, _isolate: Env, _x: i32) { unimplemented!() }

pub unsafe extern "C" fn is_u32(_p: Local) -> bool { unimplemented!() }

pub unsafe extern "C" fn is_i32(_p: Local) -> bool { unimplemented!() }

// DEPRECATE(0.2)
pub unsafe extern "C" fn integer_value(_p: Local) -> i64 { unimplemented!() }

pub unsafe extern "C" fn number(_out: &mut Local, _isolate: Env, _v: f64) { unimplemented!() }

pub unsafe extern "C" fn number_value(_p: Local) -> f64 { unimplemented!() }
