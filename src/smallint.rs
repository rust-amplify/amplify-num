// SPDX-License-Identifier: Apache-2.0
//
// Written in 2020-2026 by Dr. Maxim Orlovsky <orlovsky@ubideco.org>
//
// Copyright 2024-2026 Laboratories for Ubiquitous and Deterministic Computing,
// Institute for Distributed and Cognitive Computing (InDCS), Switzerland.
// All rights reserved.
//
// Copyright (C) 2020-2026 Dr Maxim Orlovsky.
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

use core::fmt::{self, Binary, Debug, Display, Formatter, LowerHex, Octal, UpperHex};
use core::num::IntErrorKind;
use core::ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Div, DivAssign,
    Mul, MulAssign, Not, Rem, RemAssign, Shl, ShlAssign, Shr, ShrAssign, Sub, SubAssign,
};
use core::str::FromStr;

use crate::error::{DivError, OverflowError, ParseIntError};

/// Defines unsigned integers of a bit size which is smaller than the size of
/// the primitive integer storing them.
macro_rules! smallint {
    ($($name:ident: $bits:literal bits in $inner:ident as $to_inner:ident;)+) => {$(
        #[doc = concat!($bits, "-bit unsigned integer in the range `0..2^", $bits, "`.")]
        ///
        /// The arithmetic operators panic in all build profiles if their result does not
        /// fit the type; the `checked_*`, `overflowing_*`, `saturating_*` and `wrapping_*`
        /// methods give explicit control over the overflow handling.
        #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
        #[expect(non_camel_case_types)]
        pub struct $name($inner);

        impl $name {
            /// Size of the integer in bits.
            pub const BITS: u32 = $bits;

            /// The smallest value of the type.
            pub const MIN: Self = Self(0);

            /// The largest value of the type.
            pub const MAX: Self = Self((1 << $bits) - 1);

            /// Zero.
            pub const ZERO: Self = Self(0);

            /// One.
            pub const ONE: Self = Self(1);

            /// Creates the integer from a value of the underlying primitive type.
            ///
            /// # Panics
            ///
            /// If the value exceeds [`Self::MAX`].
            pub const fn with(value: $inner) -> Self {
                assert!(value <= Self::MAX.0, "provided value exceeds Self::MAX");
                Self(value)
            }

            /// Returns the value as the underlying primitive type.
            pub const fn $to_inner(self) -> $inner { self.0 }

            smallint_arithmetics! { $inner:
                "addition": checked_add, overflowing_add, saturating_add, wrapping_add;
                "subtraction": checked_sub, overflowing_sub, saturating_sub, wrapping_sub;
                "multiplication": checked_mul, overflowing_mul, saturating_mul, wrapping_mul;
            }

            /// Checked division. Returns `None` if `rhs` is zero.
            pub fn checked_div(self, rhs: impl Into<$inner>) -> Option<Self> {
                self.0.checked_div(rhs.into()).map(Self)
            }

            /// Checked remainder. Returns `None` if `rhs` is zero.
            pub fn checked_rem(self, rhs: impl Into<$inner>) -> Option<Self> {
                self.0.checked_rem(rhs.into()).map(Self)
            }

            /// Calculates the quotient and the remainder of the division of `self` by `rhs`.
            pub fn div_rem(self, rhs: impl Into<$inner>) -> Result<(Self, Self), DivError> {
                let rhs = rhs.into();
                match (self.checked_div(rhs), self.checked_rem(rhs)) {
                    (Some(quot), Some(rem)) => Ok((quot, rem)),
                    _ => Err(DivError::ZeroDiv),
                }
            }
        }

        impl TryFrom<$inner> for $name {
            type Error = OverflowError<$inner>;

            fn try_from(value: $inner) -> Result<Self, Self::Error> {
                if value > Self::MAX.0 {
                    return Err(OverflowError { max: Self::MAX.0, value });
                }
                Ok(Self(value))
            }
        }

        impl FromStr for $name {
            type Err = ParseIntError;

            fn from_str(s: &str) -> Result<Self, ParseIntError> {
                Self::try_from(s.parse::<$inner>()?)
                    .map_err(|_| ParseIntError(IntErrorKind::PosOverflow))
            }
        }

        smallint_operators! { $name($inner):
            Add add AddAssign add_assign => $inner::checked_add, "attempt to add with overflow";
            Sub sub SubAssign sub_assign =>
                $inner::checked_sub, "attempt to subtract with overflow";
            Mul mul MulAssign mul_assign =>
                $inner::checked_mul, "attempt to multiply with overflow";
            Div div DivAssign div_assign => $inner::checked_div, "attempt to divide by zero";
            Rem rem RemAssign rem_assign => $inner::checked_rem,
                "attempt to calculate the remainder with a divisor of zero";
            BitAnd bitand BitAndAssign bitand_assign =>
                |lhs, rhs| Some(lhs & rhs), "attempt to bitand with overflow";
            BitOr bitor BitOrAssign bitor_assign =>
                |lhs, rhs| Some(lhs | rhs), "attempt to bitor with overflow";
            BitXor bitxor BitXorAssign bitxor_assign =>
                |lhs, rhs| Some(lhs ^ rhs), "attempt to bitxor with overflow";
        }

        impl Shl<u32> for $name {
            type Output = Self;

            /// Shifts the bits left, discarding those which do not fit the type.
            ///
            /// # Panics
            ///
            /// If `rhs` is not less than [`Self::BITS`].
            fn shl(self, rhs: u32) -> Self {
                assert!(rhs < Self::BITS, "attempt to shift left with overflow");
                Self((self.0 << rhs) & Self::MAX.0)
            }
        }

        impl ShlAssign<u32> for $name {
            fn shl_assign(&mut self, rhs: u32) { *self = *self << rhs }
        }

        impl Shr<u32> for $name {
            type Output = Self;

            /// Shifts the bits right.
            ///
            /// # Panics
            ///
            /// If `rhs` is not less than [`Self::BITS`].
            fn shr(self, rhs: u32) -> Self {
                assert!(rhs < Self::BITS, "attempt to shift right with overflow");
                Self(self.0 >> rhs)
            }
        }

        impl ShrAssign<u32> for $name {
            fn shr_assign(&mut self, rhs: u32) { *self = *self >> rhs }
        }

        impl Not for $name {
            type Output = Self;

            fn not(self) -> Self { Self(!self.0 & Self::MAX.0) }
        }

        smallint_fmt!($name: Debug Display Binary Octal LowerHex UpperHex);

        #[cfg(feature = "serde")]
        impl serde::Serialize for $name {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                serde::Serialize::serialize(&self.0, serializer)
            }
        }

        #[cfg(feature = "serde")]
        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let value: $inner = serde::Deserialize::deserialize(deserializer)?;
                Self::try_from(value).map_err(serde::de::Error::custom)
            }
        }
    )+};
}

