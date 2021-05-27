// Rust language amplification library providing multiple generic trait
// implementations, type wrappers, derive macros and other language enhancements
//
// Written in 2014 by
//     Andrew Poelstra <apoelstra@wpsoftware.net>
// Updated in 2020-2021 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the MIT License
// along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use core::ops::{
    Add, AddAssign, Sub, SubAssign, Mul, MulAssign, Div, DivAssign, Rem, RemAssign, BitAnd,
    BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Shl, ShlAssign, Shr, ShrAssign,
};
use core::ops::Deref;
use core::convert::TryFrom;

use crate::error::OverflowError;

macro_rules! construct_smallint {
    ($ty:ident, $inner:ident, $bits:literal, $max:expr, $doc:meta) => {
        #[$doc]
        #[derive(PartialEq, Eq, Debug, Copy, Clone, Default, PartialOrd, Ord, Hash)]
        #[cfg_attr(
            feature = "serde",
            derive(Serialize, Deserialize),
            serde(transparent)
        )]
        #[allow(non_camel_case_types)]
        pub struct $ty($inner);

        impl $ty {
            /// Bit dimension
            pub const BITS: u32 = $bits;

            /// Minimum value
            pub const MIN: Self = Self(0);

            /// Maximal value
            pub const MAX: Self = Self($max - 1);

            /// One value
            pub const ONE: Self = Self(1);

            /// One value
            pub const ZERO: Self = Self(0);

            /// Returns inner `u8` representation, which is always less or equal to `Self::MAX`
            pub fn as_u8(self) -> $inner {
                self.0 as $inner
            }

            /// Creates a new value from a provided `value.
            ///
            /// Panics if the value exceeds `Self::MAX`
            pub fn with(value: $inner) -> Self {
                assert!(value < $max, "provided value exceeds Self::MAX");
                Self(value)
            }
        }

        impl ::core::convert::TryFrom<$inner> for $ty {
            type Error = OverflowError;
            #[inline]
            fn try_from(value: $inner) -> Result<Self, Self::Error> {
                if value >= $max {
                    Err(OverflowError { max: $max as usize - 1, value: value as usize })
                } else {
                    Ok(Self(value))
                }
            }
        }

        impl From<$ty> for $inner {
            #[inline]
            fn from(val: $ty) -> Self {
                val.0
            }
        }

        impl AsRef<$inner> for $ty {
            #[inline]
            fn as_ref(&self) -> &$inner {
                &self.0
            }
        }

        impl Deref for $ty {
            type Target = $inner;

            #[inline]
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        #[cfg(feature = "std")]
        impl ::std::str::FromStr for $ty {
            type Err = ::std::num::ParseIntError;
            #[inline]
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Self::try_from($inner::from_str(s)?).map_err(|_| u8::from_str("257").unwrap_err())
            }
        }

        #[cfg(feature = "std")]
        impl ::std::fmt::Display for $ty {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                self.0.fmt(f)
            }
        }

        impl_op!($ty, $inner, Add, add, AddAssign, add_assign, +);
        impl_op!($ty, $inner, Sub, sub, SubAssign, sub_assign, -);
        impl_op!($ty, $inner, Mul, mul, MulAssign, mul_assign, *);
        impl_op!($ty, $inner, Div, div, DivAssign, div_assign, /);
        impl_op!($ty, $inner, Rem, rem, RemAssign, rem_assign, %);
        impl_op!($ty, $inner, BitAnd, bitand, BitAndAssign, bitand_assign, &);
        impl_op!($ty, $inner, BitOr, bitor, BitOrAssign, bitor_assign, |);
        impl_op!($ty, $inner, BitXor, bitxor, BitXorAssign, bitxor_assign, ^);
        impl_op!($ty, $inner, Shl, shl, ShlAssign, shl_assign, <<);
        impl_op!($ty, $inner, Shr, shr, ShrAssign, shr_assign, >>);
    };
}
macro_rules! impl_op {
    ($ty:ty, $inner:ty, $op:ident, $fn:ident, $op_assign:ident, $fn_assign:ident, $sign:tt) => {
        impl $op for $ty {
            type Output = $ty;
            #[inline]
            fn $fn(self, rhs: Self) -> Self::Output {
                Self::try_from((self.0).$fn(rhs.0)).expect(stringify!(
                    "integer overflow during ",
                    $fn,
                    " operation"
                ))
            }
        }
        impl $op for &$ty {
            type Output = $ty;
            #[inline]
            fn $fn(self, rhs: Self) -> Self::Output {
                *self $sign *rhs
            }
        }
        impl $op<&$ty> for $ty {
            type Output = $ty;
            #[inline]
            fn $fn(self, rhs: &$ty) -> Self::Output {
                self $sign *rhs
            }
        }
        impl $op<$ty> for &$ty {
            type Output = $ty;
            #[inline]
            fn $fn(self, rhs: $ty) -> Self::Output {
                *self $sign rhs
            }
        }

        impl $op<$inner> for $ty {
            type Output = $ty;
            #[inline]
            fn $fn(self, rhs: $inner) -> Self::Output {
                Self::try_from((self.0).$fn(rhs)).expect(stringify!(
                    "integer overflow during ",
                    $fn,
                    " operation"
                ))
            }
        }
        impl $op<&$inner> for &$ty {
            type Output = $ty;
            #[inline]
            fn $fn(self, rhs: &$inner) -> Self::Output {
                *self $sign *rhs
            }
        }
        impl $op<&$inner> for $ty {
            type Output = $ty;
            #[inline]
            fn $fn(self, rhs: &$inner) -> Self::Output {
                self $sign *rhs
            }
        }
        impl $op<$inner> for &$ty {
            type Output = $ty;
            #[inline]
            fn $fn(self, rhs: $inner) -> Self::Output {
                *self $sign rhs
            }
        }

        impl $op_assign for $ty {
            #[inline]
            fn $fn_assign(&mut self, rhs: Self) {
                self.0 = (*self $sign rhs).0
            }
        }
        impl $op_assign<&$ty> for $ty {
            #[inline]
            fn $fn_assign(&mut self, rhs: &$ty) {
                self.0 = (*self $sign *rhs).0
            }
        }
        impl $op_assign<$inner> for $ty {
            #[inline]
            fn $fn_assign(&mut self, rhs: $inner) {
                self.0 = (*self $sign rhs).0
            }
        }
        impl $op_assign<&$inner> for $ty {
            #[inline]
            fn $fn_assign(&mut self, rhs: &$inner) {
                self.0 = (*self $sign *rhs).0
            }
        }
    };
}

