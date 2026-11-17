// SPDX-License-Identifier: Apache-2.0
//
// Written in 2014 by
//     Andrew Poelstra <apoelstra@wpsoftware.net>
// Refactored, fixed & improved in 2021-2026 by
//     Dr. Maxim Orlovsky <orlovsky@ubideco.org>
//
// Copyright 2024-2026 Laboratories for Ubiquitous and Deterministic Computing,
// Institute for Distributed and Cognitive Computing (InDCS), Switzerland.
// All rights reserved.
//
// Copyright (C) 2021-2026 Dr Maxim Orlovsky.
// All rights reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not
// use this file except in compliance with the License. You may obtain a copy of
// the License at <http://www.apache.org/licenses/LICENSE-2.0>
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
// WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the
// License for the specific language governing permissions and limitations under
// the License.

use core::array;
use core::cmp::Ordering;
use core::fmt::{self, Binary, Debug, Display, Formatter, LowerHex, Octal, UpperHex};
use core::num::IntErrorKind;
use core::ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Div, DivAssign,
    Mul, MulAssign, Neg, Not, Rem, RemAssign, Shl, ShlAssign, Shr, ShrAssign, Sub, SubAssign,
};
use core::str::{self, FromStr};

use crate::error::{DivError, ParseIntError, ParseLengthError};
use crate::limbs;
use crate::traits::{
    AddChecked, AddOverflowing, AddSaturate, AddWrapping, DivChecked, DivOverflowing, DivSaturate,
    DivWrapping, Integer, MulChecked, MulOverflowing, MulSaturate, MulWrapping, NegChecked,
    NegOverflowing, NegSaturate, NegWrapping, Number, RemChecked, RemOverflowing, RemWrapping,
    ShlChecked, ShlOverflowing, ShlWrapping, ShrChecked, ShrOverflowing, ShrWrapping, SubChecked,
    SubOverflowing, SubSaturate, SubWrapping, Unsigned,
};

/// Integer of a fixed size, made of `N` 64-bit limbs kept in the little-endian
/// order.
///
/// `SIGNED` integers use the two's complement representation. The type mirrors
/// the API of the primitive integers, with one deliberate difference: the
/// arithmetic operators panic on overflow in all build profiles, and never wrap
/// around silently. The arithmetic traits in [`crate::traits`] provide
/// `checked_*`, `overflowing_*`, `saturating_*` and `wrapping_*` methods for
/// explicit control over the overflow handling.
///
/// The type does not allocate and can be used on bare-metal targets.
///
/// ```
/// use amplify_num::{AddChecked, SubSaturate, i256, u256};
///
/// let x = u256::from(u128::MAX) * 3u8 + 2u8;
/// assert_eq!(x.to_string(), "1020847100762815390390123822295304634367");
/// assert_eq!("1020847100762815390390123822295304634367".parse(), Ok(x));
/// assert_eq!(format!("{x:#x}"), "0x2ffffffffffffffffffffffffffffffff");
/// assert_eq!(x.div_rem(u128::MAX), Ok((u256::from(3u8), u256::from(2u8))));
/// assert_eq!(u256::MAX.checked_add(1u8), None);
/// assert_eq!(i256::MIN.saturating_sub(1), i256::MIN);
/// assert_eq!(i256::from(-7).rem_euclid(4), i256::from(1));
/// ```
///
/// Operations which can't be performed without a loss are rejected by the
/// compiler:
///
/// ```compile_fail
/// let _ = -amplify_num::u256::ONE; // unsigned integers can't be negated
/// ```
///
/// ```compile_fail
/// let _ = amplify_num::BigInt::<1, true>::from(u64::MAX); // 64 bits do not fit 63 bits
/// ```
///
/// A lossy conversion is detected when the code using it is generated, thus
/// `cargo build` reports it and `cargo check` does not. This matters only for
/// the custom sizes: the aliases, starting from [`u256`], fit every primitive.
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct BigInt<const N: usize, const SIGNED: bool>([u64; N]);

/// 256-bit unsigned integer.
#[expect(non_camel_case_types)]
pub type u256 = BigInt<4, false>;

/// 512-bit unsigned integer.
#[expect(non_camel_case_types)]
pub type u512 = BigInt<8, false>;

/// 1024-bit unsigned integer.
#[expect(non_camel_case_types)]
pub type u1024 = BigInt<16, false>;

/// 256-bit signed integer.
#[expect(non_camel_case_types)]
pub type i256 = BigInt<4, true>;

/// 512-bit signed integer.
#[expect(non_camel_case_types)]
pub type i512 = BigInt<8, true>;

/// 1024-bit signed integer.
#[expect(non_camel_case_types)]
pub type i1024 = BigInt<16, true>;

/// Implements the checked, overflowing and wrapping flavors of one of the
/// results of a combined division. The last argument is the value returned on
/// the overflow.
macro_rules! divisions {
    ($($op:literal: $div_rem:ident.$part:tt => $checked:ident, $overflowing:ident,
       $wrapping:ident, $wrapped:ident;)+) => {$(
        #[doc = concat!("Checked ", $op, ". Returns `None` if `rhs` is zero or an overflow ")]
        #[doc = "has happened."]
        pub fn $checked(self, rhs: impl Into<Self>) -> Option<Self> {
            self.$div_rem(rhs).ok().map(|res| res.$part)
        }

        #[doc = concat!("Overflowing ", $op, ". Returns the wrapped result and whether an ")]
        #[doc = "overflow has happened.\n\n# Panics\n\nIf `rhs` is zero."]
        pub fn $overflowing(self, rhs: impl Into<Self>) -> (Self, bool) {
            match self.$div_rem(rhs) {
                Err(DivError::Overflow) => (Self::$wrapped, true),
                res => (strict(res).$part, false),
            }
        }

        #[doc = concat!("Wrapping (modular) ", $op, ".\n\n# Panics\n\nIf `rhs` is zero.")]
        pub fn $wrapping(self, rhs: impl Into<Self>) -> Self { self.$overflowing(rhs).0 }
    )+};
}

/// Extracts the result of a division, panicking on an error like the primitive
/// integers do.
fn strict<T>(res: Result<T, DivError>) -> T {
    res.unwrap_or_else(|err| panic!("attempt to perform {err}"))
}

impl<const N: usize, const S: bool> BigInt<N, S> {
    /// Size of the integer in bits.
    pub const BITS: u32 = 64 * N as u32;

    /// Size of the integer in bytes.
    pub const BYTES: usize = 8 * N;

    /// Zero.
    pub const ZERO: Self = Self::from_inner([0; N]);

    /// One.
    pub const ONE: Self = {
        let mut limbs = [0; N];
        limbs[0] = 1;
        Self(limbs)
    };

    /// The smallest value of the type.
    pub const MIN: Self = {
        let mut limbs = [0; N];
        if S {
            limbs[N - 1] = 1 << 63;
        }
        Self(limbs)
    };

    /// The largest value of the type.
    pub const MAX: Self = {
        let mut limbs = [u64::MAX; N];
        if S {
            limbs[N - 1] >>= 1;
        }
        Self(limbs)
    };

    /// Constructs the integer from its limbs, given in the little-endian order.
    pub const fn from_inner(limbs: [u64; N]) -> Self {
        const { assert!(N > 0, "a big integer must have at least one limb") };
        Self(limbs)
    }

    /// Returns the limbs of the integer in the little-endian order.
    pub const fn as_inner(&self) -> &[u64; N] { &self.0 }

    /// Converts the integer into its limbs in the little-endian order.
    pub const fn into_inner(self) -> [u64; N] { self.0 }

    /// Constructs the integer from its 128 least significant bits, replicating
    /// `fill` into the limbs above them.
    fn from_low(low: u128, fill: u64) -> Self {
        Self::from_inner(array::from_fn(|i| match i {
            0 => low as u64,
            1 => (low >> 64) as u64,
            _ => fill,
        }))
    }

    /// Creates the integer from a slice of [`Self::BYTES`] bytes in the
    /// big-endian order.
    pub fn from_be_slice(bytes: &[u8]) -> Result<Self, ParseLengthError> {
        Self::chunks(bytes).map(Self::from_be_chunks)
    }

    /// Creates the integer from a slice of [`Self::BYTES`] bytes in the
    /// little-endian order.
    pub fn from_le_slice(bytes: &[u8]) -> Result<Self, ParseLengthError> {
        Self::chunks(bytes).map(Self::from_le_chunks)
    }

    /// Splits a byte slice into the limb-sized chunks.
    fn chunks(bytes: &[u8]) -> Result<&[[u8; 8]; N], ParseLengthError> {
        let err = ParseLengthError {
            actual: bytes.len(),
            expected: Self::BYTES,
        };
        match bytes.as_chunks() {
            (chunks, []) => chunks.try_into().map_err(|_| err),
            _ => Err(err),
        }
    }

    fn from_be_chunks(chunks: &[[u8; 8]; N]) -> Self {
        Self::from_inner(array::from_fn(|i| u64::from_be_bytes(chunks[N - 1 - i])))
    }

    fn from_le_chunks(chunks: &[[u8; 8]; N]) -> Self {
        Self::from_inner(chunks.map(u64::from_le_bytes))
    }

    fn to_be_chunks(self) -> [[u8; 8]; N] { array::from_fn(|i| self.0[N - 1 - i].to_be_bytes()) }

    fn to_le_chunks(self) -> [[u8; 8]; N] { self.0.map(u64::to_le_bytes) }

    /// Returns whether the bit `index`, counted from the least significant one,
    /// is set.
    ///
    /// # Panics
    ///
    /// If `index` is not less than [`Self::BITS`].
    pub const fn bit(&self, index: u32) -> bool {
        self.0[(index / 64) as usize] >> (index % 64) & 1 == 1
    }

    /// Returns the 32 least significant bits of the number.
    pub const fn low_u32(&self) -> u32 { self.0[0] as u32 }

    /// Returns the 64 least significant bits of the number.
    pub const fn low_u64(&self) -> u64 { self.0[0] }

    /// Returns `true` if the number is zero.
    pub fn is_zero(&self) -> bool { self.0 == [0; N] }

    /// Returns `true` if the number is negative, which is never the case for
    /// the unsigned integers.
    pub const fn is_negative(&self) -> bool { S && self.0[N - 1] >> 63 == 1 }

