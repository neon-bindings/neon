use super::{
    bindings as napi,
    raw::{Env, Local},
};

pub unsafe fn to_string(out: &mut Local, env: Env, value: Local) -> bool {
    let status = napi::coerce_to_string(env, value, out as *mut _);

    status == napi::Status::Ok
}
