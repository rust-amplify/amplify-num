//! Reproductions of the findings of the security audit.
//!
//! Unlike the rest of the test suite, the tests of this file do not describe
//! the desired behavior: each of them pins the *current* behavior which the
//! audit found defective, surprising or contradicting the documentation, such
//! that every finding can be reproduced by running its test. A test of this
//! file which starts to fail after a change of the library most probably
//! means that its finding got addressed: the test must be updated then.
//!
//! No finding allows a memory corruption, since the crate has no `unsafe`
//! code: the worst outcome is a panic or a wrong value.
//!
//! | ID   | Severity      | Finding                                              |
//! |------|---------------|------------------------------------------------------|
//! | A-01 | low           | Hex formatting pads to the width ignoring precision  |
//! | A-02 | low           | Zero-limb big integers bypass the compile-time guard |
//! | A-03 | informational | `BigInt::BITS` is truncated for `2^32` limbs and more |
//! | A-04 | low           | Arrays trust the length reported by a byte iterator  |
//! | A-05 | informational | Operands which are not integers are accepted         |
//! | A-06 | informational | Parsing errors of the signed integers differ from std |
//! | A-07 | informational | `ToHex` of the integers is not a byte encoding       |
//! | A-08 | informational | Panic messages of the divisions are inconsistent     |
//! | A-09 | informational | Invalid hex character is displayed as a number       |
//! | A-10 | informational | Documentation overstates the mixed-type arithmetic   |
//!
//! Findings which can't be reproduced by a test on the supported hosts:
//!
//! * A-11 (low, by inspection): `BigInt::bit` computes the limb index as
//!   `(index / 64) as usize`. On a target with 16-bit pointers an index of
//!   `2^22` and above is truncated, thus the method reads a limb instead of
//!   panicking as documented.
//! * A-12 (informational): the arithmetic, the comparison, the parsing and the
//!   formatting are not constant-time, which the documentation does not
//!   mention. The integers must not hold secrets in the code exposed to the
//!   timing attacks.
//! * A-13 (informational): `Debug` detects the `{:x?}` and `{:X?}` flags by the
//!   bits of the deprecated `Formatter::flags`, having no stable alternative.
//!   The behavior is covered by the tests of the big integers.

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

/// The probe itself is checked on the standard types.
#[test]
fn implements_probe_tells_the_implemented_traits() {
    assert!(implements!(u8: Copy));
    assert!(!implements!(String: Copy));
    assert!(implements!(u8: core::ops::Add<u8>));
    assert!(!implements!(u8: core::ops::Add<u16>));
    assert!(implements!(u16: From<u8>));
    assert!(!implements!(u8: From<u16>));
}

// ---------------------------------------------------------------------------
// A-01: hex formatting pads to the width ignoring precision
// ---------------------------------------------------------------------------

/// `format_hex` and `format_hex_reverse` compute the zero padding from the
/// full length of the data, even when a precision truncates the output. With
/// both a width and a precision the output is shorter than the requested
/// width, while the documentation promises that the width left-pads the
/// output.
///
/// Expected: the truncated output padded to the whole width, as the standard
/// library does for the strings.
#[cfg(feature = "hex")]
mod a01_hex_width_with_precision {
    use core::fmt;

    use amplify_num::hex::{format_hex, format_hex_reverse};

    const BYTES: [u8; 4] = [0x12, 0x34, 0x56, 0x78];

    struct Forward;

    impl fmt::LowerHex for Forward {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { format_hex(&BYTES, f) }
    }

    struct Reverse;

    impl fmt::LowerHex for Reverse {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { format_hex_reverse(&BYTES, f) }
    }

    #[test]
    fn output_is_shorter_than_the_width() {
        // Expected ten characters: "0000001234".
        assert_eq!(format!("{:10.4x}", Forward), "001234");
        // Expected ten characters: "0000000785".
        assert_eq!(format!("{:10.3x}", Reverse), "00785");
    }

