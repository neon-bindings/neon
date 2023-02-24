/// Tests for [`JsBigInt`]. All unit tests are prefixed with `test_` and exported by
/// [`bigint_suite`].
use std::{any, cmp::PartialEq, fmt, panic, str::FromStr};

use neon::{
    prelude::*,
    types::{
        bigint::{RangeError, Sign},
        JsBigInt,
    },
};

use num_bigint_dig::BigInt;

// Helper that converts panics to exceptions to allow `.unwrap()` usage in unit tests
fn panic_catch<'cx, F, C>(cx: &mut C, f: F) -> JsResult<'cx, JsFunction>
where
    F: Fn(&mut FunctionContext) -> NeonResult<()> + 'static,
    C: Context<'cx>,
{
    JsFunction::new(cx, move |mut cx| {
        panic::catch_unwind(panic::AssertUnwindSafe(|| f(&mut cx))).or_else(|panic| {
            if let Some(s) = panic.downcast_ref::<&str>() {
                cx.throw_error(s)
            } else if let Some(s) = panic.downcast_ref::<String>() {
                cx.throw_error(s)
            } else {
                panic::resume_unwind(panic)
            }
        })??;

        Ok(cx.undefined())
    })
}

// Export a test that is expected not to throw
fn export<F>(cx: &mut FunctionContext, o: &JsObject, f: F) -> NeonResult<()>
where
    F: Fn(&mut FunctionContext) -> NeonResult<()> + 'static,
{
    let f = panic_catch(cx, f)?;

    o.set(cx, any::type_name::<F>(), f)?;

    Ok(())
}

// Export a test that is expected to return a `bigint::Error`
fn export_lossy<T, F>(cx: &mut FunctionContext, o: &JsObject, f: F) -> NeonResult<()>
where
    F: Fn(&mut FunctionContext) -> NeonResult<Result<T, RangeError<T>>> + 'static,
{
    let f = panic_catch(cx, move |cx| {
        if f(cx)?.is_err() {
            return Ok(());
        }

        cx.throw_error("Expected a lossy error")
    })?;

    o.set(cx, any::type_name::<F>(), f)?;

    Ok(())
}

// Small helper for `eval` of a script from a Rust string. This is used
// for creating `BigInt` inline from literals (e.g., `0n`).
fn eval<'cx, C>(cx: &mut C, script: &str) -> JsResult<'cx, JsValue>
where
    C: Context<'cx>,
{
    let script = cx.string(script);

    neon::reflect::eval(cx, script)
}

// Throws an exception if `l !== r` where operands are JavaScript values
fn strict_eq<'cx, L, R, C>(l: Handle<'cx, L>, r: Handle<'cx, R>, cx: &mut C) -> NeonResult<()>
where
    L: Value,
    R: Value,
    C: Context<'cx>,
{
    if l.strict_equals(cx, r) {
        return Ok(());
    }

    let l = l.to_string(cx)?.value(cx);
    let r = r.to_string(cx)?.value(cx);

    cx.throw_error(format!("Expected {l} to equal {r}"))
}

// Throws an exception if `l != r` where operands are Rust values
fn assert_eq<'cx, L, R, C>(l: L, r: R, cx: &mut C) -> NeonResult<()>
where
    L: fmt::Debug + PartialEq<R>,
    R: fmt::Debug,
    C: Context<'cx>,
{
    if l == r {
        return Ok(());
    }

    cx.throw_error(format!("Expected {l:?} to equal {r:?}"))
}

// Create a `JsBigInt` from a `BigInt`
fn bigint<'cx, C>(cx: &mut C, n: &str) -> JsResult<'cx, JsBigInt>
where
    C: Context<'cx>,
{
    let n = BigInt::from_str(n).or_else(|err| cx.throw_error(err.to_string()))?;
    let (sign, n) = n.to_bytes_le();
    let n = n
        .chunks(8)
        .map(|c| {
            let mut x = [0; 8];

            (x[..c.len()]).copy_from_slice(c);

            u64::from_le_bytes(x)
        })
        .collect::<Vec<_>>();

    let sign = if matches!(sign, num_bigint_dig::Sign::Minus) {
        Sign::Negative
    } else {
        Sign::Positive
    };

    Ok(JsBigInt::from_digits_le(cx, sign, &n))
}

