use crate::{
    context::internal::Env,
    sys::raw,
    types::{Handle, Managed, Value},
};

pub trait ValueInternal: Managed + 'static {
    fn name() -> String;

    fn is_typeof<Other: Value>(env: Env, other: &Other) -> bool;

    fn downcast<Other: Value>(env: Env, other: &Other) -> Option<Self> {
        if Self::is_typeof(env, other) {
            Some(Self::from_raw(env, other.to_raw()))
        } else {
            None
        }
    }

    fn cast<'a, T: Value, F: FnOnce(raw::Local) -> T>(self, f: F) -> Handle<'a, T> {
        Handle::new_internal(f(self.to_raw()))
    }
}
