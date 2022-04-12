use super::{
    bindings as napi,
    raw::{Env, Local},
};

pub unsafe fn strict_equals(env: Env, lhs: Local, rhs: Local) -> bool {
    let mut result = false;
    assert_eq!(
        napi::strict_equals(env, lhs, rhs, &mut result as *mut _),
        napi::Status::Ok
    );
    result
}
