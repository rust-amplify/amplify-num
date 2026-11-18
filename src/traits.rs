// SPDX-License-Identifier: Apache-2.0
//
// Written in 2026 by Dr. Maxim Orlovsky <orlovsky@ubideco.org>
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

//! Traits for integer categories and explicit arithmetic overflow behavior.
//!
//! The traits in this module are implemented for Rust's primitive integer
//! types, this crate's custom-width integers, and the corresponding core
//! nonzero integer types. Floating-point types intentionally do not implement
//! them.
//!
//! Binary arithmetic traits have a generic right-hand side, defaulting to the
//! implementing type. This permits implementations to support lossless mixed
//! operands while keeping the result type explicit. For this crate's integers,
//! the operands of different types are converted into the wider of them, which
//! is the type of the result, both in these traits and in the operators. The
//! types of which none fits the other losslessly are rejected by the compiler,
//! as for the primitive integers:
//!
//! ```
//! use amplify_num::{AddChecked, SubWrapping, u4, u24, u256};
//!
//! assert_eq!(u4::MAX.checked_add(1u8), Some(16u8));
//! assert_eq!(SubWrapping::wrapping_sub(1u8, u24::with(2)), u24::MAX);
//! assert_eq!(u24::MAX * 2u64, 0x1FF_FFFE);
//! assert_eq!(1u64 + u256::ONE, u256::from(2u8));
//! ```
//!
//! ```compile_fail
//! let _ = amplify_num::u24::ONE + 1i8; // none of `u24` and `i8` fits the other
//! ```
//!
//! ```compile_fail
//! let _ = amplify_num::u256::ONE + 1; // an `i32` literal does not fit `u256`
//! ```
//!
//! For this crate's custom-width integers, these traits are the sole
//! implementation of `checked_*`, `overflowing_*`, `saturating_*` and
//! `wrapping_*` arithmetic:
//!
//! ```compile_fail
//! let _ = amplify_num::u256::ONE.checked_add(1u8); // requires `AddChecked` in scope
//! ```
//!
//! ```compile_fail
//! let _ = amplify_num::u4::ONE.checked_add(1u8); // requires `AddChecked` in scope
//! ```

use core::fmt::Debug;
use core::num::{
    NonZeroI8, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI128, NonZeroIsize, NonZeroU8,
    NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU128, NonZeroUsize,
};

/// A fixed-width, zero-inclusive integer number.
///
/// This trait is common to signed [`Integer`]s and unsigned [`Unsigned`]s.
/// Floating-point types are not numbers in this crate's integer-specific API.
///
/// ```compile_fail
/// use amplify_num::Number;
/// fn needs_number<T: Number>() {}
/// needs_number::<f64>();
/// ```
pub trait Number: Copy + Debug + Eq + Ord {
    /// Width of the integer in bits.
    const BITS: u32;
    /// Smallest representable value.
    const MIN: Self;
    /// Largest representable value.
    const MAX: Self;
    /// Additive identity.
    const ZERO: Self;
    /// Multiplicative identity.
    const ONE: Self;

    /// Returns whether the number is zero.
    fn is_zero(self) -> bool { self == Self::ZERO }

    /// Returns whether the number is one.
    fn is_one(self) -> bool { self == Self::ONE }
}

/// A signed integer number.
///
/// Unsigned integers implement [`Unsigned`] instead:
///
/// ```compile_fail
/// use amplify_num::Integer;
/// fn needs_signed<T: Integer>() {}
/// needs_signed::<u8>();
/// ```
pub trait Integer: Number {
    /// The additive inverse of one.
    const NEG_ONE: Self;

    /// Returns whether the number is negative.
    fn is_negative(self) -> bool;
}

/// An unsigned integer number, including zero.
pub trait Unsigned: Number {}

/// A nonzero integer wrapper and its underlying zero-inclusive number type.
///
/// Nonzero integers intentionally do not implement [`Number`], since they have
/// no zero identity.
pub trait NonZero: Copy + Debug + Eq + Ord {
    /// The zero-inclusive integer wrapped by this type.
    type Inner: Number;

    /// Extracts the nonzero integer's value.
    fn get(self) -> Self::Inner;
}

/// Checked addition. Returns `None` when the result overflows or cannot be
/// represented by the output type.
pub trait AddChecked<Rhs = Self> {
    /// Result type of the addition.
    type Output;

    /// Adds two values, returning `None` if the result is out of range.
    fn checked_add(self, rhs: Rhs) -> Option<Self::Output>;
}

