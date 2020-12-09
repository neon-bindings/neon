//! # FFI bindings to N-API symbols
//!
//! These types are manually copied from bindings generated from `bindgen`. To
//! update, use the following approach:
//!
//! * Run `cargo test --manifest-path=crates/neon-runtime/Cargo.toml --features napi`
//!   at least once to install `nodejs-sys`
//! * Open the generated bindings at `target/release/build/nodejs-sys-*/out/bindings.rs`
//! * Copy the types needed into `types.rs` and `functions.rs`
//! * Modify to match Rust naming conventions:
//!   - Remove `napi_` prefixes
//!   - Use `PascalCase` for types
//!   - Rename types that match a reserved word

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
/// ```
/// extern "C" {
///     fn get_undefined(env: Env, result: *mut Value) -> Status;
///     /* Additional functions may be included */  
/// }
/// ```
///
/// Generated output:
///
/// ```
/// // Each field is a pointer to a N-API function
/// pub(crate) struct Napi {
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
/// pub(crate) unsafe fn load() -> Result<(), libloading::Error> {
///     // Load the host process as a library
///     let host = Library::this();
///     #[cfg(windows)]
///     // On Windows, the host process might not be a library
///     let host = host?;
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
    (extern "C" {
        $(fn $name:ident($($param:ident: $ptype:ty$(,)?)*) -> $rtype:ty;)+
    }) => {
        pub(crate) struct Napi {
            $(
                $name: unsafe extern "C" fn(
                    $($param: $ptype,)*
                ) -> $rtype,
            )*
        }

        #[inline(never)]
        fn panic_load<T>() -> T {
            panic!("Must load N-API bindings")
        }

        static mut NAPI: Napi = {
            $(
                unsafe extern "C" fn $name($(_: $ptype,)*) -> $rtype {
                    panic_load()
                }
            )*

            Napi {
                $(
                    $name,
                )*
            }
        };

        pub(crate) unsafe fn load() -> Result<(), libloading::Error> {
            #[cfg(not(windows))]
            let host = libloading::os::unix::Library::this();
            #[cfg(windows)]
            let host = libloading::os::windows::Library::this()?;

            NAPI = Napi {
                $(
                    $name: *host.get(napi_name!($name).as_bytes())?,
                )*
            };

            Ok(())
        }

        $(
            #[inline]
            pub(crate) unsafe fn $name($($param: $ptype,)*) -> $rtype {
                (NAPI.$name)($($param,)*)
            }
        )*
    };
}

use std::sync::Once;

pub(crate) use functions::*;
pub(crate) use types::*;

mod types;
mod functions;

static SETUP: Once = Once::new();

/// Loads N-API symbols from host process.
/// Must be called at least once before using any functions in `neon-runtime` or
/// they will panic.
pub fn setup() {
    SETUP.call_once(|| unsafe {
        load().expect("Failed to load N-API symbols");
    });
}
