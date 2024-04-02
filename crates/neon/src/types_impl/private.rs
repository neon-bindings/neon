use crate::{
    context::internal::Env,
    handle::{internal::TransparentNoCopyWrapper, Handle},
    sys::raw,
    types::Value,
};

pub trait ValueInternal: TransparentNoCopyWrapper + 'static {
    fn name() -> String;

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
}
