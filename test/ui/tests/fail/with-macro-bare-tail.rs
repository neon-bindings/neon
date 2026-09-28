//! The `with!` macro evaluates its body in its own function scope, so `?`
//! cannot escape into the `JsResult`-returning closure that the macro
//! generates. A body that uses `?` must itself evaluate to a `Result`,
//! e.g. by writing the tail expression as `NeonResult::Ok(sum)`.

use neon::{prelude::*, types::extract::TryIntoJs};

fn bare_tail(nums: Vec<f64>) -> impl for<'cx> TryIntoJs<'cx> {
    neon::types::extract::with!(move |cx| {
        let sum = nums.into_iter().sum::<f64>();

        cx.global::<JsFunction>("parseFloat")?;

        sum
    })
}

fn main() {}
