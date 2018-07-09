//! Types and traits for working with JavaScript exceptions.

use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};
use value::{JsResult, Value};
use cx::Context;

/// An error sentinel type used by `NeonResult` (and `JsResult`) to indicate that the JS engine has entered into a throwing state.
#[derive(Debug)]
pub struct Throw;

impl Display for Throw {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        fmt.write_str("JavaScript Error")
    }
}

impl Error for Throw {
    fn description(&self) -> &str {
        "javascript error"
    }
}

/// The result of a computation that might send the JS engine into a throwing state.
pub type NeonResult<T> = Result<T, Throw>;

/// An extension trait for `Result` values that can be converted into `JsResult` values by throwing a JavaScript
/// exception in the error case.
pub trait ResultExt<'a, V: Value> {
    fn unwrap_or_throw<'b, C: Context<'b>>(self, cx: &mut C) -> JsResult<'a, V>;
}
