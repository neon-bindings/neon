#![allow(clippy::too_many_arguments)]

mod napi1 {
    use super::super::types::*;
    use std::os::raw::{c_char, c_void};

    generate!(
        extern "C" {
            fn get_undefined(env: Env, result: *mut Value) -> Status;

            fn get_null(env: Env, result: *mut Value) -> Status;

            fn get_global(env: Env, result: *mut Value) -> Status;

            fn get_boolean(env: Env, value: bool, result: *mut Value) -> Status;

            fn create_double(env: Env, value: f64, result: *mut Value) -> Status;

            fn create_object(env: Env, result: *mut Value) -> Status;

            fn get_value_bool(env: Env, value: Value, result: *mut bool) -> Status;

            fn get_value_double(env: Env, value: Value, result: *mut f64) -> Status;

            fn create_array_with_length(env: Env, length: usize, result: *mut Value) -> Status;

            fn get_array_length(env: Env, value: Value, result: *mut u32) -> Status;

            fn get_new_target(env: Env, cbinfo: CallbackInfo, result: *mut Value) -> Status;

            fn coerce_to_object(env: Env, value: Value, result: *mut Value) -> Status;

            fn coerce_to_string(env: Env, value: Value, result: *mut Value) -> Status;

            fn throw(env: Env, error: Value) -> Status;

            fn create_error(env: Env, code: Value, msg: Value, result: *mut Value) -> Status;

            fn get_and_clear_last_exception(env: Env, result: *mut Value) -> Status;

            fn is_exception_pending(env: Env, result: *mut bool) -> Status;

            fn get_value_external(env: Env, value: Value, result: *mut *mut c_void) -> Status;

            fn typeof_value(env: Env, value: Value, result: *mut ValueType) -> Status;

            fn close_escapable_handle_scope(env: Env, scope: EscapableHandleScope) -> Status;

            fn open_escapable_handle_scope(env: Env, result: *mut EscapableHandleScope) -> Status;

            fn open_handle_scope(env: Env, result: *mut HandleScope) -> Status;

            fn close_handle_scope(env: Env, scope: HandleScope) -> Status;

            fn is_arraybuffer(env: Env, value: Value, result: *mut bool) -> Status;
            fn is_typedarray(env: Env, value: Value, result: *mut bool) -> Status;
            fn is_buffer(env: Env, value: Value, result: *mut bool) -> Status;
            fn is_error(env: Env, value: Value, result: *mut bool) -> Status;
            fn is_array(env: Env, value: Value, result: *mut bool) -> Status;

            fn get_value_string_utf8(
                env: Env,
                value: Value,
                buf: *mut c_char,
                bufsize: usize,
                result: *mut usize,
            ) -> Status;

            fn create_type_error(env: Env, code: Value, msg: Value, result: *mut Value) -> Status;

            fn create_range_error(env: Env, code: Value, msg: Value, result: *mut Value) -> Status;

            fn create_string_utf8(
                env: Env,
                str: *const c_char,
                length: usize,
                result: *mut Value,
            ) -> Status;

            fn create_arraybuffer(
                env: Env,
                byte_length: usize,
                data: *mut *mut c_void,
                result: *mut Value,
            ) -> Status;

            fn get_arraybuffer_info(
                env: Env,
                arraybuffer: Value,
                data: *mut *mut c_void,
                byte_length: *mut usize,
            ) -> Status;

            fn get_typedarray_info(
                env: Env,
                typedarray: Value,
                typ: *mut TypedArrayType,
                length: *mut usize,
                data: *mut *mut c_void,
                buf: *mut Value,
                offset: *mut usize,
            ) -> Status;

            fn create_buffer(
                env: Env,
                length: usize,
                data: *mut *mut c_void,
                result: *mut Value,
            ) -> Status;

            fn get_buffer_info(
                env: Env,
                value: Value,
                data: *mut *mut c_void,
                length: *mut usize,
            ) -> Status;

            fn get_cb_info(
                env: Env,
                cbinfo: CallbackInfo,
                argc: *mut usize,
                argv: *mut Value,
                this_arg: *mut Value,
                data: *mut *mut c_void,
            ) -> Status;

            fn create_external(
                env: Env,
                data: *mut c_void,
                finalize_cb: Finalize,
                finalize_hint: *mut c_void,
                result: *mut Value,
            ) -> Status;

            fn new_instance(
                env: Env,
                constructor: Value,
                argc: usize,
                argv: *const Value,
                result: *mut Value,
            ) -> Status;

            fn call_function(
                env: Env,
                recv: Value,
                func: Value,
                argc: usize,
                argv: *const Value,
                result: *mut Value,
            ) -> Status;

            fn create_function(
                env: Env,
                utf8name: *const c_char,
                length: usize,
                cb: Callback,
                data: *mut c_void,
                result: *mut Value,
            ) -> Status;

            fn set_property(env: Env, object: Value, key: Value, value: Value) -> Status;

            fn get_property(env: Env, object: Value, key: Value, result: *mut Value) -> Status;

            fn set_element(env: Env, object: Value, index: u32, value: Value) -> Status;

            fn get_element(env: Env, object: Value, index: u32, result: *mut Value) -> Status;

            fn escape_handle(
                env: Env,
                scope: EscapableHandleScope,
                escapee: Value,
                result: *mut Value,
            ) -> Status;

            fn create_reference(
                env: Env,
                value: Value,
                initial_ref_count: u32,
                result: *mut Ref,
            ) -> Status;

            fn reference_ref(env: Env, reference: Ref, result: *mut u32) -> Status;

            fn reference_unref(env: Env, reference: Ref, result: *mut u32) -> Status;

            fn delete_reference(env: Env, reference: Ref) -> Status;

            fn get_reference_value(env: Env, reference: Ref, result: *mut Value) -> Status;

            fn strict_equals(env: Env, lhs: Value, rhs: Value, result: *mut bool) -> Status;

            fn create_external_arraybuffer(
                env: Env,
                data: *mut c_void,
                length: usize,
                finalize_cb: Finalize,
                finalize_hint: *mut c_void,
                result: *mut Value,
            ) -> Status;

            fn create_external_buffer(
                env: Env,
                length: usize,
                data: *mut c_void,
                finalize_cb: Finalize,
                finalize_hint: *mut c_void,
                result: *mut Value,
            ) -> Status;

            fn run_script(env: Env, script: Value, result: *mut Value) -> Status;
        }
    );
}

