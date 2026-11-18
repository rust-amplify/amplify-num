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

//! Hex encoding and decoding.

#[cfg(feature = "alloc")]
use alloc::{string::String, vec::Vec};
use core::{fmt, slice};

/// Hex decoding error.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Error {
    /// Non-hexadecimal character.
    InvalidChar(u8),
    /// Hex string of an odd length.
    OddLengthString(usize),
    /// Hex string of a length different from the required one: `(expected,
    /// actual)`.
    InvalidLength(usize, usize),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Error::InvalidChar(ch) => write!(f, "invalid hex character {ch}"),
            Error::OddLengthString(len) => write!(f, "odd hex string length {len}"),
            Error::InvalidLength(expected, actual) => {
                write!(f, "bad hex string length {actual} (expected {expected})")
            }
        }
    }
}

impl core::error::Error for Error {}

/// Trait for objects that can be serialized as hex strings.
#[cfg(feature = "alloc")]
pub trait ToHex {
    /// Hex representation of the object.
    fn to_hex(&self) -> String;
}

/// Trait for objects that can be deserialized from hex strings.
pub trait FromHex: Sized {
    /// Produces an object from a byte iterator.
    fn from_byte_iter<I>(iter: I) -> Result<Self, Error>
    where I: Iterator<Item = Result<u8, Error>> + ExactSizeIterator + DoubleEndedIterator;

    /// Produces an object from a hex string.
    fn from_hex(s: &str) -> Result<Self, Error> { Self::from_byte_iter(HexIterator::new(s)?) }
}

#[cfg(feature = "alloc")]
impl<T: fmt::LowerHex> ToHex for T {
    fn to_hex(&self) -> String { alloc::format!("{self:x}") }
}

#[cfg(feature = "alloc")]
impl ToHex for [u8] {
    fn to_hex(&self) -> String {
        const DIGITS: &[u8; 16] = b"0123456789abcdef";
        let mut hex = String::with_capacity(2 * self.len());
        for byte in self {
            hex.push(DIGITS[usize::from(byte >> 4)] as char);
            hex.push(DIGITS[usize::from(byte & 0xF)] as char);
        }
        hex
    }
}

#[cfg(feature = "alloc")]
impl FromHex for Vec<u8> {
    fn from_byte_iter<I>(iter: I) -> Result<Self, Error>
    where I: Iterator<Item = Result<u8, Error>> + ExactSizeIterator + DoubleEndedIterator {
        iter.collect()
    }
}

impl<const LEN: usize> FromHex for [u8; LEN] {
    fn from_byte_iter<I>(iter: I) -> Result<Self, Error>
    where I: Iterator<Item = Result<u8, Error>> + ExactSizeIterator + DoubleEndedIterator {
        if iter.len() != LEN {
            return Err(Error::InvalidLength(2 * LEN, iter.len().saturating_mul(2)));
        }
        let mut bytes = [0; LEN];
        for (dst, src) in bytes.iter_mut().zip(iter) {
            *dst = src?;
        }
        Ok(bytes)
    }
}

/// Iterator over a hex-encoded string slice which decodes hex and yields bytes.
#[derive(Clone, Debug)]
pub struct HexIterator<'a>(slice::Iter<'a, [u8; 2]>);

impl<'a> HexIterator<'a> {
    /// Constructs a new `HexIterator` from a string slice, failing if the
    /// string is of an odd length.
    pub fn new(s: &'a str) -> Result<Self, Error> {
        match s.as_bytes().as_chunks() {
            (pairs, []) => Ok(Self(pairs.iter())),
            _ => Err(Error::OddLengthString(s.len())),
        }
    }
}

/// Decodes a byte from a pair of hex characters.
fn decode(&[hi, lo]: &[u8; 2]) -> Result<u8, Error> {
    let nibble = |ch: u8| (ch as char).to_digit(16).ok_or(Error::InvalidChar(ch));
    Ok((nibble(hi)? << 4 | nibble(lo)?) as u8)
}

impl Iterator for HexIterator<'_> {
    type Item = Result<u8, Error>;

    fn next(&mut self) -> Option<Self::Item> { self.0.next().map(decode) }

    fn size_hint(&self) -> (usize, Option<usize>) { self.0.size_hint() }
}

impl DoubleEndedIterator for HexIterator<'_> {
    fn next_back(&mut self) -> Option<Self::Item> { self.0.next_back().map(decode) }
}

impl ExactSizeIterator for HexIterator<'_> {}

/// Outputs hex into a formatter, which is usually more efficient than going
/// through a `String` using [`ToHex`].
///
/// The formatter precision truncates the output, while its width left-pads it
/// with zeros.
pub fn format_hex(data: &[u8], f: &mut fmt::Formatter<'_>) -> fmt::Result {
    format_bytes(data.iter(), f)
}

