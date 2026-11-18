//! Integration tests of the public hex API: decoding, iteration and
//! formatting.
//!
//! The behavior which the security audit found defective or surprising is
//! pinned separately in `tests/audit.rs`.

#![cfg(feature = "hex")]

use core::fmt::{self, Write};

#[cfg(feature = "alloc")]
use amplify_num::hex::ToHex;
use amplify_num::hex::{Error, FromHex, HexIterator, format_hex, format_hex_reverse};

// ---------------------------------------------------------------------------
// Test harness
// ---------------------------------------------------------------------------

const BYTES: [u8; 4] = [0x12, 0x34, 0x56, 0x78];

/// Bytes formatted with [`format_hex`].
struct Forward<'a>(&'a [u8]);

impl fmt::LowerHex for Forward<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { format_hex(self.0, f) }
}

/// Bytes formatted with [`format_hex_reverse`].
struct Reverse<'a>(&'a [u8]);

impl fmt::LowerHex for Reverse<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { format_hex_reverse(self.0, f) }
}

/// Writer which accepts a limited number of characters and fails afterwards.
struct Limited {
    written: String,
    budget: usize,
}

impl Limited {
    fn new(budget: usize) -> Self {
        Self {
            written: String::new(),
            budget,
        }
    }
}

impl Write for Limited {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for ch in s.chars() {
            if self.budget == 0 {
                return Err(fmt::Error);
            }
            self.budget -= 1;
            self.written.push(ch);
        }
        Ok(())
    }
}

/// All bytes decoded by an iterator.
fn decoded(src: &str) -> Vec<Result<u8, Error>> {
    HexIterator::new(src)
        .expect("string of an even length")
        .collect()
}

// ---------------------------------------------------------------------------
// Decoding of arrays
// ---------------------------------------------------------------------------

mod array_decoding {
    use super::*;

    #[test]
    fn decodes_both_letter_cases() {
        assert_eq!(<[u8; 4]>::from_hex("12abCDef"), Ok([0x12, 0xab, 0xcd, 0xef]));
    }

    #[test]
    fn decodes_an_empty_string() {
        assert_eq!(<[u8; 0]>::from_hex(""), Ok([]));
    }

    #[test]
    fn decodes_every_byte_value() {
        let lower: String = (0..=u8::MAX).map(|byte| format!("{byte:02x}")).collect();
        let upper: String = (0..=u8::MAX).map(|byte| format!("{byte:02X}")).collect();
        let expected: [u8; 256] = core::array::from_fn(|i| i as u8);
        assert_eq!(<[u8; 256]>::from_hex(&lower), Ok(expected));
        assert_eq!(<[u8; 256]>::from_hex(&upper), Ok(expected));
    }

    #[test]
    fn rejects_an_odd_length() {
        assert_eq!(<[u8; 2]>::from_hex("123"), Err(Error::OddLengthString(3)));
        assert_eq!(<[u8; 0]>::from_hex("1"), Err(Error::OddLengthString(1)));
    }

    #[test]
    fn rejects_a_shorter_string() {
        assert_eq!(<[u8; 4]>::from_hex("1234"), Err(Error::InvalidLength(8, 4)));
        assert_eq!(<[u8; 4]>::from_hex(""), Err(Error::InvalidLength(8, 0)));
    }

    #[test]
    fn rejects_a_longer_string() {
        assert_eq!(<[u8; 1]>::from_hex("1234"), Err(Error::InvalidLength(2, 4)));
        assert_eq!(<[u8; 0]>::from_hex("12"), Err(Error::InvalidLength(0, 2)));
    }

    #[test]
    fn checks_the_length_before_the_characters() {
        assert_eq!(<[u8; 1]>::from_hex("zzzz"), Err(Error::InvalidLength(2, 4)));
        assert_eq!(<[u8; 1]>::from_hex("zzz"), Err(Error::OddLengthString(3)));
    }

    #[test]
    fn rejects_an_invalid_high_nibble() {
        assert_eq!(<[u8; 1]>::from_hex("g0"), Err(Error::InvalidChar(b'g')));
    }

    #[test]
    fn rejects_an_invalid_low_nibble() {
        assert_eq!(<[u8; 1]>::from_hex("0g"), Err(Error::InvalidChar(b'g')));
    }