    /// Returns `true` if the number is greater than zero.
    pub fn is_positive(&self) -> bool { !self.is_negative() && !self.is_zero() }

    /// Returns the number of leading zeros in the binary representation of the
    /// number.
    pub fn leading_zeros(&self) -> u32 {
        let mut zeros = 0;
        for limb in self.0.iter().rev() {
            zeros += limb.leading_zeros();
            if *limb != 0 {
                break;
            }
        }
        zeros
    }

    /// Returns the number of trailing zeros in the binary representation of the
    /// number.
    pub fn trailing_zeros(&self) -> u32 {
        let mut zeros = 0;
        for limb in &self.0 {
            zeros += limb.trailing_zeros();
            if *limb != 0 {
                break;
            }
        }
        zeros
    }

    /// Returns the number of leading ones in the binary representation of the
    /// number.
    pub fn leading_ones(&self) -> u32 { (!*self).leading_zeros() }

    /// Returns the number of trailing ones in the binary representation of the
    /// number.
    pub fn trailing_ones(&self) -> u32 { (!*self).trailing_zeros() }

    /// Returns the least number of bits needed to represent the number, which
    /// for the signed integers includes the sign bit.
    pub fn bits_required(&self) -> u32 {
        match (S, self.is_negative()) {
            (false, _) => Self::BITS - self.leading_zeros(),
            (true, false) => Self::BITS + 1 - self.leading_zeros(),
            (true, true) => Self::BITS + 1 - self.leading_ones(),
        }
    }

    /// Limbs of the absolute value of the number.
    fn magnitude(&self) -> [u64; N] {
        if self.is_negative() {
            limbs::neg(&self.0)
        } else {
            self.0
        }
    }

    /// Negates the number when `negate` is set, wrapping around at the minimal
    /// value.
    fn negated_if(self, negate: bool) -> Self { if negate { self.wrapping_neg() } else { self } }

    /// Calculates the quotient and the remainder of the division of `self` by
    /// `rhs`, with the quotient rounded towards zero and the remainder having
    /// the sign of `self`.
    pub fn div_rem(self, rhs: impl Into<Self>) -> Result<(Self, Self), DivError> {
        let rhs = rhs.into();
        if rhs.is_zero() {
            return Err(DivError::ZeroDiv);
        }
        let (quot, rem) = limbs::div_rem(&self.magnitude(), &rhs.magnitude());
        let quot = Self(quot).negated_if(self.is_negative() != rhs.is_negative());
        // The only overflow is `MIN / -1`: a positive quotient which reads as a
        // negative one.
        if quot.is_negative() && self.is_negative() == rhs.is_negative() {
            return Err(DivError::Overflow);
        }
        Ok((quot, Self(rem).negated_if(self.is_negative())))
    }

    /// Calculates the quotient and the remainder of the Euclidean division of
    /// `self` by `rhs`, where the remainder is never negative.
    pub fn div_rem_euclid(self, rhs: impl Into<Self>) -> Result<(Self, Self), DivError> {
        let rhs = rhs.into();
        let (quot, rem) = self.div_rem(rhs)?;
        // A negative remainder is less than the divisor by the absolute value,
        // thus neither of the corrections can overflow.
        Ok(match (rem.is_negative(), rhs.is_negative()) {
            (false, _) => (quot, rem),
            (true, false) => (quot.wrapping_sub(Self::ONE), rem.wrapping_add(rhs)),
            (true, true) => (quot.wrapping_add(Self::ONE), rem.wrapping_sub(rhs)),
        })
    }

    divisions! {
        "Euclidean division": div_rem_euclid.0 =>
            checked_div_euclid, overflowing_div_euclid, wrapping_div_euclid, MIN;
        "Euclidean remainder": div_rem_euclid.1 =>
            checked_rem_euclid, overflowing_rem_euclid, wrapping_rem_euclid, ZERO;
    }

    /// Calculates the quotient of the Euclidean division of `self` by `rhs`.
    ///
    /// # Panics
    ///
    /// If `rhs` is zero or the division overflows.
    pub fn div_euclid(self, rhs: impl Into<Self>) -> Self { strict(self.div_rem_euclid(rhs)).0 }

    /// Calculates the non-negative remainder of the Euclidean division of
    /// `self` by `rhs`.
    ///
    /// # Panics
    ///
    /// If `rhs` is zero or the division overflows.
    pub fn rem_euclid(self, rhs: impl Into<Self>) -> Self { strict(self.div_rem_euclid(rhs)).1 }

    /// Shifts the number left by `rhs` bits. Unlike the `<<` operator accepts
    /// any shift, with the shifts by [`Self::BITS`] and more giving zero.
    pub fn unbounded_shl(self, rhs: u32) -> Self { Self(limbs::shl(&self.0, rhs)) }

    /// Shifts the number right by `rhs` bits, preserving the sign of the signed
    /// integers. Unlike the `>>` operator accepts any shift, with the shifts by
    /// [`Self::BITS`] and more giving `-1` for the negative numbers and zero
    /// for the rest.
    pub fn unbounded_shr(self, rhs: u32) -> Self {
        Self(limbs::shr(&self.0, rhs, if self.is_negative() { u64::MAX } else { 0 }))
    }

    /// Parses the integer from a string of digits in the given `radix`,
    /// optionally preceded by a `+` sign or, for the signed integers, by a `-`
    /// sign.
    ///
    /// # Panics
    ///
    /// If `radix` is not in the range from 2 to 36.
    pub fn from_str_radix(src: &str, radix: u32) -> Result<Self, ParseIntError> {
        assert!((2..=36).contains(&radix), "radix must lie in the range `2..=36`");
        let (negative, digits) = match src.as_bytes() {
            [] => return Err(ParseIntError(IntErrorKind::Empty)),
            [b'+' | b'-'] => return Err(ParseIntError(IntErrorKind::InvalidDigit)),
            [b'+', digits @ ..] => (false, digits),
            [b'-', digits @ ..] if S => (true, digits),
            digits => (false, digits),
        };
        let overflow = if negative {
            IntErrorKind::NegOverflow
        } else {
            IntErrorKind::PosOverflow
        };

        let mut limbs = [0; N];
        for &byte in digits {
            let digit = char::from(byte)
                .to_digit(radix)
                .ok_or(ParseIntError(IntErrorKind::InvalidDigit))?;
            if limbs::mul_add_limb(&mut limbs, radix.into(), digit.into()) != 0 {
                return Err(ParseIntError(overflow));
            }
        }
        let res = Self::from_inner(limbs).negated_if(negative);
        // The magnitude fits the limbs, but a signed number must also keep its
        // sign.
        if !res.is_zero() && res.is_negative() != negative {
            return Err(ParseIntError(overflow));
        }
        Ok(res)
    }

    /// Writes the number in a radix which is a power of two, with each digit
    /// taking `bits` bits. `W` must be the number of the digits in a limb.
    fn fmt_radix<const W: usize>(
        &self,
        f: &mut Formatter<'_>,
        bits: u32,
        digits: &[u8; 16],
        prefix: &str,
    ) -> fmt::Result {
        let mut buf = [[0; W]; N];
        let buf = buf.as_flattened_mut();
        let len = (Self::BITS - self.leading_zeros()).div_ceil(bits).max(1) as usize;
        for (i, digit) in buf[..len].iter_mut().rev().enumerate() {
            let (limb, offset) = (i * bits as usize / 64, (i * bits as usize % 64) as u32);
            // A digit may cross the limb boundary and take its high bits from
            // the next limb.
            let next = self
                .0
                .get(limb + 1)
                .map_or(0, |next| next.unbounded_shl(64 - offset));
            *digit = digits[((self.0[limb] >> offset | next) % (1 << bits)) as usize];
        }
        f.pad_integral(true, prefix, str::from_utf8(&buf[..len]).map_err(|_| fmt::Error)?)
    }
}

impl<const N: usize> BigInt<N, false> {
    /// Reinterprets the bits of the number as a signed integer of the same
    /// size.
    pub const fn cast_signed(self) -> BigInt<N, true> { BigInt(self.0) }
}

impl<const N: usize> BigInt<N, true> {
    /// Reinterprets the bits of the number as an unsigned integer of the same
    /// size.
    pub const fn cast_unsigned(self) -> BigInt<N, false> { BigInt(self.0) }

    /// Returns the absolute value of the number as an unsigned integer, which
    /// can't overflow.
    pub fn unsigned_abs(self) -> BigInt<N, false> { BigInt(self.magnitude()) }

    /// Returns the absolute value of the number.
    ///
    /// # Panics
    ///
    /// If the number is [`Self::MIN`], which has no positive counterpart.
    pub fn abs(self) -> Self { if self.is_negative() { -self } else { self } }
}

/// Implements the array-based byte conversions, which can't be generic over the
/// number of limbs until the const generic expressions are stabilized.
macro_rules! byte_arrays {
    ($($limbs:literal)+) => {$(
        impl<const S: bool> BigInt<$limbs, S> {
            /// Creates the integer from its byte representation in the big-endian order.
            pub fn from_be_bytes(bytes: [u8; 8 * $limbs]) -> Self {
                let (chunks, _) = bytes.as_chunks();
                Self::from_inner(array::from_fn(|i| u64::from_be_bytes(chunks[$limbs - 1 - i])))
            }

            /// Creates the integer from its byte representation in the little-endian order.
            pub fn from_le_bytes(bytes: [u8; 8 * $limbs]) -> Self {
                let (chunks, _) = bytes.as_chunks();
                Self::from_inner(array::from_fn(|i| u64::from_le_bytes(chunks[i])))
            }

            /// Returns the byte representation of the integer in the big-endian order.
            pub fn to_be_bytes(self) -> [u8; 8 * $limbs] {
                let mut bytes = [0; 8 * $limbs];
                bytes.copy_from_slice(self.to_be_chunks().as_flattened());
                bytes
            }

            /// Returns the byte representation of the integer in the little-endian order.
            pub fn to_le_bytes(self) -> [u8; 8 * $limbs] {
                let mut bytes = [0; 8 * $limbs];
                bytes.copy_from_slice(self.to_le_chunks().as_flattened());
                bytes
            }
        }
    )+};
}
byte_arrays!(4 8 16);

impl<const N: usize, const S: bool> Default for BigInt<N, S> {
    fn default() -> Self { Self::ZERO }
}

