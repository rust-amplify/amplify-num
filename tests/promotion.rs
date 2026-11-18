//! Integration tests of the arithmetic of operands of different integer types.
//!
//! Every pair of types which the crate allows to combine is tested in both
//! orders of the operands: each operator and each arithmetic trait must return
//! the wider type and give the same result as the operation of that type made
//! on the losslessly converted operands. The lists of the pairs mirror the
//! lists of the lossless conversions provided by the library, which are
//! checked to keep the values. The last block checks that the conversions
//! and the operations which would lose the values are not provided.

use core::fmt::{Debug, Display};
use core::ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Rem, Sub};

use amplify_num::{
    AddChecked, AddOverflowing, AddSaturate, AddWrapping, BigInt, DivChecked, DivOverflowing,
    DivSaturate, DivWrapping, MulChecked, MulOverflowing, MulSaturate, MulWrapping, Number,
    RemChecked, RemOverflowing, RemWrapping, SubChecked, SubOverflowing, SubSaturate, SubWrapping,
};

// ---------------------------------------------------------------------------
// Test harness
// ---------------------------------------------------------------------------

/// All binary operators and arithmetic traits taking the right operand of the
/// type `R` and returning the type `O`.
trait Mixed<R, O>:
    Copy
    + Debug
    + Add<R, Output = O>
    + Sub<R, Output = O>
    + Mul<R, Output = O>
    + Div<R, Output = O>
    + Rem<R, Output = O>
    + BitAnd<R, Output = O>
    + BitOr<R, Output = O>
    + BitXor<R, Output = O>
    + AddChecked<R, Output = O>
    + AddOverflowing<R, Output = O>
    + AddSaturate<R, Output = O>
    + AddWrapping<R, Output = O>
    + SubChecked<R, Output = O>
    + SubOverflowing<R, Output = O>
    + SubSaturate<R, Output = O>
    + SubWrapping<R, Output = O>
    + MulChecked<R, Output = O>
    + MulOverflowing<R, Output = O>
    + MulSaturate<R, Output = O>
    + MulWrapping<R, Output = O>
    + DivChecked<R, Output = O>
    + DivOverflowing<R, Output = O>
    + DivSaturate<R, Output = O>
    + DivWrapping<R, Output = O>
    + RemChecked<R, Output = O>
    + RemOverflowing<R, Output = O>
    + RemWrapping<R, Output = O>
{
}

impl<L, R, O> Mixed<R, O> for L where L: Copy
        + Debug
        + Add<R, Output = O>
        + Sub<R, Output = O>
        + Mul<R, Output = O>
        + Div<R, Output = O>
        + Rem<R, Output = O>
        + BitAnd<R, Output = O>
        + BitOr<R, Output = O>
        + BitXor<R, Output = O>
        + AddChecked<R, Output = O>
        + AddOverflowing<R, Output = O>
        + AddSaturate<R, Output = O>
        + AddWrapping<R, Output = O>
        + SubChecked<R, Output = O>
        + SubOverflowing<R, Output = O>
        + SubSaturate<R, Output = O>
        + SubWrapping<R, Output = O>
        + MulChecked<R, Output = O>
        + MulOverflowing<R, Output = O>
        + MulSaturate<R, Output = O>
        + MulWrapping<R, Output = O>
        + DivChecked<R, Output = O>
        + DivOverflowing<R, Output = O>
        + DivSaturate<R, Output = O>
        + DivWrapping<R, Output = O>
        + RemChecked<R, Output = O>
        + RemOverflowing<R, Output = O>
        + RemWrapping<R, Output = O>
{
}