construct_smallint!(
    u2,
    u8,
    2,
    4,
    doc = "5-bit unsigned integer in the range `0..4`"
);
construct_smallint!(
    u3,
    u8,
    3,
    8,
    doc = "5-bit unsigned integer in the range `0..8`"
);
construct_smallint!(
    u4,
    u8,
    4,
    16,
    doc = "5-bit unsigned integer in the range `0..16`"
);
construct_smallint!(
    u5,
    u8,
    5,
    32,
    doc = "5-bit unsigned integer in the range `0..32`"
);
construct_smallint!(
    u6,
    u8,
    6,
    64,
    doc = "6-bit unsigned integer in the range `0..64`"
);
construct_smallint!(
    u7,
    u8,
    7,
    128,
    doc = "7-bit unsigned integer in the range `0..128`"
);
construct_smallint!(
    u24,
    u32,
    24,
    1u32 << 24,
    doc = "24-bit unsigned integer in the range `0..16_777_216`"
);

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn ubit_test() {
        let mut u_2 = u2::try_from(*u2::MAX).unwrap();
        let mut u_3 = u3::try_from(*u3::MAX).unwrap();
        let mut u_4 = u4::try_from(*u4::MAX).unwrap();
        let mut u_5 = u5::try_from(*u5::MAX).unwrap();
        let mut u_6 = u6::try_from(*u6::MAX).unwrap();
        let mut u_7 = u7::try_from(*u7::MAX).unwrap();
        let mut u_24 = u24::try_from(*u24::MAX).unwrap();

        assert_eq!(u_2, u2::with(3));
        assert_eq!(u_3, u3::with(7));
        assert_eq!(u_4, u4::with(15));
        assert_eq!(u_5, u5::with(31));
        assert_eq!(u_6, u6::with(63));
        assert_eq!(u_7, u7::with(127));

        assert_eq!(*u_2, 3u8);
        assert_eq!(*u_3, 7u8);
        assert_eq!(*u_4, 15u8);
        assert_eq!(*u_5, 31u8);
        assert_eq!(*u_6, 63u8);
        assert_eq!(*u_7, 127u8);
        assert_eq!(*u_24, (1 << 24) - 1);

        u_2 -= 1;
        u_3 -= 1;
        u_4 -= 1;
        u_5 -= 1;
        u_6 -= 1;
        u_7 -= 1;
        u_24 -= 1;

        assert_eq!(*u_2, 2u8);
        assert_eq!(*u_3, 6u8);
        assert_eq!(*u_4, 14u8);
        assert_eq!(*u_5, 30u8);
        assert_eq!(*u_6, 62u8);
        assert_eq!(*u_7, 126u8);
        assert_eq!(*u_24, (1 << 24) - 2);

        u_2 /= 2;
        u_2 *= 2;
        u_2 += 1;

        u_3 /= 2;
        u_3 *= 2;
        u_3 += 1;

        u_4 /= 2;
        u_4 *= 2;
        u_4 += 1;

        u_5 /= 2;
        u_5 *= 2;
        u_5 += 1;

        u_6 /= 2;
        u_6 *= 2;
        u_6 += 1;

        u_7 /= 2;
        u_7 *= 2;
        u_7 += 1;

        u_24 /= 2;
        u_24 *= 2;
        u_24 += 1;

        assert_eq!(*u_2, 3u8);
        assert_eq!(*u_3, 7u8);
        assert_eq!(*u_4, 15u8);
        assert_eq!(*u_5, 31u8);
        assert_eq!(*u_6, 63u8);
        assert_eq!(*u_7, 127u8);
        assert_eq!(*u_24, (1 << 24) - 1);

        assert_eq!(*u_2 % 2, 1);
        assert_eq!(*u_3 % 2, 1);
        assert_eq!(*u_4 % 2, 1);
        assert_eq!(*u_5 % 2, 1);
        assert_eq!(*u_6 % 2, 1);
        assert_eq!(*u_7 % 2, 1);
        assert_eq!(*u_24 % 2, 1);
    }

    #[test]
    #[should_panic(expected = "OverflowError { max: 3, value: 4 }")]
    fn u2_overflow_test() {
        u2::try_from(4).unwrap();
    }

    #[test]
    #[should_panic(expected = "OverflowError { max: 7, value: 8 }")]
    fn u3_overflow_test() {
        u3::try_from(8).unwrap();
    }

    #[test]
    #[should_panic(expected = "OverflowError { max: 15, value: 16 }")]
    fn u4_overflow_test() {
        u4::try_from(16).unwrap();
    }

    #[test]
    #[should_panic(expected = "OverflowError { max: 31, value: 32 }")]
    fn u5_overflow_test() {
        u5::try_from(32).unwrap();
    }

    #[test]
    #[should_panic(expected = "OverflowError { max: 63, value: 64 }")]
    fn u6_overflow_test() {
        u6::try_from(64).unwrap();
    }

    #[test]
    #[should_panic(expected = "OverflowError { max: 127, value: 128 }")]
    fn u7_overflow_test() {
        u7::try_from(128).unwrap();
    }

    #[test]
    #[should_panic(expected = "OverflowError { max: 16777215, value: 16777216 }")]
    fn u24_overflow_test() {
        u24::try_from(1 << 24).unwrap();
    }
}