/// Implements the flavors of an arithmetic operation, each on top of the
/// eponymous method of the underlying primitive type. Since the bit size of
/// that type is larger, wrapping at it and then at `2^BITS` is the same as
/// wrapping at `2^BITS` right away.
macro_rules! smallint_arithmetics {
    ($inner:ident: $($op:literal: $checked:ident, $overflowing:ident, $saturating:ident,
     $wrapping:ident;)+) => {$(
        #[doc = concat!("Checked ", $op, ". Returns `None` if an overflow has happened.")]
        pub fn $checked(self, rhs: impl Into<$inner>) -> Option<Self> {
            self.0.$checked(rhs.into()).and_then(|val| Self::try_from(val).ok())
        }

        #[doc = concat!("Overflowing ", $op, ". Returns the wrapped result and whether an ")]
        #[doc = "overflow has happened."]
        pub fn $overflowing(self, rhs: impl Into<$inner>) -> (Self, bool) {
            let (val, overflow) = self.0.$overflowing(rhs.into());
            (Self(val & Self::MAX.0), overflow || val > Self::MAX.0)
        }

        #[doc = concat!("Saturating ", $op, ". Returns the closest bound of the type if an ")]
        #[doc = "overflow has happened."]
        pub fn $saturating(self, rhs: impl Into<$inner>) -> Self {
            Self(self.0.$saturating(rhs.into()).min(Self::MAX.0))
        }

        #[doc = concat!("Wrapping (modular) ", $op, ".")]
        pub fn $wrapping(self, rhs: impl Into<$inner>) -> Self {
            Self(self.0.$wrapping(rhs.into()) & Self::MAX.0)
        }
    )+};
}

