extern crate nanny_sys;

mod internal;
pub mod mem;
pub mod vm;
pub mod scope;
pub mod value;
pub mod buffer;

use internal::vm::{Module, Throw};
use internal::mem::Handle;
use internal::value::Object;

#[no_mangle]
pub extern "C" fn nanny_init(mut module: Handle<Object>, init: fn(Module) -> Result<(), Throw>) {
    Module::initialize(&mut module, init);
}