// Convert a `JsBigInt` to a `BigInt`
fn to_bigint<'cx, V, C>(b: Handle<V>, cx: &mut C) -> NeonResult<BigInt>
where
    V: Value,
    C: Context<'cx>,
{
    let (sign, digits) = b.downcast_or_throw::<JsBigInt, _>(cx)?.to_digits_le(cx);
    let sign = match sign {
        Sign::Positive => num_bigint_dig::Sign::Plus,
        Sign::Negative => num_bigint_dig::Sign::Minus,
    };

    Ok(BigInt::from_slice_native(sign, &digits))
}

fn test_from_u64(cx: &mut FunctionContext) -> NeonResult<()> {
    strict_eq(JsBigInt::from_u64(cx, 0), eval(cx, "0n")?, cx)?;
    strict_eq(JsBigInt::from_u64(cx, 42), eval(cx, "42n")?, cx)?;
    strict_eq(
        JsBigInt::from_u64(cx, u64::MAX),
        eval(cx, &(u64::MAX.to_string() + "n"))?,
        cx,
    )?;

    Ok(())
}

fn test_from_i64(cx: &mut FunctionContext) -> NeonResult<()> {
    strict_eq(JsBigInt::from_i64(cx, 0), eval(cx, "0n")?, cx)?;
    strict_eq(JsBigInt::from_i64(cx, 42), eval(cx, "42n")?, cx)?;
    strict_eq(JsBigInt::from_i64(cx, -42), eval(cx, "-42n")?, cx)?;

    strict_eq(
        JsBigInt::from_i64(cx, i64::MAX),
        eval(cx, &(i64::MAX.to_string() + "n"))?,
        cx,
    )?;

    strict_eq(
        JsBigInt::from_i64(cx, i64::MIN),
        eval(cx, &(i64::MIN.to_string() + "n"))?,
        cx,
    )?;

    Ok(())
}

fn test_from_u128(cx: &mut FunctionContext) -> NeonResult<()> {
    strict_eq(JsBigInt::from_u128(cx, 0), eval(cx, "0n")?, cx)?;
    strict_eq(JsBigInt::from_u128(cx, 42), eval(cx, "42n")?, cx)?;

    strict_eq(
        JsBigInt::from_u128(cx, u128::MAX),
        eval(cx, "2n ** 128n - 1n")?,
        cx,
    )?;

    strict_eq(
        JsBigInt::from_u128(cx, u128::MAX - 1),
        eval(cx, "2n ** 128n - 2n")?,
        cx,
    )?;

    Ok(())
}

fn test_from_i128(cx: &mut FunctionContext) -> NeonResult<()> {
    strict_eq(JsBigInt::from_i128(cx, 0), eval(cx, "0n")?, cx)?;
    strict_eq(JsBigInt::from_i128(cx, 42), eval(cx, "42n")?, cx)?;
    strict_eq(JsBigInt::from_i128(cx, -42), eval(cx, "-42n")?, cx)?;

    strict_eq(
        JsBigInt::from_i128(cx, i128::MAX),
        eval(cx, "2n ** 127n - 1n")?,
        cx,
    )?;

    strict_eq(
        JsBigInt::from_i128(cx, i128::MAX - 1),
        eval(cx, "2n ** 127n - 2n")?,
        cx,
    )?;

    strict_eq(
        JsBigInt::from_i128(cx, i128::MIN),
        eval(cx, "-(2n ** 127n)")?,
        cx,
    )?;

    strict_eq(
        JsBigInt::from_i128(cx, i128::MIN + 1),
        eval(cx, "-(2n ** 127n - 1n)")?,
        cx,
    )?;

    Ok(())
}

