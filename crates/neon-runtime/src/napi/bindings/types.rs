use std::ffi::c_void;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Env__ {
    _unused: [u8; 0],
}

pub type Env = *mut Env__;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Value__ {
    _unused: [u8; 0],
}

pub type Value = *mut Value__;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct CallbackInfo__ {
    _unused: [u8; 0],
}

pub type CallbackInfo = *mut CallbackInfo__;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct EscapableHandleScope__ {
    _unused: [u8; 0],
}
pub type EscapableHandleScope = *mut EscapableHandleScope__;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct HandleScope__ {
    _unused: [u8; 0],
}

pub type HandleScope = *mut HandleScope__;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Ref__ {
    _unused: [u8; 0],
}

pub type Ref = *mut Ref__;

#[cfg(feature = "napi-4")]
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ThreadsafeFunction__ {
    _unused: [u8; 0],
}

#[cfg(feature = "napi-4")]
pub type ThreadsafeFunction = *mut ThreadsafeFunction__;

pub(crate) type Callback = Option<unsafe extern "C" fn(env: Env, info: CallbackInfo) -> Value>;

pub(crate) type Finalize =
    Option<unsafe extern "C" fn(env: Env, finalize_data: *mut c_void, finalize_hint: *mut c_void)>;

#[cfg(feature = "napi-4")]
pub type ThreadsafeFunctionCallJs = Option<
    unsafe extern "C" fn(env: Env, js_callback: Value, context: *mut c_void, data: *mut c_void),
>;

#[allow(dead_code)]
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
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
pub(crate) enum ValueType {
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
pub enum KeyCollectionMode {
    IncludePrototypes = 0,
    OwnOnly = 1,
}

#[allow(dead_code)]
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum KeyConversion {
    KeepNumbers = 0,
    NumbersToStrings = 1,
}

#[cfg(feature = "napi-4")]
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ThreadsafeFunctionCallMode {
    NonBlocking = 0,
    Blocking = 1,
}

#[cfg(feature = "napi-4")]
#[allow(dead_code)]
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ThreadsafeFunctionReleaseMode {
    Release = 0,
    Abort = 1,
}

#[repr(transparent)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub(crate) struct KeyFilter(pub ::std::os::raw::c_uint);

#[allow(dead_code)]
impl KeyFilter {
    pub(crate) const ALL_PROPERTIES: KeyFilter = KeyFilter(0);
    pub(crate) const WRITABLE: KeyFilter = KeyFilter(1);
    pub(crate) const CONFIGURABLE: KeyFilter = KeyFilter(4);
    pub(crate) const SKIP_STRINGS: KeyFilter = KeyFilter(8);
    pub(crate) const SKIP_SYMBOLS: KeyFilter = KeyFilter(16);
}

impl std::ops::BitOr<KeyFilter> for KeyFilter {
    type Output = Self;
    #[inline]
    fn bitor(self, other: Self) -> Self {
        KeyFilter(self.0 | other.0)
    }
}

impl std::ops::BitOrAssign for KeyFilter {
    #[inline]
    fn bitor_assign(&mut self, rhs: KeyFilter) {
        self.0 |= rhs.0;
    }
}

impl std::ops::BitAnd<KeyFilter> for KeyFilter {
    type Output = Self;
    #[inline]
    fn bitand(self, other: Self) -> Self {
        KeyFilter(self.0 & other.0)
    }
}

impl std::ops::BitAndAssign for KeyFilter {
    #[inline]
    fn bitand_assign(&mut self, rhs: KeyFilter) {
        self.0 &= rhs.0;
    }
}
