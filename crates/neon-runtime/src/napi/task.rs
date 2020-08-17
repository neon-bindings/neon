use std::mem::MaybeUninit;
use std::os::raw::c_void;

use nodejs_sys as napi;

use raw::{Env, Local};

type Perform = unsafe extern fn(*mut c_void) -> *mut c_void;
type Complete = unsafe extern fn(*mut c_void, *mut c_void, *mut c_void, &mut Local);

// Stores state for async work being executed
struct AsyncWork {
    // User-provided data context. This is passed to perform and complete
    task: *mut c_void,
    // Persistent reference to the callback function to prevent GC 
    callback: napi::napi_ref,
    // Handle to async work being queued
    request: napi::napi_async_work,
    // User provided function to perform asynchronously
    perform: Perform,
    // User provided function to call on the main-thread after task is complete
    complete: Complete,
    // Output pointer passed to `complete` function
    output: *mut c_void,
}

unsafe fn null(env: napi::napi_env) -> Local {
    let mut result = MaybeUninit::uninit();

    assert_eq!(
        napi::napi_get_null(env, result.as_mut_ptr()),
        napi::napi_status::napi_ok,
    );

    result.assume_init()
}

unsafe fn undefined(env: napi::napi_env) -> Local {
    let mut result = MaybeUninit::uninit();

    assert_eq!(
        napi::napi_get_undefined(env, result.as_mut_ptr()),
        napi::napi_status::napi_ok,
    );

    result.assume_init()
}

unsafe fn global(env: napi::napi_env) -> Local {
    let mut result = MaybeUninit::uninit();

    assert_eq!(
        napi::napi_get_global(env, result.as_mut_ptr()),
        napi::napi_status::napi_ok,
    );

    result.assume_init()
}

// Resource name to identify async work in diagnostic information
unsafe fn async_work_name(env: napi::napi_env) -> Local {
    let name = b"neon task";
    let mut result = MaybeUninit::uninit();
    let status = napi::napi_create_string_utf8(
        env,
        name.as_ptr() as *const _,
        name.len(),
        result.as_mut_ptr(),
    );

    assert_eq!(status, napi::napi_status::napi_ok);

    result.assume_init()
}

// Dereference async work and execute perform function
// Executed on the libuv thread pool
unsafe extern "C" fn perform_work(_: napi::napi_env, data: *mut c_void) {
    let mut work = &mut *(data as *mut AsyncWork);

    work.output = (work.perform)(work.task);
}

// Complete async work on the main thread
unsafe extern "C" fn complete_work(
    env: napi::napi_env,
    status: napi::napi_status,
    data: *mut c_void,
) {
    // Unbox as the first step to ensure destructor runs
    let work = Box::from_raw(data as *mut AsyncWork);

    // Output pointer for the result of user provided `complete` function
    // Should use `zeroed` in case complete does not initialize
    let mut complete_output = MaybeUninit::zeroed();
    let AsyncWork {
        task,
        callback: callback_ref,
        request,
        complete,
        output,
        ..
    } = *work;

    // Helper method to delete references when done
    let delete_refs = || {
        let delete_ref_status = napi::napi_delete_reference(env, callback_ref);
        let delete_work_status = napi::napi_delete_async_work(env, request);
    
        assert_eq!(delete_ref_status, napi::napi_status::napi_ok);
        assert_eq!(delete_work_status, napi::napi_status::napi_ok);
    };

    // Neon does not support cancelling tasks but there may be other ways for
    // tasks to become cancelled.
    // Drop references to prevent a leak and return early
    if status == napi::napi_status::napi_cancelled {
        delete_refs();

        return;
    }

    // Execute the user provided complete function with the output of `perform`
    // `complete` must not panic or memory may leak.
    complete(env as *mut _, task, output, &mut *complete_output.as_mut_ptr());

    // Unwrap the reference to the callback
    let mut result = MaybeUninit::uninit();
    let status = napi::napi_get_reference_value(
        env,
        callback_ref,
        result.as_mut_ptr(),
    );

    // Work has been completed and references can be dropped. This should
    // be performed early to ensure it is not skipped by a panic.
    delete_refs();

    // Do not assert that callback was dereferenced successfully until
    // after it was dropped to prevent a leak
    let callback = {
        assert_eq!(status, napi::napi_status::napi_ok);

        result.assume_init()
    };

    // Determine if an exception has been thrown
    let mut result = MaybeUninit::uninit();
    let status = napi::napi_is_exception_pending(env, result.as_mut_ptr());
    let is_exception = {
        assert_eq!(status, napi::napi_status::napi_ok);

        result.assume_init()
    };

    // An exception has been thrown, call with `callback(err)`
    let mut argv = if is_exception {
        let mut result = MaybeUninit::uninit();
        let status = napi::napi_get_and_clear_last_exception(
            env,
            result.as_mut_ptr(),
        );

        let exception = {
            assert_eq!(status, napi::napi_status::napi_ok);

            result.assume_init()
        };

        [exception, undefined(env)]

    // Completed successfully, call with `callback(null, output)`
    } else {
        [null(env), complete_output.assume_init()]
    };

    assert_eq!(
        napi::napi_call_function(
            env,
            global(env),
            callback,
            2,
            argv.as_mut_ptr(),
            MaybeUninit::uninit().as_mut_ptr(),
        ),
        napi::napi_status::napi_ok,
    );
}



// Schedule work to be performed asynchronously
pub unsafe extern "C" fn schedule(
    env: Env,
    task: *mut c_void,
    perform: Perform,
    complete: Complete,
    callback: Local
) {
    let name = async_work_name(env);

    // Create a persistent reference to the callback
    let mut result = MaybeUninit::uninit();
    let status = napi::napi_create_reference(
        env,
        callback,
        1,
        result.as_mut_ptr(),
    );

    let callback = {
        assert_eq!(status, napi::napi_status::napi_ok);

        result.assume_init()
    };

    // Create struct to hold state for async work
    // WARN: `AsyncWork` is self-referential. It becomes the `data` pointer
    // in the async work. `request` is initialized empty and later assigned.
    let mut work = Box::new(AsyncWork {
        task,
        callback,
        // Placeholder for the async request after it is created
        request: std::ptr::null_mut(),
        perform,
        complete,
        // Placeholder for the result of the `perform` function
        output: std::ptr::null_mut(),
    });

    // Create the async work and assign it to the `request` placeholder in `work`
    let request = &mut work.request as *mut _;
    let status = napi::napi_create_async_work(
        env,
        std::ptr::null_mut(),
        name,
        Some(perform_work),
        Some(complete_work),
        Box::into_raw(work) as *mut _,
        request,
    );

    // If the async work failed to create, delete the callback reference
    // to prevent a leak.
    if status != napi::napi_status::napi_ok {
        assert_eq!(
            napi::napi_delete_reference(env, callback),
            napi::napi_status::napi_ok,
        );
        assert_eq!(status, napi::napi_status::napi_ok);
    }

    // Queue the async work
    let status = napi::napi_queue_async_work(env, *request);

    // If the work failed to queue, delete the work and the reference
    // to prevent a leak.
    if status != napi::napi_status::napi_ok {
        let delete_ref_status = napi::napi_delete_reference(env, callback);
        let delete_work_status = napi::napi_delete_async_work(env, *request);

        assert_eq!(delete_ref_status, napi::napi_status::napi_ok);
        assert_eq!(delete_work_status, napi::napi_status::napi_ok);
        assert_eq!(status, napi::napi_status::napi_ok);
    }
}
