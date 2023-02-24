//! Types for working with [`JsBigInt`].

use std::{error, fmt, mem::MaybeUninit};

use crate::{
    context::{internal::Env, Context},
    handle::{internal::TransparentNoCopyWrapper, Handle, Managed},
    result::{NeonResult, ResultExt},
    sys::{self, raw},
    types::{private, JsBigInt, Value},
};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
/// Indicates if a `JsBigInt` is positive or negative
pub enum Sign {
    Positive,
    Negative,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
/// Indicates a lossless conversion from a [`JsBigInt`] to a Rust integer
/// could not be performed.
///
/// Failures include:
/// * Negative sign on an unsigned int
/// * Overflow of an int
/// * Underflow of a signed int
pub struct RangeError<T>(T);

impl<T> RangeError<T> {
    /// Get the lossy value read from a `BigInt`. It may be truncated,
    /// sign extended or wrapped.
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> fmt::Display for RangeError<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Loss of precision reading BigInt ({})", self.0)
    }
}

impl<T> error::Error for RangeError<T> where T: fmt::Display + fmt::Debug {}

impl<T, E> ResultExt<T> for Result<T, RangeError<E>>
where
    E: fmt::Display,
{
    fn or_throw<'a, C: Context<'a>>(self, cx: &mut C) -> NeonResult<T> {
        self.or_else(|err| cx.throw_range_error(err.to_string()))
    }
}

impl JsBigInt {
    pub const POSITIVE: Sign = Sign::Positive;
    pub const NEGATIVE: Sign = Sign::Negative;

    /// Creates a `BigInt` from an [`i64`].
    ///
    /// # Example
    ///
    /// ```
    /// # use neon::{prelude::*, types::JsBigInt};
    /// # fn example(mut cx: FunctionContext) -> JsResult<JsBigInt> {
    /// let value: Handle<JsBigInt> = JsBigInt::from_i64(&mut cx, 42);
    /// # Ok(value)
    /// # }
    /// ```
    pub fn from_i64<'cx, C>(cx: &mut C, n: i64) -> Handle<'cx, Self>
    where
        C: Context<'cx>,
    {
        let mut v = MaybeUninit::uninit();
        let v = unsafe {
            assert_eq!(
                sys::create_bigint_int64(cx.env().to_raw(), n, v.as_mut_ptr(),),
                sys::Status::Ok,
            );

            v.assume_init()
        };

        Handle::new_internal(Self(v))
    }

    /// Creates a `BigInt` from a [`u64`].
    ///
    /// # Example
    ///
    /// ```
    /// # use neon::{prelude::*, types::JsBigInt};
    /// # fn example(mut cx: FunctionContext) -> JsResult<JsBigInt> {
    /// let value: Handle<JsBigInt> = JsBigInt::from_u64(&mut cx, 42);
    /// # Ok(value)
    /// # }
    /// ```
    pub fn from_u64<'cx, C>(cx: &mut C, n: u64) -> Handle<'cx, Self>
    where
        C: Context<'cx>,
    {
        let mut v = MaybeUninit::uninit();
        let v = unsafe {
            assert_eq!(
                sys::create_bigint_uint64(cx.env().to_raw(), n, v.as_mut_ptr(),),
                sys::Status::Ok,
            );

            v.assume_init()
        };

        Handle::new_internal(Self(v))
    }

    // Internal helper for creating a _signed_ `BigInt` from a [`u128`] magnitude
    fn from_u128_sign<'cx, C>(cx: &mut C, sign: Sign, n: u128) -> Handle<'cx, Self>
    where
        C: Context<'cx>,
    {
        let n = n.to_le();
        let digits = [n as u64, (n >> 64) as u64];

        Self::from_digits_le(cx, sign, &digits)
    }

    /// Creates a `BigInt` from an [`i128`].
    ///
    /// # Example
    ///
    /// ```
    /// # use neon::{prelude::*, types::JsBigInt};
    /// # fn example(mut cx: FunctionContext) -> JsResult<JsBigInt> {
    /// let value: Handle<JsBigInt> = JsBigInt::from_i128(&mut cx, 42);
    /// # Ok(value)
    /// # }
    /// ```
    pub fn from_i128<'cx, C>(cx: &mut C, n: i128) -> Handle<'cx, Self>
    where
        C: Context<'cx>,
    {
        if n >= 0 {
            return Self::from_u128(cx, n as u128);
        }

        // Get the magnitude from a two's compliment negative
        let n = u128::MAX - (n as u128) + 1;

        Self::from_u128_sign(cx, Self::NEGATIVE, n)
    }

