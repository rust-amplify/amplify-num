// SPDX-License-Identifier: Apache-2.0
//
// Written in 2026 by Dr. Maxim Orlovsky <orlovsky@ubideco.org>
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

//! Arithmetic of the operands of different integer types.

/// Implements the binary arithmetic and bitwise operators, together with the
/// arithmetic traits, for a pair of different integer types. The result has the
/// `$out` type, which is the wider of the two, and the closure converts both
/// operands into it losslessly.
///
/// The `prim` kind is used when `$out` is a primitive integer type: its
/// operators wrap on overflow in the release builds, thus the strict methods
/// are used instead. The `own` kind is used for the integers of this crate,
/// whose operators always panic on overflow. With the `ops` prefix only the
/// operators are implemented.
macro_rules! promote {
    (@ops prim [$($gen:tt)*] $lhs:ty, $rhs:ty => $out:ty, |$l:ident, $r:ident| $conv:expr;
     $op:ident $fn:ident $strict:ident) => {
        impl<$($gen)*> ::core::ops::$op<$rhs> for $lhs {
            type Output = $out;

            fn $fn(self, rhs: $rhs) -> $out {
                let ($l, $r) = (self, rhs);
                let (lhs, rhs): ($out, $out) = $conv;
                lhs.$strict(rhs)
            }
        }
    };
    (@ops own [$($gen:tt)*] $lhs:ty, $rhs:ty => $out:ty, |$l:ident, $r:ident| $conv:expr;
     $op:ident $fn:ident $strict:ident) => {
        impl<$($gen)*> ::core::ops::$op<$rhs> for $lhs {
            type Output = $out;

            fn $fn(self, rhs: $rhs) -> $out {
                let ($l, $r) = (self, rhs);
                let (lhs, rhs): ($out, $out) = $conv;
                ::core::ops::$op::$fn(lhs, rhs)
            }
        }
    };
    (@ops $kind:ident [$($gen:tt)*] $lhs:ty, $rhs:ty => $out:ty, |$l:ident, $r:ident| $conv:expr;
     $op:ident $fn:ident $strict:ident, $($rest:tt)+) => {
        $crate::promote::promote!(@ops $kind [$($gen)*] $lhs, $rhs => $out, |$l, $r| $conv;
            $op $fn $strict);
        $crate::promote::promote!(@ops $kind [$($gen)*] $lhs, $rhs => $out, |$l, $r| $conv;
            $($rest)+);
    };
    (@traits [$($gen:tt)*] $lhs:ty, $rhs:ty => $out:ty, |$l:ident, $r:ident| $conv:expr) => {
        $crate::promote::promote!(@trait [$($gen)*] $lhs, $rhs => $out, |$l, $r| $conv;
            Option<$out>: AddChecked checked_add, SubChecked checked_sub, MulChecked checked_mul,
            DivChecked checked_div, RemChecked checked_rem);
        $crate::promote::promote!(@trait [$($gen)*] $lhs, $rhs => $out, |$l, $r| $conv;
            ($out, bool): AddOverflowing overflowing_add, SubOverflowing overflowing_sub,
            MulOverflowing overflowing_mul, DivOverflowing overflowing_div,
            RemOverflowing overflowing_rem);
        $crate::promote::promote!(@trait [$($gen)*] $lhs, $rhs => $out, |$l, $r| $conv;
            $out: AddSaturate saturating_add, SubSaturate saturating_sub,
            MulSaturate saturating_mul, DivSaturate saturating_div, AddWrapping wrapping_add,
            SubWrapping wrapping_sub, MulWrapping wrapping_mul, DivWrapping wrapping_div,
            RemWrapping wrapping_rem);
    };
    (@trait [$($gen:tt)*] $lhs:ty, $rhs:ty => $out:ty, |$l:ident, $r:ident| $conv:expr;
     $ret:ty: $trait:ident $method:ident) => {
        impl<$($gen)*> $crate::traits::$trait<$rhs> for $lhs {
            type Output = $out;

            fn $method(self, rhs: $rhs) -> $ret {
                let ($l, $r) = (self, rhs);
                let (lhs, rhs): ($out, $out) = $conv;
                <$out as $crate::traits::$trait>::$method(lhs, rhs)
            }
        }
    };
    (@trait [$($gen:tt)*] $lhs:ty, $rhs:ty => $out:ty, |$l:ident, $r:ident| $conv:expr;
     $ret:ty: $trait:ident $method:ident, $($rest:tt)+) => {
        $crate::promote::promote!(@trait [$($gen)*] $lhs, $rhs => $out, |$l, $r| $conv;
            $ret: $trait $method);
        $crate::promote::promote!(@trait [$($gen)*] $lhs, $rhs => $out, |$l, $r| $conv;
            $ret: $($rest)+);
    };
    (ops $kind:ident [$($gen:tt)*] $lhs:ty, $rhs:ty => $out:ty, |$l:ident, $r:ident| $conv:expr) => {
        $crate::promote::promote!(@ops $kind [$($gen)*] $lhs, $rhs => $out, |$l, $r| $conv;
            Add add strict_add, Sub sub strict_sub, Mul mul strict_mul, Div div strict_div,
            Rem rem strict_rem);
        $crate::promote::promote!(@ops own [$($gen)*] $lhs, $rhs => $out, |$l, $r| $conv;
            BitAnd bitand bitand, BitOr bitor bitor, BitXor bitxor bitxor);
    };
    ($kind:ident [$($gen:tt)*] $lhs:ty, $rhs:ty => $out:ty, |$l:ident, $r:ident| $conv:expr) => {
        $crate::promote::promote!(ops $kind [$($gen)*] $lhs, $rhs => $out, |$l, $r| $conv);
        $crate::promote::promote!(@traits [$($gen)*] $lhs, $rhs => $out, |$l, $r| $conv);
    };
}

pub(crate) use promote;
