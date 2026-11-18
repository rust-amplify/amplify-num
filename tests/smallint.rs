//! Integration tests of the small integers: the unsigned integers of a bit
//! size smaller than the size of the primitive integer storing them.
//!
//! The same suite runs for each of the types, with a separate test per case.
//! The expected values are computed on `u128`, which can't overflow.

use core::num::IntErrorKind;
use std::collections::HashSet;

use amplify_num::error::{DivError, OverflowError, ParseIntError};

/// Generates the tests expecting a panic with the given message.
macro_rules! panics {
    ($($name:ident: $msg:literal => $expr:expr;)+) => {$(
        #[test]
        #[should_panic(expected = $msg)]
        fn $name() { let _ = $expr; }
    )+};
}

/// Generates the test suite of a small integer. The arguments are the same
/// as of the macro defining the types in the library.
macro_rules! suite {
    ($($name:ident: $bits:literal bits in $inner:ident as $to_inner:ident;)+) => {$(
        mod $name {
            use amplify_num::*;

            use super::{DivError, HashSet, IntErrorKind, OverflowError, ParseIntError};

            type T = $name;
            type Inner = $inner;

            /// The largest value of the type.
            const MAX: u128 = (1 << $bits) - 1;

            /// The small integer having the given value.
            fn t(value: u128) -> T {
                let inner = Inner::try_from(value).expect("value fits the underlying type");
                T::try_from(inner).expect("value fits the small integer")
            }

            /// The value of a small integer.
            fn v(value: T) -> u128 { u128::from(value.$to_inner()) }

            /// A value of the underlying type.
            fn inner(value: u128) -> Inner {
                Inner::try_from(value).expect("value fits the underlying type")
            }

            // -- Constants --------------------------------------------------

            #[test]
            fn bits_is_the_declared_size() { assert_eq!(T::BITS, $bits); }

            #[test]
            fn bits_is_smaller_than_the_underlying_type() { assert!(T::BITS < Inner::BITS); }

            #[test]
            fn min_is_zero() { assert_eq!(v(T::MIN), 0); }

            #[test]
            fn max_has_all_bits_set() { assert_eq!(v(T::MAX), MAX); }

            #[test]
            fn zero_and_one_have_their_values() { assert_eq!((v(T::ZERO), v(T::ONE)), (0, 1)); }

            #[test]
            fn default_is_zero() { assert_eq!(T::default(), T::ZERO); }

            #[test]
            fn number_constants_match_the_inherent_ones() {
                assert_eq!(<T as Number>::BITS, T::BITS);
                assert_eq!(<T as Number>::MIN, T::MIN);
                assert_eq!(<T as Number>::MAX, T::MAX);
                assert_eq!(<T as Number>::ZERO, T::ZERO);
                assert_eq!(<T as Number>::ONE, T::ONE);
            }

            #[test]
            fn is_zero_holds_only_for_zero() {
                assert!(T::ZERO.is_zero());
                assert!(!T::ONE.is_zero());
                assert!(!T::MAX.is_zero());
            }

            #[test]
            fn is_one_holds_only_for_one() {
                assert!(T::ONE.is_one());
                assert!(!T::ZERO.is_one());
                assert_eq!(T::MAX.is_one(), MAX == 1);
            }

            #[test]
            fn belongs_to_the_unsigned_category() {
                fn assert_unsigned<U: Unsigned>() {}
                assert_unsigned::<T>();
            }

            // -- Construction -----------------------------------------------

            #[test]
            fn with_accepts_the_maximum() { assert_eq!(T::with(inner(MAX)), T::MAX); }

            #[test]
            fn with_works_in_constants() {
                const ONE: T = T::with(1);
                assert_eq!(ONE, T::ONE);
            }

            #[test]
            fn try_from_accepts_the_maximum() { assert_eq!(T::try_from(inner(MAX)), Ok(T::MAX)); }

            #[test]
            fn try_from_rejects_the_value_above_the_maximum() {
                let err = OverflowError { max: inner(MAX), value: inner(MAX + 1) };
                assert_eq!(T::try_from(inner(MAX + 1)), Err(err));
            }

            #[test]
            fn try_from_rejects_the_maximum_of_the_underlying_type() {
                let err = OverflowError { max: inner(MAX), value: Inner::MAX };
                assert_eq!(T::try_from(Inner::MAX), Err(err));
            }

            #[test]
            fn to_inner_returns_the_value() { assert_eq!(T::MAX.$to_inner(), inner(MAX)); }

            #[test]
            fn converts_into_u128_losslessly() { assert_eq!(u128::from(T::MAX), MAX); }

            // -- Parsing ----------------------------------------------------

            #[test]
            fn parses_the_maximum() { assert_eq!(MAX.to_string().parse(), Ok(T::MAX)); }

            #[test]
            fn parses_the_maximum_in_other_radixes() {
                assert_eq!(T::from_str_radix(&format!("{MAX:b}"), 2), Ok(T::MAX));
                assert_eq!(T::from_str_radix(&format!("{MAX:o}"), 8), Ok(T::MAX));
                assert_eq!(T::from_str_radix(&format!("{MAX:x}"), 16), Ok(T::MAX));
                assert_eq!(T::from_str_radix(&format!("{MAX:X}"), 16), Ok(T::MAX));
            }

            #[test]
            fn parses_the_largest_radix() { assert_eq!(T::from_str_radix("1", 36), Ok(T::ONE)); }

            #[test]
            fn parses_a_plus_sign() { assert_eq!("+1".parse(), Ok(T::ONE)); }

            #[test]
            fn parses_leading_zeros() { assert_eq!("0000000000000000000001".parse(), Ok(T::ONE)); }

            #[test]
            fn parsing_rejects_the_value_above_the_maximum() {
                let err = ParseIntError(IntErrorKind::PosOverflow);
                assert_eq!((MAX + 1).to_string().parse::<T>(), Err(err));
            }

            #[test]
            fn parsing_rejects_the_value_above_the_underlying_type() {
                let err = ParseIntError(IntErrorKind::PosOverflow);
                assert_eq!(u128::MAX.to_string().parse::<T>(), Err(err));
            }

            #[test]
            fn parsing_rejects_an_empty_string() {
                assert_eq!("".parse::<T>(), Err(ParseIntError(IntErrorKind::Empty)));
            }

            #[test]
            fn parsing_rejects_a_minus_sign() {
                let err = ParseIntError(IntErrorKind::InvalidDigit);
                assert_eq!("-1".parse::<T>(), Err(err));
                assert_eq!("-0".parse::<T>(), Err(err));
            }

            #[test]
            fn parsing_rejects_a_sign_without_digits() {
                let err = ParseIntError(IntErrorKind::InvalidDigit);
                assert_eq!("+".parse::<T>(), Err(err));
                assert_eq!("-".parse::<T>(), Err(err));
            }

            #[test]
            fn parsing_rejects_separators_and_prefixes() {
                let err = ParseIntError(IntErrorKind::InvalidDigit);
                for src in [" 1", "1 ", "1_0", "0x1", "1.0", "١"] {
                    assert_eq!(src.parse::<T>(), Err(err), "{src:?}");
                }
            }

            #[test]
            fn parsing_rejects_the_digits_outside_of_the_radix() {
                let err = ParseIntError(IntErrorKind::InvalidDigit);
                assert_eq!(T::from_str_radix("2", 2), Err(err));
                assert_eq!(T::from_str_radix("g", 16), Err(err));
            }

            panics! {
                with_rejects_the_value_above_the_maximum: "provided value exceeds Self::MAX" =>
                    T::with(inner(MAX + 1));
                from_str_radix_rejects_the_radix_below_2: "radix must lie in the range" =>
                    T::from_str_radix("0", 1);
                from_str_radix_rejects_the_radix_above_36: "radix must lie in the range" =>
                    T::from_str_radix("0", 37);
            }

            // -- Addition ---------------------------------------------------

            #[test]
            fn add_within_the_range() {
                assert_eq!(T::ZERO + T::ONE, T::ONE);
                assert_eq!(T::ZERO.checked_add(T::MAX), Some(T::MAX));
                assert_eq!(T::ZERO.overflowing_add(T::MAX), (T::MAX, false));
                assert_eq!(T::ZERO.saturating_add(T::MAX), T::MAX);
                assert_eq!(T::ZERO.wrapping_add(T::MAX), T::MAX);
            }

            #[test]
            fn checked_add_rejects_an_overflow() { assert_eq!(T::MAX.checked_add(T::ONE), None); }

            #[test]
            fn overflowing_add_wraps_and_reports() {
                assert_eq!(T::MAX.overflowing_add(T::ONE), (T::ZERO, true));
                assert_eq!(T::MAX.overflowing_add(T::MAX), (t(MAX - 1), true));
            }

            #[test]
            fn saturating_add_stops_at_the_maximum() {
                assert_eq!(T::MAX.saturating_add(T::ONE), T::MAX);
                assert_eq!(T::MAX.saturating_add(T::MAX), T::MAX);
            }

            #[test]
            fn wrapping_add_wraps_at_the_bit_size() {
                assert_eq!(T::MAX.wrapping_add(T::ONE), T::ZERO);
                assert_eq!(T::MAX.wrapping_add(T::MAX), t(MAX - 1));
            }

            // -- Subtraction ------------------------------------------------

            #[test]
            fn sub_within_the_range() {
                assert_eq!(T::MAX - T::MAX, T::ZERO);
                assert_eq!(T::MAX.checked_sub(T::ONE), Some(t(MAX - 1)));
                assert_eq!(T::MAX.overflowing_sub(T::ONE), (t(MAX - 1), false));
                assert_eq!(T::MAX.saturating_sub(T::ONE), t(MAX - 1));
                assert_eq!(T::MAX.wrapping_sub(T::ONE), t(MAX - 1));
            }

            #[test]
            fn checked_sub_rejects_an_overflow() { assert_eq!(T::ZERO.checked_sub(T::ONE), None); }

            #[test]
            fn overflowing_sub_wraps_and_reports() {
                assert_eq!(T::ZERO.overflowing_sub(T::ONE), (T::MAX, true));
                assert_eq!(T::ZERO.overflowing_sub(T::MAX), (T::ONE, true));
            }

            #[test]
            fn saturating_sub_stops_at_zero() {
                assert_eq!(T::ZERO.saturating_sub(T::ONE), T::ZERO);
                assert_eq!(T::ZERO.saturating_sub(T::MAX), T::ZERO);
            }

            #[test]
            fn wrapping_sub_wraps_at_the_bit_size() {
                assert_eq!(T::ZERO.wrapping_sub(T::ONE), T::MAX);
                assert_eq!(T::ZERO.wrapping_sub(T::MAX), T::ONE);
            }

            // -- Multiplication ---------------------------------------------

            #[test]
            fn mul_within_the_range() {
                assert_eq!(T::MAX * T::ONE, T::MAX);
                assert_eq!(T::MAX * T::ZERO, T::ZERO);
                assert_eq!(T::MAX.checked_mul(T::ONE), Some(T::MAX));
                assert_eq!(T::MAX.overflowing_mul(T::ONE), (T::MAX, false));
                assert_eq!(T::MAX.saturating_mul(T::ONE), T::MAX);
                assert_eq!(T::MAX.wrapping_mul(T::ONE), T::MAX);
            }

            /// The square of the maximum overflows every type but the one-bit
            /// one, and the underlying type of some of them as well.
            #[test]
            fn mul_of_the_maximums_matches_the_exact_product() {
                let exact = MAX * MAX;
                let (fits, wrapped) = (exact <= MAX, t(exact % (MAX + 1)));
                assert_eq!(T::MAX.checked_mul(T::MAX), fits.then_some(wrapped));
                assert_eq!(T::MAX.overflowing_mul(T::MAX), (wrapped, !fits));
                assert_eq!(T::MAX.saturating_mul(T::MAX), T::MAX);
                assert_eq!(T::MAX.wrapping_mul(T::MAX), wrapped);
                assert_eq!(wrapped, T::ONE);
            }

            // -- Division and remainder -------------------------------------

            #[test]
            fn div_within_the_range() {
                assert_eq!(T::MAX / T::ONE, T::MAX);
                assert_eq!(T::MAX / T::MAX, T::ONE);
                assert_eq!(T::ZERO / T::MAX, T::ZERO);
                assert_eq!(T::MAX.checked_div(T::MAX), Some(T::ONE));
                assert_eq!(T::MAX.overflowing_div(T::MAX), (T::ONE, false));
                assert_eq!(T::MAX.saturating_div(T::MAX), T::ONE);
                assert_eq!(T::MAX.wrapping_div(T::MAX), T::ONE);
            }

            #[test]
            fn rem_within_the_range() {
                assert_eq!(T::MAX % T::MAX, T::ZERO);
                assert_eq!(T::MAX % T::ONE, T::ZERO);
                assert_eq!(T::ZERO % T::MAX, T::ZERO);
                assert_eq!(T::MAX.checked_rem(T::MAX), Some(T::ZERO));
                assert_eq!(T::MAX.overflowing_rem(T::MAX), (T::ZERO, false));
                assert_eq!(T::MAX.wrapping_rem(T::MAX), T::ZERO);
            }

            #[test]
            fn checked_div_rejects_a_zero_divisor() {
                assert_eq!(T::MAX.checked_div(T::ZERO), None);
            }

            #[test]
            fn checked_rem_rejects_a_zero_divisor() {
                assert_eq!(T::MAX.checked_rem(T::ZERO), None);
            }

            #[test]
            fn div_rem_returns_the_quotient_and_the_remainder() {
                assert_eq!(T::MAX.div_rem(T::ONE), Ok((T::MAX, T::ZERO)));
                assert_eq!(T::MAX.div_rem(T::MAX), Ok((T::ONE, T::ZERO)));
            }

            #[test]
            fn div_rem_rejects_a_zero_divisor() {
                assert_eq!(T::MAX.div_rem(T::ZERO), Err(DivError::ZeroDiv));
                assert_eq!(T::MAX.div_rem(inner(0)), Err(DivError::ZeroDiv));
            }

            /// The divisor of `div_rem` has the underlying type, thus it may
            /// exceed the range of the small integer.
            #[test]
            fn div_rem_accepts_a_divisor_above_the_maximum() {
                assert_eq!(T::MAX.div_rem(Inner::MAX), Ok((T::ZERO, T::MAX)));
            }

            panics! {
                div_by_zero_panics: "attempt to divide by zero" => T::MAX / T::ZERO;
                rem_by_zero_panics: "attempt to calculate the remainder with a divisor of zero" =>
                    T::MAX % T::ZERO;
                overflowing_div_by_zero_panics: "attempt to divide by zero" =>
                    T::MAX.overflowing_div(T::ZERO);
                saturating_div_by_zero_panics: "attempt to divide by zero" =>
                    T::MAX.saturating_div(T::ZERO);
                wrapping_div_by_zero_panics: "attempt to divide by zero" =>
                    T::MAX.wrapping_div(T::ZERO);
                overflowing_rem_by_zero_panics:
                    "attempt to calculate the remainder with a divisor of zero" =>
                    T::MAX.overflowing_rem(T::ZERO);
                wrapping_rem_by_zero_panics:
                    "attempt to calculate the remainder with a divisor of zero" =>
                    T::MAX.wrapping_rem(T::ZERO);
            }

            // -- Negation ---------------------------------------------------

            #[test]
            fn wrapping_neg_gives_the_complement_to_the_bit_size() {
                assert_eq!(T::ZERO.wrapping_neg(), T::ZERO);
                assert_eq!(T::ONE.wrapping_neg(), T::MAX);
                assert_eq!(T::MAX.wrapping_neg(), T::ONE);
            }

            // -- Operators panicking on overflow ----------------------------

            panics! {
                add_overflow_panics: "attempt to add with overflow" => T::MAX + T::ONE;
                sub_overflow_panics: "attempt to subtract with overflow" => T::ZERO - T::ONE;
                add_assign_overflow_panics: "attempt to add with overflow" => {
                    let mut value = T::MAX;
                    value += 1u8;
                    value
                };
                add_assign_overflow_of_the_underlying_type_panics:
                    "attempt to add with overflow" => {
                    let mut value = T::ONE;
                    value += Inner::MAX;
                    value
                };
                sub_assign_overflow_panics: "attempt to subtract with overflow" => {
                    let mut value = T::ZERO;
                    value -= 1u8;
                    value
                };
                mul_assign_overflow_panics: "attempt to multiply with overflow" => {
                    let mut value = T::MAX;
                    value *= 2u8;
                    value
                };
                mul_assign_by_the_maximum_of_the_underlying_type_panics:
                    "attempt to multiply with overflow" => {
                    let mut value = T::MAX;
                    value *= Inner::MAX;
                    value
                };
                div_assign_by_zero_panics: "attempt to divide by zero" => {
                    let mut value = T::MAX;
                    value /= 0u8;
                    value
                };
                rem_assign_by_zero_panics:
                    "attempt to calculate the remainder with a divisor of zero" => {
                    let mut value = T::MAX;
                    value %= 0u8;
                    value
                };
                bitor_assign_above_the_maximum_panics: "attempt to bitor with overflow" => {
                    let mut value = T::ZERO;
                    value |= Inner::MAX;
                    value
                };
                bitxor_assign_above_the_maximum_panics: "attempt to bitxor with overflow" => {
                    let mut value = T::ZERO;
                    value ^= Inner::MAX;
                    value
                };
            }

            // -- Compound assignment ----------------------------------------

            #[test]
            fn add_assign_keeps_the_type() {
                let mut value = T::ZERO;
                value += 1u8;
                assert_eq!(value, T::ONE);
            }

            #[test]
            fn sub_assign_keeps_the_type() {
                let mut value = T::MAX;
                value -= 1u8;
                assert_eq!(value, t(MAX - 1));
            }

            #[test]
            fn mul_assign_keeps_the_type() {
                let mut value = T::MAX;
                value *= 1u8;
                assert_eq!(value, T::MAX);
            }

            #[test]
            fn div_assign_keeps_the_type() {
                let mut value = T::MAX;
                value /= inner(MAX);
                assert_eq!(value, T::ONE);
            }

            #[test]
            fn rem_assign_keeps_the_type() {
                let mut value = T::MAX;
                value %= inner(MAX);
                assert_eq!(value, T::ZERO);
            }

            #[test]
            fn assignment_accepts_an_operand_of_the_same_type() {
                let mut value = T::ZERO;
                value += T::ONE;
                value |= T::MAX;
                value -= T::ONE;
                assert_eq!(value, t(MAX - 1));
            }

            /// The conjunction can't exceed the left operand, thus any operand
            /// of the underlying type is accepted.
            #[test]
            fn bitand_assign_accepts_an_operand_above_the_maximum() {
                let mut value = T::MAX;
                value &= Inner::MAX;
                assert_eq!(value, T::MAX);
                value &= inner(MAX + 1);
                assert_eq!(value, T::ZERO);
            }

            #[test]
            fn bitor_assign_keeps_the_type() {
                let mut value = T::ZERO;
                value |= inner(MAX);
                assert_eq!(value, T::MAX);
            }

            #[test]
            fn bitxor_assign_keeps_the_type() {
                let mut value = T::MAX;
                value ^= inner(MAX);
                assert_eq!(value, T::ZERO);
            }

            // -- Shifts -----------------------------------------------------

            #[test]
            fn shl_by_zero_keeps_the_value() { assert_eq!(T::MAX << 0, T::MAX); }

            #[test]
            fn shl_discards_the_bits_above_the_bit_size() {
                assert_eq!(T::MAX << (T::BITS - 1), t(1 << (T::BITS - 1)));
            }

            #[test]
            fn shr_by_zero_keeps_the_value() { assert_eq!(T::MAX >> 0, T::MAX); }

            #[test]
            fn shr_moves_the_top_bit_to_the_bottom() {
                assert_eq!(T::MAX >> (T::BITS - 1), T::ONE);
            }

            #[test]
            fn shift_assignments_match_the_operators() {
                let mut value = T::ONE;
                value <<= T::BITS - 1;
                assert_eq!(value, T::ONE << (T::BITS - 1));
                value >>= T::BITS - 1;
                assert_eq!(value, T::ONE);
            }

            #[test]
            fn checked_shifts_accept_the_counts_below_the_bit_size() {
                assert_eq!(T::MAX.checked_shl(T::BITS - 1), Some(T::MAX << (T::BITS - 1)));
                assert_eq!(T::MAX.checked_shr(T::BITS - 1), Some(T::ONE));
            }

            #[test]
            fn checked_shifts_reject_the_bit_size() {
                assert_eq!(T::MAX.checked_shl(T::BITS), None);
                assert_eq!(T::MAX.checked_shr(T::BITS), None);
                assert_eq!(T::MAX.checked_shl(u32::MAX), None);
                assert_eq!(T::MAX.checked_shr(u32::MAX), None);
            }

            /// The bit size of the underlying type is not a valid shift either,
            /// though it is the one the underlying type would wrap at.
            #[test]
            fn checked_shifts_reject_the_bit_size_of_the_underlying_type() {
                assert_eq!(T::MAX.checked_shl(Inner::BITS), None);
                assert_eq!(T::MAX.checked_shr(Inner::BITS), None);
            }

            #[test]
            fn overflowing_shifts_wrap_the_count_at_the_bit_size() {
                assert_eq!(T::MAX.overflowing_shl(T::BITS), (T::MAX, true));
                assert_eq!(T::MAX.overflowing_shr(T::BITS), (T::MAX, true));
                assert_eq!(T::MAX.overflowing_shl(0), (T::MAX, false));
                assert_eq!(T::MAX.overflowing_shr(T::BITS - 1), (T::ONE, false));
                let count = 2 * T::BITS - 1;
                assert_eq!(T::MAX.overflowing_shl(count), (T::MAX << (T::BITS - 1), true));
                assert_eq!(T::MAX.overflowing_shr(count), (T::ONE, true));
            }

            #[test]
            fn wrapping_shifts_wrap_the_count_at_the_bit_size() {
                assert_eq!(T::MAX.wrapping_shl(T::BITS), T::MAX);
                assert_eq!(T::MAX.wrapping_shr(T::BITS), T::MAX);
                let count = 2 * T::BITS - 1;
                assert_eq!(T::MAX.wrapping_shl(count), T::MAX << (T::BITS - 1));
                assert_eq!(T::MAX.wrapping_shr(count), T::ONE);
                let count = u32::MAX.rem_euclid(T::BITS);
                assert_eq!(T::MAX.wrapping_shl(u32::MAX), T::MAX << count);
                assert_eq!(T::MAX.wrapping_shr(u32::MAX), T::MAX >> count);
            }

            panics! {
                shl_by_the_bit_size_panics: "attempt to shift left with overflow" =>
                    T::ONE << T::BITS;
                shr_by_the_bit_size_panics: "attempt to shift right with overflow" =>
                    T::ONE >> T::BITS;
                shl_assign_by_the_bit_size_panics: "attempt to shift left with overflow" => {
                    let mut value = T::ONE;
                    value <<= T::BITS;
                    value
                };
                shr_assign_by_the_bit_size_panics: "attempt to shift right with overflow" => {
                    let mut value = T::ONE;
                    value >>= T::BITS;
                    value
                };
            }

            // -- Bitwise operations -----------------------------------------

            #[test]
            fn not_inverts_only_the_bits_of_the_type() {
                assert_eq!(!T::ZERO, T::MAX);
                assert_eq!(!T::MAX, T::ZERO);
                assert_eq!(!T::ONE, t(MAX - 1));
            }

            #[test]
            fn bitwise_operators_follow_their_truth_tables() {
                assert_eq!(T::MAX & T::ZERO, T::ZERO);
                assert_eq!(T::MAX & T::ONE, T::ONE);
                assert_eq!(T::ZERO | T::MAX, T::MAX);
                assert_eq!(T::ZERO | T::ZERO, T::ZERO);
                assert_eq!(T::MAX ^ T::MAX, T::ZERO);
                assert_eq!(T::MAX ^ T::ONE, t(MAX - 1));
            }

            // -- Formatting -------------------------------------------------

            #[test]
            fn display_and_debug_print_the_decimal_value() {
                assert_eq!(T::MAX.to_string(), MAX.to_string());
                assert_eq!(format!("{:?}", T::MAX), MAX.to_string());
                assert_eq!(T::ZERO.to_string(), "0");
            }

            #[test]
            fn radix_formats_match_the_primitive_integers() {
                assert_eq!(format!("{:b}", T::MAX), format!("{MAX:b}"));
                assert_eq!(format!("{:o}", T::MAX), format!("{MAX:o}"));
                assert_eq!(format!("{:x}", T::MAX), format!("{MAX:x}"));
                assert_eq!(format!("{:X}", T::MAX), format!("{MAX:X}"));
                assert_eq!(format!("{:x?}", T::MAX), format!("{MAX:x?}"));
                assert_eq!(format!("{:X?}", T::MAX), format!("{MAX:X?}"));
            }

            #[test]
            fn formatting_flags_match_the_primitive_integers() {
                assert_eq!(format!("{:#b}", T::MAX), format!("{MAX:#b}"));
                assert_eq!(format!("{:#o}", T::MAX), format!("{MAX:#o}"));
                assert_eq!(format!("{:#x}", T::MAX), format!("{MAX:#x}"));
                assert_eq!(format!("{:#X}", T::MAX), format!("{MAX:#X}"));
                assert_eq!(format!("{:+}", T::MAX), format!("{MAX:+}"));
                assert_eq!(format!("{:>30}", T::MAX), format!("{MAX:>30}"));
                assert_eq!(format!("{:<30}|", T::MAX), format!("{MAX:<30}|"));
                assert_eq!(format!("{:^30}|", T::MAX), format!("{MAX:^30}|"));
                assert_eq!(format!("{:030}", T::MAX), format!("{MAX:030}"));
                assert_eq!(format!("{:#070b}", T::MAX), format!("{MAX:#070b}"));
            }

            // -- Comparison and hashing -------------------------------------

            #[test]
            fn ordering_follows_the_values() {
                assert!(T::ZERO < T::ONE);
                assert!(T::ONE <= T::MAX);
                assert!(T::MAX > T::ZERO);
                assert_eq!(T::ZERO.max(T::MAX), T::MAX);
                assert_eq!(T::ZERO.min(T::MAX), T::ZERO);
                assert_eq!(T::MAX.cmp(&T::MAX), core::cmp::Ordering::Equal);
            }

            #[test]
            fn equal_values_have_equal_hashes() {
                let set = HashSet::from([T::ZERO, T::MIN, T::MAX, t(MAX), T::default()]);
                assert_eq!(set.len(), 2);
            }
        }
    )+};
}