impl<const N: usize, const S: bool> From<bool> for BigInt<N, S> {
    fn from(value: bool) -> Self { Self::from_low(value.into(), 0) }
}

macro_rules! from_unsigned {
    ($($prim:ident)+) => {$(
        impl<const N: usize, const S: bool> From<$prim> for BigInt<N, S> {
            fn from(value: $prim) -> Self {
                const { assert!($prim::BITS + S as u32 <= Self::BITS, "lossy conversion") };
                Self::from_low(value.into(), 0)
            }
        }
    )+};
}
from_unsigned!(u8 u16 u32 u64 u128);

macro_rules! from_signed {
    ($($prim:ident)+) => {$(
        impl<const N: usize> From<$prim> for BigInt<N, true> {
            fn from(value: $prim) -> Self {
                const { assert!($prim::BITS <= Self::BITS, "lossy conversion") };
                let fill = if value < 0 { u64::MAX } else { 0 };
                Self::from_low(i128::from(value).cast_unsigned(), fill)
            }
        }
    )+};
}
from_signed!(i8 i16 i32 i64 i128);

macro_rules! from_smaller {
    ($($from:literal => $to:literal),+) => {$(
        impl<const S: bool> From<BigInt<$from, S>> for BigInt<$to, S> {
            fn from(value: BigInt<$from, S>) -> Self {
                const { assert!($from < $to, "lossy conversion") };
                let fill = if value.is_negative() { u64::MAX } else { 0 };
                Self(array::from_fn(|i| value.0.get(i).copied().unwrap_or(fill)))
            }
        }
    )+};
}
from_smaller!(4 => 8, 4 => 16, 8 => 16);

impl<const N: usize, const S: bool> Number for BigInt<N, S> {
    const BITS: u32 = Self::BITS;
    const MIN: Self = Self::MIN;
    const MAX: Self = Self::MAX;
    const ZERO: Self = Self::ZERO;
    const ONE: Self = Self::ONE;
}

impl<const N: usize> Unsigned for BigInt<N, false> {}

impl<const N: usize> Integer for BigInt<N, true> {
    const NEG_ONE: Self = Self::from_inner([u64::MAX; N]);

    fn is_negative(self) -> bool { BigInt::is_negative(&self) }
}

impl<Rhs: Into<Self>, const N: usize, const S: bool> AddOverflowing<Rhs> for BigInt<N, S> {
    type Output = Self;

    fn overflowing_add(self, rhs: Rhs) -> (Self, bool) {
        let (mut sum, rhs) = (self, rhs.into());
        let carry = limbs::add(&mut sum.0, &rhs.0);
        // A signed sum overflows when the operands share the sign which the sum
        // lacks.
        let wrapped =
            self.is_negative() == rhs.is_negative() && self.is_negative() != sum.is_negative();
        (sum, if S { wrapped } else { carry })
    }
}

impl<Rhs: Into<Self>, const N: usize, const S: bool> SubOverflowing<Rhs> for BigInt<N, S> {
    type Output = Self;

    fn overflowing_sub(self, rhs: Rhs) -> (Self, bool) {
        let (mut diff, rhs) = (self, rhs.into());
        let borrow = limbs::sub(&mut diff.0, &rhs.0);
        // A signed difference overflows when the operands differ in sign and
        // the result has the sign of the subtrahend.
        let wrapped =
            self.is_negative() != rhs.is_negative() && self.is_negative() != diff.is_negative();
        (diff, if S { wrapped } else { borrow })
    }
}

impl<Rhs: Into<Self>, const N: usize, const S: bool> MulOverflowing<Rhs> for BigInt<N, S> {
    type Output = Self;

    fn overflowing_mul(self, rhs: Rhs) -> (Self, bool) {
        let rhs = rhs.into();
        let negative = self.is_negative() != rhs.is_negative();
        let (magnitude, overflow) = limbs::mul(&self.magnitude(), &rhs.magnitude());
        let product = Self(magnitude).negated_if(negative);
        // A magnitude which fits the limbs still overflows a signed type if it
        // turns the sign of the product into the opposite of the expected one.
        (product, overflow || (!product.is_zero() && product.is_negative() != negative))
    }
}

/// Implements the checked, wrapping and saturating traits of an overflowing
/// operation. The last argument tells whether the operation saturates at the
/// minimal value.
macro_rules! arithmetics {
    ($($overflowing:ident =>
       $checked_trait:ident::$checked:ident,
       $wrapping_trait:ident::$wrapping:ident,
       $saturating_trait:ident::$saturating:ident,
       |$lhs:ident, $rhs:ident| $min:expr;)+) => {$(
        impl<Rhs: Into<Self>, const N: usize, const S: bool> $checked_trait<Rhs> for BigInt<N, S> {
            type Output = Self;

            fn $checked(self, rhs: Rhs) -> Option<Self> {
                let (res, overflow) = self.$overflowing(rhs);
                (!overflow).then_some(res)
            }
        }

        impl<Rhs: Into<Self>, const N: usize, const S: bool> $wrapping_trait<Rhs> for BigInt<N, S> {
            type Output = Self;

            fn $wrapping(self, rhs: Rhs) -> Self { self.$overflowing(rhs).0 }
        }

        impl<Rhs: Into<Self>, const N: usize, const S: bool> $saturating_trait<Rhs>
            for BigInt<N, S>
        {
            type Output = Self;

            fn $saturating(self, rhs: Rhs) -> Self {
                let ($lhs, $rhs) = (self, rhs.into());
                match $lhs.$overflowing($rhs) {
                    (res, false) => res,
                    _ if $min => Self::MIN,
                    _ => Self::MAX,
                }
            }
        }
    )+};
}

arithmetics! {
    overflowing_add => AddChecked::checked_add, AddWrapping::wrapping_add,
        AddSaturate::saturating_add, |lhs, _rhs| lhs.is_negative();
    overflowing_sub => SubChecked::checked_sub, SubWrapping::wrapping_sub,
        SubSaturate::saturating_sub, |lhs, _rhs| !S || lhs.is_negative();
    overflowing_mul => MulChecked::checked_mul, MulWrapping::wrapping_mul,
        MulSaturate::saturating_mul, |lhs, rhs| lhs.is_negative() != rhs.is_negative();
}

macro_rules! division_traits {
    ($($part:tt =>
       $checked_trait:ident::$checked:ident,
       $overflowing_trait:ident::$overflowing:ident,
       $wrapping_trait:ident::$wrapping:ident, $wrapped:ident;)+) => {$(
        impl<Rhs: Into<Self>, const N: usize, const S: bool> $checked_trait<Rhs> for BigInt<N, S> {
            type Output = Self;

            fn $checked(self, rhs: Rhs) -> Option<Self> {
                self.div_rem(rhs).ok().map(|res| res.$part)
            }
        }

        impl<Rhs: Into<Self>, const N: usize, const S: bool> $overflowing_trait<Rhs>
            for BigInt<N, S>
        {
            type Output = Self;

            fn $overflowing(self, rhs: Rhs) -> (Self, bool) {
                match self.div_rem(rhs) {
                    Err(DivError::Overflow) => (Self::$wrapped, true),
                    res => (strict(res).$part, false),
                }
            }
        }

        impl<Rhs: Into<Self>, const N: usize, const S: bool> $wrapping_trait<Rhs> for BigInt<N, S> {
            type Output = Self;

            fn $wrapping(self, rhs: Rhs) -> Self { self.$overflowing(rhs).0 }
        }
    )+};
}

division_traits! {
    0 => DivChecked::checked_div, DivOverflowing::overflowing_div, DivWrapping::wrapping_div, MIN;
    1 => RemChecked::checked_rem, RemOverflowing::overflowing_rem, RemWrapping::wrapping_rem, ZERO;
}

impl<Rhs: Into<Self>, const N: usize, const S: bool> DivSaturate<Rhs> for BigInt<N, S> {
    type Output = Self;

    fn saturating_div(self, rhs: Rhs) -> Self {
        match self.overflowing_div(rhs) {
            (quot, false) => quot,
            _ => Self::MAX,
        }
    }
}

impl<const N: usize, const S: bool> NegWrapping for BigInt<N, S> {
    type Output = Self;

    fn wrapping_neg(self) -> Self { Self(limbs::neg(&self.0)) }
}

impl<const N: usize> NegChecked for BigInt<N, true> {
    type Output = Self;

    fn checked_neg(self) -> Option<Self> { (self != Self::MIN).then(|| self.wrapping_neg()) }
}

impl<const N: usize> NegOverflowing for BigInt<N, true> {
    type Output = Self;

    fn overflowing_neg(self) -> (Self, bool) { (self.wrapping_neg(), self == Self::MIN) }
}

impl<const N: usize> NegSaturate for BigInt<N, true> {
    type Output = Self;

    fn saturating_neg(self) -> Self {
        if self == Self::MIN {
            Self::MAX
        } else {
            self.wrapping_neg()
        }
    }
}

impl<const N: usize, const S: bool> ShlChecked for BigInt<N, S> {
    type Output = Self;

    fn checked_shl(self, rhs: u32) -> Option<Self> {
        (rhs < Self::BITS).then(|| self.unbounded_shl(rhs))
    }
}

impl<const N: usize, const S: bool> ShlOverflowing for BigInt<N, S> {
    type Output = Self;

    fn overflowing_shl(self, rhs: u32) -> (Self, bool) {
        (self.unbounded_shl(rhs % Self::BITS), rhs >= Self::BITS)
    }
}

impl<const N: usize, const S: bool> ShlWrapping for BigInt<N, S> {
    type Output = Self;

    fn wrapping_shl(self, rhs: u32) -> Self { self.unbounded_shl(rhs % Self::BITS) }
}

impl<const N: usize, const S: bool> ShrChecked for BigInt<N, S> {
    type Output = Self;

    fn checked_shr(self, rhs: u32) -> Option<Self> {
        (rhs < Self::BITS).then(|| self.unbounded_shr(rhs))
    }
}

impl<const N: usize, const S: bool> ShrOverflowing for BigInt<N, S> {
    type Output = Self;

    fn overflowing_shr(self, rhs: u32) -> (Self, bool) {
        (self.unbounded_shr(rhs % Self::BITS), rhs >= Self::BITS)
    }
}

