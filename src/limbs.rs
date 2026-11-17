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

//! Multi-precision arithmetic on unsigned numbers stored as arrays of 64-bit
//! limbs in the little-endian order: the core shared by all big integers.

use core::array;

/// Adds `rhs` to `lhs`, returning the carry out of the most significant limb.
pub fn add<const N: usize>(lhs: &mut [u64; N], rhs: &[u64; N]) -> bool {
    let mut carry = false;
    for (l, r) in lhs.iter_mut().zip(rhs) {
        (*l, carry) = l.carrying_add(*r, carry);
    }
    carry
}

/// Subtracts `rhs` from `lhs`, returning the borrow out of the most significant
/// limb.
pub fn sub<const N: usize>(lhs: &mut [u64; N], rhs: &[u64; N]) -> bool {
    let mut borrow = false;
    for (l, r) in lhs.iter_mut().zip(rhs) {
        (*l, borrow) = l.borrowing_sub(*r, borrow);
    }
    borrow
}

/// Returns the two's complement of `limbs`.
pub fn neg<const N: usize>(limbs: &[u64; N]) -> [u64; N] {
    let mut res = [0; N];
    sub(&mut res, limbs);
    res
}

/// Returns the low `N` limbs of `lhs * rhs` and whether the full product does
/// not fit them.
pub fn mul<const N: usize>(lhs: &[u64; N], rhs: &[u64; N]) -> ([u64; N], bool) {
    let mut product = [0; N];
    let mut overflow = false;
    for (i, &l) in lhs.iter().enumerate() {
        let mut carry = 0;
        for (p, &r) in product[i..].iter_mut().zip(rhs) {
            (*p, carry) = l.carrying_mul_add(r, *p, carry);
        }
        // The limbs of `rhs` not multiplied above contribute past the most
        // significant limb.
        overflow |= carry != 0 || (l != 0 && rhs[N - i..].iter().any(|&r| r != 0));
    }
    (product, overflow)
}

/// Computes `limbs * factor + addend` in place, returning the carry out of the
/// most significant limb.
pub fn mul_add_limb<const N: usize>(limbs: &mut [u64; N], factor: u64, addend: u64) -> u64 {
    let mut carry = addend;
    for limb in limbs {
        (*limb, carry) = limb.carrying_mul(factor, carry);
    }
    carry
}

/// Shifts `limbs` left by `shift` bits, which may exceed the bit size of the
/// number.
pub fn shl<const N: usize>(limbs: &[u64; N], shift: u32) -> [u64; N] {
    let (skip, bits) = (usize::try_from(shift / 64).unwrap_or(usize::MAX), shift % 64);
    // The limb `below` positions under the limb `i`, or zero past the least
    // significant limb.
    let limb = |i: usize, below: usize| i.checked_sub(below).map_or(0, |i| limbs[i]);
    array::from_fn(|i| {
        limb(i, skip) << bits | limb(i, skip.saturating_add(1)).unbounded_shr(64 - bits)
    })
}

/// Shifts `limbs` right by `shift` bits, which may exceed the bit size of the
/// number. The vacated bits are taken from `fill`, which must be either zero or
/// `u64::MAX`.
pub fn shr<const N: usize>(limbs: &[u64; N], shift: u32, fill: u64) -> [u64; N] {
    let (skip, bits) = (usize::try_from(shift / 64).unwrap_or(usize::MAX), shift % 64);
    // The limb `above` positions over the limb `i`, or `fill` past the most
    // significant limb.
    let limb = |i: usize, above: usize| {
        i.checked_add(above)
            .and_then(|i| limbs.get(i))
            .copied()
            .unwrap_or(fill)
    };
    array::from_fn(|i| {
        limb(i, skip) >> bits | limb(i, skip.saturating_add(1)).unbounded_shl(64 - bits)
    })
}

/// Returns the quotient and the remainder of the division of `hi * 2^64 + lo`
/// by `den`, which must exceed `hi` and have the top bit set.
pub fn div_wide(hi: u64, lo: u64, den: u64) -> (u64, u64) {
    // x86-64 divides a double limb with a single instruction. Elsewhere this
    // takes a software routine, with which `div_rem` was measured to be 2-4
    // times slower on Cortex-M cores than with `div_halves`.
    if cfg!(target_arch = "x86_64") {
        let quot = ((u128::from(hi) << 64 | u128::from(lo)) / u128::from(den)) as u64;
        (quot, lo.wrapping_sub(quot.wrapping_mul(den)))
    } else {
        div_halves(hi, lo, den)
    }
}