/// Checks every operation of `lhs` with `rhs` against the same operation of
/// the output type made on the converted operands.
fn check_operands<L, R, O>(lhs: L, rhs: R)
where
    L: Mixed<R, O> + Into<O>,
    R: Copy + Debug + Into<O>,
    O: Mixed<O, O> + Number,
{
    let (x, y): (O, O) = (lhs.into(), rhs.into());

    assert_eq!(lhs.checked_add(rhs), x.checked_add(y), "{lhs:?} + {rhs:?}");
    assert_eq!(lhs.overflowing_add(rhs), x.overflowing_add(y), "{lhs:?} + {rhs:?}");
    assert_eq!(lhs.saturating_add(rhs), x.saturating_add(y), "{lhs:?} + {rhs:?}");
    assert_eq!(lhs.wrapping_add(rhs), x.wrapping_add(y), "{lhs:?} + {rhs:?}");

    assert_eq!(lhs.checked_sub(rhs), x.checked_sub(y), "{lhs:?} - {rhs:?}");
    assert_eq!(lhs.overflowing_sub(rhs), x.overflowing_sub(y), "{lhs:?} - {rhs:?}");
    assert_eq!(lhs.saturating_sub(rhs), x.saturating_sub(y), "{lhs:?} - {rhs:?}");
    assert_eq!(lhs.wrapping_sub(rhs), x.wrapping_sub(y), "{lhs:?} - {rhs:?}");

    assert_eq!(lhs.checked_mul(rhs), x.checked_mul(y), "{lhs:?} * {rhs:?}");
    assert_eq!(lhs.overflowing_mul(rhs), x.overflowing_mul(y), "{lhs:?} * {rhs:?}");
    assert_eq!(lhs.saturating_mul(rhs), x.saturating_mul(y), "{lhs:?} * {rhs:?}");
    assert_eq!(lhs.wrapping_mul(rhs), x.wrapping_mul(y), "{lhs:?} * {rhs:?}");

    assert_eq!(lhs.checked_div(rhs), x.checked_div(y), "{lhs:?} / {rhs:?}");
    assert_eq!(lhs.checked_rem(rhs), x.checked_rem(y), "{lhs:?} % {rhs:?}");
    // All other flavors of the division panic if the divisor is zero.
    if !y.is_zero() {
        assert_eq!(lhs.overflowing_div(rhs), x.overflowing_div(y), "{lhs:?} / {rhs:?}");
        assert_eq!(lhs.saturating_div(rhs), x.saturating_div(y), "{lhs:?} / {rhs:?}");
        assert_eq!(lhs.wrapping_div(rhs), x.wrapping_div(y), "{lhs:?} / {rhs:?}");
        assert_eq!(lhs.overflowing_rem(rhs), x.overflowing_rem(y), "{lhs:?} % {rhs:?}");
        assert_eq!(lhs.wrapping_rem(rhs), x.wrapping_rem(y), "{lhs:?} % {rhs:?}");
    }

    // The arithmetic operators panic on overflow, thus they are checked only
    // when the result exists.
    if let Some(sum) = x.checked_add(y) {
        assert_eq!(lhs + rhs, sum, "{lhs:?} + {rhs:?}");
    }
    if let Some(difference) = x.checked_sub(y) {
        assert_eq!(lhs - rhs, difference, "{lhs:?} - {rhs:?}");
    }
    if let Some(product) = x.checked_mul(y) {
        assert_eq!(lhs * rhs, product, "{lhs:?} * {rhs:?}");
    }
    if let Some(quotient) = x.checked_div(y) {
        assert_eq!(lhs / rhs, quotient, "{lhs:?} / {rhs:?}");
    }
    if let Some(remainder) = x.checked_rem(y) {
        assert_eq!(lhs % rhs, remainder, "{lhs:?} % {rhs:?}");
    }

    assert_eq!(lhs & rhs, x & y, "{lhs:?} & {rhs:?}");
    assert_eq!(lhs | rhs, x | y, "{lhs:?} | {rhs:?}");
    assert_eq!(lhs ^ rhs, x ^ y, "{lhs:?} ^ {rhs:?}");
}

/// The values of a type which provoke the overflows, the zero divisors and,
/// for the signed types, the negative operands.
fn extremes<T: Number>() -> [T; 4] { [T::MIN, T::ZERO, T::ONE, T::MAX] }

