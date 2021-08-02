use crate::context::Context;
use crate::handle::{Handle, Managed};
use crate::types::internal::ValueInternal;
use crate::types::{Env, JsString, Value};

use neon_runtime::raw;

/// A JavaScript symbol primitive value.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct JsSymbol(raw::Local);

impl JsSymbol {
    /// Create a new symbol.
    /// Equivalent to calling `Symbol()` in JavaScript
    pub fn new<'a, C: Context<'a>>(cx: &mut C) -> Handle<'a, JsSymbol> {
        JsSymbol::new_internal(cx.env(), None)
    }

    /// Create a new symbol with a description.
    /// Equivalent to calling `Symbol(description)` in JavaScript
    pub fn with_description<'a, C: Context<'a>>(
        cx: &mut C,
        desc: Handle<'a, JsString>,
    ) -> Handle<'a, JsSymbol> {
        JsSymbol::new_internal(cx.env(), Some(desc))
    }

    /// Get the optional symbol description, where `None` represents an undefined description.
    pub fn description<'a, C: Context<'a>>(self, cx: &mut C) -> Option<Handle<'a, JsString>> {
        let env = cx.env().to_raw();
        const DESCRIPTION_KEY: &[u8] = b"description\0";
        unsafe {
            neon_runtime::object::get_named(env, self.to_raw(), DESCRIPTION_KEY.as_ptr()).and_then(
                |local| {
                    if neon_runtime::tag::is_string(env, local) {
                        Some(Handle::new_internal(JsString(local)))
                    } else {
                        None
                    }
                },
            )
        }
    }

    pub(crate) fn new_internal<'a>(
        env: Env,
        desc: Option<Handle<'a, JsString>>,
    ) -> Handle<'a, JsSymbol> {
        unsafe {
            let desc_local = match desc {
                None => std::ptr::null_mut(),
                Some(h) => h.to_raw(),
            };
            let sym_local = neon_runtime::primitive::symbol(env.to_raw(), desc_local);
            Handle::new_internal(JsSymbol(sym_local))
        }
    }
}

impl Value for JsSymbol {}

impl Managed for JsSymbol {
    fn to_raw(self) -> raw::Local {
        self.0
    }

    fn from_raw(_: Env, h: raw::Local) -> Self {
        JsSymbol(h)
    }
}

impl ValueInternal for JsSymbol {
    fn name() -> String {
        "symbol".to_string()
    }

    fn is_typeof<Other: Value>(env: Env, other: Other) -> bool {
        unsafe { neon_runtime::tag::is_symbol(env.to_raw(), other.to_raw()) }
    }
}
