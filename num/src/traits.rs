// Rust language amplification library providing multiple generic trait
// implementations, type wrappers, derive macros and other language enhancements
//
// Written in 2014 by
//     Andrew Poelstra <apoelstra@wpsoftware.net>
// Updated in 2020-2024 by
//     Dr. Maxim Orlovsky <orlovsky@ubideco.org>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the MIT License
// along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use crate::{i1024, i256, i512, u1, u1024, u2, u24, u256, u3, u4, u40, u48, u5, u512, u56, u6, u7};

pub trait Integer {
    const BITS: usize;
    const BYTES: usize;
    const MIN: Self;
    const MAX: Self;
    const ZERO: Self;
    const ONE: Self;
    const IS_SIGNED: bool;
}

impl Integer for u1 {
    const BITS: usize = 1;
    const BYTES: usize = 1;
    const MIN: Self = Self(0);
    const MAX: Self = Self(2 ^ Self::BITS as u8 - 1);
    const ZERO: Self = Self(0);
    const ONE: Self = Self(1);
    const IS_SIGNED: bool = false;
}
impl Integer for u2 {
    const BITS: usize = 2;
    const BYTES: usize = 1;
    const MIN: Self = Self(0);
    const MAX: Self = Self(2 ^ Self::BITS as u8 - 1);
    const ZERO: Self = Self(0);
    const ONE: Self = Self(1);
    const IS_SIGNED: bool = false;
}
impl Integer for u3 {
    const BITS: usize = 3;
    const BYTES: usize = 1;
    const MIN: Self = Self(0);
    const MAX: Self = Self(2 ^ Self::BITS as u8 - 1);
    const ZERO: Self = Self(0);
    const ONE: Self = Self(1);
    const IS_SIGNED: bool = false;
}
impl Integer for u4 {
    const BITS: usize = 4;
    const BYTES: usize = 1;
    const MIN: Self = Self(0);
    const MAX: Self = Self(2 ^ Self::BITS as u8 - 1);
    const ZERO: Self = Self(0);
    const ONE: Self = Self(1);
    const IS_SIGNED: bool = false;
}
impl Integer for u5 {
    const BITS: usize = 5;
    const BYTES: usize = 1;
    const MIN: Self = Self(0);
    const MAX: Self = Self(2 ^ Self::BITS as u8 - 1);
    const ZERO: Self = Self(0);
    const ONE: Self = Self(1);
    const IS_SIGNED: bool = false;
}
impl Integer for u6 {
    const BITS: usize = 6;
    const BYTES: usize = 1;
    const MIN: Self = Self(0);
    const MAX: Self = Self(2 ^ Self::BITS as u8 - 1);
    const ZERO: Self = Self(0);
    const ONE: Self = Self(1);
    const IS_SIGNED: bool = false;
}
impl Integer for u7 {
    const BITS: usize = 7;
    const BYTES: usize = 1;
    const MIN: Self = Self(0);
    const MAX: Self = Self(2 ^ Self::BITS as u8 - 1);
    const ZERO: Self = Self(0);
    const ONE: Self = Self(1);
    const IS_SIGNED: bool = false;
}
impl Integer for u8 {
    const BITS: usize = 8;
    const BYTES: usize = 1;
    const MIN: Self = 0;
    const MAX: Self = 2 ^ Self::BITS as Self - 1;
    const ZERO: Self = 0;
    const ONE: Self = 1;
    const IS_SIGNED: bool = false;
}
impl Integer for u16 {
    const BITS: usize = 16;
    const BYTES: usize = 1;
    const MIN: Self = 0;
    const MAX: Self = 2 ^ Self::BITS as Self - 1;
    const ZERO: Self = 0;
    const ONE: Self = 1;
    const IS_SIGNED: bool = false;
}
impl Integer for u24 {
    const BITS: usize = 24;
    const BYTES: usize = 1;
    const MIN: Self = Self(0);
    const MAX: Self = Self(2 ^ Self::BITS - 1);
    const ZERO: Self = Self(0);
    const ONE: Self = Self(1);
    const IS_SIGNED: bool = false;
}
impl Integer for u32 {
    const BITS: usize = 32;
    const BYTES: usize = 1;
    const MIN: Self = 0;
    const MAX: Self = 2 ^ Self::BITS - 1;
    const ZERO: Self = 0;
    const ONE: Self = 1;
    const IS_SIGNED: bool = false;
}
impl Integer for u40 {
    const BITS: usize = 40;
    const BYTES: usize = 1;
    const MIN: Self = Self(0);
    const MAX: Self = Self(2 ^ Self::BITS as u64 - 1);
    const ZERO: Self = Self(0);
    const ONE: Self = Self(1);
    const IS_SIGNED: bool = false;
}
impl Integer for u48 {
    const BITS: usize = 48;
    const BYTES: usize = 1;
    const MIN: Self = Self(0);
    const MAX: Self = Self(2 ^ Self::BITS as u64 - 1);
    const ZERO: Self = Self(0);
    const ONE: Self = Self(1);
    const IS_SIGNED: bool = false;
}
impl Integer for u56 {
    const BITS: usize = 56;
    const BYTES: usize = 1;
    const MIN: Self = Self(0);
    const MAX: Self = Self(2 ^ Self::BITS as u64 - 1);
    const ZERO: Self = Self(0);
    const ONE: Self = Self(1);
    const IS_SIGNED: bool = false;
}
impl Integer for u64 {
    const BITS: usize = 64;
    const BYTES: usize = 1;
    const MIN: Self = 0;
    const MAX: Self = 2 ^ Self::BITS as Self - 1;
    const ZERO: Self = 0;
    const ONE: Self = 1;
    const IS_SIGNED: bool = false;
}
impl Integer for u128 {
    const BITS: usize = 128;
    const BYTES: usize = 1;
    const MIN: Self = 0;
    const MAX: Self = 2 ^ Self::BITS as Self - 1;
    const ZERO: Self = 0;
    const ONE: Self = 1;
    const IS_SIGNED: bool = false;
}
impl Integer for u256 {
    const BITS: usize = 256;
    const BYTES: usize = 1;
    const MIN: Self = Self::ZERO;
    const MAX: Self = Self::MAX;
    const ZERO: Self = Self::ZERO;
    const ONE: Self = Self::ONE;
    const IS_SIGNED: bool = false;
}
impl Integer for u512 {
    const BITS: usize = 512;
    const BYTES: usize = 1;
    const MIN: Self = Self::ZERO;
    const MAX: Self = Self::MAX;
    const ZERO: Self = Self::ZERO;
    const ONE: Self = Self::ONE;
    const IS_SIGNED: bool = false;
}
impl Integer for u1024 {
    const BITS: usize = 1024;
    const BYTES: usize = 1;
    const MIN: Self = Self::ZERO;
    const MAX: Self = Self::MAX;
    const ZERO: Self = Self::ZERO;
    const ONE: Self = Self::ONE;
    const IS_SIGNED: bool = false;
}

