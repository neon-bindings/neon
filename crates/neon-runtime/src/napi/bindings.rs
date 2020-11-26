use std::mem::MaybeUninit;
use lazy_static::lazy_static;
use libloading::{Library, Symbol};

/* Later we should do:
#[repr(C)]
struct NapiEnvStruct {}

#[repr(C)]
struct NapiValueStruct {}

pub(crate) type NapiEnv = *mut NapiEnvStruct;
pub(crate) type NapiValue = *mut NapiValueStruct;

But in this sample we still rely on nodejs_sys's types
*/

pub(crate) type NapiEnv = nodejs_sys::napi_env;
pub(crate) type NapiValue = nodejs_sys::napi_value;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(non_camel_case_types)]
#[allow(dead_code)]
pub(crate) enum NapiStatus {
    napi_ok,
    napi_invalid_arg,
    napi_object_expected,
    napi_string_expected,
    napi_name_expected,
    napi_function_expected,
    napi_number_expected,
    napi_boolean_expected,
    napi_array_expected,
    napi_generic_failure,
    napi_pending_exception,
    napi_cancelled,
    napi_escape_called_twice,
    napi_handle_scope_mismatch,
    napi_callback_scope_mismatch,
    napi_queue_full,
    napi_closing,
    napi_bigint_expected,
    napi_date_expected,
    napi_arraybuffer_expected,
    napi_detachable_arraybuffer_expected,
    napi_would_deadlock,
}

pub(crate) struct Napi<'a> {
    pub napi_get_undefined: Symbol<'a, unsafe extern "C" fn(env: NapiEnv, out: *mut NapiValue) -> NapiStatus>,
    pub napi_get_null: Symbol<'a, unsafe extern "C" fn(env: NapiEnv, out: *mut NapiValue) -> NapiStatus>,

    pub napi_get_boolean:
        Symbol<'a, unsafe extern "C" fn(env: NapiEnv, value: bool, out: *mut NapiValue) -> NapiStatus>,
    pub napi_get_value_bool:
        Symbol<'a, unsafe extern "C" fn(env: NapiEnv, value: NapiValue, out: *mut bool) -> NapiStatus>,

    pub napi_create_double:
        Symbol<'a, unsafe extern "C" fn(env: NapiEnv, value: f64, out: *mut NapiValue) -> NapiStatus>,
    pub napi_get_value_double:
        Symbol<'a, unsafe extern "C" fn(env: NapiEnv, value: NapiValue, out: *mut f64) -> NapiStatus>,
}

#[cfg(not(windows))]
fn get_host_library() -> Library {
    use libloading::os::unix::Library;
    Library::this().into()
}

#[cfg(windows)]
fn get_host_library() -> Library {
    use libloading::os::windows::Library;
    Library::this().into()
}

lazy_static! {
    static ref HOST: Library = get_host_library();
}

impl Napi<'_> {
    fn try_from_host() -> Result<Self, libloading::Error> {
        let host = &HOST;

        Ok(unsafe {
            Self {
                napi_get_undefined: host.get(b"napi_get_undefined")?,
                napi_get_null: host.get(b"napi_get_null")?,
                napi_get_boolean: host.get(b"napi_get_boolean")?,
                napi_get_value_bool: host.get(b"napi_get_value_bool")?,
                napi_create_double: host.get(b"napi_create_double")?,
                napi_get_value_double: host.get(b"napi_get_value_double")?,
            }
        })
    }

    pub fn from_host() -> Self {
        Self::try_from_host().unwrap()
    }
}

pub(crate) static mut NAPI: MaybeUninit<Napi> = MaybeUninit::uninit();

/// Load the N-API symbols we need.
pub(crate) unsafe fn load() {
    NAPI.as_mut_ptr().write(Napi::from_host());
}

macro_rules! napi {
    ( $name:ident ( $($args:expr),* ) ) => {
        {
            let bindings = $crate::napi::bindings::NAPI.as_ptr();
            let result: $crate::napi::bindings::NapiStatus = ((*bindings).$name)(
                $($args),*
            );
            result
        }
    }
}
