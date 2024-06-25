use std::{mem::MaybeUninit, ptr};

use super::{
    bindings as napi,
    raw::{Env, Local},
};

pub unsafe fn new(out: &mut Local, env: Env, description: Local) -> bool {
    let status = napi::create_symbol(env, description, out);

    status == napi::Status::Ok
}
