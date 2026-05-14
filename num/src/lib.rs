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

//! Custom-sized numeric types.
//!
//! Implementation of various integer types with custom bit dimension. This
//! includes:
//! * large signed and unsigned integers, named *large int types* (256, 512,
//!   1024-bit)
//! * custom sub-8-bit unsigned integers, named *small int types* (from 1 to
//!   7-bit)
//! * 10-, 12-, 14-, 20-, 24-, 40-, 48- and 56-bit unsigned integer.
//!
//! The functions here are designed to be fast.

#![cfg_attr(not(feature = "std"), no_std)]
#[cfg(feature = "alloc")]
extern crate alloc;
extern crate core;

#[cfg(feature = "serde")]
#[macro_use]
extern crate serde;

mod bigint;
pub mod error;
#[cfg(feature = "hex")]
pub mod hex;
pub mod posit;
mod smallint;

pub use bigint::{i256, i512, i1024, u256, u512, u1024};
pub use smallint::{u1, u2, u3, u4, u5, u6, u7, u10, u12, u14, u20, u24, u40, u48, u56};

// TODO: Create arbitrary precision types
// TODO: Move from using `u64` to `u128` for big int types
