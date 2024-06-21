use std::ffi::c_void;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
#[doc(hidden)]
pub struct Env__ {
    _unused: [u8; 0],
}

#[cfg_attr(docsrs, doc(cfg(feature = "napi-1")))]
/// [`napi_env`](https://nodejs.org/api/n-api.html#napi_env)
pub type Env = *mut Env__;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
#[doc(hidden)]
pub struct Value__ {
    _unused: [u8; 0],
}

#[cfg_attr(docsrs, doc(cfg(feature = "napi-1")))]
/// [`napi_value`](https://nodejs.org/api/n-api.html#napi_value)
pub type Value = *mut Value__;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
#[doc(hidden)]
pub struct CallbackInfo__ {
    _unused: [u8; 0],
}

#[cfg_attr(docsrs, doc(cfg(feature = "napi-1")))]
/// [`napi_callback_info`](https://nodejs.org/api/n-api.html#napi_callback_info)
pub type CallbackInfo = *mut CallbackInfo__;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
#[doc(hidden)]
pub struct EscapableHandleScope__ {
    _unused: [u8; 0],
}

#[cfg_attr(docsrs, doc(cfg(feature = "napi-1")))]
/// [`napi_escapable_handle_scope`](https://nodejs.org/api/n-api.html#napi_escapable_handle_scope)
pub type EscapableHandleScope = *mut EscapableHandleScope__;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
#[doc(hidden)]
pub struct HandleScope__ {
    _unused: [u8; 0],
}

#[cfg_attr(docsrs, doc(cfg(feature = "napi-1")))]
/// [`napi_handle_scope`](https://nodejs.org/api/n-api.html#napi_handle_scope)
pub type HandleScope = *mut HandleScope__;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
#[doc(hidden)]
pub struct Ref__ {
    _unused: [u8; 0],
}

#[cfg_attr(docsrs, doc(cfg(feature = "napi-1")))]
/// [`napi_ref`](https://nodejs.org/api/n-api.html#napi_ref)
pub type Ref = *mut Ref__;

#[cfg(feature = "napi-4")]
#[repr(C)]
#[derive(Debug, Copy, Clone)]
#[doc(hidden)]
pub struct UvEventLoop__ {
    _unused: [u8; 0],
}

#[cfg_attr(docsrs, doc(cfg(feature = "napi-4")))]
#[cfg(feature = "napi-4")]
/// [`napi_threadsafe_function`](https://nodejs.org/api/n-api.html#napi_threadsafe_function)
pub type UvEventLoop = *mut UvEventLoop__;

#[cfg(feature = "napi-4")]
#[repr(C)]
#[derive(Debug, Copy, Clone)]
#[doc(hidden)]
pub struct ThreadsafeFunction__ {
    _unused: [u8; 0],
}

#[cfg_attr(docsrs, doc(cfg(feature = "napi-4")))]
#[cfg(feature = "napi-4")]
/// [`napi_threadsafe_function`](https://nodejs.org/api/n-api.html#napi_threadsafe_function)
pub type ThreadsafeFunction = *mut ThreadsafeFunction__;

#[cfg_attr(docsrs, doc(cfg(feature = "napi-1")))]
/// [`napi_callback`](https://nodejs.org/api/n-api.html#napi_callback)
pub type Callback = Option<unsafe extern "C" fn(env: Env, info: CallbackInfo) -> Value>;

#[cfg_attr(docsrs, doc(cfg(feature = "napi-1")))]
/// [`napi_finalize`](https://nodejs.org/api/n-api.html#napi_finalize)
pub type Finalize =
    Option<unsafe extern "C" fn(env: Env, finalize_data: *mut c_void, finalize_hint: *mut c_void)>;

#[cfg_attr(docsrs, doc(cfg(feature = "napi-4")))]
#[cfg(feature = "napi-4")]
/// [`napi_threadsafe_function_call_js`](https://nodejs.org/api/n-api.html#napi_threadsafe_function_call_js)
pub type ThreadsafeFunctionCallJs = Option<
    unsafe extern "C" fn(env: Env, js_callback: Value, context: *mut c_void, data: *mut c_void),
>;

#[allow(dead_code)]
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(docsrs, doc(cfg(feature = "napi-1")))]
/// [`napi_status`](https://nodejs.org/api/n-api.html#napi_status)
pub enum Status {
    Ok = 0,
    InvalidArg = 1,
    ObjectExpected = 2,
    StringExpected = 3,
    NameExpected = 4,
    FunctionExpected = 5,
    NumberExpected = 6,
    BooleanExpected = 7,
    ArrayExpected = 8,
    GenericFailure = 9,
    PendingException = 10,
    Cancelled = 11,
    EscapeCalledTwice = 12,
    HandleScopeMismatch = 13,
    CallbackScopeMismatch = 14,
    QueueFull = 15,
    Closing = 16,
    BigintExpected = 17,
    DateExpected = 18,
    ArraybufferExpected = 19,
    DetachableArraybufferExpected = 20,
    WouldDeadlock = 21,
}

#[allow(dead_code)]
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(docsrs, doc(cfg(feature = "napi-6")))]
/// [`napi_valuetype`](https://nodejs.org/api/n-api.html#napi_valuetype)
pub enum ValueType {
    Undefined = 0,
    Null = 1,
    Boolean = 2,
    Number = 3,
    String = 4,
    Symbol = 5,
    Object = 6,
    Function = 7,
    External = 8,
    BigInt = 9,
}