/// Checks the operations having the narrower type `N` on the left and the
/// wider type `W`, which is the type of the result, on the right.
fn narrow_on_the_left<N, W>()
where
    N: Number + Mixed<W, W> + Into<W>,
    W: Number + Mixed<W, W>,
{
    for narrow in extremes::<N>() {
        for wide in extremes::<W>() {
            check_operands::<N, W, W>(narrow, wide);
        }
    }
}

/// Checks the operations having the wider type `W`, which is the type of the
/// result, on the left and the narrower type `N` on the right.
fn narrow_on_the_right<N, W>()
where
    N: Number + Into<W>,
    W: Number + Mixed<N, W> + Mixed<W, W>,
{
    for narrow in extremes::<N>() {
        for wide in extremes::<W>() {
            check_operands::<W, N, W>(wide, narrow);
        }
    }
}

/// Checks that the conversion of the narrower type `N` into the wider type
/// `W` keeps the values, comparing the decimal digits of the numbers.
fn conversion_is_lossless<N, W>()
where
    N: Number + Display + Into<W>,
    W: Number + Display,
{
    assert!(N::BITS <= W::BITS);
    for narrow in extremes::<N>() {
        let wide: W = narrow.into();
        assert_eq!(wide.to_string(), narrow.to_string());
        assert!(W::MIN <= wide && wide <= W::MAX, "{narrow}");
    }
    let (zero, one): (W, W) = (N::ZERO.into(), N::ONE.into());
    assert_eq!((zero, one), (W::ZERO, W::ONE));
}

/// Big integers of the sizes having no aliases in the library: the smallest
/// ones, which fit the fewest primitive types, and the ones made of an odd
/// number of limbs.
mod custom {
    use super::BigInt;

    #[expect(non_camel_case_types)]
    pub type ubig64 = BigInt<1, false>;
    #[expect(non_camel_case_types)]
    pub type ibig64 = BigInt<1, true>;
    #[expect(non_camel_case_types)]
    pub type ubig192 = BigInt<3, false>;
    #[expect(non_camel_case_types)]
    pub type ibig192 = BigInt<3, true>;
}

/// Generates the tests for each pair made of a `narrow` type and a `wide`
/// type: one of the conversion and two of the operations, one per order of the
/// operands.
macro_rules! pairs {
    ([$($narrow:ident)+] with $wide:tt) => {
        $(pairs!(@ $narrow with $wide);)+
    };
    (@ $narrow:ident with [$($wide:ident)+]) => {
        mod $narrow {
            $(
                mod $wide {
                    #[allow(unused_imports, reason = "the primitive types need no import")]
                    use amplify_num::*;

                    #[allow(unused_imports, reason = "only some pairs use the custom sizes")]
                    use crate::custom::*;

                    #[test]
                    fn conversion_is_lossless() {
                        crate::conversion_is_lossless::<$narrow, $wide>();
                    }

                    #[test]
                    fn narrow_on_the_left() { crate::narrow_on_the_left::<$narrow, $wide>(); }

                    #[test]
                    fn narrow_on_the_right() { crate::narrow_on_the_right::<$narrow, $wide>(); }
                }
            )+
        }
    };
}

/// Generates the tests for each pair of the listed types, which must be
/// ordered from the narrowest to the widest one.
macro_rules! widening {
    ($narrow:ident $($wide:ident)+) => {
        pairs!(@ $narrow with [$($wide)+]);
        widening!($($wide)+);
    };
    ($widest:ident) => {};
}

// ---------------------------------------------------------------------------
// Small integers
// ---------------------------------------------------------------------------

/// Each small integer with every wider small integer.
mod small_with_small {
    widening!(u1 u2 u3 u4 u5 u6 u7 u10 u12 u14 u20 u24 u40 u48 u56);
}

/// Each small integer with every primitive integer which fits it.
mod small_with_primitive {
    pairs!([u1 u2 u3 u4 u5 u6 u7] with [u8 u16 u32 u64 u128 usize i8 i16 i32 i64 i128 isize]);
    pairs!([u10 u12 u14] with [u16 u32 u64 u128 usize i16 i32 i64 i128 isize]);
    pairs!([u20 u24] with [u32 u64 u128 i32 i64 i128]);
    pairs!([u40 u48 u56] with [u64 u128 i64 i128]);
}