/// Implements a binary operator, together with its assigning form, for any
/// right-hand operand convertible into the underlying primitive type. The
/// operator panics with the given message if the checked operation fails or its
/// result does not fit the type.
macro_rules! smallint_operators {
    ($name:ident($inner:ident): $($op:ident $fn:ident $op_assign:ident $fn_assign:ident =>
     $checked:expr, $panic:literal;)+) => {$(
        impl<T: Into<$inner>> $op<T> for $name {
            type Output = Self;

            fn $fn(self, rhs: T) -> Self {
                let checked: fn($inner, $inner) -> Option<$inner> = $checked;
                checked(self.0, rhs.into()).and_then(|val| Self::try_from(val).ok()).expect($panic)
            }
        }

        impl<T: Into<$inner>> $op_assign<T> for $name {
            fn $fn_assign(&mut self, rhs: T) { *self = $op::$fn(*self, rhs) }
        }
    )+};
}

/// Forwards the formatting traits to the underlying primitive type.
macro_rules! smallint_fmt {
    ($name:ident: $($fmt:ident)+) => {$(
        impl $fmt for $name {
            fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result { $fmt::fmt(&self.0, f) }
        }
    )+};
}

smallint! {
    u1: 1 bits in u8 as to_u8;
    u2: 2 bits in u8 as to_u8;
    u3: 3 bits in u8 as to_u8;
    u4: 4 bits in u8 as to_u8;
    u5: 5 bits in u8 as to_u8;
    u6: 6 bits in u8 as to_u8;
    u7: 7 bits in u8 as to_u8;
    u10: 10 bits in u16 as to_u16;
    u12: 12 bits in u16 as to_u16;
    u14: 14 bits in u16 as to_u16;
    u20: 20 bits in u32 as to_u32;
    u24: 24 bits in u32 as to_u32;
    u40: 40 bits in u64 as to_u64;
    u48: 48 bits in u64 as to_u64;
    u56: 56 bits in u64 as to_u64;
}

/// Implements the lossless conversions of each integer of the list into all
/// larger ones.
macro_rules! smallint_widening {
    ($small:ident $($large:ident)*) => {
        $(impl From<$small> for $large {
            fn from(value: $small) -> Self {
                const { assert!($small::BITS < $large::BITS, "lossy conversion") };
                Self(value.0.into())
            }
        })*
        smallint_widening!($($large)*);
    };
    () => {};
}
smallint_widening!(u1 u2 u3 u4 u5 u6 u7 u10 u12 u14 u20 u24 u40 u48 u56);

/// Implements the lossless conversions of the small integers into the primitive
/// ones.
macro_rules! smallint_into {
    ($($name:ident)+ => $prims:tt) => { $(smallint_into!(@ $name => $prims);)+ };
    (@ $name:ident => [$($prim:ident)+]) => {$(
        impl From<$name> for $prim {
            fn from(value: $name) -> Self {
                const { assert!($name::MAX.0 as u128 <= $prim::MAX as u128, "lossy conversion") };
                value.0 as $prim
            }
        }
    )+};
}
smallint_into!(u1 u2 u3 u4 u5 u6 u7 => [u8 u16 u32 u64 u128 usize i8 i16 i32 i64 i128 isize]);
smallint_into!(u10 u12 u14 => [u16 u32 u64 u128 usize i16 i32 i64 i128 isize]);
smallint_into!(u20 u24 => [u32 u64 u128 i32 i64 i128]);
smallint_into!(u40 u48 u56 => [u64 u128 i64 i128]);
#[cfg(not(target_pointer_width = "16"))]
smallint_into!(u20 u24 => [usize isize]);
#[cfg(target_pointer_width = "64")]
smallint_into!(u40 u48 u56 => [usize isize]);

