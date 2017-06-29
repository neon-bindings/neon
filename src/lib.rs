//! The `neon` crate provides the entire [Neon](https://www.neon-bindings.com/) API.

extern crate neon_runtime;
extern crate cslice;

mod internal;
pub mod mem;
pub mod vm;
pub mod scope;
pub mod js;
pub mod task;

#[doc(hidden)]
pub mod macro_internal;

/// Register the current crate as a Node module, providing startup
/// logic for initializing the module object at runtime.
///
/// Example:
///
/// ```rust,ignore
/// register_module!(m, {
///     try!(m.export("foo", foo));
///     try!(m.export("bar", bar));
///     try!(m.export("baz", baz));
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
            fn __init_neon_module(mut $module: $crate::vm::Module) -> $crate::vm::VmResult<()> $init

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
                    $crate::vm::Module::initialize(m, __init_neon_module);
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
    ( $cls:ident ; $cname:ident ; $typ:ty ; $allocator:tt ; $call_ctor:tt ; $new_ctor:tt ; $mnames:tt ; $mdefs:tt ; init($call:pat) $body:block $($rest:tt)* ) => {
        class_definition!($cls ;
                          $cname ;
                          $typ ;
                          {
                              fn _______allocator_rust_y_u_no_hygienic_items_______($call: $crate::vm::FunctionCall<$crate::js::JsUndefined>) -> $crate::vm::VmResult<$typ> {
                                  $body
                              }

                              $crate::macro_internal::AllocateKernel::new(_______allocator_rust_y_u_no_hygienic_items_______)
                          } ;
                          $call_ctor ;
                          $new_ctor ;
                          $mnames ;
                          $mdefs ;
                          $($rest)*);
    };

    ( $cls:ident ; $cname:ident ; $typ:ty ; $allocator:tt ; $call_ctor:tt ; $new_ctor:tt ; ($($mname:tt)*) ; ($($mdef:tt)*) ; method $name:ident($call:pat) $body:block $($rest:tt)* ) => {
        class_definition!($cls ;
                          $cname ;
                          $typ ;
                          $allocator ;
                          $call_ctor ;
                          $new_ctor ;
                          ($($mname)* $name) ;
                          ($($mdef)* {
                              fn _______method_rust_y_u_no_hygienic_items_______($call: $crate::vm::FunctionCall<$cls>) -> $crate::vm::JsResult<$crate::js::JsValue> {
                                  $body
                              }

                              $crate::macro_internal::MethodKernel::new(_______method_rust_y_u_no_hygienic_items_______)
                          }) ;
                          $($rest)*);
    };

    ( $cls:ident ; $cname:ident ; $typ:ty ; $allocator:tt ; $call_ctor:tt ; $new_ctor:tt ; $mnames:tt ; $mdefs:tt ; constructor($call:pat) $body:block $($rest:tt)* ) => {
        class_definition!($cls ;
                          $cname ;
                          $typ ;
                          $allocator ;
                          $call_ctor ;
                          ({
                              fn _______constructor_rust_y_u_no_hygienic_items_______($call: $crate::vm::FunctionCall<$cls>) -> $crate::vm::VmResult<Option<$crate::mem::Handle<$crate::js::JsObject>>> {
                                  $body
                              }

                              $crate::macro_internal::ConstructKernel::new(_______constructor_rust_y_u_no_hygienic_items_______)
                          }) ;
                          $mnames ;
                          $mdefs ;
                          $($rest)*);
    };

    ( $cls:ident ; $cname:ident ; $typ:ty ; $allocator:tt ; $call_ctor:tt ; $new_ctor:tt ; $mnames:tt ; $mdefs:tt ; call($call:pat) $body:block $($rest:tt)* ) => {
        class_definition!($cls ;
                          $cname ;
                          $typ ;
                          $allocator ;
                          ({
                              fn _______call_rust_y_u_no_hygienic_items_______($call: $crate::vm::FunctionCall<$crate::js::JsValue>) -> $crate::vm::JsResult<$crate::js::JsValue> {
                                  $body
                              }

                              $crate::macro_internal::ConstructorCallKernel::new(_______call_rust_y_u_no_hygienic_items_______)
                          }) ;
                          $new_ctor ;
                          $mnames ;
                          $mdefs ;
                          $($rest)*);
    };

    ( $cls:ident ; $cname:ident ; $typ:ty ; $allocator:block ; ($($call_ctor:block)*) ; ($($new_ctor:block)*) ; ($($mname:ident)*) ; ($($mdef:block)*) ; $($rest:tt)* ) => {
        impl $crate::js::class::Class for $cls {
            type Internals = $typ;

            fn setup<'a, T: $crate::scope::Scope<'a>>(_: &mut T) -> $crate::vm::VmResult<$crate::js::class::ClassDescriptor<'a, Self>> {
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
///         init(call) {
///             let scope = call.scope;
///             let greeting = try!(try!(call.arguments.require(scope, 0)).to_string(scope)).value();
///             Ok(Greeter {
///                 greeting: greeting
///             })
///         }
///
///         method hello(call) {
///             let scope = call.scope;
///             let name = try!(try!(call.arguments.require(scope, 0)).to_string(scope)).value();
///             let msg = vm::lock(call.arguments.this(scope), |greeter| {
///                 format!("{}, {}!", greeter.greeting, name)
///             });
///             Ok(try!(JsString::new_or_throw(scope, &msg[..])).upcast())
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
