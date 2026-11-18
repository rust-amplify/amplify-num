// SPDX-License-Identifier: Apache-2.0
//
// Written in 2020-2026 by Dr. Maxim Orlovsky <orlovsky@ubideco.org>
//
// Copyright 2024-2026 Laboratories for Ubiquitous and Deterministic Computing,
// Institute for Distributed and Cognitive Computing (InDCS), Switzerland.
// All rights reserved.
//
// Copyright (C) 2020-2025 Dr Maxim Orlovsky.
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

//! Error types.

use core::error::Error;
use core::fmt::{self, Debug, Display, Formatter};
use core::num::IntErrorKind;

/// Error indicating that a value does not fit an integer type.
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct OverflowError<T = usize> {
    /// Maximal value of the integer type.
    pub max: T,
    /// Value that overflows.
    pub value: T,
}

impl<T: Display> Display for OverflowError<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "value {} overflows the maximum {} of the integer type", self.value, self.max)
    }
}

impl<T: Debug + Display> Error for OverflowError<T> {}

/// Error of an integer division.
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub enum DivError {
    /// Division by zero.
    ZeroDiv,
    /// The quotient does not fit the integer type, which happens only when the
    /// minimal value of a signed type is divided by `-1`.
    Overflow,
}

impl Display for DivError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            DivError::ZeroDiv => "division by zero",
            DivError::Overflow => "division with overflow",
        })
    }
}

impl Error for DivError {}

/// Error indicating that a slice has a length different from the required one.
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct ParseLengthError {
    /// The actual length of the slice.
    pub actual: usize,
    /// The required length of the slice.
    pub expected: usize,
}

impl Display for ParseLengthError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "invalid length: got {}, expected {}", self.actual, self.expected)
    }
}

impl Error for ParseLengthError {}

/// Error parsing an integer from a string, carrying the same error kinds the
/// primitive integers report.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct ParseIntError(pub IntErrorKind);

impl From<core::num::ParseIntError> for ParseIntError {
    fn from(err: core::num::ParseIntError) -> Self { Self(*err.kind()) }
}

impl Display for ParseIntError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self.0 {
            IntErrorKind::Empty => "cannot parse integer from empty string",
            IntErrorKind::InvalidDigit => "invalid digit found in string",
            IntErrorKind::PosOverflow => "number too large to fit in target type",
            IntErrorKind::NegOverflow => "number too small to fit in target type",
            _ => "invalid integer",
        })
    }
}

impl Error for ParseIntError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display() {
        let err = OverflowError {
            max: 31u8,
            value: 32,
        };
        assert_eq!(err.to_string(), "value 32 overflows the maximum 31 of the integer type");
        assert_eq!(DivError::ZeroDiv.to_string(), "division by zero");
        assert_eq!(DivError::Overflow.to_string(), "division with overflow");
        let err = ParseLengthError {
            actual: 31,
            expected: 32,
        };
        assert_eq!(err.to_string(), "invalid length: got 31, expected 32");
    }

    #[test]
    fn parse_int_error_matches_core() {
        // The messages and the kinds are the same as of the primitive integers.
        for src in ["", "x", "256", "-129", "+"] {
            let (unsigned, signed) = (src.parse::<u8>(), src.parse::<i8>());
            for err in [unsigned.err(), signed.err()].into_iter().flatten() {
                let converted = ParseIntError::from(err.clone());
                assert_eq!(converted.0, *err.kind());
                assert_eq!(converted.to_string(), err.to_string());
            }
        }
        assert_eq!(ParseIntError(IntErrorKind::Zero).to_string(), "invalid integer");
    }

    #[test]
    fn errors_are_errors() {
        fn assert_error<E: Error + Copy + Eq + Send + Sync + 'static>() {}
        assert_error::<OverflowError>();
        assert_error::<OverflowError<u64>>();
        assert_error::<DivError>();
        assert_error::<ParseLengthError>();
        assert_error::<ParseIntError>();
        #[cfg(feature = "hex")]
        assert_error::<crate::hex::Error>();
    }
}