    #[test]
    fn width_between_the_precision_and_the_length_adds_no_padding() {
        // Expected six characters: "001234".
        assert_eq!(format!("{:6.4x}", Forward), "1234");
        // Expected eight characters: "00000007".
        assert_eq!(format!("{:8.1x}", Reverse), "7");
    }

    /// The semantic of the standard library, for comparison.
    #[test]
    fn standard_strings_are_padded_after_the_truncation() {
        assert_eq!(format!("{:>10.4}", "12345678"), "      1234");
    }
}

// ---------------------------------------------------------------------------
// A-02: zero-limb big integers bypass the compile-time guard
// ---------------------------------------------------------------------------

/// `BigInt::from_inner` rejects the integers without limbs at the compile
/// time, and so do all constructors based on it. The guard is missing in the
/// `MIN` and `MAX` constants of the unsigned integers and in the casts between
/// the signed and unsigned integers, which makes `BigInt<0, _>` values
/// possible. Many operations on them panic at the run time.
///
/// Expected: a compile-time error for any use of `BigInt<0, _>`.
mod a02_zero_limb_integers {
    use amplify_num::{BigInt, Number, ShlWrapping};

    type Empty = BigInt<0, false>;

    #[test]
    fn bounds_construct_an_integer_without_limbs() {
        let value = Empty::MAX;
        assert_eq!((Empty::BITS, Empty::BYTES, size_of::<Empty>()), (0, 0, 0));
        assert!(value == Empty::MIN);
        assert!(value == <Empty as Number>::MAX);
        assert!(BigInt::is_zero(&value));
        assert_eq!(value.into_inner(), [0u64; 0]);
    }

    #[test]
    fn casts_construct_a_signed_integer_without_limbs() {
        let signed: BigInt<0, true> = Empty::MAX.cast_signed();
        assert!(signed.cast_unsigned() == Empty::MAX);
        assert_eq!(size_of_val(&signed), 0);
    }

    #[test]
    #[should_panic(expected = "index out of bounds")]
    fn display_panics() { let _ = Empty::MAX.to_string(); }

    #[test]
    #[should_panic(expected = "out of range")]
    fn lower_hex_panics() { let _ = format!("{:x}", Empty::MAX); }

    #[test]
    #[should_panic(expected = "index out of bounds")]
    fn bit_panics() { let _ = Empty::MAX.bit(0); }

    #[test]
    #[should_panic(expected = "index out of bounds")]
    fn low_bits_panic() { let _ = Empty::MAX.low_u64(); }

    #[test]
    #[should_panic(expected = "attempt to shift left with overflow")]
    fn shift_by_zero_bits_panics() { let _ = Empty::MAX << 0; }

    #[test]
    #[should_panic(expected = "divisor of zero")]
    fn wrapping_shift_panics() { let _ = Empty::MAX.wrapping_shl(0); }

    /// The message depends on the overflow checks of the build profile.
    #[test]
    #[should_panic]
    fn sign_of_the_signed_integer_panics() {
        let _ = BigInt::is_negative(&Empty::MAX.cast_signed());
    }

    #[test]
    #[should_panic]
    fn comparison_of_the_signed_integers_panics() {
        let signed = Empty::MAX.cast_signed();
        let _ = signed.cmp(&signed);
    }
}

// ---------------------------------------------------------------------------
// A-03: `BigInt::BITS` is truncated for `2^32` limbs and more
// ---------------------------------------------------------------------------

/// `BITS` is computed as `64 * N as u32`. The cast truncates the number of the
/// limbs, thus an integer of `2^32` limbs or more may report a wrong bit size
/// instead of being rejected. The sizes from `2^26` up to `2^32` limbs are
/// rejected, since the multiplication overflows when the constant is
/// evaluated. The shifts, the bit inspection and `Number::BITS` rely on that
/// constant.
///
/// Such integers take 32 GiB at least, hence the low impact. Expected: a
/// compile-time error for the sizes the bit size of which does not fit `u32`.
#[cfg(target_pointer_width = "64")]
mod a03_bits_truncation {
    use amplify_num::{BigInt, Number};