/// Outputs hex in the reverse byte order; used for hashes whose standard hex
/// encoding has the bytes reversed (like bitcoin SHA256d).
///
/// The formatter flags are interpreted in the same way as in [`format_hex`].
pub fn format_hex_reverse(data: &[u8], f: &mut fmt::Formatter<'_>) -> fmt::Result {
    format_bytes(data.iter().rev(), f)
}

fn format_bytes<'a>(
    mut bytes: impl ExactSizeIterator<Item = &'a u8>,
    f: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    let len = 2 * bytes.len();
    let prec = f.precision().unwrap_or(len);
    for _ in len..f.width().unwrap_or(len) {
        f.write_str("0")?;
    }
    for byte in bytes.by_ref().take(prec / 2) {
        write!(f, "{byte:02x}")?;
    }
    // An odd precision cuts the last byte down to its high nibble.
    if prec % 2 == 1 &&
        let Some(byte) = bytes.next()
    {
        write!(f, "{:x}", byte >> 4)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const BYTES: [u8; 10] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

    struct Forward;
    impl fmt::LowerHex for Forward {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { format_hex(&BYTES, f) }
    }

    struct Reverse;
    impl fmt::LowerHex for Reverse {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { format_hex_reverse(&BYTES, f) }
    }

    #[test]
    fn decoding() {
        let bytes = [0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef];
        assert_eq!(<[u8; 8]>::from_hex("0123456789abcdef"), Ok(bytes));
        assert_eq!(<[u8; 8]>::from_hex("0123456789ABCDEF"), Ok(bytes));
        assert_eq!(<[u8; 0]>::from_hex(""), Ok([]));

        let mut iter = HexIterator::new("00ff7f").unwrap();
        assert_eq!(iter.len(), 3);
        assert_eq!(iter.next_back(), Some(Ok(0x7f)));
        assert_eq!(iter.next(), Some(Ok(0x00)));
        assert_eq!(iter.len(), 1);
        assert_eq!(iter.next(), Some(Ok(0xff)));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next_back(), None);
    }

    #[test]
    fn decoding_errors() {
        assert_eq!(<[u8; 8]>::from_hex("0123456789abcdef0"), Err(Error::OddLengthString(17)));
        assert_eq!(<[u8; 4]>::from_hex("0123456789abcdef"), Err(Error::InvalidLength(8, 16)));
        assert_eq!(<[u8; 9]>::from_hex("0123456789abcdef"), Err(Error::InvalidLength(18, 16)));
        assert_eq!(<[u8; 8]>::from_hex("Z123456789abcdef"), Err(Error::InvalidChar(b'Z')));
        assert_eq!(<[u8; 8]>::from_hex("012Y456789abcdeb"), Err(Error::InvalidChar(b'Y')));
        assert_eq!(<[u8; 8]>::from_hex("0123456789abcd+f"), Err(Error::InvalidChar(b'+')));
        // A multibyte character is rejected by its first byte and never splits
        // the string.
        assert_eq!(<[u8; 8]>::from_hex("«23456789abcdef"), Err(Error::InvalidChar(194)));
        assert_eq!(<[u8; 1]>::from_hex("é"), Err(Error::InvalidChar(195)));
    }

    #[test]
    fn error_display() {
        assert_eq!(format!("{}", Error::InvalidChar(90)), "invalid hex character 90");
        assert_eq!(format!("{}", Error::OddLengthString(3)), "odd hex string length 3");
        assert_eq!(
            format!("{}", Error::InvalidLength(8, 6)),
            "bad hex string length 6 (expected 8)"
        );
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn roundtrip() {
        let bytes = Vec::<u8>::from_hex("0123456789ABCDEF").unwrap();
        assert_eq!(bytes, [0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef]);
        assert_eq!(bytes.to_hex(), "0123456789abcdef");
        assert_eq!([0u8, 0x0f, 0xf0, 0xff].to_hex(), "000ff0ff");
        assert_eq!(255u8.to_hex(), "ff");
        assert_eq!(Vec::<u8>::from_hex("0g"), Err(Error::InvalidChar(b'g')));
        assert_eq!(Vec::<u8>::from_hex("012"), Err(Error::OddLengthString(3)));
    }

    #[test]
    fn formatting() {
        assert_eq!(format!("{Forward:x}"), "0102030405060708090a");
        assert_eq!(format!("{Reverse:x}"), "0a090807060504030201");
        for prec in 0..=20 {
            assert_eq!(format!("{Forward:.prec$x}"), "0102030405060708090a"[..prec]);
            assert_eq!(format!("{Reverse:.prec$x}"), "0a090807060504030201"[..prec]);
        }
        assert_eq!(format!("{Forward:.30x}"), "0102030405060708090a");
        assert_eq!(format!("{Forward:25x}"), "000000102030405060708090a");
        assert_eq!(format!("{Forward:26x}"), "0000000102030405060708090a");
        assert_eq!(format!("{Reverse:25x}"), "000000a090807060504030201");
        assert_eq!(format!("{Reverse:26x}"), "0000000a090807060504030201");
    }
}
