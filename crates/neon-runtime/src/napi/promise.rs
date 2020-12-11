use std::mem::MaybeUninit;

use crate::napi::bindings as napi;
use crate::raw::{Env, Local};

use super::CallbackInfo;

pub unsafe fn new(env: Env) -> (napi::Deferred, Local) {
    let mut deferred = MaybeUninit::uninit();
    let mut promise = MaybeUninit::uninit();

    assert_eq!(
        napi::create_promise(env, deferred.as_mut_ptr(), promise.as_mut_ptr()),
        napi::Status::Ok,
    );

    (deferred.assume_init(), promise.assume_init())
}

pub unsafe fn resolve(env: Env, deferred: napi::Deferred, value: Local) {
    assert_eq!(napi::resolve_deferred(env, deferred, value), napi::Status::Ok);
}

pub unsafe fn reject(env: Env, deferred: napi::Deferred, value: Local) {
    assert_eq!(napi::reject_deferred(env, deferred, value), napi::Status::Ok);
}

unsafe extern "C" fn callback_wrapper<F>(
    env: Env,
    info: CallbackInfo,
) -> Local
where
    F: FnOnce(Env, Local) + Send + 'static,
{
    let mut argc = 1;
    let mut argv = [std::ptr::null_mut()];
    let mut this = MaybeUninit::uninit();
    let mut data = MaybeUninit::uninit();

    assert_eq!(
        napi::get_cb_info(
            env,
            info,
            &mut argc,
            argv.as_mut_ptr(),
            this.as_mut_ptr(),
            data.as_mut_ptr(),
        ),
        napi::Status::Ok,
    );

    debug_assert_eq!(argc, 1);

    let cb = Box::from_raw(data.assume_init() as *mut F);

    cb(env, argv[0]);

    let mut undefined = MaybeUninit::uninit();

    assert_eq!(
        napi::get_undefined(env, undefined.as_mut_ptr()),
        napi::Status::Ok,
    );

    undefined.assume_init()
}

unsafe fn future_callback<F>(env: Env, f: F) -> Local
where
    F: FnOnce(Env, Local) + Send + 'static,
{
    let mut local = MaybeUninit::uninit();

    assert_eq!(
        napi::create_function(
            env,
            std::ptr::null(),
            0,
            Some(callback_wrapper::<F>),
            Box::into_raw(Box::new(f)) as *mut _,
            local.as_mut_ptr(),
        ),
        napi::Status::Ok,
    );

    local.assume_init()
}

unsafe fn get_key(env: Env, o: Local, k: &str) -> Local {
    let mut key = MaybeUninit::uninit();

    assert_eq!(
        napi::create_string_utf8(
            env,
            k.as_ptr() as *const _,
            k.len(),
            key.as_mut_ptr(),
        ),
        napi::Status::Ok,
    );

    let mut prop = MaybeUninit::uninit();

    assert_eq!(
        napi::get_property(env, o, key.assume_init(), prop.as_mut_ptr()),
        napi::Status::Ok,
    );

    prop.assume_init()
}

unsafe fn call_promise_method(
    env: Env,
    promise: Local,
    method: &str,
    arg: Local,
) {
    let mut result = MaybeUninit::uninit();
    let argv = [arg];

    assert_eq!(
        napi::call_function(
            env,
            promise,
            get_key(env, promise, method),
            1,
            argv.as_ptr(),
            result.as_mut_ptr(),
        ),
        napi::Status::Ok,
    );
}

pub unsafe fn adapter<Resolve, Reject>(
    env: Env,
    maybe_promise: Local,
    resolve: Resolve,
    reject: Reject,
)
where
    Resolve: FnOnce(Env, Local) + Send + 'static,
    Reject: FnOnce(Env, Local) + Send + 'static,
{
    let (deferred, promise) = new(env);

    self::resolve(env, deferred, maybe_promise);

    let resolve = future_callback(env, resolve);
    let reject = future_callback(env, reject);

    call_promise_method(env, promise, "then", resolve);
    call_promise_method(env, promise, "catch", reject);
}
