use super::{
    bindings as napi,
    raw::{Env, Local},
};

pub unsafe fn strict_equals(env: Env, lhs: Local, rhs: Local) -> bool {
    let mut result = false;
    napi::strict_equals(env, lhs, rhs, &mut result as *mut _).unwrap();
    result
}