    #[test]
    fn reports_the_first_invalid_character() {
        assert_eq!(<[u8; 2]>::from_hex("0xyz"), Err(Error::InvalidChar(b'x')));
        assert_eq!(<[u8; 2]>::from_hex("00yz"), Err(Error::InvalidChar(b'y')));
    }

    #[test]
    fn rejects_every_ascii_character_which_is_not_a_hex_digit() {
        for byte in 0..=0x7Fu8 {
            let ch = char::from(byte);
            let (high, low) = (format!("{ch}0"), format!("0{ch}"));
            if ch.is_ascii_hexdigit() {
                let digit = ch.to_digit(16).expect("hex digit") as u8;
                assert_eq!(<[u8; 1]>::from_hex(&high), Ok([digit << 4]), "{ch:?}");
                assert_eq!(<[u8; 1]>::from_hex(&low), Ok([digit]), "{ch:?}");
            } else {
                assert_eq!(<[u8; 1]>::from_hex(&high), Err(Error::InvalidChar(byte)), "{ch:?}");
                assert_eq!(<[u8; 1]>::from_hex(&low), Err(Error::InvalidChar(byte)), "{ch:?}");
            }
        }
    }

    #[test]
    fn rejects_signs_prefixes_and_spaces() {
        assert_eq!(<[u8; 1]>::from_hex("+1"), Err(Error::InvalidChar(b'+')));
        assert_eq!(<[u8; 1]>::from_hex("-1"), Err(Error::InvalidChar(b'-')));
        assert_eq!(<[u8; 2]>::from_hex("0x12"), Err(Error::InvalidChar(b'x')));
        assert_eq!(<[u8; 1]>::from_hex(" 1"), Err(Error::InvalidChar(b' ')));
        assert_eq!(<[u8; 1]>::from_hex("1\n"), Err(Error::InvalidChar(b'\n')));
    }

    /// A multibyte character is rejected by its first byte and never splits
    /// the string in the middle of a character.
    #[test]
    fn rejects_multibyte_characters_by_their_first_byte() {
        assert_eq!(<[u8; 1]>::from_hex("é"), Err(Error::InvalidChar(0xC3)));
        assert_eq!(<[u8; 2]>::from_hex("1é2"), Err(Error::InvalidChar(0xC3)));
        assert_eq!(<[u8; 2]>::from_hex("😀"), Err(Error::InvalidChar(0xF0)));
        // The full-width digit is not a hex digit either.
        assert_eq!(<[u8; 3]>::from_hex("１２"), Err(Error::InvalidChar(0xEF)));
    }
}

// ---------------------------------------------------------------------------
// Iteration over the decoded bytes
// ---------------------------------------------------------------------------

mod iterator {
    use super::*;

    #[test]
    fn yields_the_bytes_in_order() {
        assert_eq!(decoded("00017fff"), [Ok(0x00), Ok(0x01), Ok(0x7f), Ok(0xff)]);
    }

