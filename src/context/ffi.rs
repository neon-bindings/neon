use super::internal::{ContextInternal, Env, Scope, ScopeMetadata};
use super::Context;
use neon_runtime::raw;

/// Opaque type representing the underlying env of a context
///
/// See `Context::with_raw_env` for details.
#[repr(C)]
pub struct RawEnv {
    // Create opaque type as suggested in https://doc.rust-lang.org/nomicon/ffi.html#representing-opaque-structs
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

impl RawEnv {
    /// Creates a context in the env
    pub fn with_context<T, F>(&mut self, f: F) -> T
    where
        F: for<'b> FnOnce(RawContext<'b>) -> T,
    {
        let env = unsafe { std::mem::transmute(self) };
        RawContext::with(env, f)
    }
}

pub struct RawContext<'a> {
    scope: Scope<'a, raw::HandleScope>,
}

impl<'a> ContextInternal<'a> for RawContext<'a> {
    fn scope_metadata(&self) -> &ScopeMetadata {
        &self.scope.metadata
    }
}

impl<'a> Context<'a> for RawContext<'a> {}

impl<'a> RawContext<'a> {
    fn with<T, F>(env: Env, f: F) -> T
    where
        F: for<'b> FnOnce(RawContext<'b>) -> T,
    {
        Scope::with(env, |scope| f(RawContext { scope }))
    }
}