fn test_from_digits_le(cx: &mut FunctionContext) -> NeonResult<()> {
    strict_eq(bigint(cx, "0")?, eval(cx, "0n")?, cx)?;
    strict_eq(bigint(cx, "42")?, eval(cx, "42n")?, cx)?;
    strict_eq(bigint(cx, "-42")?, eval(cx, "-42n")?, cx)?;

    strict_eq(
        bigint(cx, "170141183460469231731687303715884105727")?,
        eval(cx, "170141183460469231731687303715884105727n")?,
        cx,
    )?;

    strict_eq(
        bigint(cx, "-170141183460469231731687303715884105728")?,
        eval(cx, "-170141183460469231731687303715884105728n")?,
        cx,
    )?;

    strict_eq(
        bigint(cx, "10000000000000000000000000000000000000000")?,
        eval(cx, "10000000000000000000000000000000000000000n")?,
        cx,
    )?;

    strict_eq(
        bigint(cx, "-10000000000000000000000000000000000000000")?,
        eval(cx, "-10000000000000000000000000000000000000000n")?,
        cx,
    )?;

    Ok(())
}

fn test_to_u64(cx: &mut FunctionContext) -> NeonResult<()> {
    assert_eq(JsBigInt::from_u64(cx, 0).to_u64(cx).or_throw(cx)?, 0, cx)?;
    assert_eq(JsBigInt::from_u64(cx, 42).to_u64(cx).or_throw(cx)?, 42, cx)?;

    assert_eq(
        JsBigInt::from_u64(cx, u64::MAX).to_u64(cx).or_throw(cx)?,
        u64::MAX,
        cx,
    )?;

    Ok(())
}

fn test_to_i64(cx: &mut FunctionContext) -> NeonResult<()> {
    assert_eq(JsBigInt::from_i64(cx, 0).to_i64(cx).or_throw(cx)?, 0, cx)?;
    assert_eq(JsBigInt::from_i64(cx, 42).to_i64(cx).or_throw(cx)?, 42, cx)?;
    assert_eq(
        JsBigInt::from_i64(cx, -42).to_i64(cx).or_throw(cx)?,
        -42,
        cx,
    )?;

    assert_eq(
        JsBigInt::from_i64(cx, i64::MAX).to_i64(cx).or_throw(cx)?,
        i64::MAX,
        cx,
    )?;

    assert_eq(
        JsBigInt::from_i64(cx, i64::MIN).to_i64(cx).or_throw(cx)?,
        i64::MIN,
        cx,
    )?;

    Ok(())
}

fn test_to_u128(cx: &mut FunctionContext) -> NeonResult<()> {
    assert_eq(JsBigInt::from_u128(cx, 0).to_u128(cx).or_throw(cx)?, 0, cx)?;
    assert_eq(
        JsBigInt::from_u128(cx, 42).to_u128(cx).or_throw(cx)?,
        42,
        cx,
    )?;

    assert_eq(
        JsBigInt::from_u128(cx, u128::MAX)
            .to_u128(cx)
            .or_throw(cx)?,
        u128::MAX,
        cx,
    )?;

    // Extra trailing zeroes
    assert_eq(
        JsBigInt::from_digits_le(cx, JsBigInt::POSITIVE, &[u64::MAX, u64::MAX, 0, 0, 0, 0])
            .to_u128(cx)
            .or_throw(cx)?,
        u128::MAX,
        cx,
    )?;

    Ok(())
}

