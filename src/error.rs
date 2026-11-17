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

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
/// Error indicating that a value does not fit integer dimension
pub struct OverflowError<T = usize> {
    /// Integer bit size
    pub max: T,
    /// Value that overflows
    pub value: T,
}

impl core::fmt::Display for OverflowError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Unable to construct bit-sized integer from a value `{}` overflowing max value `{}`",
            self.value, self.max
        )
    }
}

#[cfg(feature = "std")]
impl std::error::Error for OverflowError {}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DivError {
    ZeroDiv,
    Overflow,
}

impl core::fmt::Display for DivError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match *self {
            DivError::ZeroDiv => write!(f, "division by zero"),
            DivError::Overflow => write!(f, "division with overflow"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for DivError {}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PositDecodeError {
    Zero,
    NaR,
}

impl core::fmt::Display for PositDecodeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match *self {
            PositDecodeError::Zero => write!(f, "exception value zero"),
            PositDecodeError::NaR => write!(f, "exception value NaR"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for PositDecodeError {}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
/// Invalid slice length
pub struct ParseLengthError {
    /// The length of the slice de-facto
    pub actual: usize,
    /// The required length of the slice
    pub expected: usize,
}

impl core::fmt::Display for ParseLengthError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "Invalid length: got {}, expected {}", self.actual, self.expected)
    }
}
#[cfg(feature = "std")]
impl std::error::Error for ParseLengthError {}
