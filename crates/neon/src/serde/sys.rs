//! Node-API wrappers used by serde transcoding
//!
//! In many cases, these functions provide similar functionality to functions
//! available elsewhere in `neon-`. However, keeping serde fully self contained
//! has a few benefits:
//!
//! * Wrappers can be written, altered, combined and otherwise optimized for
//!   providing the most efficient possible serde implementation
//! * All errors can be forwarded to avoid additional checks and panics
//! * The serde implementation remains self contained for potential extraction
//!   into a separate crate
//!
//! _Do not export anything from this file outside of the serde module._

use std::{mem::MaybeUninit, ptr, slice};

use crate::sys;

pub(super) use crate::sys::{Env, Status, Value, ValueType};

// Extension trait to more easily return early from a failed Node-API call
trait Verify {
    fn verify(self) -> Result<(), Status>;
}

impl Verify for Status {
    fn verify(self) -> Result<(), Status> {
        if self == Status::Ok {
            Ok(())
        } else {
            Err(self)
        }
    }
}

pub(super) unsafe fn create_bool(env: Env, v: bool) -> Result<Value, Status> {
    let mut value = MaybeUninit::uninit();
    sys::get_boolean(env, v, value.as_mut_ptr()).verify()?;
    Ok(value.assume_init())
}

pub(super) unsafe fn create_double(env: Env, v: impl Into<f64>) -> Result<Value, Status> {
    let mut value = MaybeUninit::uninit();
    sys::create_double(env, v.into(), value.as_mut_ptr()).verify()?;
    Ok(value.assume_init())
}

pub(super) unsafe fn create_string(env: Env, v: impl AsRef<str>) -> Result<Value, Status> {
    let mut value = MaybeUninit::uninit();
    let v = v.as_ref();
    sys::create_string_utf8(env, v.as_ptr().cast(), v.len(), value.as_mut_ptr()).verify()?;
    Ok(value.assume_init())
}

pub(super) unsafe fn create_arraybuffer(env: Env, v: &[u8]) -> Result<Value, Status> {
    let mut value = MaybeUninit::uninit();
    let mut data = MaybeUninit::uninit();

    sys::create_arraybuffer(env, v.len(), data.as_mut_ptr(), value.as_mut_ptr()).verify()?;

    let value = value.assume_init();
    let data = slice::from_raw_parts_mut(data.assume_init().cast(), v.len());

    data.copy_from_slice(v);

    Ok(value)
}

pub(super) unsafe fn get_null(env: Env) -> Result<Value, Status> {
    let mut value = MaybeUninit::uninit();
    sys::get_null(env, value.as_mut_ptr()).verify()?;
    Ok(value.assume_init())
}

pub(super) unsafe fn create_object(env: Env) -> Result<Value, Status> {
    let mut value = MaybeUninit::uninit();
    sys::create_object(env, value.as_mut_ptr()).verify()?;
    Ok(value.assume_init())
}

pub(super) unsafe fn set_property(env: Env, o: Value, k: Value, v: Value) -> Result<(), Status> {
    sys::set_property(env, o, k, v).verify()?;
    Ok(())
}

pub(super) unsafe fn set_named_property(
    env: Env,
    o: Value,
    k: impl AsRef<str>,
    v: Value,
) -> Result<(), Status> {
    sys::set_property(env, o, create_string(env, k)?, v).verify()?;
    Ok(())
}

pub(super) unsafe fn create_array_with_length(env: Env, len: usize) -> Result<Value, Status> {
    let mut value = MaybeUninit::uninit();
    sys::create_array_with_length(env, len, value.as_mut_ptr()).verify()?;
    Ok(value.assume_init())
}

pub(super) unsafe fn set_element(env: Env, arr: Value, k: u32, v: Value) -> Result<(), Status> {
    sys::set_element(env, arr, k, v).verify()?;
    Ok(())
}

pub(super) unsafe fn get_array_length(env: Env, value: Value) -> Result<u32, Status> {
    let mut len = 0u32;
    sys::get_array_length(env, value, &mut len as *mut u32).verify()?;
    Ok(len)
}

pub(super) unsafe fn get_element(env: Env, arr: Value, i: u32) -> Result<Value, Status> {
    let mut out = MaybeUninit::uninit();
    sys::get_element(env, arr, i, out.as_mut_ptr()).verify()?;
    Ok(out.assume_init())
}

pub(super) unsafe fn get_property_names(env: Env, value: Value) -> Result<Value, Status> {
    let mut out = MaybeUninit::uninit();
    sys::get_property_names(env, value, out.as_mut_ptr()).verify()?;
    Ok(out.assume_init())
}

pub(super) unsafe fn typeof_value(env: Env, value: Value) -> Result<ValueType, Status> {
    let mut out = MaybeUninit::uninit();
    sys::typeof_value(env, value, out.as_mut_ptr()).verify()?;
    Ok(out.assume_init())
}