/// The small integers which fit the pointer-sized integers only on some
/// targets.
mod small_with_pointer_sized {
    #[cfg(not(target_pointer_width = "16"))]
    pairs!([u20 u24] with [usize isize]);
    #[cfg(target_pointer_width = "64")]
    pairs!([u40 u48 u56] with [usize isize]);
}

/// Each unsigned primitive integer with every small integer which fits it.
mod primitive_with_small {
    pairs!([u8] with [u10 u12 u14 u20 u24 u40 u48 u56]);
    pairs!([u16] with [u20 u24 u40 u48 u56]);
    pairs!([u32] with [u40 u48 u56]);
}

// ---------------------------------------------------------------------------
// Big integers
// ---------------------------------------------------------------------------

/// Each small integer with the big integers of both signs.
mod small_with_big {
    pairs!([u1 u2 u3 u4 u5 u6 u7 u10 u12 u14 u20 u24 u40 u48 u56]
        with [ubig64 ibig64 ubig192 ibig192 u256 i256 u512 i512 u1024 i1024]);
}

/// Each unsigned primitive integer with the big integers of both signs.
mod unsigned_primitive_with_big {
    pairs!([u8 u16 u32 u64 u128 usize] with [u256 i256 u512 i512 u1024 i1024]);
}

/// Each signed primitive integer with the signed big integers.
mod signed_primitive_with_big {
    pairs!([i8 i16 i32 i64 i128 isize] with [i256 i512 i1024]);
}

/// The primitive integers with the smallest big integers which fit them: a
/// signed big integer needs one more bit than an unsigned primitive one.
mod primitive_with_smallest_big {
    pairs!([u8 u16 u32] with [ubig64 ibig64]);
    pairs!([u64 usize] with [ubig64 ubig192 ibig192]);
    pairs!([u128] with [ubig192 ibig192]);
    pairs!([i8 i16 i32 i64 isize] with [ibig64 ibig192]);
    pairs!([i128] with [ibig192]);
}

/// Each big integer with the wider big integers of the same sign.
mod big_with_big {
    pairs!([u256] with [u512 u1024]);
    pairs!([u512] with [u1024]);
    pairs!([i256] with [i512 i1024]);
    pairs!([i512] with [i1024]);
}

// ---------------------------------------------------------------------------
// Combinations which are not provided
// ---------------------------------------------------------------------------

/// Evaluates to `true` if the type implements the trait: the inherent constant
/// of the probe, which exists only in that case, shadows the constant of the
/// trait implemented for every type. The result is passed through a black box
/// to be checked when the tests run.
macro_rules! implements {
    ($ty:ty: $($bound:tt)+) => {{
        #[allow(dead_code, reason = "only one of the two constants is used by a probe")]
        trait Fallback {
            const IMPLEMENTS: bool = false;
        }
        impl<T> Fallback for T {}
        struct Probe<T>(core::marker::PhantomData<T>);
        #[allow(dead_code, reason = "only one of the two constants is used by a probe")]
        impl<T: $($bound)+> Probe<T> {
            const IMPLEMENTS: bool = true;
        }
        core::hint::black_box(<Probe<$ty>>::IMPLEMENTS)
    }};
}

/// The conversions which would lose the values are not implemented, and so
/// are the operations of the types none of which fits the other: using them
/// is a compilation error.
///
/// The conversions of the primitive integers into the big integers are
/// implemented for all sizes at once, and the ones which would lose the values
/// are rejected later, when the code is generated. Such a failure can't be
/// checked here: it is covered by the documentation tests of the library.
mod rejected {
    use core::ops::{Add, Neg};

    use amplify_num::*;

    /// The probe itself is checked on the provided combinations.
    #[test]
    fn provided_combinations_are_detected() {
        assert!(implements!(u2: From<u1>));
        assert!(implements!(u10: From<u8>));
        assert!(implements!(i8: From<u7>));
        assert!(implements!(i256: From<i8>));
        assert!(implements!(u24: Add<i32>));
        assert!(implements!(i256: Neg));
    }

