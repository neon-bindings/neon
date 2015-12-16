extern crate nanny_sys;

mod internal;
pub mod mem;
pub mod vm;
pub mod scope;
pub mod value;
pub mod error;
pub mod buffer;

use internal::vm::{Module, Throw};
use internal::mem::Handle;
use internal::value::SomeObject;

#[no_mangle]
pub extern "C" fn nanny_init(module: Handle<SomeObject>, init: fn(Module) -> Result<(), Throw>) {
    Module::initialize(module, init);
}
