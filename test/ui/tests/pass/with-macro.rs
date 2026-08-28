//! Ensure that all four arms of the `with!` macro expand correctly: with and
//! without `move`, and with a named or ignored context parameter.

use neon::{
    prelude::*,
    types::extract::{with, TryIntoJs},
};

fn all_arms(mut cx: FunctionContext) -> NeonResult<()> {
    let n = 1.0;

    let move_ident = with!(move |cx| cx.number(n));
    let move_underscore = with!(move |_| n);
    let ident = with!(|cx| cx.number(n));
    let underscore = with!(|_| n);

    move_ident.try_into_js(&mut cx)?;
    move_underscore.try_into_js(&mut cx)?;
    ident.try_into_js(&mut cx)?;
    underscore.try_into_js(&mut cx)?;

    // A body ending in a fallible call infers the error type from the tail
    // without an annotation
    let tail_inferred = with!(move |cx| cx.global::<JsObject>("console"));

    tail_inferred.try_into_js(&mut cx)?;

    Ok(())
}

fn main() {}
