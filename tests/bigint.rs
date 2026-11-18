//! Integration tests of the big integers.
//!
//! The same suites run for the integers of 1, 3, 4, 8 and 16 limbs: the
//! smallest size, a size made of an odd number of limbs, and the sizes having
//! the aliases. Each case is a separate test. The expected values are built
//! from the constants of the types and from the decimal powers of two, which
//! are computed on the strings of digits independently of the library.
//!
//! The last block checks the arithmetic of long operands against the answers
//! computed with another implementation of the arbitrary-precision integers.

use core::cmp::Ordering;
use core::num::IntErrorKind;
use std::collections::HashSet;

use amplify_num::error::{DivError, ParseIntError, ParseLengthError};
use amplify_num::*;

// ---------------------------------------------------------------------------
// Test harness
// ---------------------------------------------------------------------------

/// Generates the tests expecting a panic with the given message.
macro_rules! panics {
    ($($name:ident: $msg:literal => $expr:expr;)+) => {$(
        #[test]
        #[should_panic(expected = $msg)]
        fn $name() { let _ = $expr; }
    )+};
}

/// Decimal digits of `2^exponent`, computed by doubling a string of digits.
fn pow2(exponent: u32) -> String {
    // The digits are kept in the little-endian order.
    let mut digits = vec![1u8];
    for _ in 0..exponent {
        let mut carry = 0;
        for digit in &mut digits {
            let doubled = *digit * 2 + carry;
            (*digit, carry) = (doubled % 10, doubled / 10);
        }
        if carry > 0 {
            digits.push(carry);
        }
    }
    digits
        .iter()
        .rev()
        .map(|digit| char::from(b'0' + digit))
        .collect()
}

/// Decimal digits of the number following the given one.
fn plus_one(decimal: &str) -> String {
    let mut digits = decimal.as_bytes().to_vec();
    for digit in digits.iter_mut().rev() {
        if *digit < b'9' {
            *digit += 1;
            return String::from_utf8(digits).expect("decimal digits");
        }
        *digit = b'0';
    }
    format!("1{}", "0".repeat(decimal.len()))
}

/// Decimal digits of the number preceding the given one, which must be
/// positive.
fn minus_one(decimal: &str) -> String {
    let mut digits = decimal.as_bytes().to_vec();
    for digit in digits.iter_mut().rev() {
        if *digit > b'0' {
            *digit -= 1;
            break;
        }
        *digit = b'9';
    }
    let digits = String::from_utf8(digits).expect("decimal digits");
    match digits.trim_start_matches('0') {
        "" => "0".to_owned(),
        trimmed => trimmed.to_owned(),
    }
}

/// Octal digits of the number having the `bits` lowest bits set.
fn octal_ones(bits: u32) -> String {
    let top = ["", "1", "3"][(bits % 3) as usize];
    format!("{top}{}", "7".repeat((bits / 3) as usize))
}

/// The helpers themselves are checked against the primitive integers.
mod harness {
    use super::*;

    #[test]
    fn pow2_matches_the_primitive_integers() {
        for exponent in 0..128 {
            assert_eq!(pow2(exponent), (1u128 << exponent).to_string());
        }
    }

    #[test]
    fn plus_one_carries_through_the_nines() {
        for value in [0u128, 8, 9, 10, 99, 1099, 999_999, u128::MAX - 1] {
            assert_eq!(plus_one(&value.to_string()), (value + 1).to_string());
        }
    }

    #[test]
    fn minus_one_borrows_through_the_zeros() {
        for value in [1u128, 9, 10, 100, 1001, 1_000_000, u128::MAX] {
            assert_eq!(minus_one(&value.to_string()), (value - 1).to_string());
        }
    }

    #[test]
    fn octal_ones_matches_the_primitive_integers() {
        for bits in 1..=128 {
            assert_eq!(octal_ones(bits), format!("{:o}", u128::MAX >> (128 - bits)));
        }
    }
}

// ---------------------------------------------------------------------------
// Cases common for the signed and unsigned integers
// ---------------------------------------------------------------------------