    #[test]
    fn bit_size_does_not_match_the_byte_size() {
        type Huge = BigInt<{ (1 << 32) + 1 }, false>;
        assert_eq!(Huge::BYTES, 8 * ((1 << 32) + 1));
        assert_eq!(Huge::BITS, 64);
        assert_eq!(<Huge as Number>::BITS, 64);
    }

    #[test]
    fn bit_size_may_be_zero() {
        assert_eq!(BigInt::<{ 1 << 32 }, true>::BITS, 0);
    }
}

// ---------------------------------------------------------------------------
// A-04: arrays trust the length reported by a byte iterator
// ---------------------------------------------------------------------------

/// `<[u8; LEN]>::from_byte_iter` compares `ExactSizeIterator::len` with the
/// length of the array and does not check how many bytes the iterator yields
/// then. `ExactSizeIterator` is a safe trait, thus a wrong length is not
/// excluded: such an iterator gives an array silently padded with zeros or
/// silently truncated data.
///
/// Expected: `Error::InvalidLength` if the iterator yields less or more bytes
/// than the array has.
#[cfg(feature = "hex")]
mod a04_byte_iterator_length {
    use amplify_num::hex::{Error, FromHex};

    /// Iterator which reports a length different from the number of the bytes
    /// it yields.
    struct Misreporting<I> {
        bytes: I,
        reported: usize,
    }

    impl<I: Iterator<Item = u8>> Iterator for Misreporting<I> {
        type Item = Result<u8, Error>;

        fn next(&mut self) -> Option<Self::Item> { self.bytes.next().map(Ok) }

        fn size_hint(&self) -> (usize, Option<usize>) { (self.reported, Some(self.reported)) }
    }

    impl<I: DoubleEndedIterator<Item = u8>> DoubleEndedIterator for Misreporting<I> {
        fn next_back(&mut self) -> Option<Self::Item> { self.bytes.next_back().map(Ok) }
    }

    impl<I: Iterator<Item = u8>> ExactSizeIterator for Misreporting<I> {}

    #[test]
    fn missing_bytes_are_filled_with_zeros() {
        let iter = Misreporting {
            bytes: [0xaa, 0xbb].into_iter(),
            reported: 4,
        };
        assert_eq!(<[u8; 4]>::from_byte_iter(iter), Ok([0xaa, 0xbb, 0, 0]));
    }

    #[test]
    fn excessive_bytes_are_dropped() {
        let iter = Misreporting {
            bytes: [0xaa, 0xbb, 0xcc, 0xdd].into_iter(),
            reported: 2,
        };
        assert_eq!(<[u8; 2]>::from_byte_iter(iter), Ok([0xaa, 0xbb]));
    }

    /// The reported length is the only thing which is checked.
    #[test]
    fn reported_length_is_checked() {
        let iter = Misreporting {
            bytes: [0xaa, 0xbb].into_iter(),
            reported: 3,
        };
        assert_eq!(<[u8; 2]>::from_byte_iter(iter), Err(Error::InvalidLength(4, 6)));
    }
}

// ---------------------------------------------------------------------------
// A-05: operands which are not integers are accepted
// ---------------------------------------------------------------------------

/// The arithmetic is implemented for any right operand convertible into the
/// integer, and the standard library converts into the integers more than the
/// integers. The change log states that the operators and the arithmetic
/// traits of the small integers no longer accept `bool`, `char` and `NonZero*`
/// operands, which is true; however, the same operands are still accepted by
/// the compound assignments and `div_rem` of the small integers, by the big
/// integers (`bool`) and by the arithmetic traits of the primitive integers
/// (anything convertible, including the IP addresses).
///
/// Expected: integer operands only, as for the operators of the small
/// integers.
mod a05_non_integer_operands {
    use core::net::Ipv4Addr;
    use core::num::NonZeroU8;
    use core::ops::{Add, AddAssign};

