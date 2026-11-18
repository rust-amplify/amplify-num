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
use crate::promote::promote;
use crate::traits::{
    AddChecked, AddOverflowing, AddSaturate, AddWrapping, DivChecked, DivOverflowing, DivSaturate,
    DivWrapping, MulChecked, MulOverflowing, MulSaturate, MulWrapping, NegWrapping, Number,
    RemChecked, RemOverflowing, RemWrapping, ShlChecked, ShlOverflowing, ShlWrapping, ShrChecked,
    ShrOverflowing, ShrWrapping, SubChecked, SubOverflowing, SubSaturate, SubWrapping, Unsigned,
};

/// Defines unsigned integers of a bit size which is smaller than the size of
/// the primitive integer storing them.
macro_rules! smallint {
    ($($name:ident: $bits:literal bits in $inner:ident as $to_inner:ident;)+) => {$(
        #[doc = concat!($bits, "-bit unsigned integer in the range `0..2^", $bits, "`.")]
        ///
        /// The arithmetic operators panic in all build profiles if their result does not
        /// fit the type; the arithmetic traits in [`crate::traits`] provide `checked_*`,
        /// `overflowing_*`, `saturating_*` and `wrapping_*` methods for explicit control
        /// over the overflow handling.
        ///
        /// An operand convertible into this type uses this type as the result. Otherwise,
        /// integer operands are accepted when one of the two types fits the other
        /// losslessly, and the result has the wider of them, like the result of `u5 + u8`
        /// is `u8`. The operands of which none fits the other, like `u24` and `i8`, are
        /// rejected by the compiler as the primitive integers do. The compound assignment
        /// operators keep the type of the left operand, accepting any right operand
        /// convertible into the underlying primitive type, and panic if the result does
        /// not fit the type.
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

            /// Calculates the quotient and the remainder of the division of `self` by `rhs`.
            pub fn div_rem(self, rhs: impl Into<$inner>) -> Result<(Self, Self), DivError> {
                let rhs = rhs.into();
                if rhs == 0 {
                    return Err(DivError::ZeroDiv);
                }
                Ok((Self(self.0 / rhs), Self(self.0 % rhs)))
            }

            /// Parses the integer from a string of digits in the given `radix`.
            ///
            /// # Panics
            ///
            /// If `radix` is not in the range from 2 to 36.
            pub fn from_str_radix(src: &str, radix: u32) -> Result<Self, ParseIntError> {
                assert!((2..=36).contains(&radix), "radix must lie in the range `2..=36`");

                Self::try_from($inner::from_str_radix(src, radix)?)
                    .map_err(|_| ParseIntError(IntErrorKind::PosOverflow))
            }
        }

        smallint_trait_impls!($name, $inner);

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
                Self::from_str_radix(s, 10)
            }
        }

        smallint_operators! { $name:
            Add add => AddChecked::checked_add, "attempt to add with overflow";
            Sub sub => SubChecked::checked_sub, "attempt to subtract with overflow";
            Mul mul => MulChecked::checked_mul, "attempt to multiply with overflow";
            Div div => DivChecked::checked_div, "attempt to divide by zero";
            Rem rem => RemChecked::checked_rem,
                "attempt to calculate the remainder with a divisor of zero";
        }

        smallint_bitwise_operators!($name: BitAnd bitand &, BitOr bitor |, BitXor bitxor ^);

        smallint_assign_operators! { $name($inner):
            AddAssign add_assign => $inner::checked_add, "attempt to add with overflow";
            SubAssign sub_assign => $inner::checked_sub, "attempt to subtract with overflow";
            MulAssign mul_assign => $inner::checked_mul, "attempt to multiply with overflow";
            DivAssign div_assign => $inner::checked_div, "attempt to divide by zero";
            RemAssign rem_assign => $inner::checked_rem,
                "attempt to calculate the remainder with a divisor of zero";
            BitAndAssign bitand_assign =>
                |lhs, rhs| Some(lhs & rhs), "attempt to bitand with overflow";
            BitOrAssign bitor_assign =>
                |lhs, rhs| Some(lhs | rhs), "attempt to bitor with overflow";
            BitXorAssign bitxor_assign =>
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
                self.checked_shl(rhs).expect("attempt to shift left with overflow")
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
                self.checked_shr(rhs).expect("attempt to shift right with overflow")
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

/// Implements the flavors of an arithmetic operation as trait implementations,
/// each on top of the eponymous method of the underlying primitive type. Since
/// the bit size of that type is larger, wrapping at it and then at `2^BITS` is
/// the same as wrapping at `2^BITS` right away.
macro_rules! smallint_arithmetics {
    ($name:ident($inner:ident): $(
        $checked_trait:ident::$checked:ident,
        $overflowing_trait:ident::$overflowing:ident,
        $saturating_trait:ident::$saturating:ident,
        $wrapping_trait:ident::$wrapping:ident;
    )+) => {$(
        impl<Rhs: Into<Self>> $checked_trait<Rhs> for $name {
            type Output = Self;

            fn $checked(self, rhs: Rhs) -> Option<Self> {
                let rhs = rhs.into();
                self.0.$checked(rhs.0).and_then(|val| Self::try_from(val).ok())
            }
        }

        impl<Rhs: Into<Self>> $overflowing_trait<Rhs> for $name {
            type Output = Self;

            fn $overflowing(self, rhs: Rhs) -> (Self, bool) {
                let rhs = rhs.into();
                let (val, overflow) = self.0.$overflowing(rhs.0);
                (Self(val & Self::MAX.0), overflow || val > Self::MAX.0)
            }
        }

        impl<Rhs: Into<Self>> $saturating_trait<Rhs> for $name {
            type Output = Self;

            fn $saturating(self, rhs: Rhs) -> Self {
                let rhs = rhs.into();
                Self(self.0.$saturating(rhs.0).min(Self::MAX.0))
            }
        }

        impl<Rhs: Into<Self>> $wrapping_trait<Rhs> for $name {
            type Output = Self;

            fn $wrapping(self, rhs: Rhs) -> Self {
                let rhs = rhs.into();
                Self(self.0.$wrapping(rhs.0) & Self::MAX.0)
            }
        }
    )+};
}

macro_rules! smallint_trait_impls {
    ($name:ident, $inner:ident) => {
        impl Number for $name {
            const BITS: u32 = Self::BITS;
            const MIN: Self = Self::MIN;
            const MAX: Self = Self::MAX;
            const ZERO: Self = Self::ZERO;
            const ONE: Self = Self::ONE;
        }

        impl Unsigned for $name {}

        smallint_arithmetics! { $name($inner):
            AddChecked::checked_add, AddOverflowing::overflowing_add,
                AddSaturate::saturating_add, AddWrapping::wrapping_add;
            SubChecked::checked_sub, SubOverflowing::overflowing_sub,
                SubSaturate::saturating_sub, SubWrapping::wrapping_sub;
            MulChecked::checked_mul, MulOverflowing::overflowing_mul,
                MulSaturate::saturating_mul, MulWrapping::wrapping_mul;
        }

        impl<Rhs: Into<Self>> DivChecked<Rhs> for $name {
            type Output = Self;

            fn checked_div(self, rhs: Rhs) -> Option<Self> {
                self.0.checked_div(rhs.into().0).map(Self)
            }
        }

        impl<Rhs: Into<Self>> DivOverflowing<Rhs> for $name {
            type Output = Self;

            fn overflowing_div(self, rhs: Rhs) -> (Self, bool) {
                (Self(self.0 / rhs.into().0), false)
            }
        }

        impl<Rhs: Into<Self>> DivSaturate<Rhs> for $name {
            type Output = Self;

            fn saturating_div(self, rhs: Rhs) -> Self { Self(self.0 / rhs.into().0) }
        }

        impl<Rhs: Into<Self>> DivWrapping<Rhs> for $name {
            type Output = Self;

            fn wrapping_div(self, rhs: Rhs) -> Self { Self(self.0 / rhs.into().0) }
        }

        impl<Rhs: Into<Self>> RemChecked<Rhs> for $name {
            type Output = Self;

            fn checked_rem(self, rhs: Rhs) -> Option<Self> {
                self.0.checked_rem(rhs.into().0).map(Self)
            }
        }

        impl<Rhs: Into<Self>> RemOverflowing<Rhs> for $name {
            type Output = Self;

            fn overflowing_rem(self, rhs: Rhs) -> (Self, bool) {
                (Self(self.0 % rhs.into().0), false)
            }
        }

        impl<Rhs: Into<Self>> RemWrapping<Rhs> for $name {
            type Output = Self;

            fn wrapping_rem(self, rhs: Rhs) -> Self { Self(self.0 % rhs.into().0) }
        }

        impl NegWrapping for $name {
            type Output = Self;

            fn wrapping_neg(self) -> Self { Self(self.0.wrapping_neg() & Self::MAX.0) }
        }

        impl ShlChecked for $name {
            type Output = Self;

            fn checked_shl(self, rhs: u32) -> Option<Self> {
                (rhs < Self::BITS).then(|| Self((self.0 << rhs) & Self::MAX.0))
            }
        }

        impl ShlOverflowing for $name {
            type Output = Self;

            fn overflowing_shl(self, rhs: u32) -> (Self, bool) {
                (Self((self.0 << rhs.rem_euclid(Self::BITS)) & Self::MAX.0), rhs >= Self::BITS)
            }
        }

        impl ShlWrapping for $name {
            type Output = Self;

            fn wrapping_shl(self, rhs: u32) -> Self {
                Self((self.0 << rhs.rem_euclid(Self::BITS)) & Self::MAX.0)
            }
        }

        impl ShrChecked for $name {
            type Output = Self;

            fn checked_shr(self, rhs: u32) -> Option<Self> {
                (rhs < Self::BITS).then(|| Self(self.0 >> rhs))
            }
        }

        impl ShrOverflowing for $name {
            type Output = Self;

            fn overflowing_shr(self, rhs: u32) -> (Self, bool) {
                (Self(self.0 >> rhs.rem_euclid(Self::BITS)), rhs >= Self::BITS)
            }
        }

        impl ShrWrapping for $name {
            type Output = Self;

            fn wrapping_shr(self, rhs: u32) -> Self { Self(self.0 >> rhs.rem_euclid(Self::BITS)) }
        }
    };
}

/// Implements an arithmetic operator of two values of the type on top of its
/// checked form, panicking with the given message if the latter fails.
macro_rules! smallint_operators {
    ($name:ident: $($op:ident $fn:ident => $trait:ident::$checked:ident, $panic:literal;)+) => {$(
        impl<Rhs: Into<Self>> $op<Rhs> for $name {
            type Output = Self;

            fn $fn(self, rhs: Rhs) -> Self {
                <Self as $trait<Rhs>>::$checked(self, rhs).expect($panic)
            }
        }
    )+};
}

/// Implements a bitwise operator of two values of the type, which can't
/// overflow.
macro_rules! smallint_bitwise_operators {
    ($name:ident: $($op:ident $fn:ident $tok:tt),+) => {$(
        impl<Rhs: Into<Self>> $op<Rhs> for $name {
            type Output = Self;

            fn $fn(self, rhs: Rhs) -> Self {
                let rhs = rhs.into();
                Self(self.0 $tok rhs.0)
            }
        }
    )+};
}

/// Implements an assigning operator for any right-hand operand convertible into
/// the underlying primitive type. The operator panics with the given message if
/// the checked operation fails or its result does not fit the type.
macro_rules! smallint_assign_operators {
    ($name:ident($inner:ident): $($op_assign:ident $fn_assign:ident =>
     $checked:expr, $panic:literal;)+) => {$(
        impl<T: Into<$inner>> $op_assign<T> for $name {
            fn $fn_assign(&mut self, rhs: T) {
                let checked: fn($inner, $inner) -> Option<$inner> = $checked;
                *self = checked(self.0, rhs.into())
                    .and_then(|val| Self::try_from(val).ok())
                    .expect($panic);
            }
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

impl u2 {
    /// The value 0b10 (two) as a constant.
    pub const TWO: u2 = u2(2);

    /// The value 0b11 (three) as a constant.
    pub const THREE: u2 = u2(3);
}

impl u3 {
    /// The value 0b010 (two) as a constant.
    pub const TWO: u3 = u3(2);

    /// The value 0b011 (three) as a constant.
    pub const THREE: u3 = u3(3);

    /// The value 0b100 (four) as a constant.
    pub const FOUR: u3 = u3(4);

    /// The value 0b101 (five) as a constant.
    pub const FIVE: u3 = u3(5);

    /// The value 0b110 (six) as a constant.
    pub const SIX: u3 = u3(6);

    /// The value 0b111 (seven) as a constant.
    pub const SEVEN: u3 = u3(7);
}

/// Implements the lossless conversions of each integer of the list into all
/// larger ones, and the operations mixing them.
macro_rules! smallint_widening {
    ($small:ident $($large:ident)*) => {
        $(
            impl From<$small> for $large {
                fn from(value: $small) -> Self {
                    const { assert!($small::BITS < $large::BITS, "lossy conversion") };
                    Self(value.0.into())
                }
            }

            promote!(own [] $small, $large => $large, |lhs, rhs| (lhs.into(), rhs));
        )*
        smallint_widening!($($large)*);
    };
    () => {};
}
smallint_widening!(u1 u2 u3 u4 u5 u6 u7 u10 u12 u14 u20 u24 u40 u48 u56);

/// Implements the lossless conversions of the small integers into the primitive
/// ones, and the operations mixing them. The arithmetic traits of the primitive
/// left operands are covered by the [`crate::traits`] implementations already.
macro_rules! smallint_into {
    ($($name:ident)+ => $prims:tt) => { $(smallint_into!(@ $name => $prims);)+ };
    (@ $name:ident => [$($prim:ident)+]) => {$(
        impl From<$name> for $prim {
            #[allow(
                trivial_numeric_casts,
                reason = "macro generates identity and widening casts uniformly"
            )]
            fn from(value: $name) -> Self {
                const { assert!($name::MAX.0 as u128 <= $prim::MAX as u128, "lossy conversion") };
                value.0 as $prim
            }
        }

        promote!(prim [] $name, $prim => $prim, |lhs, rhs| (lhs.into(), rhs));
        promote!(ops prim [] $prim, $name => $prim, |lhs, rhs| (lhs, rhs.into()));
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

/// Implements the operations mixing the small integers with the primitive ones
/// which fit them losslessly.
macro_rules! smallint_from_prim {
    ($($prim:ident => [$($name:ident)+];)+) => {$($(
        impl From<$prim> for $name {
            fn from(value: $prim) -> Self {
                const { assert!($prim::BITS <= $name::BITS, "lossy conversion") };
                Self(value.into())
            }
        }

        promote!(own [] $prim, $name => $name, |lhs, rhs| ($name(lhs.into()), rhs));
    )+)+};
}
smallint_from_prim! {
    u8 => [u10 u12 u14 u20 u24 u40 u48 u56];
    u16 => [u20 u24 u40 u48 u56];
    u32 => [u40 u48 u56];
}

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

    #[derive(Copy, Clone)]
    struct CustomOperand(u8);

    impl From<CustomOperand> for u24 {
        fn from(value: CustomOperand) -> Self { Self::with(u32::from(value.0)) }
    }

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
            #[allow(
                trivial_numeric_casts,
                reason = "macro instantiates tests for u64 inner types too"
            )]
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
                        assert_eq!(
                            $name::from_str_radix(&format!("{a:x}"), 16),
                            Err(ParseIntError(IntErrorKind::PosOverflow))
                        );
                        continue;
                    }
                    let (x, a) = (small(a as i128), a as i128);
                    assert_eq!((converted, parsed), (Ok(x), Ok(x)));
                    assert_eq!(x.$to_inner(), a as $inner);
                    assert_eq!($name::from_str_radix(&format!("{x:x}"), 16), Ok(x));
                    assert_eq!($name::from_str_radix(&format!("{x:X}"), 16), Ok(x));
                    assert_eq!($name::from_str_radix(&format!("{x:o}"), 8), Ok(x));
                    assert_eq!($name::from_str_radix(&format!("{x:b}"), 2), Ok(x));
                    assert_eq!(!x, small(MAX - a));
                    assert_eq!(x.wrapping_neg(), small((-a).rem_euclid(MAX + 1)));
                    assert_fmt!(x, a as $inner => "{}" "{:?}" "{:b}" "{:o}" "{:x}" "{:X}" "{:#x}"
                        "{:#010b}" "{:>9}" "{:+}" "{:x?}");
                    for shift in 0..=2 * $name::BITS + 1 {
                        let wrapped_shift = shift.rem_euclid($name::BITS);
                        let shl = small((a << wrapped_shift) & MAX);
                        let shr = small(a >> wrapped_shift);
                        let in_range = shift < $name::BITS;
                        assert_eq!(x.checked_shl(shift), in_range.then_some(shl));
                        assert_eq!(x.overflowing_shl(shift), (shl, !in_range));
                        assert_eq!(x.wrapping_shl(shift), shl);
                        assert_eq!(x.checked_shr(shift), in_range.then_some(shr));
                        assert_eq!(x.overflowing_shr(shift), (shr, !in_range));
                        assert_eq!(x.wrapping_shr(shift), shr);
                        if in_range {
                            assert_eq!(x << shift, shl, "{a} << {shift}");
                            assert_eq!(x >> shift, shr, "{a} >> {shift}");
                        }
                    }

                    for &b in &values {
                        let (rhs, b) = (b as $inner, b as i128);

                        // The underlying type is the wider one, thus the operations
                        // with it are performed by that type.
                        let inner = a as $inner;
                        assert_eq!(x.checked_add(rhs), inner.checked_add(rhs));
                        assert_eq!(x.overflowing_add(rhs), inner.overflowing_add(rhs));
                        assert_eq!(x.saturating_add(rhs), inner.saturating_add(rhs));
                        assert_eq!(x.wrapping_add(rhs), inner.wrapping_add(rhs));
                        assert_eq!(x.checked_sub(rhs), inner.checked_sub(rhs));
                        assert_eq!(x.overflowing_sub(rhs), inner.overflowing_sub(rhs));
                        assert_eq!(x.saturating_sub(rhs), inner.saturating_sub(rhs));
                        assert_eq!(x.wrapping_sub(rhs), inner.wrapping_sub(rhs));
                        assert_eq!(x.checked_mul(rhs), inner.checked_mul(rhs));
                        assert_eq!(x.overflowing_mul(rhs), inner.overflowing_mul(rhs));
                        assert_eq!(x.saturating_mul(rhs), inner.saturating_mul(rhs));
                        assert_eq!(x.wrapping_mul(rhs), inner.wrapping_mul(rhs));
                        assert_eq!(x.checked_div(rhs), inner.checked_div(rhs));
                        assert_eq!(x.checked_rem(rhs), inner.checked_rem(rhs));
                        assert_eq!(
                            <$inner as SubChecked<$name>>::checked_sub(rhs, x),
                            rhs.checked_sub(inner)
                        );
                        if let Some(sum) = inner.checked_add(rhs) {
                            assert_eq!((x + rhs, rhs + x), (sum, sum));
                        }
                        if let Some(diff) = inner.checked_sub(rhs) {
                            assert_eq!(x - rhs, diff);
                        }
                        if let Some(diff) = rhs.checked_sub(inner) {
                            assert_eq!(rhs - x, diff);
                        }
                        if let Some(prod) = inner.checked_mul(rhs) {
                            assert_eq!((x * rhs, rhs * x), (prod, prod));
                        }
                        if b != 0 {
                            assert_eq!((x / rhs, x % rhs), (inner / rhs, inner % rhs));
                            assert_eq!(x.overflowing_div(rhs), (inner / rhs, false));
                            assert_eq!(x.saturating_div(rhs), inner / rhs);
                            assert_eq!(x.wrapping_div(rhs), inner / rhs);
                            assert_eq!(x.overflowing_rem(rhs), (inner % rhs, false));
                            assert_eq!(x.wrapping_rem(rhs), inner % rhs);
                        }
                        if a != 0 {
                            assert_eq!((rhs / x, rhs % x), (rhs / inner, rhs % inner));
                        }
                        let bitwise = (inner & rhs, inner | rhs, inner ^ rhs);
                        assert_eq!((x & rhs, x | rhs, x ^ rhs), bitwise);
                        assert_eq!((rhs & x, rhs | x, rhs ^ x), bitwise);

                        // The compound assignment keeps the type of the left operand.
                        let assign = |op: fn(&mut $name, $inner)| {
                            let mut val = x;
                            op(&mut val, rhs);
                            val
                        };
                        if a + b <= MAX {
                            assert_eq!(assign(|val, rhs| *val += rhs), small(a + b));
                        }
                        if a >= b {
                            assert_eq!(assign(|val, rhs| *val -= rhs), small(a - b));
                        }
                        if a * b <= MAX {
                            assert_eq!(assign(|val, rhs| *val *= rhs), small(a * b));
                        }
                        if b != 0 {
                            assert_eq!(assign(|val, rhs| *val /= rhs), small(a / b));
                            assert_eq!(assign(|val, rhs| *val %= rhs), small(a % b));
                        }
                        assert_eq!(assign(|val, rhs| *val &= rhs), small(a & b));
                        if b <= MAX {
                            assert_eq!(assign(|val, rhs| *val |= rhs), small(a | b));
                            assert_eq!(assign(|val, rhs| *val ^= rhs), small(a ^ b));
                        }

                        let quot = (b != 0).then(|| small(a / b));
                        let rem = (b != 0).then(|| small(a % b));
                        match quot.zip(rem) {
                            None => assert_eq!(x.div_rem(rhs), Err(DivError::ZeroDiv)),
                            Some((q, r)) => assert_eq!(x.div_rem(rhs), Ok((q, r))),
                        }

                        if b > MAX {
                            continue;
                        }
                        let y = small(b);
                        let flavors = [
                            (
                                a + b,
                                x.checked_add(y),
                                x.overflowing_add(y),
                                x.saturating_add(y),
                                x.wrapping_add(y),
                            ),
                            (
                                a - b,
                                x.checked_sub(y),
                                x.overflowing_sub(y),
                                x.saturating_sub(y),
                                x.wrapping_sub(y),
                            ),
                            (
                                a * b,
                                x.checked_mul(y),
                                x.overflowing_mul(y),
                                x.saturating_mul(y),
                                x.wrapping_mul(y),
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
                            assert_eq!(x + y, small(a + b));
                        }
                        if a >= b {
                            assert_eq!(x - y, small(a - b));
                        }
                        if a * b <= MAX {
                            assert_eq!(x * y, small(a * b));
                        }
                        assert_eq!((x & y, x | y, x ^ y), (small(a & b), small(a | b), small(a ^ b)));
                        assert_eq!(x.cmp(&y), a.cmp(&b));

                        assert_eq!((x.checked_div(y), x.checked_rem(y)), (quot, rem));
                        if let Some((q, r)) = quot.zip(rem) {
                            assert_eq!((x / y, x % y), (q, r));
                            assert_eq!(x.overflowing_div(y), (q, false));
                            assert_eq!(x.saturating_div(y), q);
                            assert_eq!(x.wrapping_div(y), q);
                            assert_eq!(x.overflowing_rem(y), (r, false));
                            assert_eq!(x.wrapping_rem(y), r);
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
    fn constants() {
        assert_eq!([u1::ZERO, u1::ONE], [0, 1].map(u1::with));
        assert_eq!([u2::ZERO, u2::ONE, u2::TWO, u2::THREE], [0, 1, 2, 3].map(u2::with));
        assert_eq!(
            [u3::ZERO, u3::ONE, u3::TWO, u3::THREE, u3::FOUR, u3::FIVE, u3::SIX, u3::SEVEN],
            [0, 1, 2, 3, 4, 5, 6, 7].map(u3::with)
        );
        assert_eq!([u2::ZERO, u2::ONE, u2::TWO, u2::THREE].map(u2::to_u8), [0, 1, 2, 3]);
        assert_eq!(
            [u3::ZERO, u3::ONE, u3::TWO, u3::THREE, u3::FOUR, u3::FIVE, u3::SIX, u3::SEVEN]
                .map(u3::to_u8),
            [0, 1, 2, 3, 4, 5, 6, 7]
        );

        assert_eq!((u1::ZERO, u1::ONE), (u1::MIN, u1::MAX));
        assert_eq!((u2::ZERO, u2::THREE), (u2::MIN, u2::MAX));
        assert_eq!((u3::ZERO, u3::SEVEN), (u3::MIN, u3::MAX));

        for raw in 0..=u2::MAX.to_u8() {
            let matched = match u2::with(raw) {
                u2::ZERO => 0,
                u2::ONE => 1,
                u2::TWO => 2,
                u2::THREE => 3,
                _ => unreachable!(),
            };
            assert_eq!(matched, raw);
        }
        for raw in 0..=u3::MAX.to_u8() {
            let matched = match u3::with(raw) {
                u3::ZERO => 0,
                u3::ONE => 1,
                u3::TWO => 2,
                u3::THREE => 3,
                u3::FOUR => 4,
                u3::FIVE => 5,
                u3::SIX => 6,
                u3::SEVEN => 7,
                _ => unreachable!(),
            };
            assert_eq!(matched, raw);
        }
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

        // A narrower right-hand operand keeps the type.
        assert_eq!(val + u5::with(4) + u1::ONE, u5::with(11));
        assert_eq!(u24::with(1000) * u7::with(100) - u14::with(5), u24::with(99_995));
        assert_eq!(u4::MAX << 1, u4::with(0b1110));
        assert_eq!(u4::MAX << 3, u4::with(0b1000));
    }

    #[test]
    fn promotion() {
        // The result has the type of the wider operand.
        assert_eq!(u5::MAX + 224u8, u8::MAX);
        assert_eq!(250u8 + u24::with(10), u24::with(260));
        assert_eq!(u24::MAX - 1u64, 0xFF_FFFEu64);
        assert_eq!(u5::with(3) * u24::with(5), u24::with(15));
        assert_eq!(u24::with(5) * u5::with(3), u24::with(15));
        assert_eq!(u14::MAX / u20::with(2), u20::with(0x1FFF));
        assert_eq!(1000u16 % u20::with(7), u20::with(6));
        assert_eq!(u7::MAX | 128u8, u8::MAX);
        assert_eq!(u12::MAX ^ u7::MAX, u12::with(0xF80));
        assert_eq!(u40::MAX & 0xFFu32, u40::with(0xFF));
        assert_eq!(u7::MAX - i8::MAX, 0i8);
        assert_eq!(u24::with(5) - 10i32, -5i32);
        #[cfg(target_pointer_width = "64")]
        assert_eq!(u56::MAX + 1usize, 1usize << 56);

        // An unsuffixed literal is an `i32`, which fits the narrower types.
        assert_eq!(u24::with(5) + 3, 8i32);
        assert_eq!(u5::ONE - 2, -1i32);

        assert_eq!(u5::MAX.checked_add(224u8), Some(u8::MAX));
        assert_eq!(u5::MAX.checked_add(225u8), None);
        assert_eq!(<u8 as AddChecked<u24>>::checked_add(u8::MAX, u24::MAX), None);
        assert_eq!(<u8 as AddWrapping<u24>>::wrapping_add(1, u24::MAX), u24::ZERO);
        assert_eq!(<u8 as SubOverflowing<u24>>::overflowing_sub(1, u24::with(2)), (u24::MAX, true));
        assert_eq!(u14::ONE.saturating_sub(u20::with(2)), u20::ZERO);
        assert_eq!(u24::with(5).overflowing_sub(6u32), (u32::MAX, true));
        assert_eq!(u24::with(5).wrapping_mul(-1i32), -5i32);
        assert_eq!(u1::ONE.saturating_mul(u56::MAX), u56::MAX);
        assert_eq!(u4::MAX.checked_div(0u16), None);
        assert_eq!(u4::MAX.checked_rem(u7::with(4)), Some(u7::with(3)));
    }

    #[test]
    fn generic_operands() {
        fn arithmetic<T: Into<u24> + Copy>(value: u24, rhs: T) -> (u24, Option<u24>, u24) {
            (value + rhs, value.checked_add(rhs), value & rhs)
        }

        assert_eq!(
            arithmetic(u24::with(10), CustomOperand(5)),
            (u24::with(15), Some(u24::with(15)), u24::ZERO)
        );
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

        assert_eq!(u10::from(u8::MAX), u10::with(0xFF));
        assert_eq!(u20::from(u16::MAX), u20::with(0xFFFF));
        assert_eq!(u40::from(u32::MAX), u40::with(0xFFFF_FFFF));
        assert_eq!(u56::from(u8::MAX), u56::with(0xFF));
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
        assert_eq!(u5::from_str_radix("1f", 16), Ok(u5::MAX));
        assert_eq!(u5::from_str_radix("37", 8), Ok(u5::with(31)));
        assert_eq!(u5::from_str_radix("11111", 2), Ok(u5::MAX));
        assert_eq!(u56::from_str_radix("zz", 36), Ok(u56::with(1295)));
        assert_eq!(u5::from_str_radix("-1", 16), Err(ParseIntError(InvalidDigit)));
        assert_eq!(u5::from_str_radix("", 16), Err(ParseIntError(Empty)));
        assert_eq!(u5::from_str_radix("20", 16), Err(ParseIntError(PosOverflow)));
        assert_eq!(u5::from_str_radix("100", 16), Err(ParseIntError(PosOverflow)));
        assert_eq!(u56::from_str_radix("100000000000000", 16), Err(ParseIntError(PosOverflow)));
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
        radix_too_small: "radix must lie in the range" => u4::from_str_radix("0", 1);
        radix_too_large: "radix must lie in the range" => u4::from_str_radix("0", 37);
        add_overflow: "attempt to add with overflow" => u4::MAX + u4::ONE;
        add_assign_overflow: "attempt to add with overflow" => {
            let mut val = u4::MAX;
            val += 1u8;
            val
        };
        add_inner_overflow: "attempt to add with overflow" => u56::with(5) + u64::MAX;
        add_promoted_overflow: "attempt to add with overflow" => 1u8 + u24::MAX;
        sub_overflow: "attempt to subtract with overflow" => u4::ZERO - u1::ONE;
        sub_inner_overflow: "attempt to subtract with overflow" => u4::ZERO - 1u8;
        mul_overflow: "attempt to multiply with overflow" => u7::MAX * u7::with(2);
        mul_inner_overflow: "attempt to multiply with overflow" => u7::MAX * 255u8;
        div_by_zero: "attempt to divide by zero" => u4::ONE / u4::ZERO;
        div_inner_by_zero: "attempt to divide by zero" => u4::ONE / 0u8;
        rem_by_zero: "attempt to calculate the remainder with a divisor of zero" =>
            u4::ONE % u4::ZERO;
        rem_promoted_by_zero: "attempt to calculate the remainder with a divisor of zero" =>
            1u8 % u12::ZERO;
        bitor_overflow: "attempt to bitor with overflow" => {
            let mut val = u4::ONE;
            val |= 16u8;
            val
        };
        bitxor_overflow: "attempt to bitxor with overflow" => {
            let mut val = u4::ONE;
            val ^= 16u8;
            val
        };
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