impl<const N: usize, const S: bool> ShrWrapping for BigInt<N, S> {
    type Output = Self;

    fn wrapping_shr(self, rhs: u32) -> Self { self.unbounded_shr(rhs % Self::BITS) }
}

impl<const N: usize, const S: bool> FromStr for BigInt<N, S> {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, ParseIntError> { Self::from_str_radix(s, 10) }
}

impl<const N: usize, const S: bool> Ord for BigInt<N, S> {
    fn cmp(&self, other: &Self) -> Ordering {
        // Numbers of the same sign are ordered as the unsigned ones, starting
        // from the most significant limb.
        other
            .is_negative()
            .cmp(&self.is_negative())
            .then_with(|| self.0.iter().rev().cmp(other.0.iter().rev()))
    }
}

impl<const N: usize, const S: bool> PartialOrd for BigInt<N, S> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}

/// Implements a binary operator, together with its assigning form, for any
/// right-hand operand convertible into the big integer.
macro_rules! operators {
    ($($op:ident $fn:ident $op_assign:ident $fn_assign:ident =>
       |$lhs:ident, $rhs:ident| $res:expr;)+) => {$(
        impl<T: Into<Self>, const N: usize, const S: bool> $op<T> for BigInt<N, S> {
            type Output = Self;

            fn $fn(self, rhs: T) -> Self {
                let ($lhs, $rhs) = (self, rhs.into());
                $res
            }
        }

        impl<T: Into<Self>, const N: usize, const S: bool> $op_assign<T> for BigInt<N, S> {
            fn $fn_assign(&mut self, rhs: T) { *self = $op::$fn(*self, rhs) }
        }
    )+};
}

operators! {
    Add add AddAssign add_assign =>
        |lhs, rhs| lhs.checked_add(rhs).expect("attempt to add with overflow");
    Sub sub SubAssign sub_assign =>
        |lhs, rhs| lhs.checked_sub(rhs).expect("attempt to subtract with overflow");
    Mul mul MulAssign mul_assign =>
        |lhs, rhs| lhs.checked_mul(rhs).expect("attempt to multiply with overflow");
    Div div DivAssign div_assign => |lhs, rhs| strict(lhs.div_rem(rhs)).0;
    Rem rem RemAssign rem_assign => |lhs, rhs| strict(lhs.div_rem(rhs)).1;
    BitAnd bitand BitAndAssign bitand_assign =>
        |lhs, rhs| Self(array::from_fn(|i| lhs.0[i] & rhs.0[i]));
    BitOr bitor BitOrAssign bitor_assign =>
        |lhs, rhs| Self(array::from_fn(|i| lhs.0[i] | rhs.0[i]));
    BitXor bitxor BitXorAssign bitxor_assign =>
        |lhs, rhs| Self(array::from_fn(|i| lhs.0[i] ^ rhs.0[i]));
}

impl<const N: usize, const S: bool> Shl<u32> for BigInt<N, S> {
    type Output = Self;

    fn shl(self, rhs: u32) -> Self {
        self.checked_shl(rhs)
            .expect("attempt to shift left with overflow")
    }
}

impl<const N: usize, const S: bool> ShlAssign<u32> for BigInt<N, S> {
    fn shl_assign(&mut self, rhs: u32) { *self = *self << rhs }
}

impl<const N: usize, const S: bool> Shr<u32> for BigInt<N, S> {
    type Output = Self;

    fn shr(self, rhs: u32) -> Self {
        self.checked_shr(rhs)
            .expect("attempt to shift right with overflow")
    }
}

impl<const N: usize, const S: bool> ShrAssign<u32> for BigInt<N, S> {
    fn shr_assign(&mut self, rhs: u32) { *self = *self >> rhs }
}

impl<const N: usize, const S: bool> Not for BigInt<N, S> {
    type Output = Self;

    fn not(self) -> Self { Self(self.0.map(|limb| !limb)) }
}

impl<const N: usize> Neg for BigInt<N, true> {
    type Output = Self;

    fn neg(self) -> Self { self.checked_neg().expect("attempt to negate with overflow") }
}

impl<const N: usize, const S: bool> Display for BigInt<N, S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        /// The largest power of ten which fits a limb.
        const CHUNK: u64 = 10u64.pow(CHUNK_DIGITS);
        const CHUNK_DIGITS: u32 = 19;

        let mut buf = [[0; 20]; N];
        let buf = buf.as_flattened_mut();
        let mut pos = buf.len();
        let (mut magnitude, mut divisor) = (self.magnitude(), [0; N]);
        divisor[0] = CHUNK;
        loop {
            let (quot, rem) = limbs::div_rem(&magnitude, &divisor);
            let (mut chunk, last) = (rem[0], quot == [0; N]);
            // All chunks but the most significant one are padded with zeros.
            for _ in 0..CHUNK_DIGITS {
                pos -= 1;
                buf[pos] = b'0' + (chunk % 10) as u8;
                chunk /= 10;
                if last && chunk == 0 {
                    break;
                }
            }
            if last {
                break;
            }
            magnitude = quot;
        }
        let digits = str::from_utf8(&buf[pos..]).map_err(|_| fmt::Error)?;
        f.pad_integral(!self.is_negative(), "", digits)
    }
}

impl<const N: usize, const S: bool> Debug for BigInt<N, S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result { Display::fmt(self, f) }
}

impl<const N: usize, const S: bool> Binary for BigInt<N, S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.fmt_radix::<64>(f, 1, b"0123456789abcdef", "0b")
    }
}

impl<const N: usize, const S: bool> Octal for BigInt<N, S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.fmt_radix::<22>(f, 3, b"0123456789abcdef", "0o")
    }
}

impl<const N: usize, const S: bool> LowerHex for BigInt<N, S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.fmt_radix::<16>(f, 4, b"0123456789abcdef", "0x")
    }
}

impl<const N: usize, const S: bool> UpperHex for BigInt<N, S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.fmt_radix::<16>(f, 4, b"0123456789ABCDEF", "0x")
    }
}

/// Human-readable formats use the hex encoding of the big-endian bytes of the
/// number, while the binary formats use the bytes themselves.
#[cfg(feature = "serde")]
impl<const N: usize, const S: bool> serde::Serialize for BigInt<N, S> {
    fn serialize<Sr: serde::Serializer>(&self, serializer: Sr) -> Result<Sr::Ok, Sr::Error> {
        if serializer.is_human_readable() {
            serializer.collect_str(&format_args!("{self:0width$x}", width = 2 * Self::BYTES))
        } else {
            serializer.serialize_bytes(self.to_be_chunks().as_flattened())
        }
    }
}