    use amplify_num::{AddChecked, AddWrapping, MulChecked, i256, u4, u24, u256};

    #[test]
    fn operators_of_the_small_integers_reject_them() {
        assert!(!implements!(u4: Add<bool>));
        assert!(!implements!(u4: Add<NonZeroU8>));
        assert!(!implements!(u24: Add<char>));
        assert!(!implements!(u4: AddChecked<bool>));
        assert!(!implements!(u24: AddChecked<char>));
    }

    #[test]
    fn compound_assignments_of_the_small_integers_accept_them() {
        assert!(implements!(u4: AddAssign<bool>));
        assert!(implements!(u4: AddAssign<NonZeroU8>));
        assert!(implements!(u24: AddAssign<char>));
        assert!(implements!(u24: AddAssign<Ipv4Addr>));

        let mut nibble = u4::ZERO;
        nibble += true;
        nibble += NonZeroU8::MIN;
        assert_eq!(nibble, u4::with(2));
        let mut word = u24::ZERO;
        word += 'a';
        word += Ipv4Addr::new(0, 0, 1, 0);
        assert_eq!(word, u24::with(97 + 256));
    }

    #[test]
    fn division_of_the_small_integers_accepts_them() {
        assert_eq!(u4::MAX.div_rem(true), Ok((u4::MAX, u4::ZERO)));
    }

    #[test]
    fn big_integers_accept_booleans() {
        assert!(implements!(u256: Add<bool>));
        assert!(!implements!(u256: Add<char>));
        assert!(!implements!(u256: Add<NonZeroU8>));

        assert_eq!(u256::ONE + true, u256::from(2u8));
        assert_eq!(u256::MAX & true, u256::ONE);
        assert_eq!(i256::ONE.checked_mul(false), Some(i256::ZERO));
    }

    #[test]
    fn traits_of_the_primitive_integers_accept_them() {
        assert!(implements!(u8: AddChecked<bool>));
        assert!(implements!(u32: AddChecked<char>));
        assert!(implements!(u8: AddChecked<NonZeroU8>));
        assert!(implements!(u32: AddChecked<Ipv4Addr>));

        assert_eq!(AddChecked::checked_add(1u8, true), Some(2));
        assert_eq!(AddChecked::checked_add(1u32, 'a'), Some(98));
        assert_eq!(AddChecked::checked_add(1u8, NonZeroU8::MAX), None);
        assert_eq!(AddWrapping::wrapping_add(1u32, Ipv4Addr::LOCALHOST), 0x7F00_0002);
    }
}

// ---------------------------------------------------------------------------
// A-06: parsing errors of the signed integers differ from std
// ---------------------------------------------------------------------------

/// `ParseIntError` is documented to carry the same error kinds the primitive
/// integers report. The primitive integers detect an overflow at the digit
/// which causes it. The big integers detect at that point only the overflow
/// of the limbs, and check the sign after the last digit: for a signed number
/// which fits the limbs and not the type, an invalid character met later is
/// reported instead of the overflow.
///
/// Both results are errors, hence the low impact. Expected: the error kinds
/// of the primitive integers.
mod a06_parsing_error_precedence {
    use core::num::IntErrorKind;

    use amplify_num::BigInt;
    use amplify_num::error::ParseIntError;

    type I64 = BigInt<1, true>;

    fn primitive(src: &str) -> Result<i64, IntErrorKind> {
        src.parse::<i64>().map_err(|err| *err.kind())
    }

    #[test]
    fn positive_overflow_is_reported_as_an_invalid_digit() {
        let src = "9223372036854775808x";
        assert_eq!(primitive(src), Err(IntErrorKind::PosOverflow));
        assert_eq!(src.parse::<I64>(), Err(ParseIntError(IntErrorKind::InvalidDigit)));
    }

