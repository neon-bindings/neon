//! A helper function for comparing `v8::Local` handles.
use raw::Local;

// FIXME(napi): #[link_name = "Neon_Mem_SameHandle"]
pub extern "C" fn same_handle(h1: Local, h2: Local) -> bool { unimplemented!() }