/// Generates the tests which do not depend on the signedness. The type `T`
/// and the constants `LIMBS` and `BITS` are defined by the module using the
/// macro.
macro_rules! common_tests {
    () => {
        /// A small non-negative number.
        fn small(value: u8) -> T { T::from(value) }

        /// The number having the given lowest limb and no other bits set.
        fn low(limb: u64) -> T {
            let mut limbs = [0; LIMBS];
            limbs[0] = limb;
            T::from_inner(limbs)
        }

        /// The number having all bits set.
        fn ones() -> T { T::from_inner([u64::MAX; LIMBS]) }

        // -- Constants and limbs ------------------------------------------

        #[test]
        fn bits_and_bytes_count_all_limbs() {
            assert_eq!((T::BITS, T::BYTES), (BITS, 8 * LIMBS));
        }

        #[test]
        fn zero_has_no_bits_set() {
            assert_eq!(T::ZERO.into_inner(), [0; LIMBS]);
        }

        #[test]
        fn one_has_the_lowest_bit_set() {
            assert_eq!(T::ONE, low(1));
        }

        #[test]
        fn default_is_zero() {
            assert_eq!(T::default(), T::ZERO);
        }

        #[test]
        fn limbs_are_kept_in_the_given_order() {
            let limbs: [u64; LIMBS] = core::array::from_fn(|i| i as u64 + 1);
            let value = T::from_inner(limbs);
            assert_eq!(value.as_inner(), &limbs);
            assert_eq!(value.into_inner(), limbs);
            assert_eq!(value.low_u64(), 1);
        }

        #[test]
        fn from_inner_works_in_constants() {
            const VALUE: T = T::from_inner([5; LIMBS]);
            const LIMBS_BACK: [u64; LIMBS] = VALUE.into_inner();
            assert_eq!(LIMBS_BACK, [5; LIMBS]);
        }

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
            for (value, zero) in
                [(T::ZERO, true), (T::ONE, false), (T::MAX, false), (ones(), false)]
            {
                assert_eq!(T::is_zero(&value), zero, "{value}");
                assert_eq!(Number::is_zero(value), zero, "{value}");
            }
        }

        #[test]
        fn is_one_holds_only_for_one() {
            assert!(Number::is_one(T::ONE));
            assert!(!Number::is_one(T::ZERO));
            assert!(!Number::is_one(T::MAX));
            assert!(!Number::is_one(small(2)));
        }

        #[test]
        fn low_bits_are_taken_from_the_lowest_limb() {
            let value = low(0x1234_5678_9ABC_DEF0);
            assert_eq!(value.low_u64(), 0x1234_5678_9ABC_DEF0);
            assert_eq!(value.low_u32(), 0x9ABC_DEF0);
            assert_eq!(ones().low_u64(), u64::MAX);
            assert_eq!(ones().low_u32(), u32::MAX);
        }

        // -- Bit inspection -----------------------------------------------

        #[test]
        fn zero_has_only_zero_bits() {
            assert_eq!(T::ZERO.leading_zeros(), BITS);
            assert_eq!(T::ZERO.trailing_zeros(), BITS);
            assert_eq!(T::ZERO.leading_ones(), 0);
            assert_eq!(T::ZERO.trailing_ones(), 0);
        }

        #[test]
        fn all_ones_has_only_one_bits() {
            assert_eq!(ones().leading_ones(), BITS);
            assert_eq!(ones().trailing_ones(), BITS);
            assert_eq!(ones().leading_zeros(), 0);
            assert_eq!(ones().trailing_zeros(), 0);
        }

        #[test]
        fn one_has_a_single_trailing_one() {
            assert_eq!(T::ONE.leading_zeros(), BITS - 1);
            assert_eq!(T::ONE.trailing_zeros(), 0);
            assert_eq!(T::ONE.trailing_ones(), 1);
            assert_eq!(T::ONE.leading_ones(), 0);
        }

        #[test]
        fn a_single_bit_is_found_at_every_position() {
            for position in 0..BITS {
                let value = T::ONE << position;
                assert_eq!(value.trailing_zeros(), position);
                assert_eq!(value.leading_zeros(), BITS - 1 - position);
                assert!(value.bit(position));
                assert_eq!((!value).trailing_ones(), position);
                assert_eq!((!value).leading_ones(), BITS - 1 - position);
                assert!(!(!value).bit(position));
            }
        }

        #[test]
        fn bit_reads_only_the_requested_position() {
            assert!(T::ONE.bit(0));
            assert!(!T::ONE.bit(1));
            assert!(!T::ONE.bit(BITS - 1));
            assert!(ones().bit(BITS - 1));
            assert!(!T::ZERO.bit(BITS - 1));
        }

        panics! {
            bit_at_the_bit_size_panics: "index out of bounds" => T::ONE.bit(BITS);
            bit_at_the_largest_index_panics: "index out of bounds" => T::ONE.bit(u32::MAX);
        }

        // -- Shifts -------------------------------------------------------

        #[test]
        fn shifts_by_zero_keep_the_value() {
            assert_eq!(ones() << 0, ones());
            assert_eq!(ones() >> 0, ones());
            assert_eq!(ones().unbounded_shl(0), ones());
            assert_eq!(ones().unbounded_shr(0), ones());
        }

        #[test]
        fn shifts_cross_all_limbs() {
            let value = small(5) << (BITS - 4);
            assert_eq!(value.trailing_zeros(), BITS - 4);
            assert_eq!(value.leading_zeros(), 1);
            assert_eq!(value >> (BITS - 4), small(5));
        }

        #[test]
        fn shifts_by_whole_limbs_move_the_limbs() {
            let shift = BITS - 64;
            assert_eq!((low(0x7654_3210) << shift).into_inner()[LIMBS - 1], 0x7654_3210);
            assert_eq!((low(0x7654_3210) << shift) >> shift, low(0x7654_3210));
        }

        #[test]
        fn checked_shifts_accept_the_counts_below_the_bit_size() {
            assert_eq!(T::ONE.checked_shl(BITS - 1), Some(T::ONE << (BITS - 1)));
            assert_eq!(small(4).checked_shr(2), Some(T::ONE));
            assert_eq!(T::ONE.checked_shr(BITS - 1), Some(T::ZERO));
        }

        #[test]
        fn checked_shifts_reject_the_bit_size() {
            assert_eq!(T::ONE.checked_shl(BITS), None);
            assert_eq!(T::ONE.checked_shr(BITS), None);
            assert_eq!(T::ONE.checked_shl(u32::MAX), None);
            assert_eq!(T::ONE.checked_shr(u32::MAX), None);
        }

        #[test]
        fn overflowing_shifts_wrap_the_count_at_the_bit_size() {
            let value = small(8);
            assert_eq!(value.overflowing_shl(3), (small(64), false));
            assert_eq!(value.overflowing_shr(3), (T::ONE, false));
            assert_eq!(value.overflowing_shl(BITS), (value, true));
            assert_eq!(value.overflowing_shr(BITS), (value, true));
            assert_eq!(value.overflowing_shl(BITS + 3), (small(64), true));
            assert_eq!(value.overflowing_shr(BITS + 3), (T::ONE, true));
        }

        #[test]
        fn wrapping_shifts_wrap_the_count_at_the_bit_size() {
            let value = small(8);
            assert_eq!(value.wrapping_shl(BITS), value);
            assert_eq!(value.wrapping_shr(BITS), value);
            assert_eq!(value.wrapping_shl(BITS + 3), small(64));
            assert_eq!(value.wrapping_shr(BITS + 3), T::ONE);
            assert_eq!(value.wrapping_shl(u32::MAX), value << (u32::MAX % BITS));
            assert_eq!(value.wrapping_shr(u32::MAX), value >> (u32::MAX % BITS));
        }

        #[test]
        fn unbounded_shl_gives_zero_from_the_bit_size() {
            assert_eq!(ones().unbounded_shl(BITS - 1), T::ONE << (BITS - 1));
            assert_eq!(ones().unbounded_shl(BITS), T::ZERO);
            assert_eq!(ones().unbounded_shl(BITS + 1), T::ZERO);
            assert_eq!(ones().unbounded_shl(u32::MAX), T::ZERO);
        }

        #[test]
        fn unbounded_shr_of_a_positive_number_gives_zero_from_the_bit_size() {
            assert_eq!(T::ONE.unbounded_shr(BITS), T::ZERO);
            assert_eq!(T::ONE.unbounded_shr(BITS + 1), T::ZERO);
            assert_eq!(T::ONE.unbounded_shr(u32::MAX), T::ZERO);
        }

        #[test]
        fn shift_assignments_match_the_operators() {
            let mut value = T::ONE;
            value <<= BITS - 2;
            assert_eq!(value, T::ONE << (BITS - 2));
            value >>= BITS - 2;
            assert_eq!(value, T::ONE);
        }

        panics! {
            shl_by_the_bit_size_panics: "attempt to shift left with overflow" => T::ONE << BITS;
            shr_by_the_bit_size_panics: "attempt to shift right with overflow" => T::ONE >> BITS;
            shl_assign_by_the_bit_size_panics: "attempt to shift left with overflow" => {
                let mut value = T::ONE;
                value <<= BITS;
                value
            };
            shr_assign_by_the_bit_size_panics: "attempt to shift right with overflow" => {
                let mut value = T::ONE;
                value >>= BITS;
                value
            };
        }

        // -- Bitwise operations -------------------------------------------

        #[test]
        fn not_inverts_every_bit() {
            assert_eq!(!T::ZERO, ones());
            assert_eq!(!ones(), T::ZERO);
            assert_eq!(!!small(5), small(5));
            assert_eq!(!T::MAX, T::MIN);
        }

        #[test]
        fn bitwise_operators_follow_their_truth_tables() {
            assert_eq!(ones() & T::ZERO, T::ZERO);
            assert_eq!(ones() & small(5), small(5));
            assert_eq!(T::ZERO | ones(), ones());
            assert_eq!(small(5) | small(3), small(7));
            assert_eq!(ones() ^ ones(), T::ZERO);
            assert_eq!(small(5) ^ small(3), small(6));
            assert_eq!(ones() ^ T::ONE, !T::ONE);
        }

        #[test]
        fn bitwise_assignments_match_the_operators() {
            let mut value = small(0b1100);
            value &= small(0b1010);
            assert_eq!(value, small(0b1000));
            value |= small(0b0011);
            assert_eq!(value, small(0b1011));
            value ^= small(0b1111);
            assert_eq!(value, small(0b0100));
        }

        // -- Addition, subtraction and multiplication ---------------------

        #[test]
        fn addition_and_subtraction_are_inverse() {
            assert_eq!(T::ONE + T::ONE, small(2));
            assert_eq!(small(2) - T::ONE, T::ONE);
            assert_eq!(small(200) + small(100) - small(100), small(200));
            assert_eq!(small(7) + T::ZERO, small(7));
            assert_eq!(small(7) - small(7), T::ZERO);
        }

        #[test]
        fn carry_and_borrow_propagate_through_all_limbs() {
            let mut limbs = [u64::MAX; LIMBS];
            limbs[LIMBS - 1] = 0;
            let (below, power) = (T::from_inner(limbs), T::ONE << (BITS - 64));
            assert_eq!(below + T::ONE, power);
            assert_eq!(power - T::ONE, below);
            assert_eq!(below.checked_add(T::ONE), Some(power));
            assert_eq!(power.checked_sub(T::ONE), Some(below));
            assert_eq!(below.overflowing_add(T::ONE), (power, false));
            assert_eq!(power.overflowing_sub(T::ONE), (below, false));
            assert_eq!(below.saturating_add(T::ONE), power);
            assert_eq!(power.saturating_sub(T::ONE), below);
            assert_eq!(below.wrapping_add(T::ONE), power);
            assert_eq!(power.wrapping_sub(T::ONE), below);
        }

        #[test]
        fn multiplication_by_zero_and_one() {
            assert_eq!(T::MAX * T::ONE, T::MAX);
            assert_eq!(T::MAX * T::ZERO, T::ZERO);
            assert_eq!(T::ONE * T::MAX, T::MAX);
            assert_eq!(T::ZERO * T::MAX, T::ZERO);
            assert_eq!(T::MAX.checked_mul(T::ONE), Some(T::MAX));
            assert_eq!(T::MAX.overflowing_mul(T::ONE), (T::MAX, false));
            assert_eq!(T::MAX.saturating_mul(T::ONE), T::MAX);
            assert_eq!(T::MAX.wrapping_mul(T::ONE), T::MAX);
        }

        #[test]
        fn multiplication_carries_into_the_upper_limbs() {
            assert_eq!(small(12) * small(12), small(144));
            assert_eq!((T::ONE << (BITS / 2 - 1)) * small(2), T::ONE << (BITS / 2));
            assert_eq!((T::ONE << (BITS - 3)) * small(2), T::ONE << (BITS - 2));
            let value = T::from(u32::MAX);
            assert_eq!(value * small(2), value + value);
            assert_eq!(low(1 << 63) * small(1), low(1 << 63));
        }

        #[test]
        fn arithmetic_assignments_match_the_operators() {
            let mut value = small(7);
            value += 3u8;
            assert_eq!(value, small(10));
            value -= 4u8;
            assert_eq!(value, small(6));
            value *= 5u8;
            assert_eq!(value, small(30));
            value /= 4u8;
            assert_eq!(value, small(7));
            value %= 4u8;
            assert_eq!(value, small(3));
        }

        // -- Division -----------------------------------------------------

        #[test]
        fn division_identities_hold() {
            let value = small(100);
            assert_eq!(value / T::ONE, value);
            assert_eq!(value % T::ONE, T::ZERO);
            assert_eq!(value / value, T::ONE);
            assert_eq!(value % value, T::ZERO);
            assert_eq!(T::ZERO / value, T::ZERO);
            assert_eq!(T::ZERO % value, T::ZERO);
            assert_eq!(T::MAX / T::MAX, T::ONE);
            assert_eq!(T::MAX % T::MAX, T::ZERO);
            assert_eq!(T::MAX / T::ONE, T::MAX);
        }

        #[test]
        fn all_flavors_divide_positive_numbers_equally() {
            let (num, den, quot, rem) = (small(7), small(2), small(3), T::ONE);
            assert_eq!(num / den, quot);
            assert_eq!(num % den, rem);
            assert_eq!(num.div_rem(den), Ok((quot, rem)));
            assert_eq!(num.checked_div(den), Some(quot));
            assert_eq!(num.checked_rem(den), Some(rem));
            assert_eq!(num.overflowing_div(den), (quot, false));
            assert_eq!(num.overflowing_rem(den), (rem, false));
            assert_eq!(num.saturating_div(den), quot);
            assert_eq!(num.wrapping_div(den), quot);
            assert_eq!(num.wrapping_rem(den), rem);
        }

        #[test]
        fn all_euclidean_flavors_divide_positive_numbers_equally() {
            let (num, den, quot, rem) = (small(7), small(2), small(3), T::ONE);
            assert_eq!(num.div_rem_euclid(den), Ok((quot, rem)));
            assert_eq!(num.div_euclid(den), quot);
            assert_eq!(num.rem_euclid(den), rem);
            assert_eq!(num.checked_div_euclid(den), Some(quot));
            assert_eq!(num.checked_rem_euclid(den), Some(rem));
            assert_eq!(num.overflowing_div_euclid(den), (quot, false));
            assert_eq!(num.overflowing_rem_euclid(den), (rem, false));
            assert_eq!(num.wrapping_div_euclid(den), quot);
            assert_eq!(num.wrapping_rem_euclid(den), rem);
        }

        #[test]
        fn division_by_a_longer_number_gives_zero() {
            let (num, den) = (T::ONE << (BITS / 2), T::ONE << (BITS - 2));
            assert_eq!(num.div_rem(den), Ok((T::ZERO, num)));
            assert_eq!(den.div_rem(num), Ok((T::ONE << (BITS / 2 - 2), T::ZERO)));
        }

        #[test]
        fn division_by_zero_is_reported() {
            assert_eq!(T::ONE.div_rem(T::ZERO), Err(DivError::ZeroDiv));
            assert_eq!(T::ONE.div_rem_euclid(T::ZERO), Err(DivError::ZeroDiv));
            assert_eq!(T::ZERO.div_rem(T::ZERO), Err(DivError::ZeroDiv));
            assert_eq!(T::ONE.checked_div(T::ZERO), None);
            assert_eq!(T::ONE.checked_rem(T::ZERO), None);
            assert_eq!(T::ONE.checked_div_euclid(T::ZERO), None);
            assert_eq!(T::ONE.checked_rem_euclid(T::ZERO), None);
        }

        panics! {
            div_by_zero_panics: "attempt to perform division by zero" => T::ONE / T::ZERO;
            rem_by_zero_panics: "attempt to perform division by zero" => T::ONE % T::ZERO;
            div_assign_by_zero_panics: "attempt to perform division by zero" => {
                let mut value = T::ONE;
                value /= T::ZERO;
                value
            };
            rem_assign_by_zero_panics: "attempt to perform division by zero" => {
                let mut value = T::ONE;
                value %= T::ZERO;
                value
            };
            overflowing_div_by_zero_panics: "attempt to perform division by zero" =>
                T::ONE.overflowing_div(T::ZERO);
            saturating_div_by_zero_panics: "attempt to perform division by zero" =>
                T::ONE.saturating_div(T::ZERO);
            wrapping_div_by_zero_panics: "attempt to perform division by zero" =>
                T::ONE.wrapping_div(T::ZERO);
            overflowing_rem_by_zero_panics: "attempt to perform division by zero" =>
                T::ONE.overflowing_rem(T::ZERO);
            wrapping_rem_by_zero_panics: "attempt to perform division by zero" =>
                T::ONE.wrapping_rem(T::ZERO);
            div_euclid_by_zero_panics: "attempt to perform division by zero" =>
                T::ONE.div_euclid(T::ZERO);
            rem_euclid_by_zero_panics: "attempt to perform division by zero" =>
                T::ONE.rem_euclid(T::ZERO);
            overflowing_div_euclid_by_zero_panics: "attempt to perform division by zero" =>
                T::ONE.overflowing_div_euclid(T::ZERO);
            wrapping_div_euclid_by_zero_panics: "attempt to perform division by zero" =>
                T::ONE.wrapping_div_euclid(T::ZERO);
            overflowing_rem_euclid_by_zero_panics: "attempt to perform division by zero" =>
                T::ONE.overflowing_rem_euclid(T::ZERO);
            wrapping_rem_euclid_by_zero_panics: "attempt to perform division by zero" =>
                T::ONE.wrapping_rem_euclid(T::ZERO);
        }

        // -- Negation -----------------------------------------------------

        #[test]
        fn wrapping_neg_gives_the_two_complement() {
            assert_eq!(T::ZERO.wrapping_neg(), T::ZERO);
            assert_eq!(T::ONE.wrapping_neg(), ones());
            assert_eq!(ones().wrapping_neg(), T::ONE);
            assert_eq!(small(5).wrapping_neg().wrapping_neg(), small(5));
        }

        // -- Parsing ------------------------------------------------------

        #[test]
        fn parses_zero_and_one() {
            assert_eq!("0".parse(), Ok(T::ZERO));
            assert_eq!("1".parse(), Ok(T::ONE));
        }

        #[test]
        fn parses_a_plus_sign() {
            assert_eq!("+1".parse(), Ok(T::ONE));
            assert_eq!("+0".parse(), Ok(T::ZERO));
        }

        #[test]
        fn parses_leading_zeros_of_any_length() {
            assert_eq!(format!("{}1", "0".repeat(1000)).parse(), Ok(T::ONE));
            assert_eq!(T::from_str_radix(&format!("{}ff", "0".repeat(1000)), 16), Ok(small(255)));
        }

        #[test]
        fn parses_every_radix() {
            for radix in 2..=36 {
                let expected = small(u8::try_from(radix).expect("radix fits a byte"));
                assert_eq!(T::from_str_radix("10", radix), Ok(expected), "radix {radix}");
                let digit = char::from_digit(radix - 1, radix).expect("the largest digit");
                let expected = Ok(expected - T::ONE);
                assert_eq!(T::from_str_radix(&digit.to_string(), radix), expected);
                let upper = digit.to_ascii_uppercase().to_string();
                assert_eq!(T::from_str_radix(&upper, radix), expected);
            }
        }

        #[test]
        fn parsing_rejects_an_empty_string() {
            assert_eq!("".parse::<T>(), Err(ParseIntError(IntErrorKind::Empty)));
            assert_eq!(T::from_str_radix("", 16), Err(ParseIntError(IntErrorKind::Empty)));
        }

        #[test]
        fn parsing_rejects_a_sign_without_digits() {
            let err = ParseIntError(IntErrorKind::InvalidDigit);
            assert_eq!("+".parse::<T>(), Err(err));
            assert_eq!("-".parse::<T>(), Err(err));
        }

        #[test]
        fn parsing_rejects_repeated_signs() {
            let err = ParseIntError(IntErrorKind::InvalidDigit);
            for src in ["++1", "--1", "+-1", "-+1", "1+", "1-"] {
                assert_eq!(src.parse::<T>(), Err(err), "{src:?}");
            }
        }

        #[test]
        fn parsing_rejects_separators_and_prefixes() {
            let err = ParseIntError(IntErrorKind::InvalidDigit);
            for src in [" 1", "1 ", "1_0", "0x1", "0b1", "1.0", "1e3", "\t1", "1\n", "١", "1é"] {
                assert_eq!(src.parse::<T>(), Err(err), "{src:?}");
            }
        }

        #[test]
        fn parsing_rejects_the_digits_outside_of_the_radix() {
            let err = ParseIntError(IntErrorKind::InvalidDigit);
            assert_eq!(T::from_str_radix("2", 2), Err(err));
            assert_eq!(T::from_str_radix("8", 8), Err(err));
            assert_eq!(T::from_str_radix("a", 10), Err(err));
            assert_eq!(T::from_str_radix("g", 16), Err(err));
            assert_eq!(T::from_str_radix("z", 35), Err(err));
        }

        panics! {
            from_str_radix_rejects_the_radix_0: "radix must lie in the range" =>
                T::from_str_radix("0", 0);
            from_str_radix_rejects_the_radix_1: "radix must lie in the range" =>
                T::from_str_radix("0", 1);
            from_str_radix_rejects_the_radix_37: "radix must lie in the range" =>
                T::from_str_radix("0", 37);
        }

        // -- Formatting ---------------------------------------------------

        #[test]
        fn zero_is_formatted_as_a_single_digit() {
            assert_eq!(T::ZERO.to_string(), "0");
            assert_eq!(format!("{:?}", T::ZERO), "0");
            assert_eq!(format!("{:x}", T::ZERO), "0");
            assert_eq!(format!("{:X}", T::ZERO), "0");
            assert_eq!(format!("{:o}", T::ZERO), "0");
            assert_eq!(format!("{:b}", T::ZERO), "0");
        }

        #[test]
        fn alternate_flag_adds_the_radix_prefix() {
            assert_eq!(format!("{:#x}", T::ZERO), "0x0");
            assert_eq!(format!("{:#X}", small(255)), "0xFF");
            assert_eq!(format!("{:#o}", small(8)), "0o10");
            assert_eq!(format!("{:#b}", small(5)), "0b101");
            assert_eq!(format!("{:#}", small(5)), "5");
        }

        #[test]
        fn width_fill_and_alignment_are_supported() {
            assert_eq!(format!("{:5}", T::ONE), "    1");
            assert_eq!(format!("{:<5}|", T::ONE), "1    |");
            assert_eq!(format!("{:^5}|", T::ONE), "  1  |");
            assert_eq!(format!("{:*>4o}", T::ONE), "***1");
            assert_eq!(format!("{:05}", T::ONE), "00001");
            assert_eq!(format!("{:+}", T::ONE), "+1");
            assert_eq!(format!("{:+05}", T::ONE), "+0001");
            assert_eq!(format!("{:#06x}", T::ONE), "0x0001");
            assert_eq!(format!("{:#010b}", T::ONE), "0b00000001");
            assert_eq!(format!("{:1}", small(123)), "123");
        }

        #[test]
        fn debug_supports_the_hex_flags() {
            assert_eq!(format!("{:?}", small(255)), "255");
            assert_eq!(format!("{:x?}", small(255)), "ff");
            assert_eq!(format!("{:X?}", small(255)), "FF");
            assert_eq!(format!("{:#x?}", small(255)), "0xff");
            assert_eq!(format!("{:#X?}", small(255)), "0xFF");
            assert_eq!(format!("{:#?}", small(255)), "255");
            assert_eq!(format!("{:06x?}", small(255)), "0000ff");
        }

        #[test]
        fn formats_match_the_primitive_integers() {
            let (value, model) = (T::from(0xDEAD_BEEFu32), 0xDEAD_BEEFu32);
            assert_eq!(value.to_string(), model.to_string());
            assert_eq!(format!("{value:x}"), format!("{model:x}"));
            assert_eq!(format!("{value:X}"), format!("{model:X}"));
            assert_eq!(format!("{value:o}"), format!("{model:o}"));
            assert_eq!(format!("{value:b}"), format!("{model:b}"));
            assert_eq!(format!("{value:#x}"), format!("{model:#x}"));
            assert_eq!(format!("{value:#o}"), format!("{model:#o}"));
            assert_eq!(format!("{value:#b}"), format!("{model:#b}"));
            assert_eq!(format!("{value:>40}"), format!("{model:>40}"));
            assert_eq!(format!("{value:#040b}"), format!("{model:#040b}"));
        }

        #[test]
        fn digits_of_every_radix_cross_the_limbs() {
            let position = BITS - 4;
            let value = T::ONE << position;
            assert_eq!(format!("{value:x}"), format!("1{}", "0".repeat((position / 4) as usize)));
            assert_eq!(format!("{value:b}"), format!("1{}", "0".repeat(position as usize)));
            let digit = ["1", "2", "4"][(position % 3) as usize];
            assert_eq!(
                format!("{value:o}"),
                format!("{digit}{}", "0".repeat((position / 3) as usize))
            );
            assert_eq!(value.to_string(), pow2(position));
        }

        /// The decimal digits are produced in chunks of 19 digits; `10^18` is
        /// the largest power of ten which fits every tested type.
        #[test]
        fn decimal_digits_fill_a_whole_chunk() {
            let power = low(1_000_000_000_000_000_000);
            assert_eq!(power.to_string(), format!("1{}", "0".repeat(18)));
            assert_eq!((power - T::ONE).to_string(), "9".repeat(18));
            assert_eq!((power + T::ONE).to_string(), format!("1{}1", "0".repeat(17)));
        }

        // -- Comparison and hashing ---------------------------------------

        #[test]
        fn ordering_follows_the_values() {
            assert!(T::ZERO < T::ONE);
            assert!(T::ONE < T::MAX);
            assert!(T::MIN <= T::ZERO);
            assert!(T::MIN < T::MAX);
            assert!(T::MAX > small(200));
            assert_eq!(T::MAX.cmp(&T::MAX), Ordering::Equal);
            assert_eq!(T::MIN.partial_cmp(&T::MAX), Some(Ordering::Less));
            assert_eq!(T::ZERO.max(T::ONE), T::ONE);
            assert_eq!(T::ZERO.min(T::ONE), T::ZERO);
        }

        #[test]
        fn limbs_are_compared_from_the_most_significant_one() {
            let (mut high, mut low_limbs) = ([0; LIMBS], [u64::MAX; LIMBS]);
            high[LIMBS - 1] = 1;
            low_limbs[LIMBS - 1] = 0;
            assert!(T::from_inner(low_limbs) < T::from_inner(high));
            assert!(T::from_inner(high) > T::from_inner(low_limbs));
        }

        #[test]
        fn equal_values_have_equal_hashes() {
            let set = HashSet::from([T::ZERO, T::default(), T::from(false), T::ONE, T::from(true)]);
            assert_eq!(set.len(), 2);
        }

        // -- Byte slices --------------------------------------------------

        #[test]
        fn from_be_slice_reads_the_most_significant_byte_first() {
            let mut bytes = vec![0u8; 8 * LIMBS];
            bytes[8 * LIMBS - 1] = 1;
            assert_eq!(T::from_be_slice(&bytes), Ok(T::ONE));
            bytes[0] = 0x40;
            assert_eq!(T::from_be_slice(&bytes), Ok(T::ONE | T::ONE << (BITS - 2)));
        }

        #[test]
        fn from_le_slice_reads_the_least_significant_byte_first() {
            let mut bytes = vec![0u8; 8 * LIMBS];
            bytes[0] = 1;
            assert_eq!(T::from_le_slice(&bytes), Ok(T::ONE));
            bytes[8 * LIMBS - 1] = 0x40;
            assert_eq!(T::from_le_slice(&bytes), Ok(T::ONE | T::ONE << (BITS - 2)));
        }

        #[test]
        fn slices_fill_the_limbs_in_their_order() {
            let bytes: Vec<u8> = (0..8 * LIMBS).map(|i| i as u8).collect();
            let (chunks, rest) = bytes.as_chunks::<8>();
            assert!(rest.is_empty());
            let be = T::from_be_slice(&bytes).expect("slice of the right length");
            let le = T::from_le_slice(&bytes).expect("slice of the right length");
            for (i, chunk) in chunks.iter().enumerate() {
                assert_eq!(be.as_inner()[LIMBS - 1 - i], u64::from_be_bytes(*chunk));
                assert_eq!(le.as_inner()[i], u64::from_le_bytes(*chunk));
            }
        }

        #[test]
        fn slices_of_a_wrong_length_are_rejected() {
            let bytes = vec![0u8; 16 * LIMBS];
            for actual in [0, 1, 7, 8 * LIMBS - 1, 8 * LIMBS + 1, 8 * LIMBS + 8, 16 * LIMBS] {
                let err = ParseLengthError {
                    actual,
                    expected: 8 * LIMBS,
                };
                assert_eq!(T::from_be_slice(&bytes[..actual]), Err(err));
                assert_eq!(T::from_le_slice(&bytes[..actual]), Err(err));
            }
        }

        // -- Conversions --------------------------------------------------

        #[test]
        fn converts_from_bool() {
            assert_eq!(T::from(true), T::ONE);
            assert_eq!(T::from(false), T::ZERO);
        }

        #[test]
        fn converts_from_unsigned_primitive_integers() {
            assert_eq!(T::from(u8::MAX), low(0xFF));
            assert_eq!(T::from(u16::MAX), low(0xFFFF));
            assert_eq!(T::from(u32::MAX), low(0xFFFF_FFFF));
        }

        #[test]
        fn converts_from_small_integers() {
            assert_eq!(T::from(u1::ONE), T::ONE);
            assert_eq!(T::from(u7::MAX), low(0x7F));
            assert_eq!(T::from(u24::MAX), low(0xFF_FFFF));
            assert_eq!(T::from(u56::MAX), low(0xFF_FFFF_FFFF_FFFF));
        }
    };
}