    #[test]
    fn is_empty_for_an_empty_string() {
        let mut iter = HexIterator::new("").expect("empty string");
        assert_eq!(iter.len(), 0);
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next_back(), None);
    }

    #[test]
    fn rejects_an_odd_length() {
        for src in ["1", "123", "12345"] {
            let err = HexIterator::new(src).expect_err("string of an odd length");
            assert_eq!(err, Error::OddLengthString(src.len()));
        }
    }

    /// The length is counted in bytes, as the decoding works with bytes.
    #[test]
    fn counts_the_length_in_bytes() {
        assert_eq!(HexIterator::new("é0").map(|_| ()), Err(Error::OddLengthString(3)));
        assert_eq!(decoded("é"), [Err(Error::InvalidChar(0xC3))]);
    }

    #[test]
    fn reports_the_exact_remaining_length() {
        let mut iter = HexIterator::new("0a0b0c").expect("valid string");
        assert_eq!((iter.len(), iter.size_hint()), (3, (3, Some(3))));
        iter.next();
        assert_eq!((iter.len(), iter.size_hint()), (2, (2, Some(2))));
        iter.next_back();
        assert_eq!((iter.len(), iter.size_hint()), (1, (1, Some(1))));
        iter.next();
        assert_eq!((iter.len(), iter.size_hint()), (0, (0, Some(0))));
    }

    #[test]
    fn is_double_ended() {
        let mut iter = HexIterator::new("0a0b0c0d").expect("valid string");
        assert_eq!(iter.next_back(), Some(Ok(0x0d)));
        assert_eq!(iter.next(), Some(Ok(0x0a)));
        assert_eq!(iter.next_back(), Some(Ok(0x0c)));
        assert_eq!(iter.next(), Some(Ok(0x0b)));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next_back(), None);
    }

    #[test]
    fn reverses() {
        let reversed: Vec<_> = HexIterator::new("0a0b0c")
            .expect("valid string")
            .rev()
            .collect();
        assert_eq!(reversed, [Ok(0x0c), Ok(0x0b), Ok(0x0a)]);
    }

    #[test]
    fn stays_exhausted_after_the_end() {
        let mut iter = HexIterator::new("ff").expect("valid string");
        assert_eq!(iter.next(), Some(Ok(0xff)));
        for _ in 0..3 {
            assert_eq!(iter.next(), None);
            assert_eq!(iter.next_back(), None);
            assert_eq!(iter.len(), 0);
        }
    }

    #[test]
    fn continues_after_an_invalid_pair() {
        assert_eq!(decoded("zz00g1ff"), [
            Err(Error::InvalidChar(b'z')),
            Ok(0x00),
            Err(Error::InvalidChar(b'g')),
            Ok(0xff)
        ]);
    }

    #[test]
    fn reports_the_high_nibble_before_the_low_one() {
        assert_eq!(decoded("gh"), [Err(Error::InvalidChar(b'g'))]);
        assert_eq!(decoded("0h"), [Err(Error::InvalidChar(b'h'))]);
    }

    #[test]
    fn clones_are_independent() {
        let mut iter = HexIterator::new("0102").expect("valid string");
        let mut copy = iter.clone();
        assert_eq!(iter.next(), Some(Ok(1)));
        assert_eq!(copy.len(), 2);
        assert_eq!(copy.next_back(), Some(Ok(2)));
        assert_eq!(iter.len(), 1);
    }

    #[test]
    fn is_debuggable() {
        let iter = HexIterator::new("0102").expect("valid string");
        assert!(format!("{iter:?}").starts_with("HexIterator"));
    }
}

// ---------------------------------------------------------------------------
// Construction from byte iterators
// ---------------------------------------------------------------------------

mod byte_iterators {
    use super::*;

    #[test]
    fn array_takes_the_bytes_of_an_iterator() {
        let iter = HexIterator::new("01020304").expect("valid string");
        assert_eq!(<[u8; 4]>::from_byte_iter(iter), Ok([1, 2, 3, 4]));
    }

    /// The hashes are often written with their bytes reversed.
    #[test]
    fn array_takes_the_bytes_of_a_reversed_iterator() {
        let iter = HexIterator::new("01020304").expect("valid string");
        assert_eq!(<[u8; 4]>::from_byte_iter(iter.rev()), Ok([4, 3, 2, 1]));
    }

    #[test]
    fn array_rejects_an_iterator_of_a_wrong_length() {
        let iter = HexIterator::new("010203").expect("valid string");
        assert_eq!(<[u8; 4]>::from_byte_iter(iter.clone()), Err(Error::InvalidLength(8, 6)));
        assert_eq!(<[u8; 2]>::from_byte_iter(iter), Err(Error::InvalidLength(4, 6)));
    }

    #[test]
    fn array_propagates_the_first_error_of_an_iterator() {
        let iter =
            [Ok(1), Err(Error::InvalidChar(b'x')), Err(Error::InvalidChar(b'y'))].into_iter();
        assert_eq!(<[u8; 3]>::from_byte_iter(iter), Err(Error::InvalidChar(b'x')));
    }