/// Overflow-reporting addition. The result is wrapped to the output type.
pub trait AddOverflowing<Rhs = Self> {
    /// Result type of the addition.
    type Output;

    /// Adds two values and reports whether the mathematical result overflowed.
    fn overflowing_add(self, rhs: Rhs) -> (Self::Output, bool);
}

/// Saturating addition.
pub trait AddSaturate<Rhs = Self> {
    /// Result type of the addition.
    type Output;

    /// Adds two values, clamping an overflow to the nearest representable
    /// bound.
    fn saturating_add(self, rhs: Rhs) -> Self::Output;
}

/// Wrapping (modular) addition.
pub trait AddWrapping<Rhs = Self> {
    /// Result type of the addition.
    type Output;

    /// Adds two values, wrapping on overflow.
    fn wrapping_add(self, rhs: Rhs) -> Self::Output;
}

/// Checked subtraction. Returns `None` when the result underflows, overflows or
/// cannot be represented by the output type.
pub trait SubChecked<Rhs = Self> {
    /// Result type of the subtraction.
    type Output;

    /// Subtracts two values, returning `None` if the result is out of range.
    fn checked_sub(self, rhs: Rhs) -> Option<Self::Output>;
}

/// Overflow-reporting subtraction. The result is wrapped to the output type.
pub trait SubOverflowing<Rhs = Self> {
    /// Result type of the subtraction.
    type Output;

    /// Subtracts two values and reports whether the mathematical result
    /// overflowed.
    fn overflowing_sub(self, rhs: Rhs) -> (Self::Output, bool);
}

/// Saturating subtraction.
pub trait SubSaturate<Rhs = Self> {
    /// Result type of the subtraction.
    type Output;

    /// Subtracts two values, clamping an overflow to the nearest representable
    /// bound.
    fn saturating_sub(self, rhs: Rhs) -> Self::Output;
}

/// Wrapping (modular) subtraction.
pub trait SubWrapping<Rhs = Self> {
    /// Result type of the subtraction.
    type Output;

    /// Subtracts two values, wrapping on overflow.
    fn wrapping_sub(self, rhs: Rhs) -> Self::Output;
}

/// Checked multiplication. Returns `None` when the result overflows.
pub trait MulChecked<Rhs = Self> {
    /// Result type of the multiplication.
    type Output;

    /// Multiplies two values, returning `None` if the result is out of range.
    fn checked_mul(self, rhs: Rhs) -> Option<Self::Output>;
}

/// Overflow-reporting multiplication. The result is wrapped to the output type.
pub trait MulOverflowing<Rhs = Self> {
    /// Result type of the multiplication.
    type Output;

    /// Multiplies two values and reports whether the mathematical result
    /// overflowed.
    fn overflowing_mul(self, rhs: Rhs) -> (Self::Output, bool);
}

/// Saturating multiplication.
pub trait MulSaturate<Rhs = Self> {
    /// Result type of the multiplication.
    type Output;

    /// Multiplies two values, clamping an overflow to the nearest representable
    /// bound.
    fn saturating_mul(self, rhs: Rhs) -> Self::Output;
}

/// Wrapping (modular) multiplication.
pub trait MulWrapping<Rhs = Self> {
    /// Result type of the multiplication.
    type Output;

    /// Multiplies two values, wrapping on overflow.
    fn wrapping_mul(self, rhs: Rhs) -> Self::Output;
}

/// Checked division. Returns `None` for division by zero or overflow.
pub trait DivChecked<Rhs = Self> {
    /// Result type of the division.
    type Output;

    /// Divides two values, returning `None` for a zero divisor or overflow.
    fn checked_div(self, rhs: Rhs) -> Option<Self::Output>;
}

/// Overflow-reporting division. A zero divisor retains the type's panic
/// behavior.
pub trait DivOverflowing<Rhs = Self> {
    /// Result type of the division.
    type Output;

    /// Divides two values and reports whether the quotient overflowed.
    fn overflowing_div(self, rhs: Rhs) -> (Self::Output, bool);
}

/// Saturating division. A zero divisor retains the type's panic behavior.
pub trait DivSaturate<Rhs = Self> {
    /// Result type of the division.
    type Output;

    /// Divides two values, clamping a quotient overflow to the nearest bound.
    fn saturating_div(self, rhs: Rhs) -> Self::Output;
}

/// Wrapping division. A zero divisor retains the type's panic behavior.
pub trait DivWrapping<Rhs = Self> {
    /// Result type of the division.
    type Output;