// ---------------------------------------------------------------------------
// Unsigned integers
// ---------------------------------------------------------------------------

/// Generates the tests of the unsigned integers.
macro_rules! unsigned_tests {
    () => {
        // -- Bounds ---------------------------------------------------------

        #[test]
        fn minimum_is_zero() {
            assert_eq!(T::MIN, T::ZERO);
        }

        #[test]
        fn maximum_has_all_bits_set() {
            assert_eq!(T::MAX.into_inner(), [u64::MAX; LIMBS]);
        }

        #[test]
        fn belongs_to_the_unsigned_category() {
            fn assert_unsigned<U: Unsigned>() {}
            assert_unsigned::<T>();
        }

        #[test]
        fn no_value_is_negative() {
            assert!(!T::MAX.is_negative());
            assert!(!T::ZERO.is_negative());
            assert!(!(T::ONE << (BITS - 1)).is_negative());
        }

        #[test]
        fn all_values_but_zero_are_positive() {
            assert!(T::MAX.is_positive());
            assert!(T::ONE.is_positive());
            assert!((T::ONE << (BITS - 1)).is_positive());
            assert!(!T::ZERO.is_positive());
        }

        #[test]
        fn bits_required_counts_the_significant_bits() {
            assert_eq!(T::ZERO.bits_required(), 0);
            assert_eq!(T::ONE.bits_required(), 1);
            assert_eq!(small(255).bits_required(), 8);
            assert_eq!(small(128).bits_required(), 8);
            assert_eq!((T::ONE << (BITS - 1)).bits_required(), BITS);
            assert_eq!(T::MAX.bits_required(), BITS);
        }

        // -- Overflow at the bounds -----------------------------------------

        #[test]
        fn add_above_the_maximum() {
            assert_eq!(T::MAX.checked_add(T::ONE), None);
            assert_eq!(T::MAX.overflowing_add(T::ONE), (T::ZERO, true));
            assert_eq!(T::MAX.saturating_add(T::ONE), T::MAX);
            assert_eq!(T::MAX.wrapping_add(T::ONE), T::ZERO);
        }

        #[test]
        fn add_of_the_maximums() {
            assert_eq!(T::MAX.checked_add(T::MAX), None);
            assert_eq!(T::MAX.overflowing_add(T::MAX), (T::MAX - T::ONE, true));
            assert_eq!(T::MAX.saturating_add(T::MAX), T::MAX);
            assert_eq!(T::MAX.wrapping_add(T::MAX), T::MAX - T::ONE);
        }

        #[test]
        fn sub_below_zero() {
            assert_eq!(T::ZERO.checked_sub(T::ONE), None);
            assert_eq!(T::ZERO.overflowing_sub(T::ONE), (T::MAX, true));
            assert_eq!(T::ZERO.saturating_sub(T::ONE), T::ZERO);
            assert_eq!(T::ZERO.wrapping_sub(T::ONE), T::MAX);
        }

        #[test]
        fn sub_of_the_maximum_from_zero() {
            assert_eq!(T::ZERO.checked_sub(T::MAX), None);
            assert_eq!(T::ZERO.overflowing_sub(T::MAX), (T::ONE, true));
            assert_eq!(T::ZERO.saturating_sub(T::MAX), T::ZERO);
            assert_eq!(T::ZERO.wrapping_sub(T::MAX), T::ONE);
        }

        #[test]
        fn mul_above_the_maximum() {
            assert_eq!(T::MAX.checked_mul(small(2)), None);
            assert_eq!(T::MAX.overflowing_mul(small(2)), (T::MAX - T::ONE, true));
            assert_eq!(T::MAX.saturating_mul(small(2)), T::MAX);
            assert_eq!(T::MAX.wrapping_mul(small(2)), T::MAX - T::ONE);
        }

        #[test]
        fn mul_of_the_maximums() {
            assert_eq!(T::MAX.checked_mul(T::MAX), None);
            assert_eq!(T::MAX.overflowing_mul(T::MAX), (T::ONE, true));
            assert_eq!(T::MAX.saturating_mul(T::MAX), T::MAX);
            assert_eq!(T::MAX.wrapping_mul(T::MAX), T::ONE);
        }

        #[test]
        fn mul_overflowing_by_a_single_bit() {
            let half = T::ONE << (BITS / 2);
            assert_eq!(half.checked_mul(half), None);
            assert_eq!(half.overflowing_mul(half), (T::ZERO, true));
            assert_eq!(half.saturating_mul(half), T::MAX);
            let top = T::ONE << (BITS - 1);
            assert_eq!(top.checked_mul(small(2)), None);
            assert_eq!((half >> 1).checked_mul(half), Some(top));
        }

        #[test]
        fn division_never_overflows() {
            assert_eq!(T::MAX.div_rem(T::ONE), Ok((T::MAX, T::ZERO)));
            assert_eq!(T::MAX.overflowing_div(T::ONE), (T::MAX, false));
            assert_eq!(T::MAX.saturating_div(T::ONE), T::MAX);
            assert_eq!(T::MAX.wrapping_div(T::ONE), T::MAX);
            assert_eq!(T::MAX.overflowing_rem(T::MAX), (T::ZERO, false));
            assert_eq!(T::MAX.div_rem_euclid(T::MAX), Ok((T::ONE, T::ZERO)));
            assert_eq!(T::MAX.overflowing_div_euclid(T::ONE), (T::MAX, false));
            assert_eq!(T::MAX.overflowing_rem_euclid(T::ONE), (T::ZERO, false));
        }

        #[test]
        fn division_of_the_maximum_by_two() {
            let half = T::MAX >> 1;
            assert_eq!(T::MAX.div_rem(small(2)), Ok((half, T::ONE)));
            assert_eq!(T::MAX.div_rem(half), Ok((small(2), T::ONE)));
            assert_eq!(T::MAX.div_rem(T::ONE << (BITS - 1)), Ok((T::ONE, half)));
        }

        #[test]
        fn wrapping_neg_of_the_maximum_is_one() {
            assert_eq!(T::MAX.wrapping_neg(), T::ONE);
        }

        panics! {
            add_overflow_panics: "attempt to add with overflow" => T::MAX + T::ONE;
            sub_overflow_panics: "attempt to subtract with overflow" => T::ZERO - T::ONE;
            mul_overflow_panics: "attempt to multiply with overflow" => T::MAX * small(2);
            add_assign_overflow_panics: "attempt to add with overflow" => {
                let mut value = T::MAX;
                value += 1u8;
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
        }

        // -- Shifts ---------------------------------------------------------

        #[test]
        fn shr_fills_the_upper_bits_with_zeros() {
            assert_eq!(T::MAX >> (BITS - 1), T::ONE);
            assert_eq!((T::ONE << (BITS - 1)) >> (BITS - 1), T::ONE);
            assert_eq!((T::MAX >> 1).leading_zeros(), 1);
        }

        #[test]
        fn shl_discards_the_upper_bits() {
            let top = T::MAX << (BITS - 1);
            assert_eq!((top.leading_zeros(), top.trailing_zeros()), (0, BITS - 1));
            assert_eq!(T::MAX << 1, T::MAX - T::ONE);
        }

        #[test]
        fn unbounded_shr_gives_zero_from_the_bit_size() {
            assert_eq!(T::MAX.unbounded_shr(BITS - 1), T::ONE);
            assert_eq!(T::MAX.unbounded_shr(BITS), T::ZERO);
            assert_eq!(T::MAX.unbounded_shr(u32::MAX), T::ZERO);
        }

        // -- Casting --------------------------------------------------------

        #[test]
        fn cast_signed_keeps_the_bits() {
            assert_eq!(T::MAX.cast_signed(), <Signed as Integer>::NEG_ONE);
            assert_eq!(T::ZERO.cast_signed(), Signed::ZERO);
            assert_eq!((T::ONE << (BITS - 1)).cast_signed(), Signed::MIN);
            assert_eq!((T::MAX >> 1).cast_signed(), Signed::MAX);
            assert_eq!(T::MAX.cast_signed().cast_unsigned(), T::MAX);
        }

        #[test]
        fn converts_from_the_largest_fitting_primitive_integer() {
            assert_eq!(T::from(u64::MAX), low(u64::MAX));
            assert_eq!(T::from(usize::MAX).bits_required(), usize::BITS);
        }

        // -- Parsing and formatting of the bounds ---------------------------

        #[test]
        fn maximum_is_formatted_in_every_radix() {
            assert_eq!(format!("{:x}", T::MAX), "f".repeat(2 * T::BYTES));
            assert_eq!(format!("{:X}", T::MAX), "F".repeat(2 * T::BYTES));
            assert_eq!(format!("{:b}", T::MAX), "1".repeat(8 * T::BYTES));
            assert_eq!(format!("{:o}", T::MAX), octal_ones(BITS));
            assert_eq!(format!("{:#x}", T::MAX), format!("0x{}", "f".repeat(2 * T::BYTES)));
        }

        #[test]
        fn maximum_is_formatted_as_a_decimal_power_of_two() {
            let expected = minus_one(&pow2(BITS));
            assert_eq!(T::MAX.to_string(), expected);
            assert_eq!(format!("{:?}", T::MAX), expected);
            assert_eq!(format!("{:+}", T::MAX), format!("+{expected}"));
        }

        #[test]
        fn top_bit_is_formatted_as_a_decimal_power_of_two() {
            assert_eq!((T::ONE << (BITS - 1)).to_string(), pow2(BITS - 1));
        }

        /// All chunks but the most significant one are padded with zeros.
        #[test]
        fn decimal_digits_cross_the_chunks_of_19_digits() {
            let chunk = low(10_000_000_000_000_000_000);
            assert_eq!(chunk.to_string(), format!("1{}", "0".repeat(19)));
            assert_eq!((chunk - T::ONE).to_string(), "9".repeat(19));
            assert_eq!((chunk + T::ONE).to_string(), format!("1{}1", "0".repeat(18)));
            assert_eq!(low(u64::MAX).to_string(), u64::MAX.to_string());
        }

        #[test]
        fn parses_the_maximum_in_every_radix() {
            assert_eq!(minus_one(&pow2(BITS)).parse(), Ok(T::MAX));
            assert_eq!(T::from_str_radix(&"f".repeat(2 * T::BYTES), 16), Ok(T::MAX));
            assert_eq!(T::from_str_radix(&"F".repeat(2 * T::BYTES), 16), Ok(T::MAX));
            assert_eq!(T::from_str_radix(&"1".repeat(8 * T::BYTES), 2), Ok(T::MAX));
            assert_eq!(T::from_str_radix(&octal_ones(BITS), 8), Ok(T::MAX));
        }

        #[test]
        fn parsing_rejects_the_value_above_the_maximum() {
            let err = ParseIntError(IntErrorKind::PosOverflow);
            assert_eq!(pow2(BITS).parse::<T>(), Err(err));
            assert_eq!(format!("{}0", minus_one(&pow2(BITS))).parse::<T>(), Err(err));
            assert_eq!(T::from_str_radix(&format!("1{}", "0".repeat(2 * T::BYTES)), 16), Err(err));
            assert_eq!(T::from_str_radix(&"f".repeat(2 * T::BYTES + 1), 16), Err(err));
            assert_eq!(T::from_str_radix(&"1".repeat(8 * T::BYTES + 1), 2), Err(err));
            assert_eq!(T::from_str_radix(&"z".repeat(8 * T::BYTES), 36), Err(err));
        }

        #[test]
        fn parsing_rejects_a_minus_sign() {
            let err = ParseIntError(IntErrorKind::InvalidDigit);
            assert_eq!("-1".parse::<T>(), Err(err));
            assert_eq!("-0".parse::<T>(), Err(err));
        }

        /// The first error met in the string is reported, as the primitive
        /// integers do.
        #[test]
        fn parsing_reports_the_first_of_overflow_and_invalid_digit() {
            let max = minus_one(&pow2(BITS));
            let invalid = ParseIntError(IntErrorKind::InvalidDigit);
            let overflow = ParseIntError(IntErrorKind::PosOverflow);
            assert_eq!(format!("{max}x").parse::<T>(), Err(invalid));
            assert_eq!(format!("{}x", pow2(BITS)).parse::<T>(), Err(overflow));
            assert_eq!(format!("x{}", pow2(BITS)).parse::<T>(), Err(invalid));
        }
    };
}

/// Generates the modules with the tests of the unsigned integers of the
/// given sizes.
macro_rules! unsigned_suite {
    ($($module:ident: $limbs:literal limbs;)+) => {$(
        mod $module {
            use crate::*;

            type T = BigInt<$limbs, false>;
            type Signed = BigInt<$limbs, true>;
            const LIMBS: usize = $limbs;
            const BITS: u32 = 64 * $limbs;

            common_tests!();
            unsigned_tests!();
        }
    )+};
}

mod unsigned {
    unsigned_suite! {
        bits64: 1 limbs;
        bits192: 3 limbs;
        bits256: 4 limbs;
        bits512: 8 limbs;
        bits1024: 16 limbs;
    }
}

// ---------------------------------------------------------------------------
// Signed integers
// ---------------------------------------------------------------------------

/// Generates the tests of the signed integers.
macro_rules! signed_tests {
    () => {
        const NEG_ONE: T = <T as Integer>::NEG_ONE;

        /// A small number of any sign.
        fn int(value: i8) -> T { T::from(value) }

        // -- Bounds ---------------------------------------------------------

        #[test]
        fn minimum_has_only_the_sign_bit_set() {
            let mut limbs = [0; LIMBS];
            limbs[LIMBS - 1] = 1 << 63;
            assert_eq!(T::MIN.into_inner(), limbs);
            assert_eq!(T::MIN, T::ONE << (BITS - 1));
        }

        #[test]
        fn maximum_has_all_bits_but_the_sign_set() {
            let mut limbs = [u64::MAX; LIMBS];
            limbs[LIMBS - 1] = u64::MAX >> 1;
            assert_eq!(T::MAX.into_inner(), limbs);
            assert_eq!(T::MAX, !T::MIN);
        }

        #[test]
        fn negative_one_has_all_bits_set() {
            assert_eq!(NEG_ONE.into_inner(), [u64::MAX; LIMBS]);
            assert_eq!(NEG_ONE, int(-1));
            assert_eq!(NEG_ONE, -T::ONE);
        }

        #[test]
        fn belongs_to_the_signed_category() {
            fn assert_signed<I: Integer>() {}
            assert_signed::<T>();
        }

        #[test]
        fn is_negative_follows_the_sign_bit() {
            for (value, negative) in [
                (T::MIN, true),
                (NEG_ONE, true),
                (T::ZERO, false),
                (T::ONE, false),
                (T::MAX, false),
            ] {
                assert_eq!(T::is_negative(&value), negative, "{value}");
                assert_eq!(Integer::is_negative(value), negative, "{value}");
            }
        }

        #[test]
        fn is_positive_excludes_zero_and_the_negative_numbers() {
            assert!(T::MAX.is_positive());
            assert!(T::ONE.is_positive());
            assert!(!T::ZERO.is_positive());
            assert!(!NEG_ONE.is_positive());
            assert!(!T::MIN.is_positive());
        }

        #[test]
        fn bits_required_includes_the_sign_bit() {
            assert_eq!(T::ZERO.bits_required(), 1);
            assert_eq!(NEG_ONE.bits_required(), 1);
            assert_eq!(T::ONE.bits_required(), 2);
            assert_eq!(int(-2).bits_required(), 2);
            assert_eq!(int(127).bits_required(), 8);
            assert_eq!(int(-128).bits_required(), 8);
            assert_eq!(T::MAX.bits_required(), BITS);
            assert_eq!(T::MIN.bits_required(), BITS);
        }

        #[test]
        fn leading_bits_follow_the_sign() {
            assert_eq!(T::MAX.leading_zeros(), 1);
            assert_eq!(T::MIN.leading_ones(), 1);
            assert_eq!(T::MIN.trailing_zeros(), BITS - 1);
            assert_eq!(T::MAX.trailing_ones(), BITS - 1);
            assert_eq!(int(-2).leading_ones(), BITS - 1);
        }

        // -- Overflow of the addition ---------------------------------------

        #[test]
        fn add_above_the_maximum() {
            assert_eq!(T::MAX.checked_add(T::ONE), None);
            assert_eq!(T::MAX.overflowing_add(T::ONE), (T::MIN, true));
            assert_eq!(T::MAX.saturating_add(T::ONE), T::MAX);
            assert_eq!(T::MAX.wrapping_add(T::ONE), T::MIN);
        }

        #[test]
        fn add_below_the_minimum() {
            assert_eq!(T::MIN.checked_add(NEG_ONE), None);
            assert_eq!(T::MIN.overflowing_add(NEG_ONE), (T::MAX, true));
            assert_eq!(T::MIN.saturating_add(NEG_ONE), T::MIN);
            assert_eq!(T::MIN.wrapping_add(NEG_ONE), T::MAX);
        }

        #[test]
        fn add_of_the_minimums() {
            assert_eq!(T::MIN.checked_add(T::MIN), None);
            assert_eq!(T::MIN.overflowing_add(T::MIN), (T::ZERO, true));
            assert_eq!(T::MIN.saturating_add(T::MIN), T::MIN);
        }

        #[test]
        fn add_of_the_maximums() {
            assert_eq!(T::MAX.checked_add(T::MAX), None);
            assert_eq!(T::MAX.overflowing_add(T::MAX), (int(-2), true));
            assert_eq!(T::MAX.saturating_add(T::MAX), T::MAX);
        }

        #[test]
        fn add_of_the_opposite_bounds_fits() {
            assert_eq!(T::MIN + T::MAX, NEG_ONE);
            assert_eq!(T::MAX.checked_add(T::MIN), Some(NEG_ONE));
            assert_eq!(T::MIN.overflowing_add(T::MAX), (NEG_ONE, false));
            assert_eq!(T::MIN.saturating_add(T::MAX), NEG_ONE);
        }

        // -- Overflow of the subtraction ------------------------------------

        #[test]
        fn sub_below_the_minimum() {
            assert_eq!(T::MIN.checked_sub(T::ONE), None);
            assert_eq!(T::MIN.overflowing_sub(T::ONE), (T::MAX, true));
            assert_eq!(T::MIN.saturating_sub(T::ONE), T::MIN);
            assert_eq!(T::MIN.wrapping_sub(T::ONE), T::MAX);
        }

        #[test]
        fn sub_above_the_maximum() {
            assert_eq!(T::MAX.checked_sub(NEG_ONE), None);
            assert_eq!(T::MAX.overflowing_sub(NEG_ONE), (T::MIN, true));
            assert_eq!(T::MAX.saturating_sub(NEG_ONE), T::MAX);
            assert_eq!(T::MAX.wrapping_sub(NEG_ONE), T::MIN);
        }

        #[test]
        fn sub_of_the_minimum_from_zero() {
            assert_eq!(T::ZERO.checked_sub(T::MIN), None);
            assert_eq!(T::ZERO.overflowing_sub(T::MIN), (T::MIN, true));
            assert_eq!(T::ZERO.saturating_sub(T::MIN), T::MAX);
            assert_eq!(T::ZERO.wrapping_sub(T::MIN), T::MIN);
        }

        #[test]
        fn sub_of_the_opposite_bounds() {
            assert_eq!(T::MAX.overflowing_sub(T::MIN), (NEG_ONE, true));
            assert_eq!(T::MAX.saturating_sub(T::MIN), T::MAX);
            assert_eq!(T::MIN.overflowing_sub(T::MAX), (T::ONE, true));
            assert_eq!(T::MIN.saturating_sub(T::MAX), T::MIN);
        }

        #[test]
        fn sub_of_the_equal_bounds_fits() {
            assert_eq!(T::MIN - T::MIN, T::ZERO);
            assert_eq!(T::MAX - T::MAX, T::ZERO);
            assert_eq!(NEG_ONE - T::MAX, T::MIN);
            assert_eq!(NEG_ONE.checked_sub(T::MIN), Some(T::MAX));
        }

        // -- Overflow of the multiplication ---------------------------------

        #[test]
        fn mul_above_the_maximum() {
            assert_eq!(T::MAX.checked_mul(int(2)), None);
            assert_eq!(T::MAX.overflowing_mul(int(2)), (int(-2), true));
            assert_eq!(T::MAX.saturating_mul(int(2)), T::MAX);
            assert_eq!(T::MAX.wrapping_mul(int(2)), int(-2));
        }

        #[test]
        fn mul_below_the_minimum() {
            assert_eq!(T::MIN.checked_mul(int(2)), None);
            assert_eq!(T::MIN.overflowing_mul(int(2)), (T::ZERO, true));
            assert_eq!(T::MIN.saturating_mul(int(2)), T::MIN);
            assert_eq!(T::MAX.saturating_mul(int(-2)), T::MIN);
            assert_eq!(T::MAX.overflowing_mul(int(-2)), (int(2), true));
        }

        #[test]
        fn mul_of_the_minimum_by_negative_one() {
            assert_eq!(T::MIN.checked_mul(NEG_ONE), None);
            assert_eq!(T::MIN.overflowing_mul(NEG_ONE), (T::MIN, true));
            assert_eq!(T::MIN.saturating_mul(NEG_ONE), T::MAX);
            assert_eq!(T::MIN.wrapping_mul(NEG_ONE), T::MIN);
            assert_eq!(NEG_ONE.overflowing_mul(T::MIN), (T::MIN, true));
        }

        #[test]
        fn mul_of_the_bounds() {
            assert_eq!(T::MAX.overflowing_mul(T::MAX), (T::ONE, true));
            assert_eq!(T::MAX.saturating_mul(T::MAX), T::MAX);
            assert_eq!(T::MIN.overflowing_mul(T::MIN), (T::ZERO, true));
            assert_eq!(T::MIN.saturating_mul(T::MIN), T::MAX);
            assert_eq!(T::MAX.overflowing_mul(T::MIN), (T::MIN, true));
            assert_eq!(T::MAX.saturating_mul(T::MIN), T::MIN);
        }

        #[test]
        fn mul_reaching_the_bounds_fits() {
            assert_eq!(T::MIN * T::ONE, T::MIN);
            assert_eq!(T::MAX * NEG_ONE, T::MIN + T::ONE);
            assert_eq!(NEG_ONE * NEG_ONE, T::ONE);
            assert_eq!((T::ONE << (BITS - 2)) * int(-2), T::MIN);
            assert_eq!((T::ONE << (BITS - 2)).checked_mul(int(2)), None);
            assert_eq!(T::MIN.overflowing_mul(T::ONE), (T::MIN, false));
        }

        #[test]
        fn mul_follows_the_signs_of_the_operands() {
            assert_eq!(int(-7) * int(6), int(-42));
            assert_eq!(int(7) * int(-6), int(-42));
            assert_eq!(int(-7) * int(-6), int(42));
            assert_eq!(int(-7) * T::ZERO, T::ZERO);
            assert_eq!(int(-7).overflowing_mul(T::ZERO), (T::ZERO, false));
        }

        // -- Negation and absolute value ------------------------------------

        #[test]
        fn neg_changes_the_sign() {
            assert_eq!(-T::ONE, NEG_ONE);
            assert_eq!(-NEG_ONE, T::ONE);
            assert_eq!(-T::ZERO, T::ZERO);
            assert_eq!(-T::MAX, T::MIN + T::ONE);
            assert_eq!(T::MAX.checked_neg(), Some(T::MIN + T::ONE));
            assert_eq!(T::MAX.overflowing_neg(), (T::MIN + T::ONE, false));
            assert_eq!(T::MAX.saturating_neg(), T::MIN + T::ONE);
            assert_eq!(T::MAX.wrapping_neg(), T::MIN + T::ONE);
        }

        #[test]
        fn neg_of_the_minimum_overflows() {
            assert_eq!(T::MIN.checked_neg(), None);
            assert_eq!(T::MIN.overflowing_neg(), (T::MIN, true));
            assert_eq!(T::MIN.saturating_neg(), T::MAX);
            assert_eq!(T::MIN.wrapping_neg(), T::MIN);
        }

        #[test]
        fn abs_drops_the_sign() {
            assert_eq!(int(-5).abs(), int(5));
            assert_eq!(int(5).abs(), int(5));
            assert_eq!(T::ZERO.abs(), T::ZERO);
            assert_eq!(T::MAX.abs(), T::MAX);
            assert_eq!((T::MIN + T::ONE).abs(), T::MAX);
        }

        #[test]
        fn unsigned_abs_fits_the_minimum() {
            assert_eq!(T::MIN.unsigned_abs(), Unsigned::ONE << (BITS - 1));
            assert_eq!(int(-5).unsigned_abs(), Unsigned::from(5u8));
            assert_eq!(int(5).unsigned_abs(), Unsigned::from(5u8));
            assert_eq!(T::MAX.unsigned_abs(), T::MAX.cast_unsigned());
            assert_eq!(T::ZERO.unsigned_abs(), Unsigned::ZERO);
        }

        #[test]
        fn cast_unsigned_keeps_the_bits() {
            assert_eq!(NEG_ONE.cast_unsigned(), Unsigned::MAX);
            assert_eq!(T::MIN.cast_unsigned(), Unsigned::ONE << (BITS - 1));
            assert_eq!(T::MAX.cast_unsigned(), Unsigned::MAX >> 1);
            assert_eq!(T::MIN.cast_unsigned().cast_signed(), T::MIN);
        }

        panics! {
            add_overflow_panics: "attempt to add with overflow" => T::MAX + T::ONE;
            add_negative_overflow_panics: "attempt to add with overflow" => T::MIN + NEG_ONE;
            sub_overflow_panics: "attempt to subtract with overflow" => T::MIN - T::ONE;
            sub_positive_overflow_panics: "attempt to subtract with overflow" => T::MAX - NEG_ONE;
            mul_overflow_panics: "attempt to multiply with overflow" => T::MAX * int(2);
            mul_negative_overflow_panics: "attempt to multiply with overflow" => T::MIN * int(2);
            mul_of_the_minimum_by_negative_one_panics: "attempt to multiply with overflow" =>
                T::MIN * NEG_ONE;
            neg_of_the_minimum_panics: "attempt to negate with overflow" => -T::MIN;
            abs_of_the_minimum_panics: "attempt to negate with overflow" => T::MIN.abs();
            add_assign_overflow_panics: "attempt to add with overflow" => {
                let mut value = T::MAX;
                value += 1i8;
                value
            };
            sub_assign_overflow_panics: "attempt to subtract with overflow" => {
                let mut value = T::MIN;
                value -= 1i8;
                value
            };
            mul_assign_overflow_panics: "attempt to multiply with overflow" => {
                let mut value = T::MIN;
                value *= -1i8;
                value
            };
        }

        // -- Division -------------------------------------------------------

        #[test]
        fn division_truncates_towards_zero() {
            assert_eq!(int(-7).div_rem(int(2)), Ok((int(-3), int(-1))));
            assert_eq!(int(7).div_rem(int(-2)), Ok((int(-3), int(1))));
            assert_eq!(int(-7).div_rem(int(-2)), Ok((int(3), int(-1))));
            assert_eq!(int(-7) / int(2), int(-3));
            assert_eq!(int(-7) % int(2), int(-1));
            assert_eq!(int(-1) / int(2), T::ZERO);
            assert_eq!(int(-1) % int(2), int(-1));
        }

        #[test]
        fn euclidean_division_gives_a_non_negative_remainder() {
            assert_eq!(int(-7).div_rem_euclid(int(2)), Ok((int(-4), int(1))));
            assert_eq!(int(7).div_rem_euclid(int(-2)), Ok((int(-3), int(1))));
            assert_eq!(int(-7).div_rem_euclid(int(-2)), Ok((int(4), int(1))));
            assert_eq!(int(-8).div_rem_euclid(int(2)), Ok((int(-4), T::ZERO)));
            assert_eq!(int(-7).div_euclid(int(2)), int(-4));
            assert_eq!(int(-7).rem_euclid(int(2)), int(1));
            assert_eq!(int(-7).checked_div_euclid(int(-2)), Some(int(4)));
            assert_eq!(int(-7).checked_rem_euclid(int(-2)), Some(int(1)));
            assert_eq!(int(-7).overflowing_div_euclid(int(2)), (int(-4), false));
            assert_eq!(int(-7).overflowing_rem_euclid(int(2)), (int(1), false));
            assert_eq!(int(-7).wrapping_div_euclid(int(2)), int(-4));
            assert_eq!(int(-7).wrapping_rem_euclid(int(2)), int(1));
        }

        #[test]
        fn division_of_the_bounds() {
            assert_eq!(T::MIN.div_rem(T::MIN), Ok((T::ONE, T::ZERO)));
            assert_eq!(T::MIN.div_rem(T::MAX), Ok((NEG_ONE, NEG_ONE)));
            assert_eq!(T::MAX.div_rem(T::MIN), Ok((T::ZERO, T::MAX)));
            assert_eq!(T::MIN.div_rem(T::ONE), Ok((T::MIN, T::ZERO)));
            assert_eq!(T::MAX.div_rem(NEG_ONE), Ok((T::MIN + T::ONE, T::ZERO)));
            assert_eq!(T::MIN.div_rem(int(2)), Ok((T::MIN >> 1, T::ZERO)));
            assert_eq!(T::MIN.div_rem(int(-2)), Ok((T::ONE << (BITS - 2), T::ZERO)));
        }

        #[test]
        fn euclidean_division_of_the_bounds() {
            assert_eq!(T::MIN.div_rem_euclid(T::MAX), Ok((int(-2), T::MAX - T::ONE)));
            assert_eq!(T::MAX.div_rem_euclid(T::MIN), Ok((T::ZERO, T::MAX)));
            assert_eq!(T::MIN.div_rem_euclid(T::MIN), Ok((T::ONE, T::ZERO)));
            assert_eq!(NEG_ONE.div_rem_euclid(T::MIN), Ok((T::ONE, T::MAX)));
            assert_eq!(NEG_ONE.div_rem_euclid(T::MAX), Ok((NEG_ONE, T::MAX - T::ONE)));
        }

        #[test]
        fn division_of_the_minimum_by_negative_one_overflows() {
            assert_eq!(T::MIN.div_rem(NEG_ONE), Err(DivError::Overflow));
            assert_eq!(T::MIN.checked_div(NEG_ONE), None);
            assert_eq!(T::MIN.overflowing_div(NEG_ONE), (T::MIN, true));
            assert_eq!(T::MIN.saturating_div(NEG_ONE), T::MAX);
            assert_eq!(T::MIN.wrapping_div(NEG_ONE), T::MIN);
        }

        #[test]
        fn remainder_of_the_minimum_by_negative_one_overflows() {
            assert_eq!(T::MIN.checked_rem(NEG_ONE), None);
            assert_eq!(T::MIN.overflowing_rem(NEG_ONE), (T::ZERO, true));
            assert_eq!(T::MIN.wrapping_rem(NEG_ONE), T::ZERO);
        }

        #[test]
        fn euclidean_division_of_the_minimum_by_negative_one_overflows() {
            assert_eq!(T::MIN.div_rem_euclid(NEG_ONE), Err(DivError::Overflow));
            assert_eq!(T::MIN.checked_div_euclid(NEG_ONE), None);
            assert_eq!(T::MIN.overflowing_div_euclid(NEG_ONE), (T::MIN, true));
            assert_eq!(T::MIN.wrapping_div_euclid(NEG_ONE), T::MIN);
            assert_eq!(T::MIN.checked_rem_euclid(NEG_ONE), None);
            assert_eq!(T::MIN.overflowing_rem_euclid(NEG_ONE), (T::ZERO, true));
            assert_eq!(T::MIN.wrapping_rem_euclid(NEG_ONE), T::ZERO);
        }

        panics! {
            div_overflow_panics: "attempt to perform division with overflow" => T::MIN / NEG_ONE;
            rem_overflow_panics: "attempt to perform division with overflow" => T::MIN % NEG_ONE;
            div_euclid_overflow_panics: "attempt to perform division with overflow" =>
                T::MIN.div_euclid(NEG_ONE);
            rem_euclid_overflow_panics: "attempt to perform division with overflow" =>
                T::MIN.rem_euclid(NEG_ONE);
            div_assign_overflow_panics: "attempt to perform division with overflow" => {
                let mut value = T::MIN;
                value /= NEG_ONE;
                value
            };
        }

        // -- Shifts ---------------------------------------------------------

        #[test]
        fn shr_keeps_the_sign() {
            assert_eq!(T::MIN >> (BITS - 1), NEG_ONE);
            assert_eq!(NEG_ONE >> 1, NEG_ONE);
            assert_eq!(NEG_ONE >> (BITS - 1), NEG_ONE);
            assert_eq!(int(-4) >> 1, int(-2));
            assert_eq!(int(-3) >> 1, int(-2));
            assert_eq!(T::MAX >> (BITS - 2), T::ONE);
        }

        #[test]
        fn shl_moves_the_bits_into_the_sign() {
            assert_eq!(T::ONE << (BITS - 1), T::MIN);
            assert_eq!(NEG_ONE << (BITS - 1), T::MIN);
            assert_eq!(T::MAX << 1, int(-2));
            assert_eq!(int(-1) << 1, int(-2));
        }

        #[test]
        fn unbounded_shr_gives_the_sign_from_the_bit_size() {
            assert_eq!(T::MIN.unbounded_shr(BITS), NEG_ONE);
            assert_eq!(T::MIN.unbounded_shr(u32::MAX), NEG_ONE);
            assert_eq!(NEG_ONE.unbounded_shr(BITS + 1), NEG_ONE);
            assert_eq!(T::MAX.unbounded_shr(BITS), T::ZERO);
            assert_eq!(T::MAX.unbounded_shr(BITS - 2), T::ONE);
        }

        #[test]
        fn wrapped_shifts_of_the_negative_numbers_keep_the_sign() {
            assert_eq!(T::MIN.overflowing_shr(BITS + 1), (T::MIN >> 1, true));
            assert_eq!(T::MIN.wrapping_shr(BITS + 1), T::MIN >> 1);
            assert_eq!(NEG_ONE.overflowing_shr(BITS), (NEG_ONE, true));
            assert_eq!(NEG_ONE.checked_shr(BITS - 1), Some(NEG_ONE));
            assert_eq!(NEG_ONE.checked_shr(BITS), None);
        }

        // -- Comparison -----------------------------------------------------

        #[test]
        fn negative_numbers_are_ordered_below_the_positive_ones() {
            let mut values = [T::ONE, T::MIN, NEG_ONE, T::MAX, T::ZERO, int(-2)];
            values.sort();
            assert_eq!(values, [T::MIN, int(-2), NEG_ONE, T::ZERO, T::ONE, T::MAX]);
            assert!(NEG_ONE < T::ZERO);
            assert!(T::MIN < NEG_ONE);
            assert_eq!(T::MIN.cmp(&T::MAX), Ordering::Less);
            assert_eq!(T::MAX.cmp(&T::MIN), Ordering::Greater);
        }

        // -- Conversions ----------------------------------------------------

        #[test]
        fn converts_from_signed_primitive_integers_extending_the_sign() {
            assert_eq!(T::from(-1i8), NEG_ONE);
            assert_eq!(T::from(-1i64), NEG_ONE);
            assert_eq!(T::from(-1isize), NEG_ONE);
            assert_eq!(T::from(i8::MIN), "-128".parse().expect("valid number"));
            assert_eq!(T::from(i8::MAX), small(127));
            let mut limbs = [u64::MAX; LIMBS];
            limbs[0] = 1 << 63;
            assert_eq!(T::from(i64::MIN).into_inner(), limbs);
            assert_eq!(T::from(i64::MAX), low(u64::MAX >> 1));
        }

        // -- Parsing and formatting of the bounds ---------------------------

        #[test]
        fn bounds_are_formatted_as_the_decimal_powers_of_two() {
            let power = pow2(BITS - 1);
            assert_eq!(T::MIN.to_string(), format!("-{power}"));
            assert_eq!(T::MAX.to_string(), minus_one(&power));
            assert_eq!(format!("{:?}", T::MIN), format!("-{power}"));
            assert_eq!(format!("{:+}", T::MAX), format!("+{}", minus_one(&power)));
            assert_eq!(format!("{:+}", T::MIN), format!("-{power}"));
        }

        #[test]
        fn negative_decimal_digits_fill_a_whole_chunk() {
            let power = -low(1_000_000_000_000_000_000);
            assert_eq!(power.to_string(), format!("-1{}", "0".repeat(18)));
            assert_eq!((power + T::ONE).to_string(), format!("-{}", "9".repeat(18)));
            assert_eq!(T::from(i64::MIN).to_string(), i64::MIN.to_string());
            assert_eq!(T::from(i64::MAX).to_string(), i64::MAX.to_string());
        }

        #[test]
        fn negative_numbers_are_formatted_with_the_sign() {
            assert_eq!(NEG_ONE.to_string(), "-1");
            assert_eq!(format!("{NEG_ONE:?}"), "-1");
            assert_eq!(format!("{NEG_ONE:5}"), "   -1");
            assert_eq!(format!("{NEG_ONE:<5}|"), "-1   |");
            assert_eq!(format!("{NEG_ONE:05}"), "-0001");
            assert_eq!(format!("{NEG_ONE:+}"), "-1");
            assert_eq!(int(-128).to_string(), "-128");
        }

        /// The radixes other than ten print the two's complement, as the
        /// primitive integers do.
        #[test]
        fn negative_numbers_are_formatted_as_the_complement_in_other_radixes() {
            assert_eq!(format!("{NEG_ONE:x}"), "f".repeat(2 * T::BYTES));
            assert_eq!(format!("{NEG_ONE:X}"), "F".repeat(2 * T::BYTES));
            assert_eq!(format!("{NEG_ONE:b}"), "1".repeat(8 * T::BYTES));
            assert_eq!(format!("{NEG_ONE:o}"), octal_ones(BITS));
            assert_eq!(format!("{NEG_ONE:x?}"), "f".repeat(2 * T::BYTES));
            assert_eq!(format!("{:x}", T::MIN), format!("8{}", "0".repeat(2 * T::BYTES - 1)));
            assert_eq!(format!("{:#b}", T::MIN), format!("0b1{}", "0".repeat(8 * T::BYTES - 1)));
            assert_eq!(format!("{:x}", T::MAX), format!("7{}", "f".repeat(2 * T::BYTES - 1)));
        }

        #[test]
        fn parses_the_bounds() {
            let power = pow2(BITS - 1);
            assert_eq!(format!("-{power}").parse(), Ok(T::MIN));
            assert_eq!(minus_one(&power).parse(), Ok(T::MAX));
            assert_eq!(format!("+{}", minus_one(&power)).parse(), Ok(T::MAX));
            let hex = "f".repeat(2 * T::BYTES - 1);
            assert_eq!(T::from_str_radix(&format!("7{hex}"), 16), Ok(T::MAX));
            let hex = "0".repeat(2 * T::BYTES - 1);
            assert_eq!(T::from_str_radix(&format!("-8{hex}"), 16), Ok(T::MIN));
        }

        #[test]
        fn parses_the_negative_numbers() {
            assert_eq!("-1".parse(), Ok(NEG_ONE));
            assert_eq!("-0".parse(), Ok(T::ZERO));
            assert_eq!("-128".parse(), Ok(int(-128)));
            assert_eq!("-0000128".parse(), Ok(int(-128)));
            assert_eq!(T::from_str_radix("-z", 36), Ok(int(-35)));
            assert_eq!(T::from_str_radix("-Ff", 16), Ok(-small(255)));
        }

        #[test]
        fn parsing_rejects_the_value_above_the_maximum() {
            let err = ParseIntError(IntErrorKind::PosOverflow);
            let power = pow2(BITS - 1);
            assert_eq!(power.parse::<T>(), Err(err));
            assert_eq!(pow2(BITS).parse::<T>(), Err(err));
            assert_eq!(format!("{power}0").parse::<T>(), Err(err));
            let hex = "0".repeat(2 * T::BYTES - 1);
            assert_eq!(T::from_str_radix(&format!("8{hex}"), 16), Err(err));
            assert_eq!(T::from_str_radix(&"f".repeat(2 * T::BYTES), 16), Err(err));
            assert_eq!(T::from_str_radix(&format!("10{hex}"), 16), Err(err));
        }

        #[test]
        fn parsing_rejects_the_value_below_the_minimum() {
            let err = ParseIntError(IntErrorKind::NegOverflow);
            let power = pow2(BITS - 1);
            assert_eq!(format!("-{}", plus_one(&power)).parse::<T>(), Err(err));
            assert_eq!(format!("-{}", pow2(BITS)).parse::<T>(), Err(err));
            assert_eq!(format!("-{power}0").parse::<T>(), Err(err));
            let hex = "0".repeat(2 * T::BYTES - 2);
            assert_eq!(T::from_str_radix(&format!("-8{hex}1"), 16), Err(err));
            assert_eq!(T::from_str_radix(&format!("-{}", "f".repeat(2 * T::BYTES)), 16), Err(err));
            assert_eq!(T::from_str_radix(&format!("-100{hex}"), 16), Err(err));
        }

        /// The hex representation of a negative number is its complement,
        /// which is a positive number not fitting the type.
        #[test]
        fn parsing_rejects_the_complement_of_a_negative_number() {
            let err = ParseIntError(IntErrorKind::PosOverflow);
            assert_eq!(T::from_str_radix(&format!("{NEG_ONE:x}"), 16), Err(err));
            assert_eq!(T::from_str_radix(&format!("{:b}", T::MIN), 2), Err(err));
        }
    };
}

/// Generates the modules with the tests of the signed integers of the given
/// sizes.
macro_rules! signed_suite {
    ($($module:ident: $limbs:literal limbs;)+) => {$(
        mod $module {
            use crate::*;

            type T = BigInt<$limbs, true>;
            type Unsigned = BigInt<$limbs, false>;
            const LIMBS: usize = $limbs;
            const BITS: u32 = 64 * $limbs;

            common_tests!();
            signed_tests!();
        }
    )+};
}

mod signed {
    signed_suite! {
        bits64: 1 limbs;
        bits192: 3 limbs;
        bits256: 4 limbs;
        bits512: 8 limbs;
        bits1024: 16 limbs;
    }
}

// ---------------------------------------------------------------------------
// Byte arrays of the aliased sizes
// ---------------------------------------------------------------------------

/// Generates the tests of the conversions between the integers and the byte
/// arrays, which are provided for the sizes having the aliases.
macro_rules! byte_array_suite {
    ($($name:ident: $len:literal bytes;)+) => {$(
        mod $name {
            use amplify_num::$name as T;

            /// Bytes with the values equal to their indexes.
            fn pattern() -> [u8; $len] { core::array::from_fn(|i| i as u8) }

            #[test]
            fn array_size_matches_the_integer() { assert_eq!(T::BYTES, $len); }

            #[test]
            fn from_be_bytes_matches_the_slice_conversion() {
                assert_eq!(Ok(T::from_be_bytes(pattern())), T::from_be_slice(&pattern()));
            }

            #[test]
            fn from_le_bytes_matches_the_slice_conversion() {
                assert_eq!(Ok(T::from_le_bytes(pattern())), T::from_le_slice(&pattern()));
            }

            #[test]
            fn to_be_bytes_restores_the_array() {
                assert_eq!(T::from_be_bytes(pattern()).to_be_bytes(), pattern());
            }

            #[test]
            fn to_le_bytes_restores_the_array() {
                assert_eq!(T::from_le_bytes(pattern()).to_le_bytes(), pattern());
            }

            #[test]
            fn byte_orders_are_reversed() {
                let mut reversed = pattern();
                reversed.reverse();
                assert_eq!(T::from_be_bytes(pattern()), T::from_le_bytes(reversed));
                assert_eq!(T::from_be_bytes(pattern()).to_le_bytes(), reversed);
            }

            #[test]
            fn one_is_the_last_big_endian_and_the_first_little_endian_byte() {
                let mut bytes = [0; $len];
                bytes[$len - 1] = 1;
                assert_eq!(T::ONE.to_be_bytes(), bytes);
                assert_eq!(T::from_be_bytes(bytes), T::ONE);
                bytes.reverse();
                assert_eq!(T::ONE.to_le_bytes(), bytes);
                assert_eq!(T::from_le_bytes(bytes), T::ONE);
            }

            #[test]
            fn bounds_have_the_expected_bytes() {
                assert_eq!(T::ZERO.to_be_bytes(), [0; $len]);
                assert_eq!((!T::ZERO).to_be_bytes(), [0xFF; $len]);
                assert_eq!((!T::ZERO).to_le_bytes(), [0xFF; $len]);
                assert_eq!(T::from_be_bytes([0xFF; $len]), !T::ZERO);
                assert_eq!(T::from_be_bytes(T::MAX.to_be_bytes()), T::MAX);
                assert_eq!(T::from_le_bytes(T::MIN.to_le_bytes()), T::MIN);
            }

            #[test]
            fn top_byte_holds_the_sign_bit() {
                let top = T::ONE << (T::BITS - 1);
                assert_eq!(top.to_be_bytes()[0], 0x80);
                assert_eq!(top.to_le_bytes()[$len - 1], 0x80);
            }
        }
    )+};
}

mod byte_arrays {
    byte_array_suite! {
        u256: 32 bytes;
        i256: 32 bytes;
        u512: 64 bytes;
        i512: 64 bytes;
        u1024: 128 bytes;
        i1024: 128 bytes;
    }
}

// ---------------------------------------------------------------------------
// Widening between the aliased sizes
// ---------------------------------------------------------------------------

mod widening {
    use amplify_num::*;

    #[test]
    fn unsigned_widening_fills_the_upper_limbs_with_zeros() {
        assert_eq!(u512::from(u256::MAX), u512::MAX >> 256);
        assert_eq!(u1024::from(u256::MAX), u1024::MAX >> 768);
        assert_eq!(u1024::from(u512::MAX), u1024::MAX >> 512);
        assert_eq!(u512::from(u256::ZERO), u512::ZERO);
    }

    #[test]
    fn signed_widening_extends_the_sign() {
        assert_eq!(i512::from(-i256::ONE), -i512::ONE);
        assert_eq!(i1024::from(-i256::ONE), -i1024::ONE);
        assert_eq!(i1024::from(-i512::ONE), -i1024::ONE);
        assert_eq!(i512::from(i256::MIN), -(i512::ONE << 255));
        assert_eq!(i1024::from(i512::MIN), -(i1024::ONE << 511));
        assert_eq!(i512::from(i256::MAX), (i512::ONE << 255) - i512::ONE);
        assert_eq!(i1024::from(i256::MAX).leading_zeros(), 769);
    }

    #[test]
    fn widening_preserves_the_decimal_representation() {
        assert_eq!(u1024::from(u256::MAX).to_string(), u256::MAX.to_string());
        assert_eq!(i1024::from(i256::MIN).to_string(), i256::MIN.to_string());
        assert_eq!(i512::from(i256::MAX).to_string(), i256::MAX.to_string());
    }
}

// ---------------------------------------------------------------------------
// Known answers
// ---------------------------------------------------------------------------

/// Generates the tests of the arithmetic of two unsigned numbers given by
/// their hex digits. The expected results were computed with the integers of
/// Python, which have an arbitrary precision, that is independently of the
/// library.
macro_rules! unsigned_vectors {
    ($($name:ident: $ty:ty, $a:literal, $b:literal => {
        add: ($sum:literal, $sum_overflow:literal),
        sub: ($diff:literal, $diff_overflow:literal),
        mul: ($prod:literal, $prod_overflow:literal),
        div: ($quot:literal, $rem:literal),
        dec: $dec:literal,
    })+) => {$(
        mod $name {
            use crate::*;

            type T = $ty;

            fn hex(src: &str) -> T { T::from_str_radix(src, 16).expect("valid hex number") }

            fn a() -> T { hex($a) }

            fn b() -> T { hex($b) }

            #[test]
            fn hex_digits_are_restored() {
                assert_eq!(format!("{:x}", a()), $a);
                assert_eq!(format!("{:x}", b()), $b);
                assert_eq!(format!("{:#X}", a()), format!("0x{}", $a.to_uppercase()));
            }

            #[test]
            fn decimal_digits() {
                assert_eq!(a().to_string(), $dec);
                assert_eq!($dec.parse(), Ok(a()));
            }

            #[test]
            fn addition() {
                let (sum, overflow): (T, bool) = (hex($sum), $sum_overflow);
                assert_eq!(a().overflowing_add(b()), (sum, overflow));
                assert_eq!(b().overflowing_add(a()), (sum, overflow));
                assert_eq!(a().checked_add(b()), (!overflow).then_some(sum));
                assert_eq!(a().wrapping_add(b()), sum);
            }

            #[test]
            fn subtraction() {
                let (diff, overflow): (T, bool) = (hex($diff), $diff_overflow);
                assert_eq!(a().overflowing_sub(b()), (diff, overflow));
                assert_eq!(a().checked_sub(b()), (!overflow).then_some(diff));
                assert_eq!(a().wrapping_sub(b()), diff);
                // The operands in the opposite order give the negated difference.
                assert_eq!(b().wrapping_sub(a()), diff.wrapping_neg());
            }

            #[test]
            fn multiplication() {
                let (prod, overflow): (T, bool) = (hex($prod), $prod_overflow);
                assert_eq!(a().overflowing_mul(b()), (prod, overflow));
                assert_eq!(b().overflowing_mul(a()), (prod, overflow));
                assert_eq!(a().checked_mul(b()), (!overflow).then_some(prod));
                assert_eq!(a().wrapping_mul(b()), prod);
            }

            #[test]
            fn division() {
                let (quot, rem) = (hex($quot), hex($rem));
                assert_eq!(a().div_rem(b()), Ok((quot, rem)));
                assert_eq!(a().div_rem_euclid(b()), Ok((quot, rem)));
                assert_eq!((a() / b(), a() % b()), (quot, rem));
                assert_eq!(quot * b() + rem, a());
                assert!(rem < b());
            }
        }
    )+};
}

/// Generates the tests of the arithmetic of two signed numbers given by their
/// decimal digits. The expected results were computed with the integers of
/// Python as well.
macro_rules! signed_vectors {
    ($($name:ident: $ty:ty, $a:literal, $b:literal => {
        add: ($sum:literal, $sum_overflow:literal),
        sub: ($diff:literal, $diff_overflow:literal),
        mul: ($prod:literal, $prod_overflow:literal),
        div: ($quot:literal, $rem:literal),
        euclid: ($euclid_quot:literal, $euclid_rem:literal),
        shr: ($shift:literal, $shifted:literal),
        hex: $hex:literal,
    })+) => {$(
        mod $name {
            use crate::*;

            type T = $ty;

            fn dec(src: &str) -> T { src.parse().expect("valid decimal number") }

            fn a() -> T { dec($a) }

            fn b() -> T { dec($b) }

            #[test]
            fn decimal_digits_are_restored() {
                assert_eq!(a().to_string(), $a);
                assert_eq!(b().to_string(), $b);
            }

            /// The hex digits are the ones of the two's complement.
            #[test]
            fn hex_digits() {
                assert_eq!(format!("{:x}", a()), $hex);
                assert_eq!(Ok(a().cast_unsigned()), BigInt::from_str_radix($hex, 16));
            }

            #[test]
            fn negation() {
                let negated = match $a.strip_prefix('-') {
                    Some(digits) => digits.to_owned(),
                    None => format!("-{}", $a),
                };
                assert_eq!((-a()).to_string(), negated);
                assert_eq!(a().checked_neg(), Some(dec(&negated)));
                assert_eq!(a().unsigned_abs().to_string(), $a.trim_start_matches('-'));
            }

            #[test]
            fn addition() {
                let (sum, overflow): (T, bool) = (dec($sum), $sum_overflow);
                assert_eq!(a().overflowing_add(b()), (sum, overflow));
                assert_eq!(b().overflowing_add(a()), (sum, overflow));
                assert_eq!(a().checked_add(b()), (!overflow).then_some(sum));
                assert_eq!(a().wrapping_add(b()), sum);
            }

            #[test]
            fn subtraction() {
                let (diff, overflow): (T, bool) = (dec($diff), $diff_overflow);
                assert_eq!(a().overflowing_sub(b()), (diff, overflow));
                assert_eq!(a().checked_sub(b()), (!overflow).then_some(diff));
                assert_eq!(a().wrapping_sub(b()), diff);
                assert_eq!(b().wrapping_sub(a()), diff.wrapping_neg());
            }

            #[test]
            fn multiplication() {
                let (prod, overflow): (T, bool) = (dec($prod), $prod_overflow);
                assert_eq!(a().overflowing_mul(b()), (prod, overflow));
                assert_eq!(b().overflowing_mul(a()), (prod, overflow));
                assert_eq!(a().checked_mul(b()), (!overflow).then_some(prod));
                assert_eq!(a().wrapping_mul(b()), prod);
            }

            #[test]
            fn division() {
                let (quot, rem) = (dec($quot), dec($rem));
                assert_eq!(a().div_rem(b()), Ok((quot, rem)));
                assert_eq!((a() / b(), a() % b()), (quot, rem));
                assert_eq!(quot * b() + rem, a());
                assert!(rem.unsigned_abs() < b().unsigned_abs());
            }

            #[test]
            fn euclidean_division() {
                let (quot, rem) = (dec($euclid_quot), dec($euclid_rem));
                assert_eq!(a().div_rem_euclid(b()), Ok((quot, rem)));
                assert_eq!((a().div_euclid(b()), a().rem_euclid(b())), (quot, rem));
                assert_eq!(quot * b() + rem, a());
                assert!(!rem.is_negative() && rem.unsigned_abs() < b().unsigned_abs());
            }

            #[test]
            fn shift_right() {
                assert_eq!(a() >> $shift, dec($shifted));
                assert_eq!(a().unbounded_shr($shift), dec($shifted));
                assert_eq!(a().checked_shr($shift), Some(dec($shifted)));
            }
        }
    )+};
}

/// Operations of the unsigned integers. The names of the vectors tell what
/// their operands exercise; the corner cases of the long division were
/// selected with a model of its algorithm.
mod known_unsigned_answers {
    unsigned_vectors! {
        random_operands: u256,
            "eaeb8ab97ae4d56324e6035e5bf2159f8941cce005ab4a4c82c6f349daea4147",
            "668b67d876f5c10e92a9e6863932bfdb5" => {
            add: ("eaeb8ab97ae4d56324e6035e5bf215a5f1f84a6775075b35ad655bad6e163efc", false),
            sub: ("eaeb8ab97ae4d56324e6035e5bf21599208b4f58964f396358288ae647be4392", false),
            mul: ("511020d2270385daf97096937465fd584e9c1cae818ed5dd1e245fdb06105233", true),
            div: ("24a78e9d7bf0ab1f190c57cb80e6bace",
                  "3bc3eee4e79e4577653d3b054069097a1"),
            dec: "1062573730702354819609534892119221079680955354597040690933501032\
                  06105535103303",
        }
        carries_through_all_limbs: u256,
            "fffffffffffffffffffffffffffffffffffffffffffffffeffffffffffffffff",
            "10000000000000001" => {
            add: ("0", true),
            sub: ("fffffffffffffffffffffffffffffffffffffffffffffffdfffffffffffffffe", false),
            mul: ("fffffffffffffffffffffffffffffffefffffffffffffffdffffffffffffffff", true),
            div: ("ffffffffffffffff0000000000000000fffffffffffffffe",
                  "1"),
            dec: "1157920892373161954235709850086879078532699846656405640394391372\
                  63839420088319",
        }
        single_limb_divisor: u256,
            "f056cd122496d0fd0472a3e084b742f8b1015428779ff6fa52fee15930a7cef4",
            "cca966e1ed316d83" => {
            add: ("f056cd122496d0fd0472a3e084b742f8b1015428779ff6fb1fa8483b1dd93c77", false),
            sub: ("f056cd122496d0fd0472a3e084b742f8b1015428779ff6f986557a7743766171", false),
            mul: ("d47a12a0dcbbcb955c426084e680cd6f1c64fdd2ae38415061841185d9b0cadc", true),
            div: ("12ca0765dd037a81318c158b109fb5e255431df13994e7848",
                  "6063fd63a6819a1c"),
            dec: "1087084478546882910885265113891986985841470646643294359257977105\
                  10731416817396",
        }
        zero_limbs_inside: u256,
            "2e0be16b4a9208000000000000000000000000000000000486a1d03ac7980cf",
            "ffffffffffffffffffffffffffffffff" => {
            add: ("2e0be16b4a9208000000000000000010000000000000000486a1d03ac7980ce", false),
            sub: ("2e0be16b4a9207fffffffffffffffff0000000000000000486a1d03ac7980d0", false),
            mul: ("fd1f41e94b56df80486a1d03ac7980ceffffffffffffffffb795e2fc53867f31", true),
            div: ("2e0be16b4a920800000000000000000",
                  "2e0be16b4a92080486a1d03ac7980cf"),
            dec: "1301711383627395044380548024925200121708595642843564956539952358\
                  822795378895",
        }
        divisor_above_the_dividend: u256,
            "d0e307687d7e58e5499e57864df244",
            "932a3a6ecb76d9ded6412085526ccd6ce3efbce49babef58ce" => {
            add: ("932a3a6ecb76d9ded641f16859d54aeb3cd50682f3323d4b12", false),
            sub: ("ffffffffffffff6cd5c5913489262129bfb05db4fbb01174f58cb9bbda5e9976", true),
            mul: ("55fa19871935c4eceb360ad3b97bb4d454db16089b99322c3cfe335d777c52b8", true),
            div: ("0",
                  "d0e307687d7e58e5499e57864df244"),
            dec: "1084602440490564082654889080018301508",
        }
        quotient_digit_estimate_is_refined: u256,
            "8000000000000000fffffffffffffffe0000000000000000",
            "8000000000000000ffffffffffffffff" => {
            add: ("80000000000000017ffffffffffffffeffffffffffffffff", false),
            sub: ("80000000000000007ffffffffffffffd0000000000000001", false),
            mul: ("ffffffffffffffff7ffffffffffffffd00000000000000020000000000000000", true),
            div: ("ffffffffffffffff",
                  "7fffffffffffffffffffffffffffffff"),
            dec: "3138550867693340382258177078524771671477658841516366364672",
        }
        divisor_is_added_back_to_a_small_quotient: u256,
            "800000000000000000000000000000000000000000000003",
            "200000000000000000000000000000000000000000000001" => {
            add: ("a00000000000000000000000000000000000000000000004", false),
            sub: ("600000000000000000000000000000000000000000000002", false),
            mul: ("e00000000000000000000000000000000000000000000003", true),
            div: ("3",
                  "200000000000000000000000000000000000000000000000"),
            dec: "3138550867693340381917894711603833208051177722232017256451",
        }
        divisor_is_added_back_to_a_large_quotient: u256,
            "7fffffffffffffff800000000000000000000000000000000000000000000000",
            "800000000000000000000000000000000000000000000001" => {
            add: ("8000000000000000000000000000000000000000000000000000000000000001", false),
            sub: ("7ffffffffffffffeffffffffffffffffffffffffffffffffffffffffffffffff", false),
            mul: ("7fffffffffffffff800000000000000000000000000000000000000000000000", true),
            div: ("fffffffffffffffe",
                  "7fffffffffffffffffffffffffffffff0000000000000002"),
            dec: "5789604461865809770864694163665061354471709762121644881167761428\
                  1724547563520",
        }
        odd_number_of_limbs: BigInt<3, false>,
            "82698e2ae9955e8e6c067bc968834753617e25ea7f86ecc2",
            "b3237467ec7a71593ee088038" => {
            add: ("82698e2ae9955e8e6c067bd49aba8dd229253b7e6d8f6cfa", false),
            sub: ("82698e2ae9955e8e6c067bbe364c00d499d71056917e6c8a", false),
            mul: ("c8725e4727d53f0b04b94239041a46d624e4d4c91df4ca70", true),
            div: ("ba5e0688f300ebc9e15dfd3",
                  "13799d5459d2e81aed23c769a"),
            dec: "3197700918212096139554243221994486007235885160886927027394",
        }
        long_quotient: u512,
            "8aaafd284c4ba20271d86bcc5908890c05301d71ea60f05039cc72207509d97d\
             c79fc5a8c50afbd17ba4749609eaf93e279045c830c4c89717eb25ee4714fa77",
            "3241bb7789bab048f5" => {
            add: ("8aaafd284c4ba20271d86bcc5908890c05301d71ea60f05039cc72207509d97d\
                   c79fc5a8c50afbd17ba4749609eaf93e279045c830c4c8c959a69d7801c5436c", false),
            sub: ("8aaafd284c4ba20271d86bcc5908890c05301d71ea60f05039cc72207509d97d\
                   c79fc5a8c50afbd17ba4749609eaf93e279045c830c4c864d62fae648c64b182", false),
            mul: ("4fba0554fdd0e5983162bcc3e9d687d832bb1aa582527744de3fd2bc82f2fbc3\
                   55291010cd6720fedac4c0dfa826347cffafc23fe6abfd4530c0c81d95552be3", true),
            div: ("2c25a384917df5b2c9021bf13cc362f69ccfe960e9e068078e6df371f71e4af6\
                   2b6e062d2bb310c5ae85a515095b97b1f856349c6eaa291",
                  "1c7609be5acd1e9db2"),
            dec: "7262628552653998438987745178758234868251216243170777908251728562\
                  0315391739075977690654912006471178648209573238957476944352549214\
                  69552660856607907985554039",
        }
        long_divisor: u1024,
            "f01e9d3a9263a7674c415b80d7e7c33dcd631437b89a93c1eb822cede26a6806\
             03399be9aa22f27f3a920d25bffaaa502adf0bc0e457f5e9f5d7b9338e0321af\
             bee7028cd964d3e26c8262ea5193eb01925a38d1fb725a4f9dc76de8bfd2b9c3\
             cd4d5188bc7e7b46eae608818a410dcf55f7cb28f2c22d469f7159f20ac4590f",
            "d411897e40c76adf23d36efe610ae39087910a1e48be190e85dbcdb25bb0d506\
             442244a7de53e084be4fb1693298a230d734db78c0724a03de118ea6c1a8a49d\
             b1499e2bd907a66dbc8a9c23f1b94892c57fbcd736ea6b026b484760641241e0\
             600304040dcc8b149c63f617be71ca091" => {
            add: ("f01e9d3a9263a7674c415b80d7e7c34b0e7bac1bc51141b428b91cd3f318a10e\
                   7c4a3dce36048367984ee84b7b07fab46d03563ec995fe35dad2cfc6b78d44bd\
                   3234ba18e08974204d9b4d566c1e34dca6f41b8f8becc12b66713027db6742f0\
                   25491efc2b252b6d9f6a7e87cb652bd556280b69cf8ade9065b0bb6df1e0f9a0", false),
            sub: ("f01e9d3a9263a7674c415b80d7e7c3308c4a7c53ac23e5cfae4b3d07d1bc2efd\
                   8a28fa051e416196dcd5320004ed59ebe8bac142ff19ed9e10dca2a06478fea2\
                   4b994b00d24033a48b69787e3709a1267dc056146af7f373d51daba9a43e3097\
                   755184154dd7cb203661927b491cefc955c78ae815f97bfcd931f87623a7b87e", false),
            mul: ("33ffe2424d627f2dc2ee0d5cbc5ba15690a45a6d534912f37a039290c632014b\
                   389c4e9a632ba45ef990bf696513f6a8450effc23cf1bd5b5426629a5b244021\
                   8857d756941dbeb5e4b237b36acc299b1e6c95eb034fcb62e65ada68b1be03c6\
                   9fe92d4e9332380c4bbb1712a0d308a64d0802f6ba98b6107c620bde1783d17f", true),
            div: ("121dcb02c755c3ec72110dc22c108266",
                  "7ae26823d4498757ba54c449b2662a80ad5c533139e0c28ede94948a31854b35\
                   7d98cbd5767f5ae85d09add5505468859558a314e5ef53c941b9960a44d16ff1\
                   58e49d9db08f6148c982215e3d1f710f19cae2ae8e4e18c6197303de86903ca0\
                   3da6400df501d0ec93bbe2ef276c2bd49"),
            dec: "1686177079744585269062271661178754879370526140432060103570947436\
                  3647296201996888228214716481545234362170827322728631433197919932\
                  5227450290156071434914280586030863704209495055050428509887739114\
                  3928111112269829125232914644702914998752788434820441090682694268\
                  11673313059276047091805972356995745499406159722076431",
        }
    }
}

/// Operations of the signed integers.
mod known_signed_answers {
    signed_vectors! {
        negative_by_positive: i256,
            "-180813748451400124089587906975303752025718933258991123298468197\
             5460011129079",
            "1103150007405600153656322912149338424" => {
            add: ("-180813748451400124089587906975303752025608618258250563283102565\
                   2547861790655", false),
            sub: ("-180813748451400124089587906975303752025829248259731683313833829\
                   8372160467503", false),
            mul: ("2591640570263704798610092529150482922326065732197373452316182731\
                   2719414348536", true),
            div: ("-1639067644813236312934658862716213783771",
                  "-992834096938286306889188592573212175"),
            euclid: ("-1639067644813236312934658862716213783772",
                     "110315910467313846767134319576126249"),
            shr: (77, "-11965250176163057103373028194760707824539793474626521"),
            hex: "fc00a16536264bacfceb590b150522d282f6ca932b84eec07c085e12d020c709",
        }
        positive_by_negative: i256,
            "2693040940593434636308308500390442987051333907748993055051372502\
             1134989500338",
            "-776741013347834246475" => {
            add: ("2693040940593434636308308500390442987051333907748993054973698400\
                   7787155253863", false),
            sub: ("2693040940593434636308308500390442987051333907748993055129046603\
                   4482823746813", false),
            mul: ("5538112994222700929266702827998626430298367517037406394874646637\
                   1532699131098", true),
            div: ("-34671027978632789757970081821485855845418981300306522000",
                  "472704254736979550338"),
            euclid: ("-34671027978632789757970081821485855845418981300306522000",
                     "472704254736979550338"),
            shr: (64, "1459900419191904059623605971425542848519850841085901977003"),
            hex: "3b8a12521718b30f860a164a2d22c8d9f14aaaea413ab5ab2c4a935d8ad71fb2",
        }
        negative_by_negative: i256,
            "-376714093091589704488060997110971011910104555685723868448164305\
             36825762033679",
            "-949366461308830907704654960364545351930" => {
            add: ("-376714093091589704488060997110971011919598220298812177525210854\
                   97190307385609", false),
            sub: ("-376714093091589704488060997110971011900610891072635559371117755\
                   76461216681749", false),
            mul: ("5602929740708141807556450286697514995553060790439370469686960749\
                   4631391186598", true),
            div: ("39680577358108695315555930373958042734",
                  "-495224208659738420638258701305752657059"),
            euclid: ("39680577358108695315555930373958042735",
                     "454142252649092487066396259058792694871"),
            shr: (191, "-12002803490276183596"),
            hex: "acb6bce8daa278ea4daaef4908b722527e462d8a1be9b4f0abaa9f03157acff1",
        }
        near_the_minimum: i256,
            "-578960446186580977117854925043439539266349923328202820197287920\
             03956564807623",
            "3" => {
            add: ("-578960446186580977117854925043439539266349923328202820197287920\
                   03956564807620", false),
            sub: ("-578960446186580977117854925043439539266349923328202820197287920\
                   03956564807626", false),
            mul: ("-578960446186580977117854925043439539266349923328202820197287920\
                   03956564782933", true),
            div: ("-192986815395526992372618308347813179755449974442734273399095973\
                   34652188269207",
                  "-2"),
            euclid: ("-192986815395526992372618308347813179755449974442734273399095973\
                      34652188269208",
                     "1"),
            shr: (1, "-289480223093290488558927462521719769633174961664101410098643960\
                      01978282403812"),
            hex: "8000000000000000000000000000000000000000000000000000000000003039",
        }
        near_the_maximum: i256,
            "5789604461865809771178549250434395392663499233282028201972879200\
             3956564765647",
            "-7" => {
            add: ("5789604461865809771178549250434395392663499233282028201972879200\
                   3956564765640", false),
            sub: ("5789604461865809771178549250434395392663499233282028201972879200\
                   3956564765654", false),
            mul: ("-578960446186580977117854925043439539266349923328202820197287920\
                   03956564439721", true),
            div: ("-827086351695115681596935607204913627523357033326004028853268457\
                   1993794966521",
                  "0"),
            euclid: ("-827086351695115681596935607204913627523357033326004028853268457\
                      1993794966521",
                     "0"),
            shr: (250, "31"),
            hex: "7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffff2bcf",
        }
        odd_number_of_limbs: BigInt<3, true>,
            "-1343404733655665296768178359928762746372332055547315893945",
            "22903977186035373256" => {
            add: ("-1343404733655665296768178359928762746349428078361280520689", false),
            sub: ("-1343404733655665296768178359928762746395236032733351267201", false),
            mul: ("-2106629087545911572189324602478250783219347196444935926920", true),
            div: ("-58653775400839264604546832786347256383",
                  "-22266522504382400897"),
            euclid: ("-58653775400839264604546832786347256384",
                     "637454681652972359"),
            shr: (100, "-1059759474269800389969539696"),
            hex: "c936333c8c340ad8d128d907d0a0419239bdcd053af9c947",
        }
        long_operands: i1024,
            "-916985098217394135754989143976412691711398229333428235506175585\
             7613948777397224975602280862967298380460368096339902797254774371\
             7175505328668423620920166129240825244500857265488017994276386276\
             4457768681239131885925663947595301264406003479274788521210171765\
             5454461271393190539101307175132389993095628286933949",
            "2474135810227784696012293167431645687527175626659955020149985331\
             0307679580163556158529091883263712783617363055259529278261319482\
             04911416939864687451614128151159966391252019438840652" => {
            add: ("-916985098217394135754989143976412691711398229333428235506175585\
                   7613948777397224975602280862967298380460368096339902797254774369\
                   2434147226390576660797234454924368369229100998888467792776532966\
                   1380972879603570300634745114958173428232372926679495738596976945\
                   0543044331528503087487179023972423601843608848093297", false),
            sub: ("-916985098217394135754989143976412691711398229333428235506175585\
                   7613948777397224975602280862967298380460368096339902797254774374\
                   1916863430946270581043097803557282119772613532087568195776239586\
                   7534564482874693471216582780232429100579634031870081303823366586\
                   0365878211257877990715435326292356384347647725774601", false),
            mul: ("8831811466459145399293051043058922087451072448083320200620919876\
                   7027809726174970635386219408744641826999372967416285205581852819\
                   6142123548319674641056188483293062320483993648157934236353254992\
                   6536804148678387083690409963706059598857190929034823648856523375\
                   3138789747106319560410114168138964591930227158697188", true),
            div: ("-370628441020370118236821449710944134985077322294959578504269452\
                   3865027848683063990659002016721232942826507651624047770521747154",
                  "-184273745603642424019833127785275911440937798608799766366588101\
                   6640858691270479192856341115276860566673845611679623581341885823\
                   945845219713845088483745833225127386088181801046429541"),
            euclid: ("-370628441020370118236821449710944134985077322294959578504269452\
                      3865027848683063990659002016721232942826507651624047770521747155",
                     "6313983541913604558139618895788865731177976405719573564841043143\
                      8990926674587642299656807304951071168789069384632934648424612425\
                      9066197226019598967868294926032580303070218392411111"),
            shr: (513, "-341959365396920638777121470661327087894813262913341471131219368\
                        3793209477644348086967007633684203506717957332943524997277301370\
                        55387712390295074337276522"),
            hex: "f2f1132b4bb7a4bc4acf9883c3beb365a3c0fc6b7a1636abf872b5f23d7c7667\
                  e8d258c6edb9e249224aaa0ea99c4d86b4d3d9b6ec6af9de9f37f7668f157b2c\
                  23f29ee22d787589bde3a96cef99bc9e849efa6d6eceb9374401710b35d0bd5e\
                  9f3e0a103a501a9954dfeaf0cebf75cebf5b2f039fbcd367dc4dd870fb82c443",
        }
    }
}