pub(super) unsafe fn is_array(env: Env, value: Value) -> Result<bool, Status> {
    let mut result = false;
    sys::is_array(env, value, &mut result).verify()?;
    Ok(result)
}

pub(super) unsafe fn get_value_bool(env: Env, value: Value) -> Result<bool, Status> {
    let mut out = false;
    sys::get_value_bool(env, value, &mut out as *mut bool).verify()?;
    Ok(out)
}

pub(super) unsafe fn get_value_double(env: Env, value: Value) -> Result<f64, Status> {
    let mut out = 0f64;
    sys::get_value_double(env, value, &mut out as *mut f64).verify()?;
    Ok(out)
}

unsafe fn get_string_len(env: Env, value: Value) -> Result<usize, Status> {
    let mut out = 0usize;
    sys::get_value_string_utf8(env, value, ptr::null_mut(), 0, &mut out as *mut usize).verify()?;
    Ok(out)
}

pub(super) unsafe fn get_value_string(env: Env, value: Value) -> Result<String, Status> {
    let mut out = 0usize;
    let string_len = get_string_len(env, value)?;
    let buf_len = string_len + 1;
    let mut buf = Vec::<u8>::with_capacity(buf_len);

    sys::get_value_string_utf8(
        env,
        value,
        buf.as_mut_ptr().cast(),
        buf_len,
        &mut out as *mut usize,
    )
    .verify()?;

    debug_assert_eq!(out, string_len);
    buf.set_len(string_len);

    Ok(String::from_utf8_unchecked(buf))
}

pub(super) unsafe fn get_value_arraybuffer(env: Env, value: Value) -> Result<Vec<u8>, Status> {
    let mut len = 0usize;
    let mut out = MaybeUninit::uninit();

    sys::get_arraybuffer_info(env, value, out.as_mut_ptr(), &mut len as *mut usize).verify()?;

    let buf = if len == 0 {
        &[]
    } else {
        slice::from_raw_parts(out.assume_init().cast(), len)
    };

    Ok(buf.to_vec())
}

pub(super) unsafe fn get_value_arrayview(env: Env, value: Value) -> Result<Vec<u8>, Status> {
    let mut len = 0usize;
    let mut typ = MaybeUninit::uninit();
    let mut out = MaybeUninit::uninit();

    sys::get_typedarray_info(
        env,
        value,
        typ.as_mut_ptr(),
        &mut len,
        out.as_mut_ptr(),
        ptr::null_mut(),
        ptr::null_mut(),
    )
    .verify()?;

    if !matches!(
        typ.assume_init(),
        sys::TypedArrayType::U8 | sys::TypedArrayType::U8Clamped
    ) {
        return Err(Status::InvalidArg);
    }

    let buf = if len == 0 {
        &[]
    } else {
        slice::from_raw_parts(out.assume_init().cast(), len)
    };

    Ok(buf.to_vec())
}

pub(super) unsafe fn get_property(env: Env, object: Value, key: Value) -> Result<Value, Status> {
    let mut out = MaybeUninit::uninit();
    sys::get_property(env, object, key, out.as_mut_ptr()).verify()?;
    Ok(out.assume_init())
}

pub(super) unsafe fn get_named_property(
    env: Env,
    object: Value,
    key: impl AsRef<str>,
) -> Result<Value, Status> {
    get_property(env, object, create_string(env, key)?)
}

pub(super) unsafe fn get_interned_string(env: Env, s: &'static str) -> Result<Value, Status> {
    use std::{cell::RefCell, collections::{HashMap, hash_map::Entry}};

    // FIXME: Do this correctly
    thread_local! {
        pub static INTERNED: RefCell<HashMap<*const str, sys::Ref>> = RefCell::new(HashMap::new());
    }

    INTERNED.with(|interned| match interned.borrow_mut().entry(s) {
        Entry::Occupied(root) => {
            let mut v = MaybeUninit::uninit();
            sys::get_reference_value(env, *root.get(), v.as_mut_ptr()).verify()?;
            Ok(v.assume_init())
        }
        Entry::Vacant(entry) => {
            let global = {
                let mut global = MaybeUninit::uninit();
                sys::get_global(env, global.as_mut_ptr()).verify()?;
                global.assume_init()
            };

            let v = {
                let s = create_string(env, s)?;
                let ctor = get_named_property(env, global, "String")?;
                let mut v = MaybeUninit::uninit();
                sys::new_instance(env, ctor, 1, [s].as_ptr(), v.as_mut_ptr()).verify()?;
                v.assume_init()
            };

            let root = {
                let mut root = MaybeUninit::uninit();
                sys::create_reference(env, v, 1, root.as_mut_ptr()).verify()?;
                root.assume_init()
            };

            entry.insert(root);

            Ok(v)
        }
    })
}