    /// Divides two values, wrapping the quotient on overflow.
    fn wrapping_div(self, rhs: Rhs) -> Self::Output;
}

/// Checked remainder. Returns `None` for division by zero or overflow.
pub trait RemChecked<Rhs = Self> {
    /// Result type of the remainder.
    type Output;

    /// Calculates a remainder, returning `None` for a zero divisor or overflow.
    fn checked_rem(self, rhs: Rhs) -> Option<Self::Output>;
}

/// Overflow-reporting remainder. A zero divisor retains the type's panic
/// behavior.
pub trait RemOverflowing<Rhs = Self> {
    /// Result type of the remainder.
    type Output;

    /// Calculates a remainder and reports whether the operation overflowed.
    fn overflowing_rem(self, rhs: Rhs) -> (Self::Output, bool);
}

/// Wrapping remainder. A zero divisor retains the type's panic behavior.
pub trait RemWrapping<Rhs = Self> {
    /// Result type of the remainder.
    type Output;

    /// Calculates a remainder, wrapping on overflow.
    fn wrapping_rem(self, rhs: Rhs) -> Self::Output;
}

/// Checked negation for signed integers.
pub trait NegChecked {
    /// Result type of the negation.
    type Output;

    /// Negates the value, returning `None` if its positive counterpart is not
    /// representable.
    fn checked_neg(self) -> Option<Self::Output>;
}

/// Overflow-reporting negation for signed integers.
pub trait NegOverflowing {
    /// Result type of the negation.
    type Output;

    /// Negates the value and reports whether the result overflowed.
    fn overflowing_neg(self) -> (Self::Output, bool);
}

/// Saturating negation for signed integers.
pub trait NegSaturate {
    /// Result type of the negation.
    type Output;

    /// Negates the value, clamping the minimum value to the maximum.
    fn saturating_neg(self) -> Self::Output;
}

/// Wrapping (modular) negation.
pub trait NegWrapping {
    /// Result type of the negation.
    type Output;

    /// Negates the value modulo `2^BITS`.
    fn wrapping_neg(self) -> Self::Output;
}

/// Checked left shift. Returns `None` when the shift is at least the bit width.
pub trait ShlChecked<Rhs = u32> {
    /// Result type of the shift.
    type Output;

    /// Shifts left, returning `None` for an out-of-range shift count.
    fn checked_shl(self, rhs: Rhs) -> Option<Self::Output>;
}

/// Overflow-reporting left shift.
pub trait ShlOverflowing<Rhs = u32> {
    /// Result type of the shift.
    type Output;

    /// Shifts left and reports whether the shift count is out of range.
    fn overflowing_shl(self, rhs: Rhs) -> (Self::Output, bool);
}

/// Wrapping left shift.
pub trait ShlWrapping<Rhs = u32> {
    /// Result type of the shift.
    type Output;

    /// Shifts left, wrapping the shift count at the number's bit width.
    fn wrapping_shl(self, rhs: Rhs) -> Self::Output;
}

/// Checked right shift. Returns `None` when the shift is at least the bit
/// width.
pub trait ShrChecked<Rhs = u32> {
    /// Result type of the shift.
    type Output;

    /// Shifts right, returning `None` for an out-of-range shift count.
    fn checked_shr(self, rhs: Rhs) -> Option<Self::Output>;
}

/// Overflow-reporting right shift.
pub trait ShrOverflowing<Rhs = u32> {
    /// Result type of the shift.
    type Output;

    /// Shifts right and reports whether the shift count is out of range.
    fn overflowing_shr(self, rhs: Rhs) -> (Self::Output, bool);
}

/// Wrapping right shift.
pub trait ShrWrapping<Rhs = u32> {
    /// Result type of the shift.
    type Output;

    /// Shifts right, wrapping the shift count at the number's bit width.
    fn wrapping_shr(self, rhs: Rhs) -> Self::Output;
}

macro_rules! impl_number {
    ($($ty:ty),+ $(,)?) => {$ (
        impl Number for $ty {
            const BITS: u32 = <$ty>::BITS;
            const MIN: Self = <$ty>::MIN;
            const MAX: Self = <$ty>::MAX;
            const ZERO: Self = 0;
            const ONE: Self = 1;
        }
    )+};
}

macro_rules! impl_natural {
    ($($ty:ty),+ $(,)?) => {$ (impl Unsigned for $ty {})+};
}

