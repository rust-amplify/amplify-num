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

pub trait DivEuclid {}
pub trait RemEuclid {}
pub trait DivRem {}
pub trait DivRemEuclid {}

pub trait CheckedNeg {}
pub trait WrappingNeg {}
pub trait SaturatingNeg {}

pub trait CheckedInc {}
pub trait WrappingInc {}
pub trait SaturatingInc {}
pub trait OverflowingInc {}

pub trait CheckedIncAssign {}
pub trait WrappingIncAssign {}
pub trait SaturatingIncAssign {}
pub trait OverflowingIncAssign {}

pub trait CheckedAdd {}
pub trait CheckedDec {}
pub trait CheckedSub {}
pub trait CheckedMul {}
pub trait CheckedDiv {}
pub trait CheckedRem {}

pub trait CheckedAddAssign {}
pub trait CheckedDecAssign {}
pub trait CheckedSubAssign {}
pub trait CheckedMulAssign {}
pub trait CheckedDivAssign {}
pub trait CheckedRemAssign {}

pub trait WrappingDiv {}
pub trait SaturatingDiv {}

pub trait WrappingRem {}
pub trait OverflowingRem {}

pub trait WrappingDivEuclid {}
pub trait OverflowingDivEuclid {}

pub trait WrappingRemEuclid {}
pub trait OverflowingRemEuclid {}

pub trait CheckedShl {}
pub trait CyclicShl {}
pub trait SaturatingShl {}
pub trait OverflowingShl {}