    #[test]
    fn negative_overflow_is_reported_as_an_invalid_digit() {
        let src = "-9223372036854775809x";
        assert_eq!(primitive(src), Err(IntErrorKind::NegOverflow));
        assert_eq!(src.parse::<I64>(), Err(ParseIntError(IntErrorKind::InvalidDigit)));
    }

    /// The overflow of the limbs is reported as by the primitive integers.
    #[test]
    fn overflow_of_the_limbs_is_reported_first() {
        let src = "18446744073709551616x";
        assert_eq!(primitive(src), Err(IntErrorKind::PosOverflow));
        assert_eq!(src.parse::<I64>(), Err(ParseIntError(IntErrorKind::PosOverflow)));
    }
}

// ---------------------------------------------------------------------------
// A-07: `ToHex` of the integers is not a byte encoding
// ---------------------------------------------------------------------------

/// `ToHex` is implemented for the byte slices, which are encoded with two
/// digits per byte, and for every type formatted as a lower hex, which
/// includes the integers. The latter are written as numbers: without the
/// leading zeros, thus with an odd number of digits for a half of the values.
/// Such strings are not accepted back by `FromHex`.
///
/// Expected: either the same encoding of the bytes, or no `ToHex` for the
/// numbers.
#[cfg(all(feature = "hex", feature = "alloc"))]
mod a07_to_hex_of_integers {
    use amplify_num::hex::{Error, FromHex, ToHex};
    use amplify_num::{i256, u4, u256};

    #[test]
    fn bytes_are_encoded_with_two_digits() {
        assert_eq!([5u8].to_hex(), "05");
    }

    #[test]
    fn integers_are_encoded_without_leading_zeros() {
        assert_eq!(5u8.to_hex(), "5");
        assert_eq!(u4::MAX.to_hex(), "f");
        assert_eq!(u256::ONE.to_hex(), "1");
        assert_eq!(u256::from(0xABCu16).to_hex(), "abc");
    }

    #[test]
    fn negative_integers_are_encoded_as_their_complement() {
        assert_eq!((-1i8).to_hex(), "ff");
        assert_eq!(i256::from(-1).to_hex(), "f".repeat(64));
    }

    #[test]
    fn encoded_integers_are_not_decodable() {
        assert_eq!(Vec::<u8>::from_hex(&5u8.to_hex()), Err(Error::OddLengthString(1)));
        assert_eq!(<[u8; 32]>::from_hex(&u256::ONE.to_hex()), Err(Error::OddLengthString(1)));
        assert_eq!(
            <[u8; 32]>::from_hex(&u256::from(0xABu8).to_hex()),
            Err(Error::InvalidLength(64, 2))
        );
    }
}

// ---------------------------------------------------------------------------
// A-08: panic messages of the divisions are inconsistent
// ---------------------------------------------------------------------------

/// The small integers panic with the messages of the primitive integers. The
/// big integers have messages of their own, which do not tell a remainder
/// from a division.
///
/// Expected: the same messages for all integers.
mod a08_division_panic_messages {
    use amplify_num::{i256, u4, u256};

    #[test]
    #[should_panic(expected = "attempt to calculate the remainder with a divisor of zero")]
    fn small_remainder_by_zero() { let _ = u4::ONE % u4::ZERO; }

    #[test]
    #[should_panic(expected = "attempt to perform division by zero")]
    fn big_remainder_by_zero() { let _ = u256::ONE % u256::ZERO; }

    #[test]
    #[should_panic(expected = "attempt to divide by zero")]
    fn small_division_by_zero() { let _ = u4::ONE / u4::ZERO; }

    #[test]
    #[should_panic(expected = "attempt to perform division by zero")]
    fn big_division_by_zero() { let _ = u256::ONE / u256::ZERO; }

    /// The primitive integers report "attempt to calculate the remainder with
    /// overflow".
    #[test]
    #[should_panic(expected = "attempt to perform division with overflow")]
    fn big_remainder_overflow() { let _ = i256::MIN % -i256::ONE; }
}

