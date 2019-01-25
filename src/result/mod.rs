//! Types and traits for working with JavaScript exceptions.

use std::fmt::{Display, Formatter, Result as FmtResult};
use types::Value;
use context::Context;

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

/// An extension trait for `Result` values that can be converted into `NeonResult` values by
/// throwsing a JavaScript exception in the error case.
pub trait NeonResultExt<'a, V: Value> {
    fn or_throw<'b, C: Context<'b>>(self, cx: &mut C) -> NeonResult<&'a V>;
}