    /// Creates a `BigInt` from a [`u128`].
    ///
    /// # Example
    ///
    /// ```
    /// # use neon::{prelude::*, types::JsBigInt};
    /// # fn example(mut cx: FunctionContext) -> JsResult<JsBigInt> {
    /// let value: Handle<JsBigInt> = JsBigInt::from_u128(&mut cx, 42);
    /// # Ok(value)
    /// # }
    /// ```
    pub fn from_u128<'cx, C>(cx: &mut C, n: u128) -> Handle<'cx, Self>
    where
        C: Context<'cx>,
    {
        Self::from_u128_sign(cx, Self::POSITIVE, n)
    }

    /// Creates a `BigInt` from a signed magnitude. The `BigInt` is calculated as:\
    /// `Sign * (digit[0] x (2⁶⁴)⁰ + digit[0] x (2⁶⁴)¹ + digit[0] x (2⁶⁴)² ...)`
    ///
    /// # Example
    ///
    /// ```
    /// # use neon::{prelude::*, types::JsBigInt};
    /// # fn example(mut cx: FunctionContext) -> JsResult<JsBigInt> {
    /// // Creates a `BigInt` equal to `2n ** 128n`
    /// let value: Handle<JsBigInt> = JsBigInt::from_digits_le(
    ///     &mut cx,
    ///     JsBigInt::POSITIVE,
    ///     &[0, 0, 1],
    /// );
    /// # Ok(value)
    /// # }
    /// ```
    //
    // XXX: It's unclear if individual digits are expected to be little endian or native.
    // The current code assumes _native_. Neon modules are currently broken on big-endian
    // platforms. If this is fixed in the future, unit tests will determine if this
    // assumption is accurate.
    pub fn from_digits_le<'cx, C>(cx: &mut C, sign: Sign, digits: &[u64]) -> Handle<'cx, Self>
    where
        C: Context<'cx>,
    {
        let sign_bit = match sign {
            Sign::Positive => 0,
            Sign::Negative => 1,
        };

        let mut v = MaybeUninit::uninit();
        let v = unsafe {
            assert_eq!(
                sys::create_bigint_words(
                    cx.env().to_raw(),
                    sign_bit,
                    digits.len(),
                    digits.as_ptr(),
                    v.as_mut_ptr(),
                ),
                sys::Status::Ok,
            );

            v.assume_init()
        };

        Handle::new_internal(Self(v))
    }

    /// Reads an `i64` from a `BigInt`.
    ///
    /// Fails on overflow and underflow.
    ///
    /// # Example
    ///
    /// See [`JsBigInt`].
    pub fn to_i64<'cx, C>(&self, cx: &mut C) -> Result<i64, RangeError<i64>>
    where
        C: Context<'cx>,
    {
        let mut n = 0;
        let mut lossless = false;

        unsafe {
            assert_eq!(
                sys::get_value_bigint_int64(cx.env().to_raw(), self.0, &mut n, &mut lossless),
                sys::Status::Ok,
            );
        }

        if lossless {
            Ok(n)
        } else {
            Err(RangeError(n))
        }
    }

    /// Reads a `u64` from a `BigInt`.
    ///
    /// Fails on overflow or a negative sign.
    pub fn to_u64<'cx, C>(&self, cx: &mut C) -> Result<u64, RangeError<u64>>
    where
        C: Context<'cx>,
    {
        let mut n = 0;
        let mut lossless = false;

        unsafe {
            assert_eq!(
                sys::get_value_bigint_uint64(cx.env().to_raw(), self.0, &mut n, &mut lossless),
                sys::Status::Ok,
            );
        }

        if lossless {
            Ok(n)
        } else {
            Err(RangeError(n))
        }
    }

    /// Reads an `i128` from a `BigInt`.
    ///
    /// Fails on overflow and underflow.
    pub fn to_i128<'cx, C>(&self, cx: &mut C) -> Result<i128, RangeError<i128>>
    where
        C: Context<'cx>,
    {
        let mut digits = [0; 2];
        let (sign, num_digits) = self.read_digits_le(cx, &mut digits);

        // Cast digits into a `u128` magnitude
        let n = (digits[0] as u128) | ((digits[1] as u128) << 64);
        let n = u128::from_le(n);

        // Verify that the magnitude leaves room for the sign bit
        let n = match sign {
            Sign::Positive => {
                if n > (i128::MAX as u128) {
                    return Err(RangeError(i128::MAX));
                } else {
                    n as i128
                }
            }
            Sign::Negative => {
                if n > (i128::MAX as u128) + 1 {
                    return Err(RangeError(i128::MIN));
                } else {
                    (n as i128).wrapping_neg()
                }
            }
        };

        // Leading zeroes are truncated and never returned. If there are additional
        // digits, the number is out of range.
        if num_digits > digits.len() {
            Err(RangeError(n))
        } else {
            Ok(n)
        }
    }