fn test_to_i128(cx: &mut FunctionContext) -> NeonResult<()> {
    assert_eq(JsBigInt::from_i128(cx, 0).to_i128(cx).or_throw(cx)?, 0, cx)?;
    assert_eq(
        JsBigInt::from_i128(cx, 42).to_i128(cx).or_throw(cx)?,
        42,
        cx,
    )?;
    assert_eq(
        JsBigInt::from_i128(cx, -42).to_i128(cx).or_throw(cx)?,
        -42,
        cx,
    )?;

    assert_eq(
        JsBigInt::from_i128(cx, i128::MAX)
            .to_i128(cx)
            .or_throw(cx)?,
        i128::MAX,
        cx,
    )?;

    assert_eq(
        JsBigInt::from_i128(cx, i128::MIN)
            .to_i128(cx)
            .or_throw(cx)?,
        i128::MIN,
        cx,
    )?;

    Ok(())
}

fn test_to_digits_le(cx: &mut FunctionContext) -> NeonResult<()> {
    assert_eq(
        to_bigint(eval(cx, "0n")?, cx)?,
        BigInt::from_str("0").unwrap(),
        cx,
    )?;

    assert_eq(
        to_bigint(eval(cx, "42n")?, cx)?,
        BigInt::from_str("42").unwrap(),
        cx,
    )?;

    assert_eq(
        to_bigint(eval(cx, "-42n")?, cx)?,
        BigInt::from_str("-42").unwrap(),
        cx,
    )?;

    assert_eq(
        to_bigint(eval(cx, "170141183460469231731687303715884105727n")?, cx)?,
        BigInt::from_str("170141183460469231731687303715884105727").unwrap(),
        cx,
    )?;

    assert_eq(
        to_bigint(eval(cx, "-170141183460469231731687303715884105728n")?, cx)?,
        BigInt::from_str("-170141183460469231731687303715884105728").unwrap(),
        cx,
    )?;

    assert_eq(
        to_bigint(eval(cx, "10000000000000000000000000000000000000000n")?, cx)?,
        BigInt::from_str("10000000000000000000000000000000000000000").unwrap(),
        cx,
    )?;

    assert_eq(
        to_bigint(eval(cx, "-10000000000000000000000000000000000000000n")?, cx)?,
        BigInt::from_str("-10000000000000000000000000000000000000000").unwrap(),
        cx,
    )?;

    Ok(())
}

fn test_very_large_number(cx: &mut FunctionContext) -> NeonResult<()> {
    // 2048-bit prime generated with `crypto.generatePrimeSync(2048)`
    // Note: Unlike the rest of the tests, this number is big-endian
    let n = BigInt::from_bytes_be(
        num_bigint_dig::Sign::Plus,
        &[
            228, 178, 58, 23, 125, 164, 107, 153, 254, 98, 85, 252, 29, 61, 8, 237, 212, 36, 173,
            205, 116, 52, 16, 155, 131, 82, 59, 211, 132, 139, 212, 101, 10, 26, 60, 44, 172, 86,
            50, 42, 9, 124, 188, 236, 77, 46, 209, 64, 239, 34, 99, 8, 235, 165, 5, 41, 159, 211,
            186, 197, 140, 111, 43, 15, 111, 132, 255, 148, 36, 12, 25, 221, 208, 162, 234, 45, 22,
            13, 251, 157, 103, 50, 181, 2, 53, 81, 15, 137, 129, 10, 130, 212, 74, 125, 80, 188,
            19, 218, 236, 189, 234, 145, 234, 232, 9, 218, 167, 111, 33, 62, 81, 96, 83, 125, 242,
            217, 179, 211, 109, 16, 210, 250, 133, 130, 86, 182, 110, 213, 74, 78, 34, 210, 88, 3,
            178, 73, 231, 53, 188, 187, 76, 247, 205, 154, 190, 200, 211, 75, 63, 34, 246, 160,
            193, 98, 7, 85, 40, 208, 47, 157, 34, 120, 235, 136, 101, 88, 174, 149, 180, 114, 197,
            230, 116, 47, 152, 253, 212, 191, 90, 151, 204, 6, 51, 179, 73, 128, 141, 192, 107, 74,
            205, 130, 56, 115, 202, 96, 79, 187, 196, 49, 118, 18, 251, 34, 64, 208, 38, 25, 35,
            195, 231, 195, 201, 224, 110, 205, 213, 92, 192, 23, 48, 165, 126, 145, 18, 30, 230,
            83, 229, 187, 138, 177, 74, 15, 209, 151, 83, 160, 246, 77, 59, 228, 57, 112, 165, 4,
            10, 11, 95, 213, 115, 187, 240, 57, 5, 117,
        ],
    );

    assert_eq(to_bigint(eval(cx, &(n.to_string() + "n"))?, cx)?, n, cx)?;

    Ok(())
}

