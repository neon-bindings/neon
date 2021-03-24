//! Represents JavaScript exceptions as a Rust `Result` type.

use context::Context;
use handle::Handle;
use std::fmt::{Display, Formatter, Result as FmtResult};
use types::Value;

/// An error sentinel type used by `NeonResult` (and `JsResult`) to indicate that the JavaScript engine
/// has entered into a throwing state.
///
/// `Throw` deliberately does not implement `std::error::Error`, because it's generally not a good idea
/// to chain JavaScript exceptions with other kinds of Rust errors, since entering into the throwing
/// state means that the JavaScript engine is unavailable until the exception is handled.
#[derive(Debug)]
pub struct Throw;

impl Display for Throw {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        fmt.write_str("JavaScript Error")
    }
}

/// The result of a computation that might send the JS engine into a throwing state.
pub type NeonResult<T> = Result<T, Throw>;

/// The result of a computation that produces a JavaScript value and might send the JS engine into a throwing state.
pub type JsResult<'b, T> = NeonResult<Handle<'b, T>>;

/// An extension trait for `Result` values that can be converted into `JsResult` values by throwing a JavaScript
/// exception in the error case.
pub trait JsResultExt<'a, V: Value> {
    fn or_throw<'b, C: Context<'b>>(self, cx: &mut C) -> JsResult<'a, V>;
}