#[cfg(feature = "serde")]
impl<'de, const N: usize, const S: bool> serde::Deserialize<'de> for BigInt<N, S> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::{Error, Unexpected, Visitor};

        struct BytesVisitor<const N: usize, const S: bool>;

        impl<const N: usize, const S: bool> Visitor<'_> for BytesVisitor<N, S> {
            type Value = BigInt<N, S>;

            fn expecting(&self, f: &mut Formatter<'_>) -> fmt::Result {
                write!(f, "{} bytes or a hex string of {} characters", 8 * N, 16 * N)
            }

            fn visit_str<E: Error>(self, s: &str) -> Result<Self::Value, E> {
                let bytes = crate::hex::HexIterator::new(s)
                    .ok()
                    .filter(|bytes| bytes.len() == 8 * N)
                    .ok_or_else(|| E::invalid_length(s.len(), &self))?;
                let mut chunks = [[0; 8]; N];
                for (dst, src) in chunks.as_flattened_mut().iter_mut().zip(bytes) {
                    *dst = src.map_err(|_| E::invalid_value(Unexpected::Str(s), &self))?;
                }
                Ok(BigInt::from_be_chunks(&chunks))
            }

            fn visit_bytes<E: Error>(self, bytes: &[u8]) -> Result<Self::Value, E> {
                BigInt::from_be_slice(bytes).map_err(|_| E::invalid_length(bytes.len(), &self))
            }
        }

        if deserializer.is_human_readable() {
            deserializer.deserialize_str(BytesVisitor)
        } else {
            deserializer.deserialize_bytes(BytesVisitor)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type U128 = BigInt<2, false>;
    type I128 = BigInt<2, true>;

    fn u(val: u128) -> U128 { BigInt::from_inner([val as u64, (val >> 64) as u64]) }
    fn i(val: i128) -> I128 { u(val as u128).cast_signed() }
    fn flagged<T, B>(conv: fn(T) -> B) -> impl Fn((T, bool)) -> (B, bool) {
        move |(val, flag)| (conv(val), flag)
    }

    /// Xorshift generator of limbs, biased towards the values provoking carries
    /// and other corner cases of the multi-precision arithmetic.
    struct Rng(u64);

    impl Rng {
        fn next(&mut self) -> u64 {
            self.0 ^= self.0 << 13;
            self.0 ^= self.0 >> 7;
            self.0 ^= self.0 << 17;
            self.0
        }

        fn limb(&mut self) -> u64 {
            match self.next() % 10 {
                0 => 0,
                1 => 1,
                2 => u64::MAX,
                3 => u64::MAX - 1,
                4 => 1 << 63,
                5 => (1 << 63) - 1,
                6 => self.next() >> (self.next() % 64),
                _ => self.next(),
            }
        }

        /// Number with a random count of significant limbs.
        fn limbs<const N: usize>(&mut self) -> [u64; N] {
            let len = self.next() as usize % (N + 1);
            array::from_fn(|i| if i < len { self.limb() } else { 0 })
        }
    }

    /// Edge cases of 128-bit arithmetic mixed with random numbers.
    fn samples() -> Vec<u128> {
        let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
        let mut samples = vec![0, 1, 2, 3, 4, 5, 7, 10, 255, 256, 60000, 70000];
        for pow in [1u128 << 31, 1 << 32, 1 << 63, 1 << 64, 1 << 65, 1 << 126, 1 << 127] {
            samples.extend([
                pow - 1,
                pow,
                pow + 1,
                pow.wrapping_neg(),
                !pow,
                0u128.wrapping_sub(pow + 1),
            ]);
        }
        samples.extend([u128::MAX, u128::MAX - 1, u128::MAX - 2, u128::MAX / 3, u128::MAX / 10]);
        samples.extend((0..40).map(|_| u128::from(rng.limb()) << 64 | u128::from(rng.limb())));
        samples
    }

    const SHIFTS: [u32; 14] = [0, 1, 2, 31, 63, 64, 65, 100, 126, 127, 128, 129, 1000, u32::MAX];

    macro_rules! assert_fmt {
        ($big:expr, $prim:expr => $($spec:literal)+) => {$(
            assert_eq!(format!($spec, $big), format!($spec, $prim), "{} of {}", $spec, $prim);
        )+};
    }

    macro_rules! assert_same_fmt {
        ($big:expr, $prim:expr) => {
            assert_fmt!($big, $prim => "{}" "{:?}" "{:b}" "{:o}" "{:x}" "{:X}" "{:#b}" "{:#o}"
                "{:#x}" "{:#X}" "{:+}" "{:45}" "{:<45}" "{:^45}" "{:>+45}" "{:045}" "{:+045}"
                "{:#040x}" "{:_<#40X}" "{:*^#140b}" "{:#050o}" "{:1}" "{:#1x}");
        };
    }

    #[test]
    fn unsigned_matches_u128() {
        let samples = samples();
        for &a in &samples {
            let x = u(a);
            assert_eq!(x.is_zero(), a == 0);
            assert_eq!(x.is_positive(), a > 0);
            assert!(!x.is_negative());
            assert_eq!(x.leading_zeros(), a.leading_zeros());
            assert_eq!(x.leading_ones(), a.leading_ones());
            assert_eq!(x.trailing_zeros(), a.trailing_zeros());
            assert_eq!(x.trailing_ones(), a.trailing_ones());
            assert_eq!(x.bits_required(), 128 - a.leading_zeros());
            assert_eq!(x.low_u64(), a as u64);
            assert_eq!(x.low_u32(), a as u32);
            assert_eq!(!x, u(!a));
            assert_eq!(x.wrapping_neg(), u(a.wrapping_neg()));
            assert_eq!(x.cast_signed(), i(a as i128));
            assert_eq!(U128::from(a), x);
            for bit in 0..128 {
                assert_eq!(x.bit(bit), a >> bit & 1 == 1);
            }
            for shift in SHIFTS {
                assert_eq!(x.unbounded_shl(shift), u(a.unbounded_shl(shift)));
                assert_eq!(x.unbounded_shr(shift), u(a.unbounded_shr(shift)));
                assert_eq!(x.checked_shl(shift), a.checked_shl(shift).map(u));
                assert_eq!(x.checked_shr(shift), a.checked_shr(shift).map(u));
                assert_eq!(x.overflowing_shl(shift), flagged(u)(a.overflowing_shl(shift)));
                assert_eq!(x.overflowing_shr(shift), flagged(u)(a.overflowing_shr(shift)));
                assert_eq!(x.wrapping_shl(shift), u(a.wrapping_shl(shift)));
                assert_eq!(x.wrapping_shr(shift), u(a.wrapping_shr(shift)));
            }
            assert_same_fmt!(x, a);
            for radix in [2, 3, 8, 10, 16, 36] {
                let digits = match radix {
                    2 => format!("{a:b}"),
                    8 => format!("{a:o}"),
                    16 => format!("{a:x}"),
                    _ => format!("{a}"),
                };
                for src in [
                    digits.clone(),
                    format!("+{digits}"),
                    format!("-{digits}"),
                    format!("{digits}0"),
                ] {
                    let expected = u128::from_str_radix(&src, radix)
                        .map(u)
                        .map_err(|err| *err.kind());
                    assert_eq!(
                        U128::from_str_radix(&src, radix).map_err(|err| err.0),
                        expected,
                        "{src}"
                    );
                }
            }

            for &b in &samples {
                let y = u(b);
                assert_eq!(x.cmp(&y), a.cmp(&b));
                assert_eq!(x & y, u(a & b));
                assert_eq!(x | y, u(a | b));
                assert_eq!(x ^ y, u(a ^ b));

                assert_eq!(x.overflowing_add(y), flagged(u)(a.overflowing_add(b)));
                assert_eq!(x.overflowing_sub(y), flagged(u)(a.overflowing_sub(b)));
                assert_eq!(x.overflowing_mul(y), flagged(u)(a.overflowing_mul(b)));
                assert_eq!(x.checked_add(y), a.checked_add(b).map(u));
                assert_eq!(x.checked_sub(y), a.checked_sub(b).map(u));
                assert_eq!(x.checked_mul(y), a.checked_mul(b).map(u));
                assert_eq!(x.wrapping_add(y), u(a.wrapping_add(b)));
                assert_eq!(x.wrapping_sub(y), u(a.wrapping_sub(b)));
                assert_eq!(x.wrapping_mul(y), u(a.wrapping_mul(b)));
                assert_eq!(x.saturating_add(y), u(a.saturating_add(b)));
                assert_eq!(x.saturating_sub(y), u(a.saturating_sub(b)));
                assert_eq!(x.saturating_mul(y), u(a.saturating_mul(b)));

                assert_eq!(x.checked_div(y), a.checked_div(b).map(u));
                assert_eq!(x.checked_rem(y), a.checked_rem(b).map(u));
                assert_eq!(x.checked_div_euclid(y), a.checked_div_euclid(b).map(u));
                assert_eq!(x.checked_rem_euclid(y), a.checked_rem_euclid(b).map(u));
                if b == 0 {
                    assert_eq!(x.div_rem(y), Err(DivError::ZeroDiv));
                    assert_eq!(x.div_rem_euclid(y), Err(DivError::ZeroDiv));
                    continue;
                }
                assert_eq!(x.div_rem(y), Ok((u(a / b), u(a % b))));
                assert_eq!(x / y, u(a / b));
                assert_eq!(x % y, u(a % b));
                assert_eq!(x.overflowing_div(y), (u(a / b), false));
                assert_eq!(x.overflowing_rem(y), (u(a % b), false));
                assert_eq!(x.wrapping_div(y), u(a / b));
                assert_eq!(x.wrapping_rem(y), u(a % b));
                assert_eq!(x.saturating_div(y), u(a / b));
                assert_eq!(x.div_euclid(y), u(a / b));
                assert_eq!(x.rem_euclid(y), u(a % b));
            }
        }
    }

    #[test]
    fn signed_matches_i128() {
        let samples: Vec<i128> = samples().into_iter().map(|val| val as i128).collect();
        for &a in &samples {
            let x = i(a);
            assert_eq!(x.is_zero(), a == 0);
            assert_eq!(x.is_positive(), a > 0);
            assert_eq!(x.is_negative(), a < 0);
            assert_eq!(x.leading_zeros(), a.leading_zeros());
            assert_eq!(x.leading_ones(), a.leading_ones());
            assert_eq!(x.trailing_zeros(), a.trailing_zeros());
            assert_eq!(x.trailing_ones(), a.trailing_ones());
            let sign_bits = if a < 0 {
                a.leading_ones()
            } else {
                a.leading_zeros()
            };
            assert_eq!(x.bits_required(), 129 - sign_bits);
            assert_eq!(!x, i(!a));
            assert_eq!(x.checked_neg(), a.checked_neg().map(i));
            assert_eq!(x.overflowing_neg(), flagged(i)(a.overflowing_neg()));
            assert_eq!(x.saturating_neg(), i(a.saturating_neg()));
            assert_eq!(x.wrapping_neg(), i(a.wrapping_neg()));
            assert_eq!(x.unsigned_abs(), u(a.unsigned_abs()));
            assert_eq!(x.cast_unsigned(), u(a as u128));
            assert_eq!(I128::from(a), x);
            if a != i128::MIN {
                assert_eq!(-x, i(-a));
                assert_eq!(x.abs(), i(a.abs()));
            }
            for shift in SHIFTS {
                assert_eq!(x.unbounded_shl(shift), i(a.unbounded_shl(shift)));
                assert_eq!(x.unbounded_shr(shift), i(a.unbounded_shr(shift)));
                assert_eq!(x.checked_shl(shift), a.checked_shl(shift).map(i));
                assert_eq!(x.checked_shr(shift), a.checked_shr(shift).map(i));
                assert_eq!(x.overflowing_shl(shift), flagged(i)(a.overflowing_shl(shift)));
                assert_eq!(x.overflowing_shr(shift), flagged(i)(a.overflowing_shr(shift)));
                assert_eq!(x.wrapping_shl(shift), i(a.wrapping_shl(shift)));
                assert_eq!(x.wrapping_shr(shift), i(a.wrapping_shr(shift)));
            }
            assert_same_fmt!(x, a);
            for radix in [2, 7, 10, 16, 36] {
                let digits = match radix {
                    2 => format!("{:b}", a.unsigned_abs()),
                    16 => format!("{:X}", a.unsigned_abs()),
                    _ => format!("{}", a.unsigned_abs()),
                };
                for src in [
                    digits.clone(),
                    format!("+{digits}"),
                    format!("-{digits}"),
                    format!("-{digits}1"),
                ] {
                    let expected = i128::from_str_radix(&src, radix)
                        .map(i)
                        .map_err(|err| *err.kind());
                    assert_eq!(
                        I128::from_str_radix(&src, radix).map_err(|err| err.0),
                        expected,
                        "{src}"
                    );
                }
            }

            for &b in &samples {
                let y = i(b);
                assert_eq!(x.cmp(&y), a.cmp(&b));
                assert_eq!(x & y, i(a & b));
                assert_eq!(x | y, i(a | b));
                assert_eq!(x ^ y, i(a ^ b));

                assert_eq!(x.overflowing_add(y), flagged(i)(a.overflowing_add(b)));
                assert_eq!(x.overflowing_sub(y), flagged(i)(a.overflowing_sub(b)));
                assert_eq!(x.overflowing_mul(y), flagged(i)(a.overflowing_mul(b)));
                assert_eq!(x.checked_add(y), a.checked_add(b).map(i));
                assert_eq!(x.checked_sub(y), a.checked_sub(b).map(i));
                assert_eq!(x.checked_mul(y), a.checked_mul(b).map(i));
                assert_eq!(x.wrapping_add(y), i(a.wrapping_add(b)));
                assert_eq!(x.wrapping_sub(y), i(a.wrapping_sub(b)));
                assert_eq!(x.wrapping_mul(y), i(a.wrapping_mul(b)));
                assert_eq!(x.saturating_add(y), i(a.saturating_add(b)));
                assert_eq!(x.saturating_sub(y), i(a.saturating_sub(b)));
                assert_eq!(x.saturating_mul(y), i(a.saturating_mul(b)));

                assert_eq!(x.checked_div(y), a.checked_div(b).map(i));
                assert_eq!(x.checked_rem(y), a.checked_rem(b).map(i));
                assert_eq!(x.checked_div_euclid(y), a.checked_div_euclid(b).map(i));
                assert_eq!(x.checked_rem_euclid(y), a.checked_rem_euclid(b).map(i));
                if b == 0 {
                    assert_eq!(x.div_rem(y), Err(DivError::ZeroDiv));
                    assert_eq!(x.div_rem_euclid(y), Err(DivError::ZeroDiv));
                    continue;
                }
                assert_eq!(x.overflowing_div(y), flagged(i)(a.overflowing_div(b)));
                assert_eq!(x.overflowing_rem(y), flagged(i)(a.overflowing_rem(b)));
                assert_eq!(x.overflowing_div_euclid(y), flagged(i)(a.overflowing_div_euclid(b)));
                assert_eq!(x.overflowing_rem_euclid(y), flagged(i)(a.overflowing_rem_euclid(b)));
                assert_eq!(x.wrapping_div(y), i(a.wrapping_div(b)));
                assert_eq!(x.wrapping_rem(y), i(a.wrapping_rem(b)));
                assert_eq!(x.wrapping_div_euclid(y), i(a.wrapping_div_euclid(b)));
                assert_eq!(x.wrapping_rem_euclid(y), i(a.wrapping_rem_euclid(b)));
                assert_eq!(x.saturating_div(y), i(a.saturating_div(b)));
                if a == i128::MIN && b == -1 {
                    assert_eq!(x.div_rem(y), Err(DivError::Overflow));
                    assert_eq!(x.div_rem_euclid(y), Err(DivError::Overflow));
                    continue;
                }
                assert_eq!(x.div_rem(y), Ok((i(a / b), i(a % b))));
                assert_eq!(x / y, i(a / b));
                assert_eq!(x % y, i(a % b));
                assert_eq!(x.div_euclid(y), i(a.div_euclid(b)));
                assert_eq!(x.rem_euclid(y), i(a.rem_euclid(b)));
            }
        }
    }

    #[test]
    fn single_limb_matches_u64() {
        let mut rng = Rng(42);
        for _ in 0..10_000 {
            let (a, b) = (rng.limb(), rng.limb());
            let (x, y) = (BigInt::<1, false>::from(a), BigInt::<1, false>::from(b));
            assert_eq!(x.overflowing_add(y), flagged(BigInt::from)(a.overflowing_add(b)));
            assert_eq!(x.overflowing_sub(y), flagged(BigInt::from)(a.overflowing_sub(b)));
            assert_eq!(x.overflowing_mul(y), flagged(BigInt::from)(a.overflowing_mul(b)));
            assert_eq!(x.checked_div(y), a.checked_div(b).map(BigInt::from));
            assert_eq!(x.checked_rem(y), a.checked_rem(b).map(BigInt::from));
            assert_eq!(x.to_string(), a.to_string());
            assert_eq!(format!("{x:#o}"), format!("{a:#o}"));
            let (x, y) = (BigInt::<1, true>::from(a as i64), BigInt::<1, true>::from(b as i64));
            let (a, b) = (a as i64, b as i64);
            assert_eq!(x.overflowing_mul(y), flagged(BigInt::from)(a.overflowing_mul(b)));
            assert_eq!(x.checked_div(y), a.checked_div(b).map(BigInt::from));
            assert_eq!(x.checked_rem_euclid(y), a.checked_rem_euclid(b).map(BigInt::from));
            assert_eq!(x.to_string(), a.to_string());
            assert_eq!(x.to_string().parse(), Ok(x));
        }
    }

    /// Schoolbook multiplication producing all `2 * N` limbs of the product.
    fn mul_reference<const N: usize>(lhs: &[u64; N], rhs: &[u64; N]) -> ([u64; N], bool) {
        let mut product = vec![0u64; 2 * N];
        for i in 0..N {
            let mut carry = 0;
            for j in 0..N {
                let acc =
                    u128::from(lhs[i]) * u128::from(rhs[j]) + u128::from(product[i + j]) + carry;
                product[i + j] = acc as u64;
                carry = acc >> 64;
            }
            product[i + N] = carry as u64;
        }
        (array::from_fn(|i| product[i]), product[N..].iter().any(|&limb| limb != 0))
    }

    /// Binary long division, one bit of the quotient at a time.
    fn div_rem_reference<const N: usize>(num: &[u64; N], den: &[u64; N]) -> ([u64; N], [u64; N]) {
        let (mut quot, mut rem) = ([0; N], [0; N]);
        for bit in (0..64 * N).rev() {
            let carry = rem[N - 1] >> 63 == 1;
            rem = limbs::shl(&rem, 1);
            rem[0] |= num[bit / 64] >> (bit % 64) & 1;
            if carry || rem.iter().rev().ge(den.iter().rev()) {
                limbs::sub(&mut rem, den);
                quot[bit / 64] |= 1 << (bit % 64);
            }
        }
        (quot, rem)
    }

    fn check_limbs<const N: usize>(rounds: usize) {
        let mut rng = Rng(0xDEAD_BEEF ^ N as u64);
        for _ in 0..rounds {
            let (a, b): ([u64; N], [u64; N]) = (rng.limbs(), rng.limbs());
            assert_eq!(limbs::mul(&a, &b), mul_reference(&a, &b), "{a:x?} * {b:x?}");
            if b == [0; N] {
                continue;
            }
            let (quot, rem) = limbs::div_rem(&a, &b);
            assert_eq!((quot, rem), div_rem_reference(&a, &b), "{a:x?} / {b:x?}");
            // num == quot * den + rem
            let (mut num, overflow) = limbs::mul(&quot, &b);
            assert!(!overflow && !limbs::add(&mut num, &rem));
            assert_eq!(num, a);
        }
    }

    #[test]
    fn div_wide_matches_u128() {
        const HIGH: u64 = 1 << 63;
        // The extreme operands, followed by the ones which need the second
        // correction of the quotient while the remainder is equal to the
        // divisor, which random numbers practically never reach.
        let corner_cases = [
            (0, 0, HIGH),
            (HIGH - 1, u64::MAX, HIGH),
            (u64::MAX - 1, u64::MAX, u64::MAX),
            (0x6000_0000_0000_0000, u64::MAX - 1, HIGH + 2),
            (0x4000_0000_0000_0000, u64::MAX - 2, HIGH + 3),
            (0x5555_5555_5555_5555, u64::MAX - 7, HIGH + 3),
        ];
        let mut rng = Rng(0xD1F1_D1F1);
        let random = (0..200_000).map(|_| {
            // The divisor must have the top bit set and exceed the high limb.
            let den = rng.limb() | HIGH;
            (rng.limb() % den, rng.limb(), den)
        });
        for (hi, lo, den) in corner_cases.into_iter().chain(random) {
            let (num, wide) = (u128::from(hi) << 64 | u128::from(lo), u128::from(den));
            let expected = ((num / wide) as u64, (num % wide) as u64);
            assert_eq!(limbs::div_wide(hi, lo, den), expected, "{num:#x} / {den:#x}");
            assert_eq!(limbs::div_halves(hi, lo, den), expected, "{num:#x} / {den:#x}");
        }
    }

    #[test]
    fn limbs_match_reference() {
        check_limbs::<1>(5_000);
        check_limbs::<2>(5_000);
        check_limbs::<3>(5_000);
        check_limbs::<4>(20_000);
        check_limbs::<5>(5_000);
        check_limbs::<8>(5_000);
        check_limbs::<16>(2_000);
    }

    #[test]
    fn division_corner_cases() {
        const HIGH: u64 = 1 << 63;
        // The quotient digit estimate is one too large and the divisor must be
        // added back (step D6 of the algorithm, which random numbers
        // practically never reach).
        let cases: [([u64; 4], [u64; 4]); 8] = [
            ([3, 0, HIGH, 0], [1, 0, HIGH >> 2, 0]),
            ([0, u64::MAX - 1, HIGH, 0], [u64::MAX, HIGH, 0, 0]),
            ([0, 0, HIGH, HIGH - 1], [1, 0, HIGH, 0]),
            ([0, 0, 0, HIGH], [1, 0, HIGH, 0]),
            ([u64::MAX, 0, u64::MAX, HIGH], [u64::MAX, 0, HIGH, 0]),
            ([0, 0, 1, 0], [u64::MAX, 0, 0, 0]),
            ([u64::MAX; 4], [u64::MAX, u64::MAX, 0, 0]),
            ([u64::MAX; 4], [1, 0, 0, 0]),
        ];
        for (num, den) in cases {
            assert_eq!(
                limbs::div_rem(&num, &den),
                div_rem_reference(&num, &den),
                "{num:x?} / {den:x?}"
            );
        }
        assert_eq!(limbs::div_rem(&cases[0].0, &cases[0].1), ([3, 0, 0, 0], [0, 0, HIGH >> 2, 0]));
    }

    #[test]
    fn signed_division_properties() {
        let mut rng = Rng(7);
        for _ in 0..5_000 {
            let (a, b) = (i256::from_inner(rng.limbs()), i256::from_inner(rng.limbs()));
            if b.is_zero() || (a == i256::MIN && b == -i256::ONE) {
                continue;
            }
            let (quot, rem) = a.div_rem(b).unwrap();
            assert_eq!(quot * b + rem, a);
            assert!(rem.unsigned_abs() < b.unsigned_abs());
            assert!(rem.is_zero() || rem.is_negative() == a.is_negative());
            assert!(quot.is_zero() || quot.is_negative() == (a.is_negative() != b.is_negative()));

            let (quot, rem) = a.div_rem_euclid(b).unwrap();
            assert_eq!(quot.wrapping_mul(b).wrapping_add(rem), a);
            assert!(!rem.is_negative() && rem.unsigned_abs() < b.unsigned_abs());
        }
        assert_eq!(i256::MIN.div_rem(-1), Err(DivError::Overflow));
        assert_eq!(i256::MIN.div_rem(1), Ok((i256::MIN, i256::ZERO)));
        assert_eq!(i256::MIN.div_rem(i256::MIN), Ok((i256::ONE, i256::ZERO)));
        assert_eq!(i256::MAX.div_rem(i256::MIN), Ok((i256::ZERO, i256::MAX)));
        assert_eq!(i256::MIN.div_rem(i256::MAX), Ok((-i256::ONE, -i256::ONE)));
        assert_eq!(i256::MIN / 2, i256::MIN >> 1);
        assert_eq!(i256::from(-1) / 5, i256::ZERO);
        assert_eq!(i256::from(-7) % 100, i256::from(-7));
        assert_eq!(i256::from(-7).div_euclid(i256::MIN), i256::ONE);
        assert_eq!(i256::from(-7).rem_euclid(i256::MIN), i256::MAX - 6);
        assert_eq!(i512::MAX.checked_div(-1), Some(i512::MIN + 1));
        assert_eq!(i1024::MIN.overflowing_div_euclid(-1), (i1024::MIN, true));
        assert_eq!(i1024::MIN.overflowing_rem_euclid(-1), (i1024::ZERO, true));
        assert_eq!(i256::MIN.saturating_div(-1), i256::MAX);
    }

    #[test]
    fn constants() {
        assert_eq!((u256::BITS, u512::BITS, u1024::BITS), (256, 512, 1024));
        assert_eq!((i256::BYTES, i512::BYTES, i1024::BYTES), (32, 64, 128));
        assert_eq!(u256::MIN.into_inner(), [0; 4]);
        assert_eq!(u1024::MAX.into_inner(), [u64::MAX; 16]);
        assert_eq!(i256::MIN.into_inner(), [0, 0, 0, 1 << 63]);
        assert_eq!(i256::MAX.into_inner(), [u64::MAX, u64::MAX, u64::MAX, u64::MAX >> 1]);
        assert_eq!(u512::ZERO, u512::default());
        assert_eq!(i512::ONE.as_inner(), &[1, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(i256::MIN, i256::ONE << 255);
        assert_eq!(i256::MAX, !i256::MIN);
        assert_eq!(i256::MIN.wrapping_sub(1), i256::MAX);
        assert_eq!(u256::MAX.wrapping_add(1u8), u256::MIN);
    }

    #[test]
    fn arithmetic_vectors() {
        let init = u256::from(0xDEADBEEFDEADBEEFu64);
        let add = init + init;
        assert_eq!(add, u256::from_inner([0xBD5B7DDFBD5B7DDE, 1, 0, 0]));
        let shl = add << 88;
        assert_eq!(shl, u256::from_inner([0, 0xDFBD5B7DDE000000, 0x1BD5B7D, 0]));
        let shr = shl >> 40;
        assert_eq!(shr, u256::from_inner([0x7DDE000000000000, 0x0001BD5B7DDFBD5B, 0, 0]));
        let mut incr = shr;
        incr += 1u32;
        assert_eq!(incr, u256::from_inner([0x7DDE000000000001, 0x0001BD5B7DDFBD5B, 0, 0]));
        let sub = incr - init;
        assert_eq!(sub, u256::from_inner([0x9F30411021524112, 0x0001BD5B7DDFBD5A, 0, 0]));
        let mult = sub * 300u32;
        assert_eq!(mult, u256::from_inner([0x8C8C3EE70C644118, 0x0209E7378231E632, 0, 0]));
        assert_eq!(mult / 300u64, sub);
        assert_eq!((mult * 39842u64 + 9054u64) % 39842u64, u256::from(9054u64));

        let mut pow = init;
        for _ in 0..6 {
            pow *= 0xFFFFFFFFu32;
        }
        let expected =
            [0xA69B4555DEADBEEF, 0xA69B455CD41BB662, 0xD41BB662A69B4550, 0xDEADBEEAA69B455C];
        assert_eq!(pow, u256::from_inner(expected));
        let square = init * init;
        assert_eq!(square, u256::from_inner([0x048D1354216DA321, 0xC1B1CD13A4D13D46, 0, 0]));
        let expected =
            [0xF4E166AAD40D0A41, 0xF5CF7F3618C2C886, 0x4AFCFF6F0375C608, 0x928D92B4D7F5DF33];
        assert_eq!(square * square, u256::from_inner(expected));

        assert_eq!(u256::MAX.overflowing_add(u256::MAX), (u256::MAX - 1u8, true));
        assert_eq!(i256::MAX.overflowing_mul(2), (i256::from(-2), true));
        assert_eq!(i256::MIN.overflowing_mul(2), (i256::ZERO, true));
        assert_eq!(i256::MAX.overflowing_mul(i256::MAX), (i256::ONE, true));
        assert_eq!(i256::MIN.overflowing_mul(1), (i256::MIN, false));
        assert_eq!(i256::MIN.overflowing_mul(-1), (i256::MIN, true));
        assert_eq!(i256::from(-4) >> 2, i256::from(-1));
        assert_eq!(i256::MIN >> 255, i256::from(-1));
        assert_eq!(u256::MAX >> 255, u256::ONE);
        assert_eq!(u1024::ONE << 1023 >> 1023, u1024::ONE);
    }

    #[test]
    fn conversions() {
        assert_eq!(u256::from(true), u256::ONE);
        assert_eq!(i256::from(false), i256::ZERO);
        assert_eq!(u256::from(u128::MAX).into_inner(), [u64::MAX, u64::MAX, 0, 0]);
        assert_eq!(i256::from(u128::MAX).into_inner(), [u64::MAX, u64::MAX, 0, 0]);
        assert_eq!(i256::from(i128::MIN).into_inner(), [0, 1 << 63, u64::MAX, u64::MAX]);
        assert_eq!(i256::from(-1i8), !i256::ZERO);
        assert_eq!(i1024::from(-2i64) + 1u64, i1024::from(-1i16));
        assert_eq!(i256::from(-5).unsigned_abs(), u256::from(5u8));
        assert_eq!(i256::MIN.unsigned_abs(), u256::ONE << 255);
        assert_eq!(u256::MAX.cast_signed(), i256::from(-1));
        assert_eq!(i256::from(-1).cast_unsigned(), u256::MAX);

        assert_eq!(u512::from(u256::MAX).into_inner(), [
            u64::MAX,
            u64::MAX,
            u64::MAX,
            u64::MAX,
            0,
            0,
            0,
            0
        ]);
        assert_eq!(u1024::from(u256::from(30u8)), u1024::from(30u8));
        assert_eq!(u1024::from(u512::MAX) >> 448, u1024::from(u64::MAX));
        assert_eq!(i512::from(i256::from(-30)), i512::from(-30));
        assert_eq!(i1024::from(i256::MIN), -(i1024::ONE << 255));
        assert_eq!(i1024::from(i512::MAX), (i1024::ONE << 511) - 1);
    }

    #[test]
    fn bytes() {
        let be = [
            0x1b, 0xad, 0xca, 0xfe, 0xde, 0xad, 0xbe, 0xef, 0xde, 0xaf, 0xba, 0xbe, 0x2b, 0xed,
            0xfe, 0xed, 0xba, 0xad, 0xf0, 0x0d, 0xde, 0xfa, 0xce, 0xda, 0x11, 0xfe, 0xd2, 0xba,
            0xd1, 0xc0, 0xff, 0xe0,
        ];
        let mut le = be;
        le.reverse();
        let val = u256::from_inner([
            0x11fed2bad1c0ffe0,
            0xbaadf00ddefaceda,
            0xdeafbabe2bedfeed,
            0x1badcafedeadbeef,
        ]);
        assert_eq!(u256::from_be_bytes(be), val);
        assert_eq!(u256::from_le_bytes(le), val);
        assert_eq!(u256::from_be_slice(&be), Ok(val));
        assert_eq!(u256::from_le_slice(&le), Ok(val));
        assert_eq!(val.to_be_bytes(), be);
        assert_eq!(val.to_le_bytes(), le);
        assert_eq!(i256::from_be_bytes(be).cast_unsigned(), val);
        assert_eq!(i256::from(-2).to_be_bytes()[24..], [
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfe
        ]);
        assert_eq!(u1024::from_le_bytes(u1024::MAX.to_le_bytes()), u1024::MAX);
        assert_eq!(i512::from_be_bytes(i512::MIN.to_be_bytes()), i512::MIN);
        assert_eq!(
            U128::from_le_slice(&le[..16]),
            Ok(u(u128::from_le_bytes(*le.first_chunk().unwrap())))
        );
        assert_eq!(
            U128::from_be_slice(&be[..16]),
            Ok(u(u128::from_be_bytes(*be.first_chunk().unwrap())))
        );

        for len in [0, 1, 8, 31, 33, 64] {
            let err = ParseLengthError {
                actual: len,
                expected: 32,
            };
            assert_eq!(u256::from_be_slice(&[0; 64][..len]), Err(err));
            assert_eq!(i256::from_le_slice(&[0; 64][..len]), Err(err));
        }
    }

    #[test]
    fn formatting() {
        let val = u256::from_inner([0, 0xAAAAAAAABBBBBBBB, 0x0000000111122222, 0]);
        assert_eq!(format!("{val:X}"), "111122222AAAAAAAABBBBBBBB0000000000000000");
        assert_eq!(format!("{val:#x}"), "0x111122222aaaaaaaabbbbbbbb0000000000000000");
        assert_eq!(
            format!("{:#066x}", u256::from(0xDEADBEEFu32)),
            format!("0x{:0>64}", "deadbeef")
        );
        assert_eq!(format!("{:x}", i256::from(-1)), "f".repeat(64));
        assert_eq!(format!("{:o}", u256::MAX), format!("17{}", "7".repeat(84)));
        assert_eq!(format!("{:b}", u1024::MAX), "1".repeat(1024));
        assert_eq!(format!("{:b}", i1024::MIN), format!("1{}", "0".repeat(1023)));
        assert_eq!(format!("{:w^#7X}", u256::ONE), "ww0x1ww");
        assert_eq!(
            format!("{:5}|{:<5}|{:05}", i256::from(-1), i256::ONE, i256::from(-1)),
            "   -1|1    |-0001"
        );

        let max = "115792089237316195423570985008687907853269984665640564039457584007913129639935";
        assert_eq!(u256::MAX.to_string(), max);
        assert_eq!(format!("{:?}", u256::MAX), max);
        let min = "-57896044618658097711785492504343953926634992332820282019728792003956564819968";
        assert_eq!(i256::MIN.to_string(), min);
        assert_eq!(i256::MAX.to_string(), min[1..77].to_owned() + "7");
        assert_eq!(u256::from(10u64.pow(19)).to_string(), "10000000000000000000");
        assert_eq!(
            (u256::from(10u64.pow(19)) * 10u64.pow(19)).to_string(),
            format!("1{}", "0".repeat(38))
        );
        assert_eq!(u1024::MAX.to_string().len(), 309);
        assert_eq!(i1024::MIN.to_string().len(), 309);
    }

    #[test]
    fn parsing() {
        use IntErrorKind::*;

        let max = "115792089237316195423570985008687907853269984665640564039457584007913129639935";
        assert_eq!(max.parse(), Ok(u256::MAX));
        assert_eq!(u256::from_str_radix(&"f".repeat(64), 16), Ok(u256::MAX));
        assert_eq!(u256::from_str_radix(&"F".repeat(65), 16), Err(ParseIntError(PosOverflow)));
        assert_eq!((max[..77].to_owned() + "6").parse::<u256>(), Err(ParseIntError(PosOverflow)));
        assert_eq!(format!("{max}0").parse::<u256>(), Err(ParseIntError(PosOverflow)));
        assert_eq!(format!("-{max}").parse(), Ok(-u512::from(u256::MAX).cast_signed()));
        for val in [i256::MIN, i256::MIN + 1, -i256::ONE, i256::ZERO, i256::ONE, i256::MAX] {
            assert_eq!(val.to_string().parse(), Ok(val));
            assert_eq!(i256::from_str_radix(&format!("{val:b}"), 2).is_ok(), !val.is_negative());
        }
        let min = "57896044618658097711785492504343953926634992332820282019728792003956564819968";
        assert_eq!(min.parse::<i256>(), Err(ParseIntError(PosOverflow)));
        assert_eq!(format!("-{min}").parse(), Ok(i256::MIN));
        assert_eq!(format!("-{}9", &min[..76]).parse::<i256>(), Err(ParseIntError(NegOverflow)));
        assert_eq!(u1024::from_str_radix("zz", 36), Ok(u1024::from(1295u16)));
        assert_eq!(i256::from_str_radix("-Zz", 36), Ok(i256::from(-1295)));

        for (src, kind) in [
            ("", Empty),
            ("+", InvalidDigit),
            ("-", InvalidDigit),
            ("--1", InvalidDigit),
            ("+-1", InvalidDigit),
            ("1 ", InvalidDigit),
            (" 1", InvalidDigit),
            ("1_000", InvalidDigit),
            ("0x10", InvalidDigit),
            ("12a", InvalidDigit),
            ("١", InvalidDigit),
            ("1é", InvalidDigit),
        ] {
            assert_eq!(src.parse::<i256>(), Err(ParseIntError(kind)), "{src:?}");
            assert_eq!(src.parse::<u256>(), Err(ParseIntError(kind)), "{src:?}");
        }
        assert_eq!("-1".parse::<u256>(), Err(ParseIntError(InvalidDigit)));
        assert_eq!("-0".parse::<u256>(), Err(ParseIntError(InvalidDigit)));
        assert_eq!("-0".parse(), Ok(i256::ZERO));
        assert_eq!("+0".parse(), Ok(u256::ZERO));
        assert_eq!("+00012".parse(), Ok(u256::from(12u8)));
        assert_eq!(i256::from_str_radix("2", 2), Err(ParseIntError(InvalidDigit)));
    }

    #[test]
    fn bits_required() {
        assert_eq!(u256::ZERO.bits_required(), 0);
        assert_eq!(u256::from(255u8).bits_required(), 8);
        assert_eq!(u256::from(256u16).bits_required(), 9);
        assert_eq!((u256::from(70000u32) << 200).bits_required(), 217);
        assert_eq!(u256::from(70000u32).unbounded_shl(300).bits_required(), 0);
        assert_eq!(u256::MAX.bits_required(), 256);
        assert_eq!(i256::ZERO.bits_required(), 1);
        assert_eq!(i256::from(255).bits_required(), 9);
        assert_eq!(i256::from(-128).bits_required(), 8);
        assert_eq!(i256::from(-129).bits_required(), 9);
        assert_eq!(i256::from(-1).bits_required(), 1);
        assert_eq!(i256::MIN.bits_required(), 256);
        assert_eq!(i256::MAX.bits_required(), 256);
        assert_eq!(u512::ZERO.leading_zeros(), 512);
        assert_eq!(u512::ZERO.trailing_zeros(), 512);
        assert_eq!(u512::MAX.leading_ones(), 512);
        assert_eq!(u512::MAX.trailing_ones(), 512);
        assert_eq!(i256::from(-2).leading_ones(), 255);
        assert!(u256::from(10u8).bit(1) && u256::from(10u8).bit(3) && !u256::from(10u8).bit(2));
        assert!(i256::MIN.bit(255) && !i256::MAX.bit(255));
    }

    #[test]
    fn ordering() {
        let mut vals =
            [i256::ONE, i256::MIN, i256::from(-1), i256::MAX, i256::ZERO, i256::from(i64::MIN)];
        vals.sort();
        assert_eq!(vals, [
            i256::MIN,
            i256::from(i64::MIN),
            i256::from(-1),
            i256::ZERO,
            i256::ONE,
            i256::MAX
        ]);
        let small = u256::from_inner([10, 0, 0, 0]);
        let big = u256::from_inner([0x8C8C3EE70C644118, 0x0209E7378231E632, 0, 0]);
        let bigger = u256::from_inner([0x9C8C3EE70C644118, 0x0209E7378231E632, 0, 0]);
        let biggest = u256::from_inner([0x5C8C3EE70C644118, 0x0209E7378231E632, 0, 1]);
        assert!(small < big && big < bigger && bigger < biggest && biggest <= biggest);
        assert!(u256::MAX > biggest && u256::MAX.cast_signed() < small.cast_signed());
    }

    macro_rules! panics {
        ($($name:ident: $msg:literal => $expr:expr;)+) => {$(
            #[test]
            #[should_panic(expected = $msg)]
            fn $name() { let _ = $expr; }
        )+};
    }

    panics! {
        add_overflow: "attempt to add with overflow" => u256::MAX + 1u8;
        add_signed_overflow: "attempt to add with overflow" => i256::MIN + i256::MIN;
        sub_overflow: "attempt to subtract with overflow" => u256::ZERO - 1u8;
        sub_signed_overflow: "attempt to subtract with overflow" => i256::ZERO - i256::MIN;
        mul_overflow: "attempt to multiply with overflow" => u256::MAX * 2u8;
        mul_signed_overflow: "attempt to multiply with overflow" => i256::MIN * -1;
        div_by_zero: "attempt to perform division by zero" => u256::ONE / 0u8;
        div_overflow: "attempt to perform division with overflow" => i256::MIN / -1;
        rem_by_zero: "attempt to perform division by zero" => i256::ONE % 0;
        rem_overflow: "attempt to perform division with overflow" => i256::MIN % -i256::ONE;
        overflowing_div_by_zero: "attempt to perform division by zero" =>
            i256::MAX.overflowing_div(0);
        wrapping_rem_by_zero: "attempt to perform division by zero" => u256::MAX.wrapping_rem(0u8);
        saturating_div_by_zero: "attempt to perform division by zero" =>
            u256::MAX.saturating_div(0u8);
        div_euclid_by_zero: "attempt to perform division by zero" => i256::MAX.div_euclid(0);
        rem_euclid_overflow: "attempt to perform division with overflow" =>
            i256::MIN.rem_euclid(-1);
        neg_overflow: "attempt to negate with overflow" => -i256::MIN;
        abs_overflow: "attempt to negate with overflow" => i256::MIN.abs();
        shl_overflow: "attempt to shift left with overflow" => u256::ONE << 256;
        shr_overflow: "attempt to shift right with overflow" => i256::ONE >> 256;
        bit_out_of_range: "index out of bounds" => u256::ONE.bit(256);
        radix_too_small: "radix must lie in the range" => u256::from_str_radix("0", 1);
        radix_too_large: "radix must lie in the range" => u256::from_str_radix("0", 37);
        limbs_div_by_zero: "attempt to divide by zero" => limbs::div_rem(&[1, 2], &[0, 0]);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_human_readable() {
        let check = |val: u256, hex: &str| {
            let json = format!("\"{hex}\"");
            assert_eq!(serde_json::to_string(&val).unwrap(), json);
            assert_eq!(serde_json::from_str::<u256>(&json).unwrap(), val);
            assert_eq!(serde_json::to_string(&val.cast_signed()).unwrap(), json);
            assert_eq!(serde_json::from_str::<i256>(&json).unwrap(), val.cast_signed());
        };
        check(u256::ZERO, &"0".repeat(64));
        check(u256::from(0xDEADBEEFu32), &format!("{:0>64}", "deadbeef"));
        check(
            u256::from_inner([0xaa11, 0xbb22, 0xcc33, 0xdd44]),
            "000000000000dd44000000000000cc33000000000000bb22000000000000aa11",
        );
        check(u256::MAX, &"f".repeat(64));
        check(
            u256::from_inner([
                0xA69B4555DEADBEEF,
                0xA69B455CD41BB662,
                0xD41BB662A69B4550,
                0xDEADBEEAA69B455C,
            ]),
            "deadbeeaa69b455cd41bb662a69b4550a69b455cd41bb662a69b4555deadbeef",
        );
        assert_eq!(
            serde_json::from_str::<u256>(&format!("\"{}\"", "F".repeat(64))).unwrap(),
            u256::MAX
        );
        assert_eq!(
            serde_json::to_string(&i512::from(-2)).unwrap(),
            format!("\"{}e\"", "f".repeat(127))
        );

        for invalid in [
            format!("{}g", "f".repeat(63)),
            "f".repeat(62),
            "f".repeat(63),
            "f".repeat(66),
            format!("+{}", "f".repeat(63)),
            format!("{}é", "f".repeat(62)),
            String::new(),
        ] {
            assert!(serde_json::from_str::<u256>(&format!("\"{invalid}\"")).is_err(), "{invalid}");
        }
        assert!(serde_json::from_str::<u256>("12").is_err());
    }
}
