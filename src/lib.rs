//! The `neon` crate provides the entire [Neon](https://www.neon-bindings.com/) API.

extern crate neon_runtime;
extern crate cslice;
extern crate semver;

#[cfg(test)]
extern crate rustc_version;

#[cfg(test)]
#[macro_use]
extern crate lazy_static;

pub mod mem;
pub mod vm;
pub mod js;
pub mod task;
pub mod meta;

#[doc(hidden)]
pub mod macro_internal;

// FIXME: document that the first argument is an identifier that binds a ModuleContext

/// Register the current crate as a Node module, providing startup
/// logic for initializing the module object at runtime.
///
/// Example:
///
/// ```rust,ignore
/// register_module!(cx, {
///     cx.export_function("foo", foo)?;
///     cx.export_function("bar", bar)?;
///     cx.export_function("baz", baz)?;
///     Ok(())
/// });
/// ```
#[macro_export]
macro_rules! register_module {
    ($module:ident, $init:block) => {
        // Mark this function as a global constructor (like C++).
        #[allow(improper_ctypes)]
        #[cfg_attr(target_os = "linux", link_section = ".ctors")]
        #[cfg_attr(target_os = "macos", link_section = "__DATA,__mod_init_func")]
        #[cfg_attr(target_os = "windows", link_section = ".CRT$XCU")]
        pub static __LOAD_NEON_MODULE: extern "C" fn() = {
            fn __init_neon_module(mut $module: $crate::vm::ModuleContext) -> $crate::vm::VmResult<()> $init

            extern "C" fn __load_neon_module() {
                // Put everything else in the ctor fn so the user fn can't see it.
                #[repr(C)]
                struct __NodeModule {
                    version: i32,
                    flags: u32,
                    dso_handle: *mut u8,
                    filename: *const u8,
                    register_func: Option<extern "C" fn(
                        $crate::mem::Handle<$crate::js::JsObject>, *mut u8, *mut u8)>,
                    context_register_func: Option<extern "C" fn(
                        $crate::mem::Handle<$crate::js::JsObject>, *mut u8, *mut u8, *mut u8)>,
                    modname: *const u8,
                    priv_data: *mut u8,
                    link: *mut __NodeModule
                }

                static mut __NODE_MODULE: __NodeModule = __NodeModule {
                    version: 0,
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
                        m: $crate::mem::Handle<$crate::js::JsObject>, _: *mut u8, _: *mut u8) {
                    $crate::macro_internal::initialize_module(m, __init_neon_module);
                }

                extern "C" {
                    fn node_module_register(module: *mut __NodeModule);
                }

                // Suppress the default Rust panic hook, which prints diagnostics to stderr.
                ::std::panic::set_hook(::std::boxed::Box::new(|_| { }));

                unsafe {
                    // Set the ABI version based on the NODE_MODULE_VERSION constant provided by the current node headers.
                    __NODE_MODULE.version = $crate::macro_internal::runtime::module::get_version();
                    node_module_register(&mut __NODE_MODULE);
                }
            }

            __load_neon_module
        };
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! class_definition {
    ( $cls:ident ; $cname:ident ; $typ:ty ; $allocator:tt ; $call_ctor:tt ; $new_ctor:tt ; $mnames:tt ; $mdefs:tt ; init($call:ident) $body:block $($rest:tt)* ) => {
        class_definition!($cls ;
                          $cname ;
                          $typ ;
                          {
                              fn _______allocator_rust_y_u_no_hygienic_items_______(mut $call: $crate::vm::CallContext<$crate::js::JsUndefined>) -> $crate::vm::VmResult<$typ> {
                                  $body
                              }

                              $crate::macro_internal::AllocateCallback(_______allocator_rust_y_u_no_hygienic_items_______)
                          } ;
                          $call_ctor ;
                          $new_ctor ;
                          $mnames ;
                          $mdefs ;
                          $($rest)*);
    };

    ( $cls:ident ; $cname:ident ; $typ:ty ; $allocator:tt ; $call_ctor:tt ; $new_ctor:tt ; ($($mname:tt)*) ; ($($mdef:tt)*) ; method $name:ident($call:ident) $body:block $($rest:tt)* ) => {
        class_definition!($cls ;
                          $cname ;
                          $typ ;
                          $allocator ;
                          $call_ctor ;
                          $new_ctor ;
                          ($($mname)* $name) ;
                          ($($mdef)* {
                              fn _______method_rust_y_u_no_hygienic_items_______(mut $call: $crate::vm::CallContext<$cls>) -> $crate::vm::JsResult<$crate::js::JsValue> {
                                  $body
                              }

                              $crate::macro_internal::MethodCallback(_______method_rust_y_u_no_hygienic_items_______)
                          }) ;
                          $($rest)*);
    };

    ( $cls:ident ; $cname:ident ; $typ:ty ; $allocator:tt ; $call_ctor:tt ; $new_ctor:tt ; $mnames:tt ; $mdefs:tt ; constructor($call:ident) $body:block $($rest:tt)* ) => {
        class_definition!($cls ;
                          $cname ;
                          $typ ;
                          $allocator ;
                          $call_ctor ;
                          ({
                              fn _______constructor_rust_y_u_no_hygienic_items_______(mut $call: $crate::vm::CallContext<$cls>) -> $crate::vm::VmResult<Option<$crate::mem::Handle<$crate::js::JsObject>>> {
                                  $body
                              }

                              $crate::macro_internal::ConstructCallback::new(_______constructor_rust_y_u_no_hygienic_items_______)
                          }) ;
                          $mnames ;
                          $mdefs ;
                          $($rest)*);
    };

    ( $cls:ident ; $cname:ident ; $typ:ty ; $allocator:tt ; $call_ctor:tt ; $new_ctor:tt ; $mnames:tt ; $mdefs:tt ; call($call:ident) $body:block $($rest:tt)* ) => {
        class_definition!($cls ;
                          $cname ;
                          $typ ;
                          $allocator ;
                          ({
                              fn _______call_rust_y_u_no_hygienic_items_______(mut $call: $crate::vm::CallContext<$crate::js::JsValue>) -> $crate::vm::JsResult<$crate::js::JsValue> {
                                  $body
                              }

                              $crate::macro_internal::ConstructorCallCallback::new(_______call_rust_y_u_no_hygienic_items_______)
                          }) ;
                          $new_ctor ;
                          $mnames ;
                          $mdefs ;
                          $($rest)*);
    };

    ( $cls:ident ; $cname:ident ; $typ:ty ; $allocator:block ; ($($call_ctor:block)*) ; ($($new_ctor:block)*) ; ($($mname:ident)*) ; ($($mdef:block)*) ; $($rest:tt)* ) => {
        impl $crate::js::class::Class for $cls {
            type Internals = $typ;

            fn setup<'a, C: $crate::vm::Context<'a>>(_: &mut C) -> $crate::vm::VmResult<$crate::js::class::ClassDescriptor<'a, Self>> {
                ::std::result::Result::Ok(Self::describe(stringify!($cname), $allocator)
                                             $(.construct($new_ctor))*
                                             $(.call($call_ctor))*
                                             $(.method(stringify!($mname), $mdef))*)
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_managed {
    ($cls:ident) => {
        impl $crate::mem::Managed for $cls {
            fn to_raw(self) -> $crate::macro_internal::runtime::raw::Local {
                let $cls(raw) = self;
                raw
            }

            fn from_raw(raw: $crate::macro_internal::runtime::raw::Local) -> Self {
                $cls(raw)
            }
        }
    }
}

/// Declare custom native JavaScript types with Rust implementations.
///
/// Example:
///
/// ```rust,ignore
/// pub struct Greeter {
///     greeting: String
/// }
///
/// declare_types! {
///
///     /// A class for generating greeting strings.
///     pub class JsGreeter for Greeter {
///         init(cx) {
///             let greeting = cx.argument(0)?.to_string(&mut cx)?.value();
///             Ok(Greeter {
///                 greeting: greeting
///             })
///         }
///
///         method hello(cx) {
///             let name = cx.argument(0)?.to_string(&mut cx)?.value();
///             let this = cx.this();
///             let msg = {
///                 let guard = cx.lock();
///                 let greeter = this.borrow(&guard);
///                 format!("{}, {}!", greeter.greeting, name)
///             };
///             Ok(cx.string(&msg[..])?.upcast())
///         }
///     }
///
/// }
/// ```
#[macro_export]
macro_rules! declare_types {
    { $(#[$attr:meta])* pub class $cls:ident for $typ:ident { $($body:tt)* } $($rest:tt)* } => {
        declare_types! { $(#[$attr])* pub class $cls as $typ for $typ { $($body)* } $($rest)* }
    };

    { $(#[$attr:meta])* class $cls:ident for $typ:ident { $($body:tt)* } $($rest:tt)* } => {
        declare_types! { $(#[$attr])* class $cls as $typ for $typ { $($body)* } $($rest)* }
    };

    { $(#[$attr:meta])* pub class $cls:ident as $cname:ident for $typ:ty { $($body:tt)* } $($rest:tt)* } => {
        #[derive(Copy, Clone)]
        #[repr(C)]
        $(#[$attr])*
        pub struct $cls($crate::macro_internal::runtime::raw::Local);

        impl_managed!($cls);

        class_definition!($cls ; $cname ; $typ ; () ; () ; () ; () ; () ; $($body)*);

        declare_types! { $($rest)* }
    };

    { $(#[$attr:meta])* class $cls:ident as $cname:ident for $typ:ty { $($body:tt)* } $($rest:tt)* } => {
        #[derive(Copy, Clone)]
        #[repr(C)]
        $(#[$attr])*
        struct $cls($crate::macro_internal::runtime::raw::Local);

        impl_managed!($cls);

        class_definition!($cls ; $cname ; $typ ; () ; () ; () ; () ; () ; $($body)*);

        declare_types! { $($rest)* }
    };

    { } => { };
}

#[cfg(all(windows, not(neon_profile = "release")))]
compile_error!("Neon only builds with --release. For tests, try `cargo test --release`.");

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use std::sync::Mutex;

    use rustc_version::{version_meta, Channel};

    // Create a mutex to enforce sequential running of the tests.
    lazy_static! {
        static ref TEST_MUTEX: Mutex<()> = Mutex::new(());
    }

    fn project_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
    }

    fn log(test_name: &str) {
        eprintln!("======================================================");
        eprintln!("Neon test: {}", test_name);
        eprintln!("======================================================");
    }

    fn run(cmd: &str, dir: &Path) {
        let (shell, command_flag) = if cfg!(windows) {
            ("cmd", "/C")
        } else {
            ("sh", "-c")
        };

        assert!(Command::new(&shell)
                        .current_dir(dir)
                        .args(&[&command_flag, cmd])
                        .status()
                        .expect("failed to execute test command")
                        .success());
    }

    fn cli_setup() {
        let cli = project_root().join("cli");

        run("npm install", &cli);
        run("npm run transpile", &cli);
    }

    #[test]
    fn cli_test() {
        let _guard = TEST_MUTEX.lock();

        log("cli_test");

        cli_setup();

        let test_cli = project_root().join("test").join("cli");
        run("npm install", &test_cli);
        run("npm run transpile", &test_cli);
        run("npm test", &test_cli);
    }

    #[cfg(not(windows))]
    #[test]
    fn static_test() {
        let _guard = TEST_MUTEX.lock();

        log("static_test");

        if version_meta().unwrap().channel != Channel::Nightly {
            return;
        }

        run("cargo test --release", &project_root().join("test").join("static"));
    }

    #[test]
    fn dynamic_test() {
        let _guard = TEST_MUTEX.lock();

        log("dynamic_test");

        cli_setup();

        let test_dynamic = project_root().join("test").join("dynamic");
        run("npm install", &test_dynamic);
        run("npm test", &test_dynamic);
    }
}