// ---------------------------------------------------------------------------
// A-09: invalid hex character is displayed as a number
// ---------------------------------------------------------------------------

/// The message of `Error::InvalidChar` prints the decimal code of the byte,
/// which reads as if the string had contained that number.
///
/// Expected: the character itself or its code marked as such.
#[cfg(feature = "hex")]
mod a09_invalid_character_message {
    use amplify_num::hex::{Error, FromHex};

    #[test]
    fn character_is_displayed_as_its_decimal_code() {
        let err = <[u8; 1]>::from_hex("Z0").expect_err("invalid character");
        assert_eq!(err, Error::InvalidChar(b'Z'));
        assert_eq!(err.to_string(), "invalid hex character 90");
    }
}

// ---------------------------------------------------------------------------
// A-10: documentation overstates the mixed-type arithmetic
// ---------------------------------------------------------------------------

/// The statements of the documentation which do not hold:
///
/// * `README.md`: the integers of different types "can be combined when one
///   type fits the other losslessly". The documentation of `BigInt`: the other
///   operand may be "a big one with fewer limbs". In fact the big integers are
///   combined with each other only if they have the same signedness and both
///   have 4, 8 or 16 limbs.
/// * `README.md`: an unsuffixed literal "can be used only with the integers
///   narrower than 32 bits". It is accepted by the signed big integers as well,
///   which the examples of the same file use.
/// * `README.md`: the list of the checked traits ends with "also implemented
///   for `NonZero*`", while the shifts are not implemented for those types, and
///   the negation is implemented only for the signed ones.
///
/// Expected: the documentation matching the implementation.
mod a10_documentation {
    use core::num::{NonZeroI8, NonZeroU8};
    use core::ops::Add;

    use amplify_num::{
        AddChecked, BigInt, DivChecked, MulChecked, NegChecked, RemChecked, ShlChecked, ShrChecked,
        SubChecked, i256, i512, u256, u512,
    };

    #[test]
    fn big_integers_of_different_signs_are_not_combined() {
        assert!(!implements!(u256: Add<i512>));
        assert!(!implements!(i512: Add<u256>));
        assert!(!implements!(i512: From<u256>));
    }

    #[test]
    fn big_integers_of_custom_sizes_are_not_combined() {
        assert!(!implements!(BigInt<2, false>: Add<u256>));
        assert!(!implements!(u256: Add<BigInt<2, false>>));
        assert!(!implements!(u256: From<BigInt<2, false>>));
        assert!(!implements!(BigInt<5, true>: From<i256>));
    }

    /// The combinations the documentation is right about.
    #[test]
    fn aliased_big_integers_of_the_same_sign_are_combined() {
        assert!(implements!(u256: Add<u512>));
        assert!(implements!(u512: Add<u256>));
        assert!(implements!(i512: From<i256>));
    }

    #[test]
    fn unsuffixed_literals_are_accepted_by_the_signed_big_integers() {
        assert_eq!(i256::ONE + 1, i256::from(2));
        assert_eq!(i512::ONE * -3, i512::from(-3));
        assert!(implements!(i256: Add<i32>));
        assert!(!implements!(u256: Add<i32>));
    }

    #[test]
    fn nonzero_integers_implement_only_some_checked_traits() {
        assert!(implements!(NonZeroU8: AddChecked));
        assert!(implements!(NonZeroU8: SubChecked));
        assert!(implements!(NonZeroU8: MulChecked));
        assert!(implements!(NonZeroU8: DivChecked));
        assert!(implements!(NonZeroU8: RemChecked));
        assert!(implements!(NonZeroI8: NegChecked));
        assert!(!implements!(NonZeroU8: NegChecked));
        assert!(!implements!(NonZeroU8: ShlChecked));
        assert!(!implements!(NonZeroU8: ShrChecked));
    }
}