macro_rules! impl_integer {
    ($($ty:ty),+ $(,)?) => {$ (
        impl Integer for $ty {
            const NEG_ONE: Self = -1;

            fn is_negative(self) -> bool { self < 0 }
        }
    )+};
}

macro_rules! impl_binary_op {
    (
        $ty:ty,
        $rhs:ty,
        $checked_trait:ident =>
        $checked_method:ident,
        $overflowing_trait:ident =>
        $overflowing_method:ident,
        $saturate_trait:ident =>
        $saturate_method:ident,
        $wrapping_trait:ident =>
        $wrapping_method:ident
    ) => {
        impl<Rhs: Into<$rhs>> $checked_trait<Rhs> for $ty {
            type Output = Self;

            fn $checked_method(self, rhs: Rhs) -> Option<Self> { self.$checked_method(rhs.into()) }
        }

        impl<Rhs: Into<$rhs>> $overflowing_trait<Rhs> for $ty {
            type Output = Self;

            fn $overflowing_method(self, rhs: Rhs) -> (Self, bool) {
                self.$overflowing_method(rhs.into())
            }
        }

        impl<Rhs: Into<$rhs>> $saturate_trait<Rhs> for $ty {
            type Output = Self;

            fn $saturate_method(self, rhs: Rhs) -> Self { self.$saturate_method(rhs.into()) }
        }

        impl<Rhs: Into<$rhs>> $wrapping_trait<Rhs> for $ty {
            type Output = Self;

            fn $wrapping_method(self, rhs: Rhs) -> Self { self.$wrapping_method(rhs.into()) }
        }
    };
}

macro_rules! impl_arithmetic {
    ($ty:ty, $rhs:ty) => {
        impl_binary_op!($ty, $rhs,
            AddChecked => checked_add, AddOverflowing => overflowing_add,
            AddSaturate => saturating_add, AddWrapping => wrapping_add);
        impl_binary_op!($ty, $rhs,
            SubChecked => checked_sub, SubOverflowing => overflowing_sub,
            SubSaturate => saturating_sub, SubWrapping => wrapping_sub);
        impl_binary_op!($ty, $rhs,
            MulChecked => checked_mul, MulOverflowing => overflowing_mul,
            MulSaturate => saturating_mul, MulWrapping => wrapping_mul);
    };
}

macro_rules! impl_div_rem {
    ($ty:ty, $rhs:ty) => {
        impl<Rhs: Into<$rhs>> DivChecked<Rhs> for $ty {
            type Output = Self;

            fn checked_div(self, rhs: Rhs) -> Option<Self> { self.checked_div(rhs.into()) }
        }

        impl<Rhs: Into<$rhs>> DivOverflowing<Rhs> for $ty {
            type Output = Self;

            fn overflowing_div(self, rhs: Rhs) -> (Self, bool) { self.overflowing_div(rhs.into()) }
        }

        impl<Rhs: Into<$rhs>> DivSaturate<Rhs> for $ty {
            type Output = Self;

            fn saturating_div(self, rhs: Rhs) -> Self { self.saturating_div(rhs.into()) }
        }

        impl<Rhs: Into<$rhs>> DivWrapping<Rhs> for $ty {
            type Output = Self;

            fn wrapping_div(self, rhs: Rhs) -> Self { self.wrapping_div(rhs.into()) }
        }

        impl<Rhs: Into<$rhs>> RemChecked<Rhs> for $ty {
            type Output = Self;

            fn checked_rem(self, rhs: Rhs) -> Option<Self> { self.checked_rem(rhs.into()) }
        }

        impl<Rhs: Into<$rhs>> RemOverflowing<Rhs> for $ty {
            type Output = Self;

            fn overflowing_rem(self, rhs: Rhs) -> (Self, bool) { self.overflowing_rem(rhs.into()) }
        }

        impl<Rhs: Into<$rhs>> RemWrapping<Rhs> for $ty {
            type Output = Self;

            fn wrapping_rem(self, rhs: Rhs) -> Self { self.wrapping_rem(rhs.into()) }
        }
    };
}

macro_rules! impl_signed_negation {
    ($($ty:ty),+ $(,)?) => {$ (
        impl NegChecked for $ty {
            type Output = Self;

            fn checked_neg(self) -> Option<Self> { self.checked_neg() }
        }

        impl NegOverflowing for $ty {
            type Output = Self;

            fn overflowing_neg(self) -> (Self, bool) { self.overflowing_neg() }
        }

        impl NegSaturate for $ty {
            type Output = Self;

            fn saturating_neg(self) -> Self { self.saturating_neg() }
        }
    )+};
}