fn test_i64_out_of_range(cx: &mut FunctionContext) -> NeonResult<Result<i64, RangeError<i64>>> {
    Ok(JsBigInt::from_i128(cx, (i64::MIN as i128) - 1).to_i64(cx))
}

fn test_u64_out_of_range(cx: &mut FunctionContext) -> NeonResult<Result<u64, RangeError<u64>>> {
    Ok(JsBigInt::from_u128(cx, (u64::MAX as u128) + 1).to_u64(cx))
}

fn test_i128_extra_digits(cx: &mut FunctionContext) -> NeonResult<Result<i128, RangeError<i128>>> {
    let res = eval(cx, "2n ** 128n")?
        .downcast_or_throw::<JsBigInt, _>(cx)?
        .to_i128(cx);

    Ok(res)
}

fn test_i128_overflow(cx: &mut FunctionContext) -> NeonResult<Result<i128, RangeError<i128>>> {
    let res = eval(cx, "2n ** 127n")?
        .downcast_or_throw::<JsBigInt, _>(cx)?
        .to_i128(cx);

    Ok(res)
}

fn test_i128_underflow(cx: &mut FunctionContext) -> NeonResult<Result<i128, RangeError<i128>>> {
    let res = eval(cx, "-(2n ** 127n + 1n)")?
        .downcast_or_throw::<JsBigInt, _>(cx)?
        .to_i128(cx);

    Ok(res)
}

fn test_u128_overflow(cx: &mut FunctionContext) -> NeonResult<Result<i128, RangeError<i128>>> {
    let res = eval(cx, "2n ** 127n")?
        .downcast_or_throw::<JsBigInt, _>(cx)?
        .to_i128(cx);

    Ok(res)
}

fn test_u128_underflow(cx: &mut FunctionContext) -> NeonResult<Result<u128, RangeError<u128>>> {
    let res = eval(cx, "-1n")?
        .downcast_or_throw::<JsBigInt, _>(cx)?
        .to_u128(cx);

    Ok(res)
}

// Creates a map (object) of test name to functions to be executed by a JavaScript
// test runner.
pub fn bigint_suite(mut cx: FunctionContext) -> JsResult<JsObject> {
    let o = cx.empty_object();

    // `Ok` tests
    export(&mut cx, &o, test_from_u64)?;
    export(&mut cx, &o, test_from_i64)?;
    export(&mut cx, &o, test_from_u128)?;
    export(&mut cx, &o, test_from_i128)?;
    export(&mut cx, &o, test_from_digits_le)?;
    export(&mut cx, &o, test_to_u64)?;
    export(&mut cx, &o, test_to_i64)?;
    export(&mut cx, &o, test_to_u128)?;
    export(&mut cx, &o, test_to_i128)?;
    export(&mut cx, &o, test_to_digits_le)?;
    export(&mut cx, &o, test_very_large_number)?;

    // `Err` tests
    export_lossy(&mut cx, &o, test_i64_out_of_range)?;
    export_lossy(&mut cx, &o, test_u64_out_of_range)?;
    export_lossy(&mut cx, &o, test_i128_extra_digits)?;
    export_lossy(&mut cx, &o, test_i128_overflow)?;
    export_lossy(&mut cx, &o, test_i128_underflow)?;
    export_lossy(&mut cx, &o, test_u128_overflow)?;
    export_lossy(&mut cx, &o, test_u128_underflow)?;

    Ok(o)
}
