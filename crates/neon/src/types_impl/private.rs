use std::{ffi::c_void, mem::MaybeUninit};

use crate::{
    context::{internal::Env, Context},
    handle::{internal::TransparentNoCopyWrapper, Handle},
    result::{JsResult, NeonResult, Throw},
    sys::{self, bindings as napi, raw},
    types::Value,
};

use super::JsValue;

// Maximum number of function arguments in V8.
const V8_ARGC_LIMIT: usize = 65535;

pub(crate) unsafe fn prepare_call<'a, 'b, C: Context<'a>>(
    cx: &mut C,
    args: &[Handle<'b, JsValue>],
) -> NeonResult<(usize, *const c_void)> {
    // Note: This cast is only save because `Handle<'_, JsValue>` is
    // guaranteed to have the same layout as a pointer because `Handle`
    // and `JsValue` are both `repr(C)` newtypes.
    let argv = args.as_ptr().cast();
    let argc = args.len();
    if argc > V8_ARGC_LIMIT {
        return cx.throw_range_error("too many arguments");
    }
    Ok((argc, argv))
}

pub trait ValueInternal: TransparentNoCopyWrapper + 'static {
    fn name() -> &'static str;

    fn is_typeof<Other: Value>(env: Env, other: &Other) -> bool;

    fn downcast<Other: Value>(env: Env, other: &Other) -> Option<Self> {
        if Self::is_typeof(env, other) {
            // # Safety
            // `is_typeof` check ensures this is the correct JavaScript type
            Some(unsafe { Self::from_local(env, other.to_local()) })
        } else {
            None
        }
    }

    fn cast<'a, T: Value, F: FnOnce(raw::Local) -> T>(self, f: F) -> Handle<'a, T> {
        Handle::new_internal(f(self.to_local()))
    }

    fn to_local(&self) -> raw::Local;

    // # Safety
    // JavaScript value must be of type `Self`
    unsafe fn from_local(env: Env, h: raw::Local) -> Self;

    unsafe fn try_call<'a, 'b, C: Context<'a>, T, AS>(
        &self,
        cx: &mut C,
        this: Handle<'b, T>,
        args: AS,
    ) -> JsResult<'a, JsValue>
    where
        T: Value,
        AS: AsRef<[Handle<'b, JsValue>]>,
    {
        let callee = self.to_local();
        let (argc, argv) = unsafe { prepare_call(cx, args.as_ref()) }?;
        let env = cx.env();
        let mut result: MaybeUninit<raw::Local> = MaybeUninit::zeroed();

        let status = napi::call_function(
            env.to_raw(),
            this.to_local(),
            callee,
            argc,
            argv.cast(),
            result.as_mut_ptr(),
        );

        check_call_status(cx, callee, status)?;

        Ok(Handle::new_internal(JsValue::from_local(
            env,
            result.assume_init(),
        )))
    }

    unsafe fn try_construct<'a, 'b, C: Context<'a>, AS>(
        &self,
        cx: &mut C,
        args: AS,
    ) -> JsResult<'a, JsValue>
    where
        AS: AsRef<[Handle<'b, JsValue>]>,
    {
        let callee = self.to_local();
        let (argc, argv) = unsafe { prepare_call(cx, args.as_ref()) }?;
        let env = cx.env();
        let mut result: MaybeUninit<raw::Local> = MaybeUninit::zeroed();
        let status =
            napi::new_instance(env.to_raw(), callee, argc, argv.cast(), result.as_mut_ptr());

        check_call_status(cx, callee, status)?;

        Ok(Handle::new_internal(JsValue::from_local(
            env,
            result.assume_init(),
        )))
    }
}

unsafe fn check_call_status<'a, C: Context<'a>>(
    cx: &mut C,
    callee: raw::Local,
    status: Result<(), sys::Status>,
) -> NeonResult<()> {
    match status {
        Err(sys::Status::InvalidArg) if !sys::tag::is_function(cx.env().to_raw(), callee) => {
            return cx.throw_error("not a function");
        }
        Err(sys::Status::PendingException) => {
            return Err(Throw::new());
        }
        status => status.unwrap(),
    }

    Ok(())
}