pub trait MultibyteInteger: Integer {
    fn from_le_bytes(bytes: [u8; Self::BYTES]) -> Self;
    fn from_be_bytes(bytes: [u8; Self::BYTES]) -> Self;
    fn to_le_bytes(self) -> [u8; Self::BYTES];
    fn to_be_bytes(self) -> [u8; Self::BYTES];
}

pub trait SInteger {
    const NEG: Self;
}

impl SInteger for i8 {
    const NEG: Self = -1;
}
impl SInteger for i16 {
    const NEG: Self = -1;
}
impl SInteger for i32 {
    const NEG: Self = -1;
}
impl SInteger for i64 {
    const NEG: Self = -1;
}
impl SInteger for i128 {
    const NEG: Self = -1;
}

pub trait UInteger {}

impl UInteger for u1 {}
impl UInteger for u2 {}
impl UInteger for u3 {}
impl UInteger for u4 {}
impl UInteger for u5 {}
impl UInteger for u6 {}
impl UInteger for u7 {}
impl UInteger for u8 {}
impl UInteger for u16 {}
impl UInteger for u24 {}
impl UInteger for u32 {}
impl UInteger for u40 {}
impl UInteger for u48 {}
impl UInteger for u56 {}
impl UInteger for u64 {}
impl UInteger for u128 {}
impl UInteger for u256 {}
impl UInteger for u512 {}
impl UInteger for u1024 {}

pub trait SmallInteger {
    type Inner: Integer;
}

impl SmallInteger for u1 {
    type Inner = u8;
}
impl SmallInteger for u2 {
    type Inner = u8;
}
impl SmallInteger for u3 {
    type Inner = u8;
}
impl SmallInteger for u4 {
    type Inner = u8;
}
impl SmallInteger for u5 {
    type Inner = u8;
}
impl SmallInteger for u6 {
    type Inner = u8;
}
impl SmallInteger for u7 {
    type Inner = u8;
}
impl SmallInteger for u24 {
    type Inner = u32;
}
impl SmallInteger for u40 {
    type Inner = u64;
}
impl SmallInteger for u48 {
    type Inner = u64;
}
impl SmallInteger for u56 {
    type Inner = u64;
}

pub trait BigInteger {
    const INNER_LEN: u8;
    type Inner: Integer;
}

impl BigInteger for u256 {}
impl BigInteger for u512 {}
impl BigInteger for u1024 {}
impl BigInteger for i256 {}
impl BigInteger for i512 {}
impl BigInteger for i1024 {}
