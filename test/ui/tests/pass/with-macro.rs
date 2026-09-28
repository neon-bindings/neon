//! Ensure that all arms of the `with!` macro expand correctly: with and
//! without `move`, with a named or ignored context parameter, and with or
//! without a return type annotation.

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

    // A return type annotation names the error type, allowing a bare `Ok(..)` tail
    let move_ident_ret = with!(move |cx| -> NeonResult<_> { Ok(cx.number(n)) });
    let move_underscore_ret = with!(move |_| -> NeonResult<f64> { Ok(n) });
    let ident_ret = with!(|cx| -> NeonResult<_> { Ok(cx.number(n)) });
    let underscore_ret = with!(|_| -> NeonResult<_> { Ok(n) });

    move_ident_ret.try_into_js(&mut cx)?;
    move_underscore_ret.try_into_js(&mut cx)?;
    ident_ret.try_into_js(&mut cx)?;
    underscore_ret.try_into_js(&mut cx)?;

    // A body ending in a fallible call infers the error type from the tail
    // without an annotation
    let tail_inferred = with!(move |cx| cx.global::<JsObject>("console"));

    // A wrapped tail value can name the error type through the alias instead
    let alias_tail = with!(move |_| NeonResult::Ok(n));

    tail_inferred.try_into_js(&mut cx)?;
    alias_tail.try_into_js(&mut cx)?;

    Ok(())
}

fn main() {}
