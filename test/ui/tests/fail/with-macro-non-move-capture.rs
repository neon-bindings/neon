//! The `with!` macro does not implicitly `move` its closure; captured
//! variables are borrowed exactly as the code reads. Mutating a captured
//! variable while the wrapped closure is still live must fail to borrow
//! check instead of silently capturing a stale copy.

#![allow(unused_assignments)]

use neon::{prelude::*, types::extract::TryIntoJs};

fn stale_capture(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let mut n = 1.0;
    let w = neon::types::extract::with!(|cx| cx.number(n));

    n = 2.0;

    w.try_into_js(&mut cx)
}

fn main() {}