/// Implements the byte conversions for the integers made of a whole number of
/// bytes.
macro_rules! smallint_bytes {
    ($($name:ident: $len:literal bytes in $inner:ident;)+) => {$(
        impl $name {
            const PADDING: usize = {
                assert!(Self::BITS == 8 * $len);
                size_of::<$inner>() - $len
            };

            /// Creates the integer from its byte representation in the little-endian order.
            pub const fn from_le_bytes(bytes: [u8; $len]) -> Self {
                let mut inner = [0; size_of::<$inner>()];
                inner.split_at_mut($len).0.copy_from_slice(&bytes);
                Self($inner::from_le_bytes(inner))
            }

            /// Creates the integer from its byte representation in the big-endian order.
            pub const fn from_be_bytes(bytes: [u8; $len]) -> Self {
                let mut inner = [0; size_of::<$inner>()];
                inner.split_at_mut(Self::PADDING).1.copy_from_slice(&bytes);
                Self($inner::from_be_bytes(inner))
            }

            /// Returns the byte representation of the integer in the little-endian order.
            pub const fn to_le_bytes(self) -> [u8; $len] {
                let mut bytes = [0; $len];
                bytes.copy_from_slice(self.0.to_le_bytes().split_at($len).0);
                bytes
            }

            /// Returns the byte representation of the integer in the big-endian order.
            pub const fn to_be_bytes(self) -> [u8; $len] {
                let mut bytes = [0; $len];
                bytes.copy_from_slice(self.0.to_be_bytes().split_at(Self::PADDING).1);
                bytes
            }
        }
    )+};
}