// ---------------------------------------------------------------------------
// The suite of each type
// ---------------------------------------------------------------------------

suite! {
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

// ---------------------------------------------------------------------------
// Multiplication overflow of the operator
// ---------------------------------------------------------------------------

/// The product of two values of the one-bit integer always fits it, thus the
/// overflow of the multiplication operator is tested for the wider types.
mod mul_overflow_panics {
    macro_rules! mul_overflow {
        ($($name:ident)+) => {$(
            #[test]
            #[should_panic(expected = "attempt to multiply with overflow")]
            fn $name() { let _ = amplify_num::$name::MAX * amplify_num::$name::MAX; }
        )+};
    }

    mul_overflow!(u2 u3 u4 u5 u6 u7 u10 u12 u14 u20 u24 u40 u48 u56);

    #[test]
    fn one_bit_product_never_overflows() {
        use amplify_num::{MulChecked, u1};

        for (a, b) in [(u1::ZERO, u1::ZERO), (u1::ZERO, u1::ONE), (u1::ONE, u1::ONE)] {
            assert_eq!(a.checked_mul(b), Some(a & b));
            assert_eq!(a * b, a & b);
        }
    }
}

// ---------------------------------------------------------------------------
// Named constants of the smallest types
// ---------------------------------------------------------------------------

mod named_constants {
    use amplify_num::{u2, u3};

    #[test]
    fn u2_constants_cover_all_values() {
        assert_eq!([u2::ZERO, u2::ONE, u2::TWO, u2::THREE].map(u2::to_u8), [0, 1, 2, 3]);
        assert_eq!(u2::THREE, u2::MAX);
    }

    #[test]
    fn u3_constants_cover_all_values() {
        let all = [u3::ZERO, u3::ONE, u3::TWO, u3::THREE, u3::FOUR, u3::FIVE, u3::SIX, u3::SEVEN];
        assert_eq!(all.map(u3::to_u8), [0, 1, 2, 3, 4, 5, 6, 7]);
        assert_eq!(u3::SEVEN, u3::MAX);
    }
}

// ---------------------------------------------------------------------------
// Byte conversions of the types made of whole bytes
// ---------------------------------------------------------------------------

/// Generates the tests of the byte conversions. `$be` are the big-endian
/// bytes of the value `$value`.
macro_rules! bytes {
    ($($name:ident: $len:literal bytes, $value:literal = $be:expr;)+) => {$(
        mod $name {
            use amplify_num::$name as T;

            const BE: [u8; $len] = $be;

            fn le() -> [u8; $len] {
                let mut le = BE;
                le.reverse();
                le
            }

            #[test]
            fn from_be_bytes_reads_the_most_significant_byte_first() {
                assert_eq!(T::from_be_bytes(BE), T::with($value));
            }

            #[test]
            fn from_le_bytes_reads_the_least_significant_byte_first() {
                assert_eq!(T::from_le_bytes(le()), T::with($value));
            }

            #[test]
            fn to_be_bytes_writes_the_most_significant_byte_first() {
                assert_eq!(T::with($value).to_be_bytes(), BE);
            }

            #[test]
            fn to_le_bytes_writes_the_least_significant_byte_first() {
                assert_eq!(T::with($value).to_le_bytes(), le());
            }

            #[test]
            fn all_ones_bytes_give_the_maximum() {
                assert_eq!(T::from_be_bytes([0xFF; $len]), T::MAX);
                assert_eq!(T::from_le_bytes([0xFF; $len]), T::MAX);
                assert_eq!(T::MAX.to_be_bytes(), [0xFF; $len]);
                assert_eq!(T::MAX.to_le_bytes(), [0xFF; $len]);
            }

            #[test]
            fn zero_bytes_give_zero() {
                assert_eq!(T::from_be_bytes([0; $len]), T::ZERO);
                assert_eq!(T::from_le_bytes([0; $len]), T::ZERO);
                assert_eq!(T::ZERO.to_be_bytes(), [0; $len]);
                assert_eq!(T::ZERO.to_le_bytes(), [0; $len]);
            }

            #[test]
            fn one_is_the_last_big_endian_and_the_first_little_endian_byte() {
                assert_eq!(T::ONE.to_be_bytes()[$len - 1], 1);
                assert_eq!(T::ONE.to_le_bytes()[0], 1);
            }

            #[test]
            fn conversions_work_in_constants() {
                const VALUE: T = T::from_be_bytes(BE);
                const BYTES: [u8; $len] = VALUE.to_le_bytes();
                assert_eq!(T::from_le_bytes(BYTES), VALUE);
            }
        }
    )+};
}

mod bytes {
    bytes! {
        u24: 3 bytes, 0x12_3456 = [0x12, 0x34, 0x56];
        u40: 5 bytes, 0x12_3456_789A = [0x12, 0x34, 0x56, 0x78, 0x9A];
        u48: 6 bytes, 0x1234_5678_9ABC = [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC];
        u56: 7 bytes, 0x12_3456_789A_BCDE = [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE];
    }
}
