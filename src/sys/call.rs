use std::{mem::MaybeUninit, ptr::null_mut};

use smallvec::SmallVec;

use super::{
    bindings as napi,
    raw::{Env, FunctionCallbackInfo, Local},
};

// Number of arguments to allocate on the stack. This should be large enough
// to cover most use cases without being wasteful.
//
// * If the number is too large, too much space is allocated and then filled
//   with `undefined`.
// * If the number is too small, getting arguments frequently takes two tries
//   and requires heap allocation.
const ARGV_SIZE: usize = 4;

#[repr(transparent)]
/// List of JavaScript arguments to a function
// `Arguments` is intended to be a small abstraction to hide the usage of
// `SmallVec` allowing changes to `ARGV_SIZE` in a single location
pub struct Arguments(SmallVec<[Local; ARGV_SIZE]>);

impl Arguments {
    #[inline]
    /// Get an argument at a specific position
    pub fn get(&self, i: usize) -> Option<Local> {
        self.0.get(i).cloned()
    }
}

pub unsafe fn is_construct(env: Env, info: FunctionCallbackInfo) -> bool {
    let mut target: MaybeUninit<Local> = MaybeUninit::zeroed();

    let status = napi::get_new_target(env, info, target.as_mut_ptr());

    assert_eq!(status, napi::Status::Ok);

    // get_new_target is guaranteed to assign to target, so it's initialized.
    let target: Local = target.assume_init();

    // By the get_new_target contract, target will either be NULL if the current
    // function was called without `new`, or a valid napi_value handle if the current
    // function was called with `new`.
    !target.is_null()
}

pub unsafe fn this(env: Env, info: FunctionCallbackInfo, out: &mut Local) {
    let status = napi::get_cb_info(env, info, null_mut(), null_mut(), out as *mut _, null_mut());
    assert_eq!(status, napi::Status::Ok);
}

/// Gets the number of arguments passed to the function.
// TODO: Remove this when `CallContext` is refactored to get call info upfront.
pub unsafe fn len(env: Env, info: FunctionCallbackInfo) -> i32 {
    let mut argc = 0usize;
    let status = napi::get_cb_info(
        env,
        info,
        &mut argc as *mut _,
        null_mut(),
        null_mut(),
        null_mut(),
    );
    assert_eq!(status, napi::Status::Ok);
    argc as i32
}

/// Returns the function arguments for a call
pub unsafe fn argv(env: Env, info: FunctionCallbackInfo) -> Arguments {
    // Allocate space on the stack for up to `ARGV_SIZE` values
    let mut argv = MaybeUninit::<[Local; ARGV_SIZE]>::uninit();

    // Starts as the size allocated; after `get_cb_info` it is the number of arguments
    let mut argc = ARGV_SIZE;

    assert_eq!(
        napi::get_cb_info(
            env,
            info,
            &mut argc as *mut _,
            argv.as_mut_ptr().cast(),
            null_mut(),
            null_mut(),
        ),
        napi::Status::Ok,
    );

    // We did not allocate enough space; allocate on the heap and try again
    let argv = if argc > ARGV_SIZE {
        // We know exactly how much space to reserve
        let mut argv = Vec::with_capacity(argc);

        assert_eq!(
            napi::get_cb_info(
                env,
                info,
                &mut argc as *mut _,
                argv.as_mut_ptr(),
                null_mut(),
                null_mut(),
            ),
            napi::Status::Ok,
        );

        // Set the size of `argv` to the number of initialized elements
        argv.set_len(argc);
        SmallVec::from_vec(argv)

        // There were `ARGV_SIZE` or fewer arguments, use the stack allocated space
    } else {
        SmallVec::from_buf_and_len(argv.assume_init(), argc)
    };

    Arguments(argv)
}
