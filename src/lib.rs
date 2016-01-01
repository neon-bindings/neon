//! Neon is a safe Rust abstraction layer for writing native Node.js modules.

extern crate neon_sys;

mod internal;
pub mod mem;
pub mod vm;
pub mod scope;
pub mod value;
pub mod error;
pub mod buffer;

/// The module version for Node.js 4.x is 46.
// TODO detect this based on what we're compiling for.
pub const NODE_MODULE_VERSION: i32 = 46;

#[macro_export]
macro_rules! neon_module {
    ($name:ident($module:ident) $init:block) => {
        // Mark this function as a global constructor (like C++).
        // TODO Support more OSes here.
        #[cfg_attr(target_os = "linux", link_section = ".ctors")]
        #[cfg_attr(target_os = "macos", link_section = "__DATA,__mod_init_func")]
        #[cfg_attr(target_os = "windows", link_section = ".CRT$XCU")]
        pub static __LOAD_NEON_MODULE: extern "C" fn() = {
            fn $name(mut $module: $crate::vm::Module) -> $crate::vm::Result<()> $init

            extern "C" fn __load_neon_module() {
                // Put everything else in the ctor fn so the user fn can't see it.
                #[repr(C)]
                struct __NodeModule {
                    version: i32,
                    flags: u32,
                    dso_handle: *mut u8,
                    filename: *const u8,
                    register_func: Option<extern "C" fn(
                        $crate::mem::Handle<$crate::value::SomeObject>, *mut u8, *mut u8)>,
                    context_register_func: Option<extern "C" fn(
                        $crate::mem::Handle<$crate::value::SomeObject>, *mut u8, *mut u8, *mut u8)>,
                    modname: *const u8,
                    priv_data: *mut u8,
                    link: *mut __NodeModule
                }

                static mut __NODE_MODULE: __NodeModule = __NodeModule {
                    version: $crate::NODE_MODULE_VERSION,
                    flags: 0,
                    dso_handle: 0 as *mut _,
                    filename: b"neon_source.rs\0" as *const u8,
                    register_func: Some(__register_neon_module),
                    context_register_func: None,
                    modname: b"neon_module\0" as *const u8,
                    priv_data: 0 as *mut _,
                    link: 0 as *mut _
                };

                extern "C" fn __register_neon_module(
                        m: $crate::mem::Handle<$crate::value::SomeObject>, _: *mut u8, _: *mut u8) {
                    $crate::vm::Module::initialize(m, $name);
                }

                extern "C" {
                    fn node_module_register(module: *mut __NodeModule);
                }

                unsafe {
                    node_module_register(&mut __NODE_MODULE);
                }
            }

            __load_neon_module
        };
    }
}