#[allow(dead_code)]
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(docsrs, doc(cfg(feature = "napi-1")))]
/// [`napi_typedarray_type`](https://nodejs.org/api/n-api.html#napi_typedarray_type)
pub enum TypedArrayType {
    I8 = 0,
    U8 = 1,
    U8Clamped = 2,
    I16 = 3,
    U16 = 4,
    I32 = 5,
    U32 = 6,
    F32 = 7,
    F64 = 8,
    I64 = 9,
    U64 = 10,
}

#[allow(dead_code)]
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(docsrs, doc(cfg(feature = "napi-6")))]
#[cfg(feature = "napi-6")]
/// [`napi_key_collection_mode`](https://nodejs.org/api/n-api.html#napi_key_collection_mode)
pub enum KeyCollectionMode {
    IncludePrototypes = 0,
    OwnOnly = 1,
}

#[allow(dead_code)]
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(docsrs, doc(cfg(feature = "napi-6")))]
#[cfg(feature = "napi-6")]
/// [`napi_key_conversion`](https://nodejs.org/api/n-api.html#napi_key_conversion)
pub enum KeyConversion {
    KeepNumbers = 0,
    NumbersToStrings = 1,
}

#[cfg_attr(docsrs, doc(cfg(feature = "napi-4")))]
#[cfg(feature = "napi-4")]
#[allow(dead_code)]
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
/// [`napi_threadsafe_function_call_mode`](https://nodejs.org/api/n-api.html#napi_threadsafe_function_call_mode)
pub enum ThreadsafeFunctionCallMode {
    NonBlocking = 0,
    Blocking = 1,
}

#[cfg_attr(docsrs, doc(cfg(feature = "napi-4")))]
#[cfg(feature = "napi-4")]
#[allow(dead_code)]
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
/// [`napi_threadsafe_function_release_mode`](https://nodejs.org/api/n-api.html#napi_threadsafe_function_release_mode)
pub enum ThreadsafeFunctionReleaseMode {
    Release = 0,
    Abort = 1,
}

#[repr(transparent)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(docsrs, doc(cfg(feature = "napi-6")))]
#[cfg(feature = "napi-6")]
/// [`napi_key_filter`](https://nodejs.org/api/n-api.html#napi_key_filter)
pub struct KeyFilter(pub ::std::os::raw::c_uint);

#[allow(dead_code)]
#[cfg(feature = "napi-6")]
impl KeyFilter {
    pub const ALL_PROPERTIES: KeyFilter = KeyFilter(0);
    pub const WRITABLE: KeyFilter = KeyFilter(1);
    pub const CONFIGURABLE: KeyFilter = KeyFilter(4);
    pub const SKIP_STRINGS: KeyFilter = KeyFilter(8);
    pub const SKIP_SYMBOLS: KeyFilter = KeyFilter(16);
}

#[cfg(feature = "napi-6")]
impl std::ops::BitOr<KeyFilter> for KeyFilter {
    type Output = Self;
    #[inline]
    fn bitor(self, other: Self) -> Self {
        KeyFilter(self.0 | other.0)
    }
}

#[cfg(feature = "napi-6")]
impl std::ops::BitOrAssign for KeyFilter {
    #[inline]
    fn bitor_assign(&mut self, rhs: KeyFilter) {
        self.0 |= rhs.0;
    }
}

#[cfg(feature = "napi-6")]
impl std::ops::BitAnd<KeyFilter> for KeyFilter {
    type Output = Self;
    #[inline]
    fn bitand(self, other: Self) -> Self {
        KeyFilter(self.0 & other.0)
    }
}

#[cfg(feature = "napi-6")]
impl std::ops::BitAndAssign for KeyFilter {
    #[inline]
    fn bitand_assign(&mut self, rhs: KeyFilter) {
        self.0 &= rhs.0;
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
#[doc(hidden)]
pub struct AsyncWork__ {
    _unused: [u8; 0],
}

#[cfg_attr(docsrs, doc(cfg(feature = "napi-1")))]
/// [`napi_async_work`](https://nodejs.org/api/n-api.html#napi_async_work)
pub type AsyncWork = *mut AsyncWork__;

#[cfg_attr(docsrs, doc(cfg(feature = "napi-1")))]
/// [`napi_async_execute_callback`](https://nodejs.org/api/n-api.html#napi_async_execute_callback)
pub type AsyncExecuteCallback = Option<unsafe extern "C" fn(env: Env, data: *mut c_void)>;

#[cfg_attr(docsrs, doc(cfg(feature = "napi-1")))]
/// [`napi_async_complete_callback`](https://nodejs.org/api/n-api.html#napi_async_complete_callback)
pub type AsyncCompleteCallback =
    Option<unsafe extern "C" fn(env: Env, status: Status, data: *mut c_void)>;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
#[doc(hidden)]
pub struct Deferred__ {
    _unused: [u8; 0],
}

#[cfg_attr(docsrs, doc(cfg(feature = "napi-1")))]
/// [`napi_deferred`](https://nodejs.org/api/n-api.html#napi_deferred)
pub type Deferred = *mut Deferred__;

#[cfg_attr(docsrs, doc(cfg(feature = "napi-8")))]
#[cfg(feature = "napi-8")]
#[repr(C)]
#[derive(Debug, Copy, Clone)]
/// [`napi_type_tag`](https://nodejs.org/api/n-api.html#napi_type_tag)
pub struct TypeTag {
    pub lower: u64,
    pub upper: u64,
}