#[cfg(feature = "napi-4")]
mod napi4 {
    use super::super::types::*;
    use std::os::raw::c_void;

    generate!(
        extern "C" {
            fn create_threadsafe_function(
                env: Env,
                func: Value,
                async_resource: Value,
                async_resource_name: Value,
                max_queue_size: usize,
                initial_thread_count: usize,
                thread_finalize_data: *mut c_void,
                thread_finalize_cb: Finalize,
                context: *mut c_void,
                call_js_cb: ThreadsafeFunctionCallJs,
                result: *mut ThreadsafeFunction,
            ) -> Status;

            fn call_threadsafe_function(
                func: ThreadsafeFunction,
                data: *mut c_void,
                is_blocking: ThreadsafeFunctionCallMode,
            ) -> Status;

            fn release_threadsafe_function(
                func: ThreadsafeFunction,
                mode: ThreadsafeFunctionReleaseMode,
            ) -> Status;

            fn ref_threadsafe_function(env: Env, func: ThreadsafeFunction) -> Status;

            fn unref_threadsafe_function(env: Env, func: ThreadsafeFunction) -> Status;
        }
    );
}

#[cfg(feature = "napi-5")]
mod napi5 {
    use super::super::types::*;

    generate!(
        extern "C" {
            fn create_date(env: Env, value: f64, result: *mut Value) -> Status;

            fn get_date_value(env: Env, value: Value, result: *mut f64) -> Status;

            fn is_date(env: Env, value: Value, result: *mut bool) -> Status;
        }
    );
}

#[cfg(feature = "napi-6")]
mod napi6 {
    use super::super::types::*;
    use std::os::raw::c_void;

    generate!(
        extern "C" {
            fn get_all_property_names(
                env: Env,
                object: Value,
                key_mode: KeyCollectionMode,
                key_filter: KeyFilter,
                key_conversion: KeyConversion,
                result: *mut Value,
            ) -> Status;

            fn set_instance_data(
                env: Env,
                data: *mut c_void,
                finalize_cb: Finalize,
                finalize_hint: *mut c_void,
            ) -> Status;

            fn get_instance_data(env: Env, data: *mut *mut c_void) -> Status;
        }
    );
}

pub(crate) use napi1::*;
#[cfg(feature = "napi-4")]
pub(crate) use napi4::*;
#[cfg(feature = "napi-5")]
pub(crate) use napi5::*;
#[cfg(feature = "napi-6")]
pub(crate) use napi6::*;

use super::{Env, Status};

// This symbol is loaded separately because it is a prerequisite
unsafe fn get_version(host: &libloading::Library, env: Env) -> Result<u32, libloading::Error> {
    let get_version = host.get::<fn(Env, *mut u32) -> Status>(b"napi_get_version")?;
    let mut version = 0;

    assert_eq!(get_version(env, &mut version as *mut _), Status::Ok,);

    Ok(version)
}

pub(crate) unsafe fn load(env: Env) -> Result<(), libloading::Error> {
    #[cfg(not(windows))]
    let host = libloading::os::unix::Library::this().into();
    #[cfg(windows)]
    let host = libloading::os::windows::Library::this()?.into();

    // This never fail since `get_version` is in N-API Version 1 and the module will fail
    // with `Error: Module did not self-register` if N-API does not exist.
    let version = get_version(&host, env).expect("Failed to find N-API version");

    napi1::load(&host, version, 1)?;

    #[cfg(feature = "napi-4")]
    napi4::load(&host, version, 4)?;

    #[cfg(feature = "napi-5")]
    napi5::load(&host, version, 5)?;

    #[cfg(feature = "napi-6")]
    napi6::load(&host, version, 6)?;

    Ok(())
}
