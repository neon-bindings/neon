//! Procedural macros supporting [Neon](https://docs.rs/neon/latest/neon/)

#[cfg(feature = "napi")]
mod napi;
#[cfg(feature = "napi")]
use napi as macros;

#[cfg(not(feature = "napi"))]
mod legacy;
#[cfg(not(feature = "napi"))]
use legacy as macros;

// Proc macro definitions must be in the root of the crate
// Implementations are in the backend dependent module

#[proc_macro_attribute]
/// Marks a function as the main entry point for initialization in
/// a Neon module.
///
/// This attribute should only be used _once_ in a module and will
/// be called each time the module is initialized in a context.
///
/// ```ignore
/// #[neon::main]
/// fn init(mut cx: ModuleContext) -> NeonResult<()> {
///     let version = cx.string("1.0.0");
///
///     cx.export_value("version", version)?;
///
///     Ok(())
/// }
/// ```
///
/// If multiple functions are marked with `#[neon::main]`, there may be a compile error:
///
/// ```sh
/// error: symbol `napi_register_module_v1` is already defined
/// ```
pub fn main(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    macros::main(attr, item)
}