    #[test]
    fn small_integers_are_not_narrowed() {
        assert!(!implements!(u1: From<u2>));
        assert!(!implements!(u2: From<u3>));
        assert!(!implements!(u3: From<u4>));
        assert!(!implements!(u4: From<u5>));
        assert!(!implements!(u5: From<u6>));
        assert!(!implements!(u6: From<u7>));
        assert!(!implements!(u7: From<u10>));
        assert!(!implements!(u10: From<u12>));
        assert!(!implements!(u12: From<u14>));
        assert!(!implements!(u14: From<u20>));
        assert!(!implements!(u20: From<u24>));
        assert!(!implements!(u24: From<u40>));
        assert!(!implements!(u40: From<u48>));
        assert!(!implements!(u48: From<u56>));
    }

    #[test]
    fn primitive_integers_are_not_narrowed_into_the_small_ones() {
        assert!(!implements!(u7: From<u8>));
        assert!(!implements!(u14: From<u16>));
        assert!(!implements!(u24: From<u32>));
        assert!(!implements!(u56: From<u64>));
        assert!(!implements!(u56: From<u128>));
        assert!(!implements!(u56: From<usize>));
    }

    #[test]
    fn small_integers_are_not_narrowed_into_the_primitive_ones() {
        assert!(!implements!(u8: From<u10>));
        assert!(!implements!(i8: From<u10>));
        assert!(!implements!(u16: From<u20>));
        assert!(!implements!(i16: From<u20>));
        assert!(!implements!(u32: From<u40>));
        assert!(!implements!(i32: From<u40>));
    }

    /// The pointer-sized integers fit the small ones depending on the target.
    #[test]
    fn small_integers_are_not_narrowed_into_the_pointer_sized_ones() {
        assert!(implements!(usize: From<u14>));
        assert_eq!(implements!(usize: From<u24>), usize::BITS >= 32);
        assert_eq!(implements!(isize: From<u24>), usize::BITS >= 32);
        assert_eq!(implements!(usize: From<u56>), usize::BITS >= 64);
        assert_eq!(implements!(isize: From<u56>), usize::BITS >= 64);
    }

    #[test]
    fn signed_integers_are_not_converted_into_the_unsigned_ones() {
        assert!(!implements!(u10: From<i8>));
        assert!(!implements!(u56: From<i32>));
        assert!(!implements!(u256: From<i8>));
        assert!(!implements!(u256: From<i128>));
        assert!(!implements!(u256: From<isize>));
        assert!(!implements!(u256: From<i256>));
    }

    #[test]
    fn big_integers_are_not_narrowed() {
        assert!(!implements!(u256: From<u512>));
        assert!(!implements!(u512: From<u1024>));
        assert!(!implements!(i256: From<i512>));
        assert!(!implements!(i256: From<u256>));
        assert!(!implements!(u64: From<u256>));
        assert!(!implements!(u56: From<u256>));
    }

    #[test]
    fn integers_which_do_not_fit_each_other_are_not_combined() {
        assert!(!implements!(u24: Add<i8>));
        assert!(!implements!(i8: Add<u24>));
        assert!(!implements!(u40: Add<i32>));
        assert!(!implements!(i32: Add<u40>));
        assert!(!implements!(u256: Add<i32>));
        assert!(!implements!(i32: Add<u256>));
        assert!(!implements!(u256: Add<i256>));
        assert!(!implements!(i256: Add<u256>));
        assert!(!implements!(u24: AddChecked<i8>));
        assert!(!implements!(u256: AddChecked<i32>));
    }

    #[test]
    fn unsigned_integers_are_not_negated() {
        assert!(!implements!(u4: Neg));
        assert!(!implements!(u256: Neg));
        assert!(!implements!(u4: NegChecked));
        assert!(!implements!(u256: NegChecked));
        assert!(!implements!(u256: NegOverflowing));
        assert!(!implements!(u256: NegSaturate));
    }
}
