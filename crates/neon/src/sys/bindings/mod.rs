//! # FFI bindings to Node-API symbols
//!
//! Rust types generated from [Node-API](https://nodejs.org/api/n-api.html).

// These types are manually copied from bindings generated from `bindgen`. To
// update, use the following approach:
//
// * Run a debug build of Neon at least once to install `nodejs-sys`
// * Open the generated bindings at `target/debug/build/nodejs-sys-*/out/bindings.rs`
// * Copy the types needed into `types.rs` and `functions.rs`
// * Modify to match Rust naming conventions:
//   - Remove `napi_` prefixes
//   - Use `PascalCase` for types
//   - Rename types that match a reserved word

/// Constructs the name of a N-API symbol as a string from a function identifier
/// E.g., `get_undefined` becomes `"napi_get_undefined"`
macro_rules! napi_name {
    // Explicitly replace identifiers that have been renamed from the N-API
    // symbol because they would match a reserved word.
    (typeof_value) => {
        "napi_typeof"
    };
    // Default case: Stringify the identifier and prefix with `napi_`
    ($name:ident) => {
        concat!("napi_", stringify!($name))
    };
}

/// Generate dynamic bindings to N-API symbols from definitions in an
/// block `extern "C"`.
///
/// * A single global mutable struct holds references to the N-API functions
/// * The global `Napi` struct  is initialized with stubs that panic if called
/// * A `load` function is generated that loads the N-API symbols from the
///   host process and replaces the global struct with real implementations
/// * `load` should be called exactly once before using any N-API functions
/// * Wrapper functions are generated to delegate to fields in the `Napi` struct
///
/// Sample input:
///
/// ```ignore
/// extern "C" {
///     fn get_undefined(env: Env, result: *mut Value) -> Status;
///     /* Additional functions may be included */  
/// }
/// ```
///
/// Generated output:
///
/// ```ignore
/// // Each field is a pointer to a N-API function
/// struct Napi {
///     get_undefined: unsafe extern "C" fn(env: Env, result: *mut Value) -> Status,
///     /* ... repeat for each N-API function */
/// }
///
/// // Defines a panic function that is called if symbols have not been loaded
/// #[inline(never)]
/// fn panic_load<T>() -> T {
///     panic!("Must load N-API bindings")
/// }
///
/// // Mutable global instance of the Napi struct
/// // Initialized with stubs of N-API methods that panic
/// static mut NAPI: Napi = {
///     // Stubs are defined in a block to prevent naming conflicts with wrappers
///     unsafe extern "C" fn get_undefined(_: Env, _: *mut Value) -> Status {
///         panic_load()
///     }
///     /* ... repeat for each N-API function */
///
///     Napi {
///         get_undefined,
///         /* ... repeat for each N-API function */
///     }
/// };
///
/// // Load N-API symbols from the host process
/// // # Safety: Must only be called once
/// pub(super) unsafe fn load(
///     host: &libloading::Library,
///     actual_napi_version: u32,
///     expected_napi_version: u32,
/// ) -> Result<(), libloading::Error> {
///     assert!(
///         actual_napi_version >= expected_napi_version,
///         "Minimum required N-API version {}, found {}.",
///         expected_napi_version,
///         actual_napi_version,
///     );
///
///     NAPI = Napi {
///         // Load each N-API symbol
///         get_undefined: *host.get("napi_get_undefined".as_bytes())?,
///         /* ... repeat for each N-API function */
///     };
///
///     Ok(())
/// }
///
/// // Each N-API function has wrapper for easy calling. These calls are optimized
/// // to a single pointer dereference.
/// #[inline]
/// pub(crate) unsafe fn get_undefined(env: Env, result: *mut Value) -> Status {
///     (NAPI.get_undefined)(env, result)
/// }
/// ```
macro_rules! generate {
    (#[$extern_attr:meta] extern "C" {
        $($(#[$attr:meta])? fn $name:ident($($param:ident: $ptype:ty$(,)?)*)$( -> $rtype:ty)?;)+
    }) => {
        struct Napi {
            $(
                $name: unsafe extern "C" fn(
                    $($param: $ptype,)*
                )$( -> $rtype)?,
            )*
        }

        #[inline(never)]
        fn panic_load<T>() -> T {
            panic!("Node-API symbol has not been loaded")
        }

        static mut NAPI: Napi = {
            $(
                unsafe extern "C" fn $name($(_: $ptype,)*) $(-> $rtype)? {
                    panic_load()
                }
            )*

            Napi {
                $(
                    $name,
                )*
            }
        };

        pub(super) unsafe fn load(host: &libloading::Library) {
            let print_warn = |err| eprintln!("WARN: {}", err);

            unsafe {
                NAPI = Napi {
                    $(
                        $name: match host.get(napi_name!($name).as_bytes()) {
                            Ok(f) => *f,
                            // Node compatible runtimes may not have full coverage of Node-API
                            // (e.g., bun). Instead of failing to start, warn on start and
                            // panic when the API is called.
                            // https://github.com/Jarred-Sumner/bun/issues/158
                            Err(err) => {
                                print_warn(err);
                                NAPI.$name
                            },
                        },
                    )*
                };
            }
        }

        $(
            impl_napi_wrapper! {
                attributes:
                    #[$extern_attr]
                    $(#[$attr])?
                    #[inline]
                    #[doc = concat!(
                        "[`",
                        napi_name!($name),
                        "`](https://nodejs.org/api/n-api.html#",
                        napi_name!($name),
                        ")",
                    )],
                name: $name,
                parameters: ($($param: $ptype,)*)
                $(, return_type: $rtype)?
            }
        )*
    };
}

macro_rules! impl_napi_wrapper {
    {
        attributes: $(#[$attr:meta])*,
        name: $name:ident,
        parameters: ($($param:ident: $ptype:ty,)*)
    } => {
        $(#[$attr])*
        pub unsafe fn $name($($param: $ptype,)*) {
            unsafe { (NAPI.$name)($($param,)*); }
        }
    };

    {
        attributes: $(#[$attr:meta])*,
        name: $name:ident,
        parameters: ($($param:ident: $ptype:ty,)*),
        return_type:$rtype:ty
    } => {
        $(#[$attr])*
        pub unsafe fn $name($($param: $ptype,)*) -> ::core::result::Result<(), $rtype> {
            let r: $rtype = unsafe { (NAPI.$name)($($param,)*) };
            match r {
                <$rtype>::Ok => Ok(()),
                status => Err(status)
            }
        }
    };
}

pub use self::{functions::*, types::*};

mod functions;
mod types;