smallint_bytes! {
    u24: 3 bytes in u32;
    u40: 5 bytes in u64;
    u48: 6 bytes in u64;
    u56: 7 bytes in u64;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Values of the underlying primitive type to test with: all of them if the
    /// type is `u8`, or the corner cases mixed with pseudo-random numbers
    /// otherwise.
    fn samples(max: u64, inner_max: u64) -> Vec<u64> {
        if inner_max == u64::from(u8::MAX) {
            return (0..=inner_max).collect();
        }
        let mut samples = vec![0, 1, 2, 3, 7, max / 2, max / 2 + 1, max - 1, max, max + 1, max + 2];
        samples.extend([2 * max + 1, inner_max / 2, inner_max - 1, inner_max]);
        let mut state = 0x2545_F491_4F6C_DD1D ^ max;
        for _ in 0..30 {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            samples.extend([state & max, state & inner_max, (state >> 32) & max & 0xFFFF]);
        }
        samples
    }

    macro_rules! assert_fmt {
        ($small:expr, $prim:expr => $($spec:literal)+) => {$(
            assert_eq!(format!($spec, $small), format!($spec, $prim), "{} of {}", $spec, $prim);
        )+};
    }

    /// Checks each type against the exact arithmetic on `i128`, which can't
    /// overflow.
    macro_rules! model_tests {
        ($($test:ident: $name:ident($inner:ident) $to_inner:ident;)+) => {$(
            #[test]
            fn $test() {
                const MAX: i128 = $name::MAX.0 as i128;
                let small = |val: i128| $name::with(val as $inner);
                let values = samples(MAX as u64, $inner::MAX.into());

                assert_eq!(MAX, (1 << $name::BITS) - 1);
                assert_eq!(($name::MIN, $name::ZERO, $name::ONE), (small(0), small(0), small(1)));
                assert_eq!($name::default(), $name::ZERO);

                for &a in &values {
                    let converted = $name::try_from(a as $inner);
                    let parsed = a.to_string().parse::<$name>();
                    if a as i128 > MAX {
                        let err = OverflowError { max: MAX as $inner, value: a as $inner };
                        assert_eq!(converted, Err(err));
                        assert_eq!(parsed, Err(ParseIntError(IntErrorKind::PosOverflow)));
                        continue;
                    }
                    let (x, a) = (small(a as i128), a as i128);
                    assert_eq!((converted, parsed), (Ok(x), Ok(x)));
                    assert_eq!(x.$to_inner(), a as $inner);
                    assert_eq!(!x, small(MAX - a));
                    assert_fmt!(x, a as $inner => "{}" "{:?}" "{:b}" "{:o}" "{:x}" "{:X}" "{:#x}"
                        "{:#010b}" "{:>9}" "{:+}" "{:x?}");
                    for shift in 0..$name::BITS {
                        assert_eq!(x << shift, small((a << shift) & MAX), "{a} << {shift}");
                        assert_eq!(x >> shift, small(a >> shift), "{a} >> {shift}");
                    }

                    for &b in &values {
                        let (rhs, b) = (b as $inner, b as i128);
                        let flavors = [
                            (
                                a + b,
                                x.checked_add(rhs),
                                x.overflowing_add(rhs),
                                x.saturating_add(rhs),
                                x.wrapping_add(rhs),
                            ),
                            (
                                a - b,
                                x.checked_sub(rhs),
                                x.overflowing_sub(rhs),
                                x.saturating_sub(rhs),
                                x.wrapping_sub(rhs),
                            ),
                            (
                                a * b,
                                x.checked_mul(rhs),
                                x.overflowing_mul(rhs),
                                x.saturating_mul(rhs),
                                x.wrapping_mul(rhs),
                            ),
                        ];
                        for (exact, checked, overflowing, saturating, wrapping) in flavors {
                            let fits = (0..=MAX).contains(&exact);
                            let wrapped = small(exact.rem_euclid(MAX + 1));
                            assert_eq!(checked, fits.then_some(wrapped), "{a} and {b}");
                            assert_eq!(overflowing, (wrapped, !fits), "{a} and {b}");
                            assert_eq!(saturating, small(exact.clamp(0, MAX)), "{a} and {b}");
                            assert_eq!(wrapping, wrapped, "{a} and {b}");
                        }

                        if a + b <= MAX {
                            assert_eq!(x + rhs, small(a + b));
                        }
                        if a >= b {
                            assert_eq!(x - rhs, small(a - b));
                        }
                        if a * b <= MAX {
                            assert_eq!(x * rhs, small(a * b));
                        }
                        assert_eq!(x & rhs, small(a & b));
                        if b <= MAX {
                            assert_eq!(x | rhs, small(a | b));
                            assert_eq!(x ^ rhs, small(a ^ b));
                            assert_eq!(x.cmp(&small(b)), a.cmp(&b));
                        }

                        let quot = (b != 0).then(|| small(a / b));
                        let rem = (b != 0).then(|| small(a % b));
                        assert_eq!((x.checked_div(rhs), x.checked_rem(rhs)), (quot, rem));
                        match quot.zip(rem) {
                            None => assert_eq!(x.div_rem(rhs), Err(DivError::ZeroDiv)),
                            Some(res) => {
                                assert_eq!(x.div_rem(rhs), Ok(res));
                                assert_eq!((x / rhs, x % rhs), res);
                            }
                        }
                    }
                }
            }
        )+};
    }

    model_tests! {
        u1_model: u1(u8) to_u8;
        u2_model: u2(u8) to_u8;
        u3_model: u3(u8) to_u8;
        u4_model: u4(u8) to_u8;
        u5_model: u5(u8) to_u8;
        u6_model: u6(u8) to_u8;
        u7_model: u7(u8) to_u8;
        u10_model: u10(u16) to_u16;
        u12_model: u12(u16) to_u16;
        u14_model: u14(u16) to_u16;
        u20_model: u20(u32) to_u32;
        u24_model: u24(u32) to_u32;
        u40_model: u40(u64) to_u64;
        u48_model: u48(u64) to_u64;
        u56_model: u56(u64) to_u64;
    }

    #[test]
    fn assignments() {
        let mut val = u5::with(3);
        val += 4u8;
        assert_eq!(val, u5::with(7));
        val -= 2u8;
        assert_eq!(val, u5::with(5));
        val *= 6u8;
        assert_eq!(val, u5::with(30));
        val /= 4u8;
        assert_eq!(val, u5::with(7));
        val %= 4u8;
        assert_eq!(val, u5::with(3));
        val <<= 3;
        assert_eq!(val, u5::with(24));
        val >>= 2;
        assert_eq!(val, u5::with(6));
        val |= 9u8;
        assert_eq!(val, u5::with(15));
        val &= 12u8;
        assert_eq!(val, u5::with(12));
        val ^= 10u8;
        assert_eq!(val, u5::with(6));

        // The right-hand operand may be any integer which fits the underlying
        // type.
        assert_eq!(val + u5::with(4) + u1::ONE, u5::with(11));
        assert_eq!(u24::with(1000) * u7::with(100) - u14::with(5), u24::with(99_995));
        assert_eq!(u4::MAX << 1, u4::with(0b1110));
        assert_eq!(u4::MAX << 3, u4::with(0b1000));
    }

    #[test]
    fn conversions() {
        assert_eq!((u8::from(u7::MAX), i8::from(u7::MAX)), (127, 127));
        assert_eq!((u16::from(u14::MAX), i16::from(u14::MAX)), (0x3FFF, 0x3FFF));
        assert_eq!((u32::from(u24::MAX), i32::from(u24::MAX)), (0xFF_FFFF, 0xFF_FFFF));
        assert_eq!((u64::from(u56::MAX), i64::from(u56::MAX)), ((1 << 56) - 1, (1 << 56) - 1));
        assert_eq!((u128::from(u56::MAX), i128::from(u56::MAX)), ((1 << 56) - 1, (1 << 56) - 1));
        assert_eq!((u128::from(u1::ONE), i128::from(u1::ONE)), (1, 1));
        assert_eq!((usize::from(u14::MAX), isize::from(u14::MAX)), (0x3FFF, 0x3FFF));
        #[cfg(not(target_pointer_width = "16"))]
        assert_eq!((usize::from(u24::MAX), isize::from(u24::MAX)), (0xFF_FFFF, 0xFF_FFFF));
        #[cfg(target_pointer_width = "64")]
        assert_eq!((usize::from(u40::MAX), isize::from(u48::MAX)), ((1 << 40) - 1, (1 << 48) - 1));

        assert_eq!(u2::from(u1::ONE), u2::ONE);
        assert_eq!(u7::from(u6::MAX), u7::with(63));
        assert_eq!(u10::from(u7::MAX), u10::with(127));
        assert_eq!(u20::from(u14::MAX), u20::with(0x3FFF));
        assert_eq!(u40::from(u24::MAX), u40::with(0xFF_FFFF));
        assert_eq!(u56::from(u48::MAX), u56::with(0xFFFF_FFFF_FFFF));
        assert_eq!(u56::from(u1::ONE), u56::ONE);
    }

    #[test]
    fn bytes() {
        let val = u24::with(0x12_3456);
        assert_eq!(
            (val.to_le_bytes(), val.to_be_bytes()),
            ([0x56, 0x34, 0x12], [0x12, 0x34, 0x56])
        );
        assert_eq!(u24::from_le_bytes([0x56, 0x34, 0x12]), val);
        assert_eq!(u24::from_be_bytes([0x12, 0x34, 0x56]), val);

        let val = u40::with(0x12_3456_789A);
        assert_eq!(val.to_le_bytes(), [0x9A, 0x78, 0x56, 0x34, 0x12]);
        assert_eq!(val.to_be_bytes(), [0x12, 0x34, 0x56, 0x78, 0x9A]);
        assert_eq!(u40::from_le_bytes(val.to_le_bytes()), val);
        assert_eq!(u40::from_be_bytes(val.to_be_bytes()), val);

        let val = u48::with(0x1234_5678_9ABC);
        assert_eq!(val.to_le_bytes(), [0xBC, 0x9A, 0x78, 0x56, 0x34, 0x12]);
        assert_eq!(val.to_be_bytes(), [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC]);
        assert_eq!(u48::from_le_bytes(val.to_le_bytes()), val);
        assert_eq!(u48::from_be_bytes(val.to_be_bytes()), val);

        let val = u56::with(0x12_3456_789A_BCDE);
        assert_eq!(val.to_le_bytes(), [0xDE, 0xBC, 0x9A, 0x78, 0x56, 0x34, 0x12]);
        assert_eq!(val.to_be_bytes(), [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE]);
        assert_eq!(u56::from_le_bytes(val.to_le_bytes()), val);
        assert_eq!(u56::from_be_bytes(val.to_be_bytes()), val);

        assert_eq!(
            (u24::from_le_bytes([0xFF; 3]), u24::from_be_bytes([0xFF; 3])),
            (u24::MAX, u24::MAX)
        );
        assert_eq!(
            (u56::from_le_bytes([0xFF; 7]), u56::from_be_bytes([0xFF; 7])),
            (u56::MAX, u56::MAX)
        );
        assert_eq!(u56::MAX.to_be_bytes(), [0xFF; 7]);
    }

    #[test]
    fn parsing() {
        use IntErrorKind::*;

        assert_eq!("31".parse(), Ok(u5::MAX));
        assert_eq!("+7".parse(), Ok(u5::with(7)));
        assert_eq!("72057594037927935".parse(), Ok(u56::MAX));
        for (src, kind) in
            [("", Empty), ("x", InvalidDigit), ("-1", InvalidDigit), ("3 ", InvalidDigit)]
        {
            assert_eq!(src.parse::<u5>(), Err(ParseIntError(kind)), "{src:?}");
        }
        // The value may overflow either the type itself, or the underlying
        // primitive type.
        for src in ["32", "255", "256", "99999999999999999999999"] {
            assert_eq!(src.parse::<u5>(), Err(ParseIntError(PosOverflow)), "{src:?}");
        }
        assert_eq!("72057594037927936".parse::<u56>(), Err(ParseIntError(PosOverflow)));
    }

    macro_rules! panics {
        ($($name:ident: $msg:literal => $expr:expr;)+) => {$(
            #[test]
            #[should_panic(expected = $msg)]
            fn $name() { let _ = $expr; }
        )+};
    }

    panics! {
        with_overflow: "provided value exceeds Self::MAX" => u4::with(16);
        add_overflow: "attempt to add with overflow" => u4::MAX + 1u8;
        add_inner_overflow: "attempt to add with overflow" => u56::with(5) + u64::MAX;
        sub_overflow: "attempt to subtract with overflow" => u4::ZERO - 1u8;
        mul_overflow: "attempt to multiply with overflow" => u7::MAX * 127u8;
        mul_inner_overflow: "attempt to multiply with overflow" => u7::MAX * 255u8;
        div_by_zero: "attempt to divide by zero" => u4::ONE / 0u8;
        rem_by_zero: "attempt to calculate the remainder with a divisor of zero" => u4::ONE % 0u8;
        bitor_overflow: "attempt to bitor with overflow" => u4::ONE | 16u8;
        bitxor_overflow: "attempt to bitxor with overflow" => u4::ONE ^ 16u8;
        shl_overflow: "attempt to shift left with overflow" => u4::ONE << 4;
        shr_overflow: "attempt to shift right with overflow" => u4::ONE >> 4;
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_validates_range() {
        assert_eq!(serde_json::to_string(&u5::MAX).unwrap(), "31");
        assert_eq!(serde_json::from_str::<u5>("31").unwrap(), u5::MAX);
        assert_eq!(serde_json::from_str::<u1>("1").unwrap(), u1::ONE);
        assert_eq!(serde_json::to_string(&u56::MAX).unwrap(), "72057594037927935");
        assert_eq!(serde_json::from_str::<u56>("72057594037927935").unwrap(), u56::MAX);

        // Values outside of the range of the type must never be constructed.
        for invalid in ["32", "255", "256", "-1", "1.5", "\"1\""] {
            assert!(serde_json::from_str::<u5>(invalid).is_err(), "{invalid}");
        }
        assert!(serde_json::from_str::<u1>("2").is_err());
        assert!(serde_json::from_str::<u24>("16777216").is_err());
        assert!(serde_json::from_str::<u56>("72057594037927936").is_err());
        let err = serde_json::from_str::<u5>("32").unwrap_err().to_string();
        assert!(err.contains("value 32 overflows the maximum 31"), "{err}");
    }
}
