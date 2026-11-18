//! Integration tests of the small integers: the unsigned integers of a bit
//! size smaller than the size of the primitive integer storing them.
//!
//! The same suite runs for each of the types, with a separate test per case.
//! The sub-byte integers `u1` through `u7` are additionally tested exhaustively
//! across all values, operand pairs, radices and mixed-type promotions. The
//! expected values are computed on `u128` and `i128`, which can't overflow.

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

// ---------------------------------------------------------------------------
// Exhaustive tests of the sub-byte integers (u1 through u7)
// ---------------------------------------------------------------------------

mod exhaustive {
    use core::fmt::Debug;
    use core::hash::{Hash, Hasher};
    use core::num::IntErrorKind;
    use core::ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Rem, Sub};
    use std::collections::HashSet;
    use std::collections::hash_map::DefaultHasher;

    use amplify_num::error::{DivError, OverflowError, ParseIntError};
    use amplify_num::*;

    /// All binary operators and arithmetic traits taking a right operand `R`
    /// and returning `O`.
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

    /// Checks all binary operators and arithmetic traits of `lhs` and `rhs`
    /// against exact `i128` arithmetic on an output type spanning `min..=max`
    /// (both unsigned `0..=2^B-1` and signed two's complement
    /// `-2^(B-1)..=2^(B-1)-1`).
    fn check_exact_binary<L, R, O>(
        lhs: L,
        rhs: R,
        a: i128,
        b: i128,
        min: i128,
        max: i128,
        out: fn(i128) -> O,
    ) where
        L: Mixed<R, O>,
        R: Copy + Debug,
        O: Copy + Debug + Eq,
    {
        let modulus = max - min + 1;
        let wrap = |exact: i128| out((exact - min).rem_euclid(modulus) + min);
        let sat = |exact: i128| out(exact.clamp(min, max));

        let sum = a + b;
        let sum_fits = (min..=max).contains(&sum);
        assert_eq!(lhs.checked_add(rhs), sum_fits.then(|| wrap(sum)), "{lhs:?} + {rhs:?}");
        assert_eq!(lhs.overflowing_add(rhs), (wrap(sum), !sum_fits), "{lhs:?} + {rhs:?}");
        assert_eq!(lhs.saturating_add(rhs), sat(sum), "{lhs:?} + {rhs:?}");
        assert_eq!(lhs.wrapping_add(rhs), wrap(sum), "{lhs:?} + {rhs:?}");
        if sum_fits {
            assert_eq!(lhs + rhs, out(sum), "{lhs:?} + {rhs:?}");
        }

        let diff = a - b;
        let diff_fits = (min..=max).contains(&diff);
        assert_eq!(lhs.checked_sub(rhs), diff_fits.then(|| wrap(diff)), "{lhs:?} - {rhs:?}");
        assert_eq!(lhs.overflowing_sub(rhs), (wrap(diff), !diff_fits), "{lhs:?} - {rhs:?}");
        assert_eq!(lhs.saturating_sub(rhs), sat(diff), "{lhs:?} - {rhs:?}");
        assert_eq!(lhs.wrapping_sub(rhs), wrap(diff), "{lhs:?} - {rhs:?}");
        if diff_fits {
            assert_eq!(lhs - rhs, out(diff), "{lhs:?} - {rhs:?}");
        }

        let prod = a * b;
        let prod_fits = (min..=max).contains(&prod);
        assert_eq!(lhs.checked_mul(rhs), prod_fits.then(|| wrap(prod)), "{lhs:?} * {rhs:?}");
        assert_eq!(lhs.overflowing_mul(rhs), (wrap(prod), !prod_fits), "{lhs:?} * {rhs:?}");
        assert_eq!(lhs.saturating_mul(rhs), sat(prod), "{lhs:?} * {rhs:?}");
        assert_eq!(lhs.wrapping_mul(rhs), wrap(prod), "{lhs:?} * {rhs:?}");
        if prod_fits {
            assert_eq!(lhs * rhs, out(prod), "{lhs:?} * {rhs:?}");
        }

        if b == 0 {
            assert_eq!(lhs.checked_div(rhs), None, "{lhs:?} / {rhs:?}");
            assert_eq!(lhs.checked_rem(rhs), None, "{lhs:?} % {rhs:?}");
        } else if min < 0 && a == min && b == -1 {
            assert_eq!(lhs.checked_div(rhs), None, "{lhs:?} / {rhs:?}");
            assert_eq!(lhs.overflowing_div(rhs), (out(min), true), "{lhs:?} / {rhs:?}");
            assert_eq!(lhs.saturating_div(rhs), out(max), "{lhs:?} / {rhs:?}");
            assert_eq!(lhs.wrapping_div(rhs), out(min), "{lhs:?} / {rhs:?}");
            assert_eq!(lhs.checked_rem(rhs), None, "{lhs:?} % {rhs:?}");
            assert_eq!(lhs.overflowing_rem(rhs), (out(0), true), "{lhs:?} % {rhs:?}");
            assert_eq!(lhs.wrapping_rem(rhs), out(0), "{lhs:?} % {rhs:?}");
        } else {
            let (q, r) = (out(a / b), out(a % b));
            assert_eq!(lhs / rhs, q, "{lhs:?} / {rhs:?}");
            assert_eq!(lhs % rhs, r, "{lhs:?} % {rhs:?}");
            assert_eq!(lhs.checked_div(rhs), Some(q), "{lhs:?} / {rhs:?}");
            assert_eq!(lhs.overflowing_div(rhs), (q, false), "{lhs:?} / {rhs:?}");
            assert_eq!(lhs.saturating_div(rhs), q, "{lhs:?} / {rhs:?}");
            assert_eq!(lhs.wrapping_div(rhs), q, "{lhs:?} / {rhs:?}");
            assert_eq!(lhs.checked_rem(rhs), Some(r), "{lhs:?} % {rhs:?}");
            assert_eq!(lhs.overflowing_rem(rhs), (r, false), "{lhs:?} % {rhs:?}");
            assert_eq!(lhs.wrapping_rem(rhs), r, "{lhs:?} % {rhs:?}");
        }

        let (ba, bb) = (a.rem_euclid(modulus), b.rem_euclid(modulus));
        assert_eq!(lhs & rhs, wrap(ba & bb), "{lhs:?} & {rhs:?}");
        assert_eq!(lhs | rhs, wrap(ba | bb), "{lhs:?} | {rhs:?}");
        assert_eq!(lhs ^ rhs, wrap(ba ^ bb), "{lhs:?} ^ {rhs:?}");
    }

    /// Checks all binary operators and arithmetic traits of `lhs` and `rhs`
    /// against the operations of the promoted output type `O` (used for 128-bit
    /// and big integers whose full range exceeds `i128`).
    fn check_promoted_against_wide<L, R, O>(lhs: L, rhs: R)
    where
        L: Mixed<R, O> + Into<O>,
        R: Copy + Debug + Into<O>,
        O: Mixed<O, O> + Number,
    {
        let (x, y): (O, O) = (lhs.into(), rhs.into());

        assert_eq!(lhs.checked_add(rhs), x.checked_add(y));
        assert_eq!(lhs.overflowing_add(rhs), x.overflowing_add(y));
        assert_eq!(lhs.saturating_add(rhs), x.saturating_add(y));
        assert_eq!(lhs.wrapping_add(rhs), x.wrapping_add(y));

        assert_eq!(lhs.checked_sub(rhs), x.checked_sub(y));
        assert_eq!(lhs.overflowing_sub(rhs), x.overflowing_sub(y));
        assert_eq!(lhs.saturating_sub(rhs), x.saturating_sub(y));
        assert_eq!(lhs.wrapping_sub(rhs), x.wrapping_sub(y));

        assert_eq!(lhs.checked_mul(rhs), x.checked_mul(y));
        assert_eq!(lhs.overflowing_mul(rhs), x.overflowing_mul(y));
        assert_eq!(lhs.saturating_mul(rhs), x.saturating_mul(y));
        assert_eq!(lhs.wrapping_mul(rhs), x.wrapping_mul(y));

        assert_eq!(lhs.checked_div(rhs), x.checked_div(y));
        assert_eq!(lhs.checked_rem(rhs), x.checked_rem(y));
        if !y.is_zero() {
            assert_eq!(lhs.overflowing_div(rhs), x.overflowing_div(y));
            assert_eq!(lhs.saturating_div(rhs), x.saturating_div(y));
            assert_eq!(lhs.wrapping_div(rhs), x.wrapping_div(y));
            assert_eq!(lhs.overflowing_rem(rhs), x.overflowing_rem(y));
            assert_eq!(lhs.wrapping_rem(rhs), x.wrapping_rem(y));
        }

        if let Some(sum) = x.checked_add(y) {
            assert_eq!(lhs + rhs, sum);
        }
        if let Some(diff) = x.checked_sub(y) {
            assert_eq!(lhs - rhs, diff);
        }
        if let Some(prod) = x.checked_mul(y) {
            assert_eq!(lhs * rhs, prod);
        }
        if let Some(quot) = x.checked_div(y) {
            assert_eq!(lhs / rhs, quot);
        }
        if let Some(rem) = x.checked_rem(y) {
            assert_eq!(lhs % rhs, rem);
        }

        assert_eq!(lhs & rhs, x & y);
        assert_eq!(lhs | rhs, x | y);
        assert_eq!(lhs ^ rhs, x ^ y);
    }

    /// Formats a `u8` in `radix` (`2..=36`) into a stack buffer without heap
    /// allocation.
    fn format_radix(mut value: u8, radix: u32, upper: bool) -> ([u8; 8], usize) {
        let mut buf = [0u8; 8];
        let mut pos = buf.len();
        loop {
            pos -= 1;
            let digit = u8::try_from(u32::from(value) % radix).expect("digit < 36");
            buf[pos] = match digit {
                0..=9 => b'0' + digit,
                _ if upper => b'A' + (digit - 10),
                _ => b'a' + (digit - 10),
            };
            value = u8::try_from(u32::from(value) / radix).expect("quotient fits u8");
            if value == 0 {
                break;
            }
        }
        (buf, pos)
    }

    /// Character of the digit `digit` (`0..36`), used to construct the smallest
    /// invalid digit for a radix.
    fn digit_char(digit: u32) -> char {
        let byte = u8::try_from(digit).expect("digit < 36");
        char::from(if byte < 10 {
            b'0' + byte
        } else {
            b'a' + (byte - 10)
        })
    }

    fn hash_of<H: Hash>(value: H) -> u64 {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        hasher.finish()
    }

    /// Shift counts exercising all valid counts (`0..BITS`), the bit width,
    /// multiples of the bit width, the underlying primitive widths and `u32`
    /// extremes.
    fn shift_counts(bits: u32) -> impl Iterator<Item = u32> {
        (0..=32).chain([63, 64, 65, 127, 128, u32::MAX - bits, u32::MAX - 1, u32::MAX])
    }

    /// Self-tests of the exact reference oracle and radix formatter on `u8` and
    /// `i8`.
    mod harness {
        use super::*;

        #[test]
        fn format_radix_matches_u8_from_str_radix() {
            for raw in 0..=u8::MAX {
                for radix in 2..=36 {
                    for upper in [false, true] {
                        let (buf, pos) = format_radix(raw, radix, upper);
                        let text = core::str::from_utf8(&buf[pos..]).expect("ascii digits");
                        assert_eq!(u8::from_str_radix(text, radix), Ok(raw));
                    }
                }
            }
        }

        #[test]
        fn exact_oracle_matches_u8_and_i8_extremes() {
            for a in [0u8, 1, 2, 127, 128, 254, 255] {
                for b in [0u8, 1, 2, 127, 128, 254, 255] {
                    check_exact_binary(a, b, i128::from(a), i128::from(b), 0, 255, |v| {
                        u8::try_from(v).expect("u8 range")
                    });
                }
            }
            for a in [i8::MIN, -127, -2, -1, 0, 1, 2, 126, i8::MAX] {
                for b in [i8::MIN, -127, -2, -1, 0, 1, 2, 126, i8::MAX] {
                    check_exact_binary(a, b, i128::from(a), i128::from(b), -128, 127, |v| {
                        i8::try_from(v).expect("i8 range")
                    });
                }
            }
        }
    }

    /// Generates the exhaustive test suite for each sub-byte integer `u1..=u7`.
    macro_rules! subbyte_suite {
        ($($name:ident: $bits:literal;)+) => {$(
            mod $name {
                use super::*;

                type T = amplify_num::$name;

                const COUNT: usize = 1 << $bits;
                const MAX_U8: u8 = (1 << $bits) - 1;
                const MAX: i128 = (1 << $bits) - 1;
                const MOD: u16 = 1 << $bits;

                fn t(v: i128) -> T { T::with(u8::try_from(v).expect("fits u8")) }

                #[derive(Copy, Clone, Debug)]
                struct CustomSelf(u8);

                impl From<CustomSelf> for T {
                    fn from(value: CustomSelf) -> Self { T::with(value.0) }
                }

                #[derive(Copy, Clone, Debug)]
                struct CustomU8(u8);

                impl From<CustomU8> for u8 {
                    fn from(value: CustomU8) -> Self { value.0 }
                }

                // -- Construction, conversions & predicates ------------------

                #[test]
                fn const_with_and_to_u8_cover_every_value() {
                    const ALL: [T; COUNT] = {
                        let mut arr = [T::ZERO; COUNT];
                        let mut i = 0;
                        while i < COUNT {
                            arr[i] = T::with(i as u8);
                            i += 1;
                        }
                        arr
                    };
                    const RAW: [u8; COUNT] = {
                        let mut arr = [0u8; COUNT];
                        let mut i = 0;
                        while i < COUNT {
                            arr[i] = ALL[i].to_u8();
                            i += 1;
                        }
                        arr
                    };

                    for raw in 0..=MAX_U8 {
                        let idx = usize::from(raw);
                        let x = ALL[idx];
                        assert_eq!(RAW[idx], raw);
                        assert_eq!(x, T::with(raw));
                        assert_eq!(x.to_u8(), raw);
                        assert_eq!(x.clone(), x);
                        assert_eq!(x.is_zero(), raw == 0);
                        assert_eq!(x.is_one(), raw == 1);
                        assert!(T::MIN <= x && x <= T::MAX);
                    }
                }

                #[test]
                fn try_from_and_try_into_cover_every_u8() {
                    for raw in 0..=u8::MAX {
                        let via_try_into: Result<T, _> = raw.try_into();
                        if raw <= MAX_U8 {
                            let expected = Ok(T::with(raw));
                            assert_eq!(T::try_from(raw), expected);
                            assert_eq!(via_try_into, expected);
                        } else {
                            let expected = Err(OverflowError { max: MAX_U8, value: raw });
                            assert_eq!(T::try_from(raw), expected);
                            assert_eq!(via_try_into, expected);
                        }
                    }
                }

                #[test]
                fn conversions_into_wider_types_cover_every_value() {
                    for raw in 0..=MAX_U8 {
                        let x = T::with(raw);
                        assert_eq!(u8::from(x), raw);
                        assert_eq!(u16::from(x), u16::from(raw));
                        assert_eq!(u32::from(x), u32::from(raw));
                        assert_eq!(u64::from(x), u64::from(raw));
                        assert_eq!(u128::from(x), u128::from(raw));
                        assert_eq!(usize::from(x), usize::from(raw));
                        assert_eq!(i8::from(x), raw.cast_signed());
                        assert_eq!(i16::from(x), i16::from(raw));
                        assert_eq!(i32::from(x), i32::from(raw));
                        assert_eq!(i64::from(x), i64::from(raw));
                        assert_eq!(i128::from(x), i128::from(raw));
                        assert_eq!(isize::from(x), isize::from(raw));

                        assert_eq!(u10::from(x).to_u16(), u16::from(raw));
                        assert_eq!(u12::from(x).to_u16(), u16::from(raw));
                        assert_eq!(u14::from(x).to_u16(), u16::from(raw));
                        assert_eq!(u20::from(x).to_u32(), u32::from(raw));
                        assert_eq!(u24::from(x).to_u32(), u32::from(raw));
                        assert_eq!(u40::from(x).to_u64(), u64::from(raw));
                        assert_eq!(u48::from(x).to_u64(), u64::from(raw));
                        assert_eq!(u56::from(x).to_u64(), u64::from(raw));

                        assert_eq!(BigInt::<1, false>::from(x), BigInt::<1, false>::from(raw));
                        assert_eq!(BigInt::<1, true>::from(x), BigInt::<1, true>::from(raw));
                        assert_eq!(u256::from(x), u256::from(raw));
                        assert_eq!(i256::from(x), i256::from(raw));
                        assert_eq!(u512::from(x), u512::from(raw));
                        assert_eq!(i512::from(x), i512::from(raw));
                        assert_eq!(u1024::from(x), u1024::from(raw));
                        assert_eq!(i1024::from(x), i1024::from(raw));
                    }
                }

                // -- Comparison and hashing ----------------------------------

                #[test]
                fn comparison_and_hashing_cover_all_values_and_pairs() {
                    let mut set = HashSet::new();
                    for a in 0..=MAX_U8 {
                        let x = T::with(a);
                        assert!(set.insert(x));
                        assert_eq!(hash_of(x), hash_of(T::with(a)));

                        for b in 0..=MAX_U8 {
                            let y = T::with(b);
                            assert_eq!(x == y, a == b);
                            assert_eq!(x != y, a != b);
                            assert_eq!(x < y, a < b);
                            assert_eq!(x <= y, a <= b);
                            assert_eq!(x > y, a > b);
                            assert_eq!(x >= y, a >= b);
                            assert_eq!(x.partial_cmp(&y), a.partial_cmp(&b));
                            assert_eq!(x.cmp(&y), a.cmp(&b));
                            assert_eq!(x.min(y), T::with(a.min(b)));
                            assert_eq!(x.max(y), T::with(a.max(b)));

                            let (lo, hi) = (a.min(b), a.max(b));
                            assert_eq!(
                                x.clamp(T::with(lo), T::with(hi)),
                                T::with(a.clamp(lo, hi))
                            );
                            assert_eq!(
                                x.clamp(T::with(b / 2), y),
                                T::with(a.clamp(b / 2, b))
                            );
                            let mid_hi = b + (MAX_U8 - b) / 2;
                            assert_eq!(
                                x.clamp(y, T::with(mid_hi)),
                                T::with(a.clamp(b, mid_hi))
                            );
                            assert_eq!(x.clamp(y, T::MAX), T::with(a.clamp(b, MAX_U8)));
                            assert_eq!(x.clamp(T::MIN, y), T::with(a.clamp(0, b)));
                        }
                    }
                    assert_eq!(set.len(), COUNT);
                }

                // -- Same-type arithmetics -----------------------------------

                #[test]
                fn addition_covers_all_pairs() {
                    for a in 0..=MAX_U8 {
                        let x = T::with(a);
                        for b in 0..=MAX_U8 {
                            let y = T::with(b);
                            let exact = u16::from(a) + u16::from(b);
                            let fits = exact <= u16::from(MAX_U8);
                            let wrapped = T::with(u8::try_from(exact % MOD).unwrap());
                            let saturated =
                                T::with(u8::try_from(exact.min(u16::from(MAX_U8))).unwrap());

                            assert_eq!(x.checked_add(y), fits.then_some(wrapped));
                            assert_eq!(x.overflowing_add(y), (wrapped, !fits));
                            assert_eq!(x.saturating_add(y), saturated);
                            assert_eq!(x.wrapping_add(y), wrapped);
                            if fits {
                                assert_eq!(x + y, wrapped);
                                let mut acc = x;
                                acc += y;
                                assert_eq!(acc, wrapped);
                            }
                        }
                    }
                }

                #[test]
                fn subtraction_covers_all_pairs() {
                    let mod_i16 = i16::try_from(MOD).unwrap();
                    for a in 0..=MAX_U8 {
                        let x = T::with(a);
                        for b in 0..=MAX_U8 {
                            let y = T::with(b);
                            let exact = i16::from(a) - i16::from(b);
                            let fits = exact >= 0;
                            let wrapped =
                                T::with(u8::try_from(exact.rem_euclid(mod_i16)).unwrap());
                            let saturated = T::with(u8::try_from(exact.max(0)).unwrap());

                            assert_eq!(x.checked_sub(y), fits.then_some(wrapped));
                            assert_eq!(x.overflowing_sub(y), (wrapped, !fits));
                            assert_eq!(x.saturating_sub(y), saturated);
                            assert_eq!(x.wrapping_sub(y), wrapped);
                            assert_eq!(x.wrapping_sub(y), x.wrapping_add(y.wrapping_neg()));
                            if fits {
                                assert_eq!(x - y, wrapped);
                                let mut acc = x;
                                acc -= y;
                                assert_eq!(acc, wrapped);
                            }
                        }
                    }
                }

                #[test]
                fn multiplication_covers_all_pairs() {
                    for a in 0..=MAX_U8 {
                        let x = T::with(a);
                        for b in 0..=MAX_U8 {
                            let y = T::with(b);
                            let exact = u16::from(a) * u16::from(b);
                            let fits = exact <= u16::from(MAX_U8);
                            let wrapped = T::with(u8::try_from(exact % MOD).unwrap());
                            let saturated =
                                T::with(u8::try_from(exact.min(u16::from(MAX_U8))).unwrap());

                            assert_eq!(x.checked_mul(y), fits.then_some(wrapped));
                            assert_eq!(x.overflowing_mul(y), (wrapped, !fits));
                            assert_eq!(x.saturating_mul(y), saturated);
                            assert_eq!(x.wrapping_mul(y), wrapped);
                            if fits {
                                assert_eq!(x * y, wrapped);
                                let mut acc = x;
                                acc *= y;
                                assert_eq!(acc, wrapped);
                            }
                        }
                    }
                }

                #[test]
                fn division_and_remainder_cover_all_pairs_and_u8_divisors() {
                    for a in 0..=MAX_U8 {
                        let x = T::with(a);
                        for b in 0..=MAX_U8 {
                            let y = T::with(b);
                            if b == 0 {
                                assert_eq!(x.checked_div(y), None);
                                assert_eq!(x.checked_rem(y), None);
                                assert_eq!(x.div_rem(y), Err(DivError::ZeroDiv));
                            } else {
                                let (q, r) = (T::with(a / b), T::with(a % b));
                                assert_eq!(
                                    u16::from(q.to_u8()) * u16::from(b) + u16::from(r.to_u8()),
                                    u16::from(a)
                                );
                                assert!(r < y);
                                assert_eq!(x / y, q);
                                assert_eq!(x % y, r);
                                assert_eq!(x.checked_div(y), Some(q));
                                assert_eq!(x.overflowing_div(y), (q, false));
                                assert_eq!(x.saturating_div(y), q);
                                assert_eq!(x.wrapping_div(y), q);
                                assert_eq!(x.checked_rem(y), Some(r));
                                assert_eq!(x.overflowing_rem(y), (r, false));
                                assert_eq!(x.wrapping_rem(y), r);
                                assert_eq!(x.div_rem(y), Ok((q, r)));

                                let mut div_acc = x;
                                div_acc /= y;
                                assert_eq!(div_acc, q);

                                let mut rem_acc = x;
                                rem_acc %= y;
                                assert_eq!(rem_acc, r);
                            }
                        }

                        for rhs in 0..=u8::MAX {
                            if rhs == 0 {
                                assert_eq!(x.div_rem(rhs), Err(DivError::ZeroDiv));
                            } else {
                                assert_eq!(
                                    x.div_rem(rhs),
                                    Ok((T::with(a / rhs), T::with(a % rhs)))
                                );
                            }
                        }
                    }
                }

                // -- Negation, bitwise, shifts & compound assignments --------

                #[test]
                fn negation_and_bitwise_operations_cover_all_values_and_pairs() {
                    let mod_i16 = i16::try_from(MOD).unwrap();
                    for a in 0..=MAX_U8 {
                        let x = T::with(a);
                        let neg = T::with(u8::try_from((-i16::from(a)).rem_euclid(mod_i16)).unwrap());
                        let inv = T::with(MAX_U8 - a);

                        assert_eq!(x.wrapping_neg(), neg);
                        assert_eq!(x.wrapping_add(neg), T::ZERO);
                        assert_eq!(T::ZERO.wrapping_sub(x), neg);
                        assert_eq!(!x, inv);
                        assert_eq!(!!x, x);
                        assert_eq!(inv.wrapping_add(T::ONE), neg);
                        assert_eq!((x & inv, x | inv, x ^ inv), (T::ZERO, T::MAX, T::MAX));

                        for b in 0..=MAX_U8 {
                            let y = T::with(b);
                            let (and, or, xor) =
                                (T::with(a & b), T::with(a | b), T::with(a ^ b));
                            assert_eq!((x & y, x | y, x ^ y), (and, or, xor));
                            assert_eq!(!(x & y), !x | !y);
                            assert_eq!(!(x | y), !x & !y);

                            let mut v = x;
                            v &= y;
                            assert_eq!(v, and);
                            let mut v = x;
                            v |= y;
                            assert_eq!(v, or);
                            let mut v = x;
                            v ^= y;
                            assert_eq!(v, xor);
                        }
                    }
                }

                #[test]
                fn compound_assignments_with_u8_cover_all_pairs() {
                    for a in 0..=MAX_U8 {
                        let x = T::with(a);
                        for rhs in 0..=u8::MAX {
                            if u16::from(a) + u16::from(rhs) <= u16::from(MAX_U8) {
                                let mut v = x;
                                v += rhs;
                                assert_eq!(v, T::with(a + rhs));
                            }
                            if a >= rhs {
                                let mut v = x;
                                v -= rhs;
                                assert_eq!(v, T::with(a - rhs));
                            }
                            let prod = u16::from(a) * u16::from(rhs);
                            if prod <= u16::from(MAX_U8) {
                                let mut v = x;
                                v *= rhs;
                                assert_eq!(v, T::with(u8::try_from(prod).unwrap()));
                            }
                            if rhs != 0 {
                                let mut v = x;
                                v /= rhs;
                                assert_eq!(v, T::with(a / rhs));
                                let mut v = x;
                                v %= rhs;
                                assert_eq!(v, T::with(a % rhs));
                            }
                            let mut v = x;
                            v &= rhs;
                            assert_eq!(v, T::with(a & rhs));

                            if rhs <= MAX_U8 {
                                let mut v = x;
                                v |= rhs;
                                assert_eq!(v, T::with(a | rhs));
                                let mut v = x;
                                v ^= rhs;
                                assert_eq!(v, T::with(a ^ rhs));
                            }
                        }
                    }
                }

                #[test]
                fn shifts_cover_every_value_and_shift_counts() {
                    for a in 0..=MAX_U8 {
                        let x = T::with(a);
                        for shift in shift_counts(T::BITS) {
                            let rem = shift.rem_euclid(T::BITS);
                            let shl = T::with((a << rem) & MAX_U8);
                            let shr = T::with(a >> rem);
                            let in_range = shift < T::BITS;

                            assert_eq!(x.checked_shl(shift), in_range.then_some(shl));
                            assert_eq!(x.overflowing_shl(shift), (shl, !in_range));
                            assert_eq!(x.wrapping_shl(shift), shl);
                            assert_eq!(x.checked_shr(shift), in_range.then_some(shr));
                            assert_eq!(x.overflowing_shr(shift), (shr, !in_range));
                            assert_eq!(x.wrapping_shr(shift), shr);
                            if in_range {
                                assert_eq!(x << shift, shl);
                                assert_eq!(x >> shift, shr);
                                let mut v = x;
                                v <<= shift;
                                assert_eq!(v, shl);
                                let mut v = x;
                                v >>= shift;
                                assert_eq!(v, shr);
                            }
                        }
                    }
                }

                #[test]
                fn generic_into_operands_cover_all_pairs() {
                    for a in 0..=MAX_U8 {
                        let x = T::with(a);
                        for b in 0..=MAX_U8 {
                            check_exact_binary(
                                x,
                                CustomSelf(b),
                                i128::from(a),
                                i128::from(b),
                                0,
                                MAX,
                                t,
                            );
                        }
                        for rhs in 0..=u8::MAX {
                            let custom = CustomU8(rhs);
                            if rhs == 0 {
                                assert_eq!(x.div_rem(custom), Err(DivError::ZeroDiv));
                            } else {
                                assert_eq!(
                                    x.div_rem(custom),
                                    Ok((T::with(a / rhs), T::with(a % rhs)))
                                );
                                let mut v = x;
                                v /= custom;
                                assert_eq!(v, T::with(a / rhs));
                                let mut v = x;
                                v %= custom;
                                assert_eq!(v, T::with(a % rhs));
                            }
                            if u16::from(a) + u16::from(rhs) <= u16::from(MAX_U8) {
                                let mut v = x;
                                v += custom;
                                assert_eq!(v, T::with(a + rhs));
                            }
                            if a >= rhs {
                                let mut v = x;
                                v -= custom;
                                assert_eq!(v, T::with(a - rhs));
                            }
                            let prod = u16::from(a) * u16::from(rhs);
                            if prod <= u16::from(MAX_U8) {
                                let mut v = x;
                                v *= custom;
                                assert_eq!(v, T::with(u8::try_from(prod).unwrap()));
                            }
                            let mut v = x;
                            v &= custom;
                            assert_eq!(v, T::with(a & rhs));
                            if rhs <= MAX_U8 {
                                let mut v = x;
                                v |= custom;
                                assert_eq!(v, T::with(a | rhs));
                                let mut v = x;
                                v ^= custom;
                                assert_eq!(v, T::with(a ^ rhs));
                            }
                        }
                    }
                }

                // -- Formatting, parsing & serde -----------------------------

                #[test]
                fn formatting_matches_u8_for_every_value() {
                    for a in 0..=MAX_U8 {
                        let x = T::with(a);
                        assert_eq!(format!("{x}"), format!("{a}"));
                        assert_eq!(format!("{x:?}"), format!("{a:?}"));
                        assert_eq!(format!("{x:b}"), format!("{a:b}"));
                        assert_eq!(format!("{x:o}"), format!("{a:o}"));
                        assert_eq!(format!("{x:x}"), format!("{a:x}"));
                        assert_eq!(format!("{x:X}"), format!("{a:X}"));
                        assert_eq!(format!("{x:#b}"), format!("{a:#b}"));
                        assert_eq!(format!("{x:#o}"), format!("{a:#o}"));
                        assert_eq!(format!("{x:#x}"), format!("{a:#x}"));
                        assert_eq!(format!("{x:#X}"), format!("{a:#X}"));
                        assert_eq!(format!("{x:+}"), format!("{a:+}"));
                        assert_eq!(format!("{x:>6}"), format!("{a:>6}"));
                        assert_eq!(format!("{x:<6}|"), format!("{a:<6}|"));
                        assert_eq!(format!("{x:^6}|"), format!("{a:^6}|"));
                        assert_eq!(format!("{x:06}"), format!("{a:06}"));
                        assert_eq!(format!("{x:#010b}"), format!("{a:#010b}"));
                        assert_eq!(format!("{x:x?}"), format!("{a:x?}"));
                        assert_eq!(format!("{x:X?}"), format!("{a:X?}"));
                        assert_eq!(format!("{x:#x?}"), format!("{a:#x?}"));
                        assert_eq!(format!("{x:#X?}"), format!("{a:#X?}"));
                        #[cfg(all(feature = "hex", feature = "alloc"))]
                        assert_eq!(amplify_num::hex::ToHex::to_hex(&x), format!("{a:x}"));
                    }
                }

                #[test]
                fn parsing_covers_every_byte_and_every_radix() {
                    let overflow = Err(ParseIntError(IntErrorKind::PosOverflow));
                    let invalid = Err(ParseIntError(IntErrorKind::InvalidDigit));

                    for raw in 0..=u8::MAX {
                        let expected = (raw <= MAX_U8)
                            .then(|| T::with(raw))
                            .ok_or(ParseIntError(IntErrorKind::PosOverflow));
                        assert_eq!(raw.to_string().parse::<T>(), expected);
                        assert_eq!(format!("+{raw}").parse::<T>(), expected);
                        assert_eq!(format!("000{raw}").parse::<T>(), expected);

                        for radix in 2..=36 {
                            for upper in [false, true] {
                                let (buf, pos) = format_radix(raw, radix, upper);
                                let src = core::str::from_utf8(&buf[pos..]).expect("ascii");
                                assert_eq!(T::from_str_radix(src, radix), expected, "{src}@{radix}");
                            }
                        }
                    }

                    for radix in 2..=35 {
                        let bad = digit_char(radix).to_string();
                        assert_eq!(T::from_str_radix(&bad, radix), invalid, "radix {radix}");
                        assert_eq!(
                            T::from_str_radix(&bad.to_uppercase(), radix),
                            invalid,
                            "radix {radix}"
                        );
                    }
                    assert_eq!("256".parse::<T>(), overflow);
                    assert_eq!(u128::MAX.to_string().parse::<T>(), overflow);
                }

                #[cfg(feature = "serde")]
                #[test]
                fn serde_covers_every_byte() {
                    for raw in 0..=u8::MAX {
                        let json = raw.to_string();
                        if raw <= MAX_U8 {
                            let x = T::with(raw);
                            assert_eq!(serde_json::to_string(&x).unwrap(), json);
                            assert_eq!(serde_json::from_str::<T>(&json).unwrap(), x);
                        } else {
                            let err = serde_json::from_str::<T>(&json)
                                .expect_err("value above Self::MAX")
                                .to_string();
                            assert!(
                                err.contains(&format!(
                                    "value {raw} overflows the maximum {MAX_U8}"
                                )),
                                "{err}"
                            );
                        }
                    }
                }

                // -- Mixed-type / promoted arithmetics -----------------------

                #[test]
                fn mixed_with_u8_covers_all_pairs() {
                    let to_u8 = |v: i128| u8::try_from(v).expect("fits u8");
                    for a in 0..=MAX_U8 {
                        let x = T::with(a);
                        let a = i128::from(a);
                        for rhs in 0..=u8::MAX {
                            let b = i128::from(rhs);
                            check_exact_binary(x, rhs, a, b, 0, 255, to_u8);
                            check_exact_binary(rhs, x, b, a, 0, 255, to_u8);
                        }
                    }
                }

                #[test]
                fn mixed_with_i8_covers_all_pairs() {
                    let to_i8 = |v: i128| i8::try_from(v).expect("fits i8");
                    for a in 0..=MAX_U8 {
                        let x = T::with(a);
                        let a = i128::from(a);
                        for rhs in i8::MIN..=i8::MAX {
                            let b = i128::from(rhs);
                            check_exact_binary(x, rhs, a, b, -128, 127, to_i8);
                            check_exact_binary(rhs, x, b, a, -128, 127, to_i8);
                        }
                    }
                }

                #[test]
                fn mixed_with_wider_types_covers_every_value() {
                    macro_rules! check_wider_unsigned {
                        ($wide:ty, $to_wide:expr) => {{
                            let min = 0i128;
                            let max = i128::try_from(<$wide>::MAX).unwrap();
                            let samples = [
                                <$wide>::MIN,
                                <$wide>::ONE,
                                $to_wide(3),
                                $to_wide(100),
                                $to_wide(max / 3),
                                $to_wide(max - i128::from(MAX_U8 / 2 + 1)),
                                <$wide>::MAX,
                            ];
                            for a in 0..=MAX_U8 {
                                let x = T::with(a);
                                let a = i128::from(a);
                                for w in samples {
                                    let b = i128::try_from(w).unwrap();
                                    check_exact_binary(x, w, a, b, min, max, $to_wide);
                                    check_exact_binary(w, x, b, a, min, max, $to_wide);
                                }
                            }
                        }};
                    }

                    macro_rules! check_wider_small {
                        ($wide:ident, $inner:ty, $to_inner:ident) => {{
                            let max = i128::from($wide::MAX.$to_inner());
                            let make = |v: i128| $wide::with(<$inner>::try_from(v).unwrap());
                            let samples = [
                                $wide::MIN,
                                $wide::ONE,
                                make(3),
                                make(100),
                                make(max / 3),
                                make(max - i128::from(MAX_U8 / 2 + 1)),
                                $wide::MAX,
                            ];
                            for a in 0..=MAX_U8 {
                                let x = T::with(a);
                                let a = i128::from(a);
                                for w in samples {
                                    let b = i128::from(w.$to_inner());
                                    check_exact_binary(x, w, a, b, 0, max, make);
                                    check_exact_binary(w, x, b, a, 0, max, make);
                                }
                            }
                        }};
                    }

                    macro_rules! check_wider_signed {
                        ($wide:ty) => {{
                            let min = i128::try_from(<$wide>::MIN).unwrap();
                            let max = i128::try_from(<$wide>::MAX).unwrap();
                            let make = |v: i128| <$wide>::try_from(v).unwrap();
                            let half = i128::from(MAX_U8 / 2 + 1);
                            let samples = [
                                <$wide>::MIN,
                                make(min + half),
                                make(min / 3),
                                -2,
                                -1,
                                0,
                                1,
                                2,
                                make(max / 3),
                                make(max - half),
                                <$wide>::MAX,
                            ];
                            for a in 0..=MAX_U8 {
                                let x = T::with(a);
                                let a = i128::from(a);
                                for w in samples {
                                    let b = i128::try_from(w).unwrap();
                                    check_exact_binary(x, w, a, b, min, max, make);
                                    check_exact_binary(w, x, b, a, min, max, make);
                                }
                            }
                        }};
                    }

                    check_wider_small!(u10, u16, to_u16);
                    check_wider_small!(u12, u16, to_u16);
                    check_wider_small!(u14, u16, to_u16);
                    check_wider_small!(u20, u32, to_u32);
                    check_wider_small!(u24, u32, to_u32);
                    check_wider_small!(u40, u64, to_u64);
                    check_wider_small!(u48, u64, to_u64);
                    check_wider_small!(u56, u64, to_u64);

                    check_wider_unsigned!(u16, |v| u16::try_from(v).unwrap());
                    check_wider_unsigned!(u32, |v| u32::try_from(v).unwrap());
                    check_wider_unsigned!(u64, |v| u64::try_from(v).unwrap());
                    check_wider_unsigned!(usize, |v| usize::try_from(v).unwrap());

                    check_wider_signed!(i16);
                    check_wider_signed!(i32);
                    check_wider_signed!(i64);
                    check_wider_signed!(isize);

                    let half_u128 = u128::from(MAX_U8 / 2 + 1);
                    let half_i128 = i128::from(MAX_U8 / 2 + 1);
                    for a in 0..=MAX_U8 {
                        let x = T::with(a);
                        for w in [0u128, 1, 3, u128::MAX / 3, u128::MAX - half_u128, u128::MAX] {
                            check_promoted_against_wide::<T, u128, u128>(x, w);
                            check_promoted_against_wide::<u128, T, u128>(w, x);
                        }
                        for w in [
                            i128::MIN,
                            i128::MIN + half_i128,
                            i128::MIN / 3,
                            -2,
                            -1,
                            0,
                            1,
                            2,
                            i128::MAX / 3,
                            i128::MAX - half_i128,
                            i128::MAX,
                        ] {
                            check_promoted_against_wide::<T, i128, i128>(x, w);
                            check_promoted_against_wide::<i128, T, i128>(w, x);
                        }
                        for w in [
                            BigInt::<1, false>::MIN,
                            BigInt::<1, false>::ONE,
                            BigInt::<1, false>::MAX - (MAX_U8 / 2 + 1),
                            BigInt::<1, false>::MAX,
                        ] {
                            check_promoted_against_wide::<T, _, BigInt<1, false>>(x, w);
                            check_promoted_against_wide::<_, T, BigInt<1, false>>(w, x);
                        }
                        for w in [
                            BigInt::<1, true>::MIN,
                            BigInt::<1, true>::NEG_ONE,
                            BigInt::<1, true>::ZERO,
                            BigInt::<1, true>::ONE,
                            BigInt::<1, true>::MAX,
                        ] {
                            check_promoted_against_wide::<T, _, BigInt<1, true>>(x, w);
                            check_promoted_against_wide::<_, T, BigInt<1, true>>(w, x);
                        }
                        for w in [u256::MIN, u256::ONE, u256::MAX - (MAX_U8 / 2 + 1), u256::MAX] {
                            check_promoted_against_wide::<T, u256, u256>(x, w);
                            check_promoted_against_wide::<u256, T, u256>(w, x);
                        }
                        for w in [i256::MIN, i256::NEG_ONE, i256::ZERO, i256::ONE, i256::MAX] {
                            check_promoted_against_wide::<T, i256, i256>(x, w);
                            check_promoted_against_wide::<i256, T, i256>(w, x);
                        }
                        for w in [u512::MIN, u512::ONE, u512::MAX] {
                            check_promoted_against_wide::<T, u512, u512>(x, w);
                            check_promoted_against_wide::<u512, T, u512>(w, x);
                        }
                        for w in [i512::MIN, i512::NEG_ONE, i512::ZERO, i512::ONE, i512::MAX] {
                            check_promoted_against_wide::<T, i512, i512>(x, w);
                            check_promoted_against_wide::<i512, T, i512>(w, x);
                        }
                        for w in [u1024::MIN, u1024::ONE, u1024::MAX] {
                            check_promoted_against_wide::<T, u1024, u1024>(x, w);
                            check_promoted_against_wide::<u1024, T, u1024>(w, x);
                        }
                        for w in [i1024::MIN, i1024::NEG_ONE, i1024::ZERO, i1024::ONE, i1024::MAX] {
                            check_promoted_against_wide::<T, i1024, i1024>(x, w);
                            check_promoted_against_wide::<i1024, T, i1024>(w, x);
                        }
                    }
                }

                // -- Boundary and promoted panics ----------------------------

                panics! {
                    with_rejects_u8_max: "provided value exceeds Self::MAX" => T::with(u8::MAX);
                    from_str_radix_rejects_zero: "radix must lie in the range" =>
                        T::from_str_radix("0", 0);
                    from_str_radix_rejects_u32_max: "radix must lie in the range" =>
                        T::from_str_radix("0", u32::MAX);
                    bitor_assign_at_max_plus_one_panics: "attempt to bitor with overflow" => {
                        let mut v = T::ZERO;
                        v |= MAX_U8 + 1;
                        v
                    };
                    bitxor_assign_at_max_plus_one_panics: "attempt to bitxor with overflow" => {
                        let mut v = T::ZERO;
                        v ^= MAX_U8 + 1;
                        v
                    };
                    shl_by_u8_bits_panics: "attempt to shift left with overflow" => T::ONE << 8;
                    shr_by_u8_bits_panics: "attempt to shift right with overflow" => T::ONE >> 8;
                    shl_by_u32_max_panics: "attempt to shift left with overflow" =>
                        T::ONE << u32::MAX;
                    shr_by_u32_max_panics: "attempt to shift right with overflow" =>
                        T::ONE >> u32::MAX;
                    promoted_u8_add_lhs_overflow_panics: "attempt to add with overflow" =>
                        T::ONE + u8::MAX;
                    promoted_u8_add_rhs_overflow_panics: "attempt to add with overflow" =>
                        u8::MAX + T::ONE;
                    promoted_u8_sub_lhs_overflow_panics: "attempt to subtract with overflow" =>
                        T::ZERO - 1u8;
                    promoted_u8_sub_rhs_overflow_panics: "attempt to subtract with overflow" =>
                        0u8 - T::ONE;
                    promoted_i8_add_lhs_overflow_panics: "attempt to add with overflow" =>
                        T::ONE + i8::MAX;
                    promoted_i8_add_rhs_overflow_panics: "attempt to add with overflow" =>
                        i8::MAX + T::ONE;
                    promoted_i8_sub_min_lhs_overflow_panics: "attempt to subtract with overflow" =>
                        T::ZERO - i8::MIN;
                    promoted_i8_sub_min_rhs_overflow_panics: "attempt to subtract with overflow" =>
                        i8::MIN - T::ONE;
                    promoted_u8_div_by_zero_lhs_panics: "attempt to divide by zero" =>
                        T::ONE / 0u8;
                    promoted_u8_div_by_zero_rhs_panics: "attempt to divide by zero" =>
                        1u8 / T::ZERO;
                    promoted_u8_rem_by_zero_lhs_panics:
                        "attempt to calculate the remainder with a divisor of zero" =>
                        T::ONE % 0u8;
                    promoted_u8_rem_by_zero_rhs_panics:
                        "attempt to calculate the remainder with a divisor of zero" =>
                        1u8 % T::ZERO;
                    promoted_i8_div_by_zero_lhs_panics: "attempt to divide by zero" =>
                        T::ONE / 0i8;
                    promoted_i8_div_by_zero_rhs_panics: "attempt to divide by zero" =>
                        1i8 / T::ZERO;
                    promoted_i8_rem_by_zero_lhs_panics:
                        "attempt to calculate the remainder with a divisor of zero" =>
                        T::ONE % 0i8;
                    promoted_i8_rem_by_zero_rhs_panics:
                        "attempt to calculate the remainder with a divisor of zero" =>
                        1i8 % T::ZERO;
                    promoted_wrapping_div_by_zero_panics: "attempt to divide by zero" =>
                        T::ONE.wrapping_div(0u8);
                    promoted_wrapping_rem_by_zero_panics:
                        "attempt to calculate the remainder with a divisor of zero" =>
                        T::ONE.wrapping_rem(0i8);
                    clamp_min_above_max_panics: "min <= max" => T::ZERO.clamp(T::MAX, T::ZERO);
                    mul_assign_at_exact_overflow_boundary_panics:
                        "attempt to multiply with overflow" => {
                        let mut v = T::ONE;
                        v *= MAX_U8 + 1;
                        v
                    };
                }
            }
        )+};
    }

    subbyte_suite! {
        u1: 1;
        u2: 2;
        u3: 3;
        u4: 4;
        u5: 5;
        u6: 6;
        u7: 7;
    }

    /// Exhaustively tests every pair of distinct sub-byte integers `(N, W)`
    /// with `N::BITS < W::BITS` across all operand pairs, both operand orders,
    /// `div_rem` and compound assignments.
    mod pairs {
        use super::*;

        macro_rules! pair_suite {
            ($narrow:ident with [$($wide:ident)+]) => {
                mod $narrow {
                    $(
                        mod $wide {
                            use super::super::*;

                            type N = amplify_num::$narrow;
                            type W = amplify_num::$wide;

                            const N_MAX: u8 = N::MAX.to_u8();
                            const W_MAX: u8 = W::MAX.to_u8();

                            fn w(v: i128) -> W { W::with(u8::try_from(v).expect("fits u8")) }

                            #[test]
                            fn all_pairs_operators_traits_and_assignments() {
                                let max = i128::from(W_MAX);
                                for a in 0..=N_MAX {
                                    let n = N::with(a);
                                    assert_eq!(W::from(n), W::with(a));

                                    for b in 0..=W_MAX {
                                        let wide = W::with(b);
                                        check_exact_binary(
                                            n,
                                            wide,
                                            i128::from(a),
                                            i128::from(b),
                                            0,
                                            max,
                                            w,
                                        );
                                        check_exact_binary(
                                            wide,
                                            n,
                                            i128::from(b),
                                            i128::from(a),
                                            0,
                                            max,
                                            w,
                                        );

                                        // `div_rem` works in both directions via `Into<u8>`.
                                        if a == 0 {
                                            assert_eq!(wide.div_rem(n), Err(DivError::ZeroDiv));
                                        } else {
                                            assert_eq!(
                                                wide.div_rem(n),
                                                Ok((W::with(b / a), W::with(b % a)))
                                            );
                                        }
                                        if b == 0 {
                                            assert_eq!(n.div_rem(wide), Err(DivError::ZeroDiv));
                                        } else {
                                            assert_eq!(
                                                n.div_rem(wide),
                                                Ok((N::with(a / b), N::with(a % b)))
                                            );
                                        }

                                        // Compound assignments on `W` with `N`, and on `N` with `W`.
                                        if u16::from(b) + u16::from(a) <= u16::from(W_MAX) {
                                            let mut v = wide;
                                            v += n;
                                            assert_eq!(v, W::with(b + a));
                                        }
                                        if u16::from(a) + u16::from(b) <= u16::from(N_MAX) {
                                            let mut v = n;
                                            v += wide;
                                            assert_eq!(v, N::with(a + b));
                                        }
                                        if b >= a {
                                            let mut v = wide;
                                            v -= n;
                                            assert_eq!(v, W::with(b - a));
                                        }
                                        if a >= b {
                                            let mut v = n;
                                            v -= wide;
                                            assert_eq!(v, N::with(a - b));
                                        }
                                        let prod = u16::from(a) * u16::from(b);
                                        if prod <= u16::from(W_MAX) {
                                            let mut v = wide;
                                            v *= n;
                                            assert_eq!(v, W::with(u8::try_from(prod).unwrap()));
                                        }
                                        if prod <= u16::from(N_MAX) {
                                            let mut v = n;
                                            v *= wide;
                                            assert_eq!(v, N::with(u8::try_from(prod).unwrap()));
                                        }
                                        if a != 0 {
                                            let mut v = wide;
                                            v /= n;
                                            assert_eq!(v, W::with(b / a));
                                            let mut v = wide;
                                            v %= n;
                                            assert_eq!(v, W::with(b % a));
                                        }
                                        if b != 0 {
                                            let mut v = n;
                                            v /= wide;
                                            assert_eq!(v, N::with(a / b));
                                            let mut v = n;
                                            v %= wide;
                                            assert_eq!(v, N::with(a % b));
                                        }
                                        let mut v = wide;
                                        v &= n;
                                        assert_eq!(v, W::with(b & a));
                                        let mut v = wide;
                                        v |= n;
                                        assert_eq!(v, W::with(b | a));
                                        let mut v = wide;
                                        v ^= n;
                                        assert_eq!(v, W::with(b ^ a));

                                        let mut v = n;
                                        v &= wide;
                                        assert_eq!(v, N::with(a & b));
                                        if b <= N_MAX {
                                            let mut v = n;
                                            v |= wide;
                                            assert_eq!(v, N::with(a | b));
                                            let mut v = n;
                                            v ^= wide;
                                            assert_eq!(v, N::with(a ^ b));
                                        }
                                    }
                                }
                            }

                            panics! {
                                narrow_add_wide_overflow_panics: "attempt to add with overflow" =>
                                    N::ONE + W::MAX;
                                wide_add_narrow_overflow_panics: "attempt to add with overflow" =>
                                    W::MAX + N::ONE;
                                narrow_sub_wide_overflow_panics:
                                    "attempt to subtract with overflow" => N::ZERO - W::ONE;
                                wide_sub_narrow_overflow_panics:
                                    "attempt to subtract with overflow" => W::ZERO - N::ONE;
                                narrow_div_wide_zero_panics: "attempt to divide by zero" =>
                                    N::ONE / W::ZERO;
                                wide_div_narrow_zero_panics: "attempt to divide by zero" =>
                                    W::ONE / N::ZERO;
                                narrow_rem_wide_zero_panics:
                                    "attempt to calculate the remainder with a divisor of zero" =>
                                    N::ONE % W::ZERO;
                                wide_rem_narrow_zero_panics:
                                    "attempt to calculate the remainder with a divisor of zero" =>
                                    W::ONE % N::ZERO;
                                narrow_bitor_assign_wide_max_panics:
                                    "attempt to bitor with overflow" => {
                                    let mut v = N::ZERO;
                                    v |= W::MAX;
                                    v
                                };
                                narrow_bitxor_assign_wide_max_panics:
                                    "attempt to bitxor with overflow" => {
                                    let mut v = N::ZERO;
                                    v ^= W::MAX;
                                    v
                                };
                            }
                        }
                    )+
                }
            };
        }

        macro_rules! subbyte_widening {
            ($narrow:ident $($wide:ident)+) => {
                pair_suite!($narrow with [$($wide)+]);
                subbyte_widening!($($wide)+);
            };
            ($widest:ident) => {};
        }

        subbyte_widening!(u1 u2 u3 u4 u5 u6 u7);

        /// Multiplication of `u1` with any integer never overflows, whereas
        /// every wider sub-byte pair overflows at `(N::MAX, W::MAX)`.
        mod mul_overflow {
            use amplify_num::*;

            #[test]
            fn u1_products_never_overflow() {
                for a in [u1::ZERO, u1::ONE] {
                    for b in 0..=u8::MAX {
                        assert_eq!(a * b, if a.is_one() { b } else { 0 });
                        assert_eq!(b * a, if a.is_one() { b } else { 0 });
                    }
                    for b in i8::MIN..=i8::MAX {
                        assert_eq!(a * b, if a.is_one() { b } else { 0 });
                        assert_eq!(b * a, if a.is_one() { b } else { 0 });
                    }
                }
            }

            macro_rules! mul_panics {
                ($($name:ident: $lhs:expr, $rhs:expr;)+) => {$(
                    #[test]
                    #[should_panic(expected = "attempt to multiply with overflow")]
                    fn $name() { let _ = $lhs * $rhs; }
                )+};
            }

            mul_panics! {
                u2_by_u3: u2::MAX, u3::MAX;
                u3_by_u2: u3::MAX, u2::MAX;
                u2_by_u4: u2::MAX, u4::MAX;
                u4_by_u2: u4::MAX, u2::MAX;
                u2_by_u5: u2::MAX, u5::MAX;
                u5_by_u2: u5::MAX, u2::MAX;
                u2_by_u6: u2::MAX, u6::MAX;
                u6_by_u2: u6::MAX, u2::MAX;
                u2_by_u7: u2::MAX, u7::MAX;
                u7_by_u2: u7::MAX, u2::MAX;
                u3_by_u4: u3::MAX, u4::MAX;
                u4_by_u3: u4::MAX, u3::MAX;
                u3_by_u5: u3::MAX, u5::MAX;
                u5_by_u3: u5::MAX, u3::MAX;
                u3_by_u6: u3::MAX, u6::MAX;
                u6_by_u3: u6::MAX, u3::MAX;
                u3_by_u7: u3::MAX, u7::MAX;
                u7_by_u3: u7::MAX, u3::MAX;
                u4_by_u5: u4::MAX, u5::MAX;
                u5_by_u4: u5::MAX, u4::MAX;
                u4_by_u6: u4::MAX, u6::MAX;
                u6_by_u4: u6::MAX, u4::MAX;
                u4_by_u7: u4::MAX, u7::MAX;
                u7_by_u4: u7::MAX, u4::MAX;
                u5_by_u6: u5::MAX, u6::MAX;
                u6_by_u5: u6::MAX, u5::MAX;
                u5_by_u7: u5::MAX, u7::MAX;
                u7_by_u5: u7::MAX, u5::MAX;
                u6_by_u7: u6::MAX, u7::MAX;
                u7_by_u6: u7::MAX, u6::MAX;
                u2_by_u8: u2::MAX, u8::MAX;
                u8_by_u2: u8::MAX, u2::MAX;
                u7_by_u8: u7::MAX, u8::MAX;
                u8_by_u7: u8::MAX, u7::MAX;
                u2_by_i8_max: u2::MAX, i8::MAX;
                i8_max_by_u2: i8::MAX, u2::MAX;
                u2_by_i8_min: u2::TWO, -65i8;
                i8_min_by_u2: -65i8, u2::TWO;
                u7_by_i8_max: u7::MAX, i8::MAX;
                i8_max_by_u7: i8::MAX, u7::MAX;
                u7_by_i8_min: u7::MAX, i8::MIN;
                i8_min_by_u7: i8::MIN, u7::MAX;
            }
        }
    }
}