    /// Reads a `u128` from a `BigInt`.
    ///
    /// Fails on overflow or a negative sign.
    pub fn to_u128<'cx, C>(&self, cx: &mut C) -> Result<u128, RangeError<u128>>
    where
        C: Context<'cx>,
    {
        let mut digits = [0; 2];
        let (sign, num_digits) = self.read_digits_le(cx, &mut digits);

        // Cast digits into a `u128` magnitude
        let n = (digits[0] as u128) | ((digits[1] as u128) << 64);
        let n = u128::from_le(n);

        // Leading zeroes are truncated and never returned. If there are additional
        // digits, the number is out of range.
        if matches!(sign, Sign::Negative) || num_digits > digits.len() {
            Err(RangeError(n))
        } else {
            Ok(n)
        }
    }

    /// Gets a signed magnitude pair from a `BigInt`.
    ///
    /// The `BigInt` is calculated as:\
    /// `Sign * (digit[0] x (2⁶⁴)⁰ + digit[0] x (2⁶⁴)¹ + digit[0] x (2⁶⁴)² ...)`
    pub fn to_digits_le<'cx, C>(&self, cx: &mut C) -> (Sign, Vec<u64>)
    where
        C: Context<'cx>,
    {
        let mut v = vec![0; self.len(cx)];
        let (sign, len) = self.read_digits_le(cx, &mut v);

        // It shouldn't be possible for the number of digits to change. If it
        // it does, it's a correctness issue and not a soundness bug.
        debug_assert_eq!(v.len(), len);

        (sign, v)
    }

    /// Gets the sign from a `BigInt` and reads digits into a buffer.
    /// The returned `usize` is the total number of digits in the `BigInt`.
    ///
    /// # Example
    ///
    /// Read a `u256` from a `BigInt`.
    ///
    /// ```
    /// # use std::error::Error;
    /// # use neon::{prelude::*, types::JsBigInt};
    /// fn bigint_to_u256(cx: &mut FunctionContext, n: Handle<JsBigInt>) -> NeonResult<[u64; 4]> {
    ///     let mut digits = [0; 4];
    ///     let (sign, num_digits) = n.read_digits_le(cx, &mut digits);
    ///
    ///     if sign == JsBigInt::NEGATIVE {
    ///         return cx.throw_error("Underflow reading u256 from BigInt");
    ///     }
    ///
    ///     if num_digits > digits.len() {
    ///         return cx.throw_error("Overflow reading u256 from BigInt");
    ///     }
    ///
    ///     Ok(digits)
    /// }
    /// ```
    pub fn read_digits_le<'cx, C>(&self, cx: &mut C, digits: &mut [u64]) -> (Sign, usize)
    where
        C: Context<'cx>,
    {
        let mut sign_bit = 0;
        let mut word_count = digits.len();

        unsafe {
            assert_eq!(
                sys::get_value_bigint_words(
                    cx.env().to_raw(),
                    self.0,
                    &mut sign_bit,
                    &mut word_count,
                    digits.as_mut_ptr(),
                ),
                sys::Status::Ok,
            );
        }

        let sign = if sign_bit == 0 {
            Sign::Positive
        } else {
            Sign::Negative
        };

        (sign, word_count)
    }

    /// Gets the number of `u64` digits in a `BigInt`
    pub fn len<'cx, C>(&self, cx: &mut C) -> usize
    where
        C: Context<'cx>,
    {
        // Get the length by reading into an empty slice and ignoring the sign
        self.read_digits_le(cx, &mut []).1
    }
}

impl Value for JsBigInt {}

unsafe impl TransparentNoCopyWrapper for JsBigInt {
    type Inner = raw::Local;

    fn into_inner(self) -> Self::Inner {
        self.0
    }
}

impl Managed for JsBigInt {
    fn to_raw(&self) -> raw::Local {
        self.0
    }

    fn from_raw(_: Env, h: raw::Local) -> Self {
        Self(h)
    }
}

impl private::ValueInternal for JsBigInt {
    fn name() -> String {
        "BigInt".to_string()
    }

    fn is_typeof<Other: Value>(env: Env, other: &Other) -> bool {
        unsafe { sys::tag::is_bigint(env.to_raw(), other.to_raw()) }
    }
}