macro_rules! impl_shifts {
    ($ty:ty) => {
        impl ShlChecked for $ty {
            type Output = Self;

            fn checked_shl(self, rhs: u32) -> Option<Self> { self.checked_shl(rhs) }
        }

        impl ShlOverflowing for $ty {
            type Output = Self;

            fn overflowing_shl(self, rhs: u32) -> (Self, bool) { self.overflowing_shl(rhs) }
        }

        impl ShlWrapping for $ty {
            type Output = Self;

            fn wrapping_shl(self, rhs: u32) -> Self { self.wrapping_shl(rhs) }
        }

        impl ShrChecked for $ty {
            type Output = Self;

            fn checked_shr(self, rhs: u32) -> Option<Self> { self.checked_shr(rhs) }
        }

        impl ShrOverflowing for $ty {
            type Output = Self;

            fn overflowing_shr(self, rhs: u32) -> (Self, bool) { self.overflowing_shr(rhs) }
        }

        impl ShrWrapping for $ty {
            type Output = Self;

            fn wrapping_shr(self, rhs: u32) -> Self { self.wrapping_shr(rhs) }
        }
    };
}

macro_rules! impl_primitive {
    ($ty:ty, $category:ident) => {
        impl_number!($ty);
        $category!($ty);
        impl_arithmetic!($ty, $ty);
        impl_div_rem!($ty, $ty);
        impl_shifts!($ty);

        impl NegWrapping for $ty {
            type Output = Self;

            fn wrapping_neg(self) -> Self { self.wrapping_neg() }
        }
    };
}

impl_primitive!(u8, impl_natural);
impl_primitive!(u16, impl_natural);
impl_primitive!(u32, impl_natural);
impl_primitive!(u64, impl_natural);
impl_primitive!(u128, impl_natural);
impl_primitive!(usize, impl_natural);
impl_primitive!(i8, impl_integer);
impl_primitive!(i16, impl_integer);
impl_primitive!(i32, impl_integer);
impl_primitive!(i64, impl_integer);
impl_primitive!(i128, impl_integer);
impl_primitive!(isize, impl_integer);
impl_signed_negation!(i8, i16, i32, i64, i128, isize);

macro_rules! impl_nonzero {
    ($nz:ty, $inner:ty) => {
        impl NonZero for $nz {
            type Inner = $inner;

            fn get(self) -> $inner { self.get() }
        }

        impl AddChecked for $nz {
            type Output = Self;

            fn checked_add(self, rhs: Self) -> Option<Self> {
                self.get().checked_add(rhs.get()).and_then(Self::new)
            }
        }

        impl SubChecked for $nz {
            type Output = Self;

            fn checked_sub(self, rhs: Self) -> Option<Self> {
                self.get().checked_sub(rhs.get()).and_then(Self::new)
            }
        }

        impl MulChecked for $nz {
            type Output = Self;

            fn checked_mul(self, rhs: Self) -> Option<Self> {
                self.get().checked_mul(rhs.get()).and_then(Self::new)
            }
        }

        impl DivChecked for $nz {
            type Output = Self;

            fn checked_div(self, rhs: Self) -> Option<Self> {
                self.get().checked_div(rhs.get()).and_then(Self::new)
            }
        }

        impl RemChecked for $nz {
            type Output = Self;

            fn checked_rem(self, rhs: Self) -> Option<Self> {
                self.get().checked_rem(rhs.get()).and_then(Self::new)
            }
        }
    };
}

impl_nonzero!(NonZeroU8, u8);
impl_nonzero!(NonZeroU16, u16);
impl_nonzero!(NonZeroU32, u32);
impl_nonzero!(NonZeroU64, u64);
impl_nonzero!(NonZeroU128, u128);
impl_nonzero!(NonZeroUsize, usize);
impl_nonzero!(NonZeroI8, i8);
impl_nonzero!(NonZeroI16, i16);
impl_nonzero!(NonZeroI32, i32);
impl_nonzero!(NonZeroI64, i64);
impl_nonzero!(NonZeroI128, i128);
impl_nonzero!(NonZeroIsize, isize);

macro_rules! impl_nonzero_negation {
    ($($ty:ty),+ $(,)?) => {$ (
        impl NegChecked for $ty {
            type Output = Self;

            fn checked_neg(self) -> Option<Self> {
                self.get().checked_neg().and_then(Self::new)
            }
        }
    )+};
}
impl_nonzero_negation!(NonZeroI8, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI128, NonZeroIsize);