/// Does the same as [`div_wide`] using no operations on double limbs.
///
/// This is the same long division as in [`div_rem`], made with the halves of
/// the limbs as the digits.
pub fn div_halves(hi: u64, lo: u64, den: u64) -> (u64, u64) {
    const HALF: u64 = 1 << 32;
    let (den_hi, den_lo) = (den >> 32, den % HALF);
    // Divides `rem * 2^32 + digit` by `den`, where `rem < den`. The quotient is
    // estimated with the high half of the divisor and made exact with the low
    // one.
    let step = |rem: u64, digit: u64| {
        let (mut q, mut r) = (rem / den_hi, rem % den_hi);
        while q >= HALF || q * den_lo > (r << 32 | digit) {
            q -= 1;
            r += den_hi;
            if r >= HALF {
                break;
            }
        }
        (q, (rem << 32 | digit).wrapping_sub(q.wrapping_mul(den)))
    };
    let (q_hi, rem) = step(hi, lo >> 32);
    let (q_lo, rem) = step(rem, lo % HALF);
    (q_hi << 32 | q_lo, rem)
}

/// Returns the quotient and the remainder of the division of `num` by `den`,
/// using the algorithm D from D. Knuth, "The Art of Computer Programming", vol.
/// 2, section 4.3.1.
///
/// # Panics
///
/// If `den` is zero.
pub fn div_rem<const N: usize>(num: &[u64; N], den: &[u64; N]) -> ([u64; N], [u64; N]) {
    let len = N - den.iter().rev().take_while(|&&limb| limb == 0).count();
    assert!(len > 0, "attempt to divide by zero");

    // D1. Normalize: shift both numbers such that the top bit of the divisor is
    // set. The bits shifted out of the dividend go to the extra limb `rem[N]`.
    let shift = den[len - 1].leading_zeros();
    let den = shl(den, shift);
    let mut rem = [shl(num, shift), [0; N]];
    rem[1][0] = num[N - 1].unbounded_shr(64 - shift);
    let window = rem.as_flattened_mut();

    let mut quot = [0; N];
    let top = den[len - 1];
    for j in (0..=N - len).rev() {
        // D3. Estimate the quotient digit `q` from the two top limbs of the
        // remainder. The remainder is less than the divisor, so its top limb
        // can't exceed `top`, and if they are equal the digit is the largest.
        let (hi, lo) = (window[j + len], window[j + len - 1]);
        debug_assert!(hi <= top);
        let (mut q, mut r) = if hi < top {
            let (q, r) = div_wide(hi, lo, top);
            (q, u128::from(r))
        } else {
            (u64::MAX, u128::from(lo) + u128::from(top))
        };
        // Refine the estimate with the next limb, such that it exceeds the true
        // digit by one at most; `r` not fitting a limb means it already does.
        while len > 1 &&
            r <= u128::from(u64::MAX) &&
            u128::from(q) * u128::from(den[len - 2]) >
                (r << 64 | u128::from(window[j + len - 2]))
        {
            q -= 1;
            r += u128::from(top);
        }

        // D4. Subtract the divisor multiplied by `q` from the remainder.
        let (mut carry, mut borrow) = (0, false);
        for (w, &d) in window[j..j + len].iter_mut().zip(&den) {
            let product;
            (product, carry) = d.carrying_mul(q, carry);
            (*w, borrow) = w.borrowing_sub(product, borrow);
        }
        (window[j + len], borrow) = window[j + len].borrowing_sub(carry, borrow);

        // D6. A borrow means that `q` was one too large: add the divisor back.
        if borrow {
            q -= 1;
            let mut carry = false;
            for (w, &d) in window[j..j + len].iter_mut().zip(&den) {
                (*w, carry) = w.carrying_add(d, carry);
            }
            window[j + len] = window[j + len].wrapping_add(u64::from(carry));
        }
        quot[j] = q;
    }

    // D8. Denormalize the remainder.
    (quot, shr(&rem[0], shift, 0))
}