    #[test]
    fn array_accepts_any_exact_iterator() {
        let iter = [7u8, 8, 9].into_iter().map(Ok);
        assert_eq!(<[u8; 3]>::from_byte_iter(iter), Ok([7, 8, 9]));
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn vector_collects_all_bytes_of_an_iterator() {
        let iter = HexIterator::new("01020304").expect("valid string");
        assert_eq!(Vec::<u8>::from_byte_iter(iter.rev()), Ok(vec![4, 3, 2, 1]));
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn vector_propagates_the_first_error_of_an_iterator() {
        let iter = [Ok(1), Err(Error::InvalidChar(b'x')), Ok(2)].into_iter();
        assert_eq!(Vec::<u8>::from_byte_iter(iter), Err(Error::InvalidChar(b'x')));
    }
}

// ---------------------------------------------------------------------------
// Decoding of vectors
// ---------------------------------------------------------------------------

#[cfg(feature = "alloc")]
mod vector_decoding {
    use super::*;

    #[test]
    fn decodes_strings_of_any_even_length() {
        assert_eq!(Vec::<u8>::from_hex("ff"), Ok(vec![0xff]));
        assert_eq!(
            Vec::<u8>::from_hex("0123456789abcdefABCDEF"),
            Ok(vec![0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0xab, 0xcd, 0xef])
        );
    }

    #[test]
    fn decodes_an_empty_string() {
        assert_eq!(Vec::<u8>::from_hex(""), Ok(vec![]));
    }

    #[test]
    fn decodes_a_long_string() {
        let bytes = Vec::<u8>::from_hex(&"a5".repeat(100_000)).expect("valid string");
        assert_eq!(bytes, vec![0xa5; 100_000]);
    }

    #[test]
    fn rejects_an_odd_length() {
        assert_eq!(Vec::<u8>::from_hex("012"), Err(Error::OddLengthString(3)));
    }

    #[test]
    fn rejects_invalid_characters() {
        assert_eq!(Vec::<u8>::from_hex("0g"), Err(Error::InvalidChar(b'g')));
        assert_eq!(Vec::<u8>::from_hex("00 1"), Err(Error::InvalidChar(b' ')));
        assert_eq!(Vec::<u8>::from_hex("0x00"), Err(Error::InvalidChar(b'x')));
    }
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

mod errors {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn invalid_character_is_displayed() {
        assert_eq!(Error::InvalidChar(b'g').to_string(), "invalid hex character 103");
    }

    #[test]
    fn odd_length_is_displayed() {
        assert_eq!(Error::OddLengthString(7).to_string(), "odd hex string length 7");
    }

    /// The variant keeps the expected length first, while the message starts
    /// with the actual one.
    #[test]
    fn invalid_length_is_displayed() {
        assert_eq!(Error::InvalidLength(8, 6).to_string(), "bad hex string length 6 (expected 8)");
    }

    #[test]
    fn is_a_standard_error_without_a_source() {
        fn assert_error<E: core::error::Error + Send + Sync + 'static>(err: E) {
            assert!(err.source().is_none());
        }
        assert_error(Error::InvalidChar(0));
        assert_error(Error::OddLengthString(1));
        assert_error(Error::InvalidLength(2, 4));
    }

    #[test]
    fn is_comparable_and_hashable() {
        let errors = [
            Error::InvalidChar(1),
            Error::InvalidChar(1),
            Error::OddLengthString(1),
            Error::InvalidLength(1, 2),
            Error::InvalidLength(2, 1),
        ];
        assert_eq!(HashSet::from(errors).len(), 4);
        assert!(Error::InvalidChar(1) < Error::InvalidChar(2));
        assert!(Error::InvalidChar(255) < Error::OddLengthString(0));
        assert!(Error::OddLengthString(usize::MAX) < Error::InvalidLength(0, 0));
        assert!(Error::InvalidLength(1, 2) < Error::InvalidLength(2, 1));
        assert_eq!(Error::InvalidChar(1).max(Error::InvalidChar(2)), Error::InvalidChar(2));
    }

    #[test]
    fn is_copyable_and_debuggable() {
        let err = Error::InvalidLength(2, 4);
        let copy = err;
        assert_eq!(err, copy);
        assert_eq!(format!("{err:?}"), "InvalidLength(2, 4)");
    }
}

// ---------------------------------------------------------------------------
// Formatting
// ---------------------------------------------------------------------------

mod formatting {
    use super::*;

    // -- Plain output ---------------------------------------------------------

    #[test]
    fn writes_two_lowercase_digits_per_byte() {
        assert_eq!(format!("{:x}", Forward(&BYTES)), "12345678");
        assert_eq!(format!("{:x}", Forward(&[0x00, 0x0a, 0xa0, 0xff])), "000aa0ff");
    }

    #[test]
    fn reverse_writes_the_bytes_in_the_opposite_order() {
        assert_eq!(format!("{:x}", Reverse(&BYTES)), "78563412");
        assert_eq!(format!("{:x}", Reverse(&[0x00, 0x0a, 0xa0, 0xff])), "ffa00a00");
    }

    #[test]
    fn empty_slice_gives_an_empty_string() {
        assert_eq!(format!("{:x}", Forward(&[])), "");
        assert_eq!(format!("{:x}", Reverse(&[])), "");
    }

    #[test]
    fn writes_every_byte_value() {
        let bytes: Vec<u8> = (0..=u8::MAX).collect();
        let expected: String = bytes.iter().map(|byte| format!("{byte:02x}")).collect();
        assert_eq!(format!("{:x}", Forward(&bytes)), expected);
    }

    // -- Precision ------------------------------------------------------------

    #[test]
    fn precision_truncates_the_output() {
        assert_eq!(format!("{:.6x}", Forward(&BYTES)), "123456");
        assert_eq!(format!("{:.2x}", Forward(&BYTES)), "12");
        assert_eq!(format!("{:.6x}", Reverse(&BYTES)), "785634");
    }

    #[test]
    fn odd_precision_ends_with_a_high_nibble() {
        assert_eq!(format!("{:.5x}", Forward(&BYTES)), "12345");
        assert_eq!(format!("{:.1x}", Forward(&BYTES)), "1");
        assert_eq!(format!("{:.7x}", Reverse(&BYTES)), "7856341");
    }

    #[test]
    fn zero_precision_gives_an_empty_string() {
        assert_eq!(format!("{:.0x}", Forward(&BYTES)), "");
        assert_eq!(format!("{:.0x}", Reverse(&BYTES)), "");
    }

    #[test]
    fn precision_above_the_length_changes_nothing() {
        assert_eq!(format!("{:.8x}", Forward(&BYTES)), "12345678");
        assert_eq!(format!("{:.9x}", Forward(&BYTES)), "12345678");
        assert_eq!(format!("{:.100x}", Reverse(&BYTES)), "78563412");
        assert_eq!(format!("{:.3x}", Forward(&[])), "");
    }

    #[test]
    fn every_precision_gives_a_prefix_of_the_full_output() {
        for precision in 0..=10 {
            let len = precision.min(8);
            assert_eq!(format!("{:.precision$x}", Forward(&BYTES)), "12345678"[..len]);
            assert_eq!(format!("{:.precision$x}", Reverse(&BYTES)), "78563412"[..len]);
        }
    }

    // -- Width ----------------------------------------------------------------

    #[test]
    fn width_left_pads_with_zeros() {
        assert_eq!(format!("{:10x}", Forward(&BYTES)), "0012345678");
        assert_eq!(format!("{:9x}", Forward(&BYTES)), "012345678");
        assert_eq!(format!("{:11x}", Reverse(&BYTES)), "00078563412");
        assert_eq!(format!("{:3x}", Forward(&[])), "000");
    }

    #[test]
    fn width_up_to_the_length_changes_nothing() {
        for width in 0..=8 {
            assert_eq!(format!("{:width$x}", Forward(&BYTES)), "12345678");
            assert_eq!(format!("{:width$x}", Reverse(&BYTES)), "78563412");
        }
    }

    /// The padding does not depend on the flags: the documentation promises
    /// the zeros on the left.
    #[test]
    fn fill_and_alignment_do_not_change_the_padding() {
        let padded = "0012345678";
        assert_eq!(format!("{:<10x}", Forward(&BYTES)), padded);
        assert_eq!(format!("{:^10x}", Forward(&BYTES)), padded);
        assert_eq!(format!("{:>10x}", Forward(&BYTES)), padded);
        assert_eq!(format!("{:*>10x}", Forward(&BYTES)), padded);
        assert_eq!(format!("{:010x}", Forward(&BYTES)), padded);
    }

    #[test]
    fn alternate_and_sign_flags_add_nothing() {
        assert_eq!(format!("{:#x}", Forward(&BYTES)), "12345678");
        assert_eq!(format!("{:+x}", Forward(&BYTES)), "12345678");
        assert_eq!(format!("{:#10x}", Reverse(&BYTES)), "0078563412");
    }

    // -- Writer errors --------------------------------------------------------

    /// A failure of the writer is reported wherever it happens: in the
    /// padding or in the bytes.
    #[test]
    fn writer_errors_in_the_padding_and_in_the_bytes_are_propagated() {
        for (full, reverse) in [("0012345678", false), ("0078563412", true)] {
            for budget in 0..full.len() {
                let mut writer = Limited::new(budget);
                let res = if reverse {
                    write!(writer, "{:10x}", Reverse(&BYTES))
                } else {
                    write!(writer, "{:10x}", Forward(&BYTES))
                };
                assert_eq!(res, Err(fmt::Error), "budget {budget}");
                assert_eq!(writer.written, full[..budget], "budget {budget}");
            }
        }
    }

    /// The same for the nibble which ends the output of an odd precision.
    #[test]
    fn writer_errors_in_the_last_nibble_are_propagated() {
        for (full, reverse) in [("12345", false), ("78563", true)] {
            for budget in 0..full.len() {
                let mut writer = Limited::new(budget);
                let res = if reverse {
                    write!(writer, "{:.5x}", Reverse(&BYTES))
                } else {
                    write!(writer, "{:.5x}", Forward(&BYTES))
                };
                assert_eq!(res, Err(fmt::Error), "budget {budget}");
                assert_eq!(writer.written, full[..budget], "budget {budget}");
            }
        }
    }

    #[test]
    fn writer_of_a_sufficient_size_gets_the_whole_output() {
        let mut writer = Limited::new(10);
        assert_eq!(write!(writer, "{:10x}", Forward(&BYTES)), Ok(()));
        assert_eq!(writer.written, "0012345678");
        let mut writer = Limited::new(5);
        assert_eq!(write!(writer, "{:.5x}", Reverse(&BYTES)), Ok(()));
        assert_eq!(writer.written, "78563");
    }
}

// ---------------------------------------------------------------------------
// Encoding into strings
// ---------------------------------------------------------------------------

#[cfg(feature = "alloc")]
mod encoding {
    use super::*;

    #[test]
    fn slices_are_encoded_with_two_digits_per_byte() {
        assert_eq!(BYTES[..].to_hex(), "12345678");
        assert_eq!([0x00u8, 0x0f, 0xf0, 0xff][..].to_hex(), "000ff0ff");
    }

    #[test]
    fn arrays_and_vectors_are_encoded_as_slices() {
        assert_eq!(BYTES.to_hex(), "12345678");
        assert_eq!(Vec::from(BYTES).to_hex(), "12345678");
        assert_eq!([0x05u8].to_hex(), "05");
    }

    #[test]
    fn empty_slice_is_encoded_as_an_empty_string() {
        assert_eq!([0u8; 0].to_hex(), "");
        assert_eq!(Vec::<u8>::new().to_hex(), "");
    }

    #[test]
    fn encoding_is_reverted_by_decoding() {
        let bytes: Vec<u8> = (0..=u8::MAX).collect();
        let hex = bytes.to_hex();
        assert_eq!(hex.len(), 512);
        assert!(
            hex.bytes()
                .all(|ch| matches!(ch, b'0'..=b'9' | b'a'..=b'f'))
        );
        assert_eq!(Vec::<u8>::from_hex(&hex), Ok(bytes));
    }

    #[test]
    fn decoding_is_reverted_by_encoding_in_lower_case() {
        let bytes = Vec::<u8>::from_hex("00FfAb12").expect("valid string");
        assert_eq!(bytes.to_hex(), "00ffab12");
    }

    /// The types formatted as a lower hex use that formatting.
    #[test]
    fn lower_hex_types_are_encoded_with_their_formatting() {
        assert_eq!(Forward(&BYTES).to_hex(), "12345678");
        assert_eq!(Reverse(&BYTES).to_hex(), "78563412");
        assert_eq!(0xABCDu16.to_hex(), "abcd");
    }
}
