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

/// Unit tests of each operation on its corner cases. The expected values were
/// computed with the arbitrary-precision integers of Python. The operations
/// are also compared with their reference implementations on random numbers
/// by the tests of the big integers.
#[cfg(test)]
mod tests {
    use super::*;

    const MAX: u64 = u64::MAX;
    const TOP: u64 = 1 << 63;

    // -- Addition ---------------------------------------------------------

    #[test]
    fn add_without_carries() {
        let mut lhs = [1, 2, 3];
        assert!(!add(&mut lhs, &[10, 20, 30]));
        assert_eq!(lhs, [11, 22, 33]);
    }

    #[test]
    fn add_propagates_the_carry_through_the_limbs() {
        let mut lhs = [MAX, MAX, 0];
        assert!(!add(&mut lhs, &[1, 0, 0]));
        assert_eq!(lhs, [0, 0, 1]);
    }

    #[test]
    fn add_returns_the_carry_out_of_the_top_limb() {
        let mut lhs = [MAX; 3];
        assert!(add(&mut lhs, &[1, 0, 0]));
        assert_eq!(lhs, [0; 3]);
    }

    #[test]
    fn add_of_the_largest_numbers() {
        let mut lhs = [MAX; 3];
        assert!(add(&mut lhs, &[MAX; 3]));
        assert_eq!(lhs, [MAX - 1, MAX, MAX]);
    }

    #[test]
    fn add_of_no_limbs_gives_no_carry() {
        assert!(!add::<0>(&mut [], &[]));
    }

    // -- Subtraction ------------------------------------------------------

    #[test]
    fn sub_without_borrows() {
        let mut lhs = [11, 22, 33];
        assert!(!sub(&mut lhs, &[1, 2, 3]));
        assert_eq!(lhs, [10, 20, 30]);
    }

    #[test]
    fn sub_propagates_the_borrow_through_the_limbs() {
        let mut lhs = [0, 0, 1];
        assert!(!sub(&mut lhs, &[1, 0, 0]));
        assert_eq!(lhs, [MAX, MAX, 0]);
    }

    #[test]
    fn sub_returns_the_borrow_out_of_the_top_limb() {
        let mut lhs = [0; 3];
        assert!(sub(&mut lhs, &[1, 0, 0]));
        assert_eq!(lhs, [MAX; 3]);
    }

    #[test]
    fn sub_of_the_largest_number_from_zero() {
        let mut lhs = [0; 3];
        assert!(sub(&mut lhs, &[MAX; 3]));
        assert_eq!(lhs, [1, 0, 0]);
    }

    #[test]
    fn sub_of_equal_numbers_gives_zero() {
        let mut lhs = [5, 6, 7];
        assert!(!sub(&mut lhs, &[5, 6, 7]));
        assert_eq!(lhs, [0; 3]);
    }

    // -- Negation ---------------------------------------------------------

    #[test]
    fn neg_of_zero_is_zero() {
        assert_eq!(neg(&[0, 0]), [0, 0]);
    }

    #[test]
    fn neg_is_the_two_complement() {
        assert_eq!(neg(&[1, 0]), [MAX, MAX]);
        assert_eq!(neg(&[MAX, MAX]), [1, 0]);
        assert_eq!(neg(&[5, 7]), [MAX - 4, MAX - 7]);
    }

    #[test]
    fn neg_of_the_top_bit_is_itself() {
        assert_eq!(neg(&[0, TOP]), [0, TOP]);
    }

    // -- Multiplication ---------------------------------------------------

    #[test]
    fn mul_by_zero_and_one() {
        assert_eq!(mul(&[7, 8, 9], &[0; 3]), ([0; 3], false));
        assert_eq!(mul(&[7, 8, 9], &[1, 0, 0]), ([7, 8, 9], false));
        assert_eq!(mul(&[1, 0, 0], &[7, 8, 9]), ([7, 8, 9], false));
    }

    #[test]
    fn mul_carries_into_the_next_limb() {
        assert_eq!(mul(&[MAX, 0], &[MAX, 0]), ([1, MAX - 1], false));
        assert_eq!(mul(&[3, 5], &[7, 0]), ([21, 35], false));
    }

    #[test]
    fn mul_reports_the_carry_out_of_the_top_limb() {
        assert_eq!(mul(&[MAX, MAX], &[2, 0]), ([MAX - 1, MAX], true));
        assert_eq!(mul(&[MAX, MAX], &[MAX, 0]), ([1, MAX], true));
        assert_eq!(mul(&[0, TOP], &[2, 0]), ([0, 0], true));
        assert_eq!(mul(&[MAX, MAX], &[MAX, MAX]), ([1, 0], true));
    }

    /// The products of the limbs which lie entirely above the result are not
    /// computed, though they must be reported as an overflow.
    #[test]
    fn mul_reports_the_limbs_above_the_result() {
        assert_eq!(mul(&[0, 1], &[0, 1]), ([0, 0], true));
        assert_eq!(mul(&[0, 1], &[1, 1]), ([0, 1], true));
        assert_eq!(mul(&[0, 0, 1], &[0, 1, 0]), ([0; 3], true));
        assert_eq!(mul(&[0, 1, 0], &[0, 0, 1]), ([0; 3], true));
        assert_eq!(mul(&[0, 0, 5], &[0, 0, 7]), ([0; 3], true));
    }

    #[test]
    fn mul_up_to_the_bound_does_not_overflow() {
        assert_eq!(mul(&[MAX, MAX], &[1, 0]), ([MAX, MAX], false));
        assert_eq!(mul(&[0, TOP], &[1, 0]), ([0, TOP], false));
        assert_eq!(mul(&[1 << 32, 0], &[1 << 32, 0]), ([0, 1], false));
        assert_eq!(mul(&[0, (1 << 32) - 1], &[1 << 32, 0]), ([0, 0xFFFF_FFFF_0000_0000], false));
        assert_eq!(mul(&[0, 1, 0], &[0, 1, 0]), ([0, 0, 1], false));
    }

    #[test]
    fn mul_by_a_single_bit_above_the_bound_overflows() {
        assert_eq!(mul(&[0, 1 << 32], &[1 << 32, 0]), ([0, 0], true));
    }

    // -- Multiplication by a limb -----------------------------------------

    #[test]
    fn mul_add_limb_multiplies_and_adds() {
        let mut limbs = [1, 2];
        assert_eq!(mul_add_limb(&mut limbs, 10, 5), 0);
        assert_eq!(limbs, [15, 20]);
    }

    #[test]
    fn mul_add_limb_carries_into_the_next_limb() {
        let mut limbs = [MAX, 0];
        assert_eq!(mul_add_limb(&mut limbs, 2, 0), 0);
        assert_eq!(limbs, [MAX - 1, 1]);
    }

    #[test]
    fn mul_add_limb_returns_the_carry_out_of_the_top_limb() {
        let mut limbs = [MAX, MAX];
        assert_eq!(mul_add_limb(&mut limbs, 1, 1), 1);
        assert_eq!(limbs, [0, 0]);
    }

    #[test]
    fn mul_add_limb_of_the_largest_operands() {
        let mut limbs = [MAX, MAX];
        assert_eq!(mul_add_limb(&mut limbs, MAX, MAX), MAX);
        assert_eq!(limbs, [0, 0]);
    }

    #[test]
    fn mul_add_limb_by_zero_gives_the_addend() {
        let mut limbs = [5, 7];
        assert_eq!(mul_add_limb(&mut limbs, 0, 9), 0);
        assert_eq!(limbs, [9, 0]);
        let mut limbs = [0, 0];
        assert_eq!(mul_add_limb(&mut limbs, MAX, MAX), 0);
        assert_eq!(limbs, [MAX, 0]);
    }

    // -- Shift left -------------------------------------------------------

    #[test]
    fn shl_by_zero_keeps_the_number() {
        assert_eq!(shl(&[1, 2, 3], 0), [1, 2, 3]);
    }

    #[test]
    fn shl_moves_the_bits_into_the_next_limb() {
        assert_eq!(shl(&[TOP, 0, 0], 1), [0, 1, 0]);
        assert_eq!(shl(&[MAX, 0, 0], 4), [MAX << 4, 0xF, 0]);
        assert_eq!(shl(&[MAX; 3], 63), [TOP, MAX, MAX]);
    }

    #[test]
    fn shl_by_whole_limbs_moves_the_limbs() {
        assert_eq!(shl(&[1, 2, 3], 64), [0, 1, 2]);
        assert_eq!(shl(&[1, 2, 3], 128), [0, 0, 1]);
    }

    #[test]
    fn shl_by_limbs_and_bits() {
        assert_eq!(shl(&[1, 2, 3], 65), [0, 2, 4]);
    }

    #[test]
    fn shl_discards_the_bits_above_the_top_limb() {
        assert_eq!(shl(&[0, 0, MAX], 1), [0, 0, MAX - 1]);
        assert_eq!(shl(&[MAX; 3], 191), [0, 0, TOP]);
    }

    #[test]
    fn shl_by_the_bit_size_and_more_gives_zero() {
        for shift in [192, 193, 255, 256, 1 << 16, 1 << 31, u32::MAX] {
            assert_eq!(shl(&[MAX; 3], shift), [0; 3], "{shift}");
        }
    }

    // -- Shift right ------------------------------------------------------

    #[test]
    fn shr_by_zero_keeps_the_number() {
        assert_eq!(shr(&[1, 2, 3], 0, 0), [1, 2, 3]);
        assert_eq!(shr(&[1, 2, 3], 0, MAX), [1, 2, 3]);
    }

    #[test]
    fn shr_moves_the_bits_into_the_previous_limb() {
        assert_eq!(shr(&[0, 1, 0], 1, 0), [TOP, 0, 0]);
        assert_eq!(shr(&[0, 0, MAX], 4, 0), [0, 0xF << 60, MAX >> 4]);
        assert_eq!(shr(&[MAX; 3], 63, 0), [MAX, MAX, 1]);
    }

    #[test]
    fn shr_by_whole_limbs_moves_the_limbs() {
        assert_eq!(shr(&[1, 2, 3], 64, 0), [2, 3, 0]);
        assert_eq!(shr(&[1, 2, 3], 128, 0), [3, 0, 0]);
    }

    #[test]
    fn shr_by_limbs_and_bits() {
        assert_eq!(shr(&[1, 2, 3], 65, 0), [TOP | 1, 1, 0]);
        assert_eq!(shr(&[MAX; 3], 191, 0), [1, 0, 0]);
    }

    /// The fill of ones makes the shift arithmetic for the negative numbers.
    #[test]
    fn shr_takes_the_vacated_bits_from_the_fill() {
        assert_eq!(shr(&[0, 0, TOP], 1, MAX), [0, 0, 0b11 << 62]);
        assert_eq!(shr(&[0, 0, TOP], 64, MAX), [0, TOP, MAX]);
        assert_eq!(shr(&[0, 0, TOP], 65, MAX), [0, 0b11 << 62, MAX]);
        assert_eq!(shr(&[1, 2, TOP], 128, MAX), [TOP, MAX, MAX]);
        assert_eq!(shr(&[0, 0, TOP], 191, MAX), [MAX; 3]);
        assert_eq!(shr(&[0; 3], 4, MAX), [0, 0, 0xF << 60]);
    }

    #[test]
    fn shr_by_the_bit_size_and_more_gives_the_fill() {
        for shift in [192, 193, 255, 256, 1 << 16, 1 << 31, u32::MAX] {
            assert_eq!(shr(&[1, 2, 3], shift, 0), [0; 3], "{shift}");
            assert_eq!(shr(&[1, 2, 3], shift, MAX), [MAX; 3], "{shift}");
        }
    }

    // -- Division of a double limb ----------------------------------------

    /// Both implementations are checked, whichever of them the target uses.
    #[test]
    fn div_wide_at_the_bounds_of_the_operands() {
        let cases = [
            ((0, 0, TOP), (0, 0)),
            ((0, MAX, TOP), (1, TOP - 1)),
            ((1, 0, TOP), (2, 0)),
            ((TOP - 1, MAX, TOP), (MAX, TOP - 1)),
            ((MAX - 1, MAX, MAX), (MAX, MAX - 1)),
            ((0, 5, MAX), (0, 5)),
        ];
        for ((hi, lo, den), expected) in cases {
            assert_eq!(div_wide(hi, lo, den), expected, "{hi:#x} {lo:#x} / {den:#x}");
            assert_eq!(div_halves(hi, lo, den), expected, "{hi:#x} {lo:#x} / {den:#x}");
        }
    }

    // -- Division ---------------------------------------------------------

    #[test]
    fn div_rem_by_one() {
        assert_eq!(div_rem(&[5, 6, 7], &[1, 0, 0]), ([5, 6, 7], [0; 3]));
    }

    #[test]
    fn div_rem_by_itself() {
        assert_eq!(div_rem(&[5, 6, 7], &[5, 6, 7]), ([1, 0, 0], [0; 3]));
    }

    #[test]
    fn div_rem_of_a_smaller_dividend() {
        assert_eq!(div_rem(&[5, 6, 0], &[0, 0, 1]), ([0; 3], [5, 6, 0]));
        assert_eq!(div_rem(&[5, 6, 7], &[6, 6, 7]), ([0; 3], [5, 6, 7]));
        assert_eq!(div_rem(&[0; 3], &[6, 6, 7]), ([0; 3], [0; 3]));
    }

    #[test]
    fn div_rem_by_a_single_limb() {
        const FIVES: u64 = 0x5555_5555_5555_5555;
        assert_eq!(div_rem(&[0, 0, 1], &[3, 0, 0]), ([FIVES, FIVES, 0], [1, 0, 0]));
        assert_eq!(div_rem(&[MAX; 3], &[MAX, 0, 0]), ([1, 1, 1], [0; 3]));
        assert_eq!(div_rem(&[MAX; 3], &[2, 0, 0]), ([MAX, MAX, TOP - 1], [1, 0, 0]));
        assert_eq!(div_rem(&[1, 2, 3], &[TOP, 0, 0]), ([4, 6, 0], [1, 0, 0]));
    }

    #[test]
    fn div_rem_by_all_limbs() {
        assert_eq!(div_rem(&[MAX; 3], &[0, 0, 1]), ([MAX, 0, 0], [MAX, MAX, 0]));
        assert_eq!(div_rem(&[MAX; 3], &[0, 0, MAX]), ([1, 0, 0], [MAX, MAX, 0]));
        assert_eq!(div_rem(&[MAX; 3], &[MAX, MAX, MAX - 1]), ([1, 0, 0], [0, 0, 1]));
        assert_eq!(div_rem(&[0, 0, TOP], &[MAX, MAX, TOP - 1]), ([1, 0, 0], [1, 0, 0]));
    }

    /// The divisor is shifted to have its top bit set: by 63 bits at most, or
    /// not at all.
    #[test]
    fn div_rem_normalizes_the_divisor() {
        assert_eq!(div_rem(&[0, 0, MAX], &[0, 1, 0]), ([0, MAX, 0], [0; 3]));
        assert_eq!(div_rem(&[MAX; 3], &[0, 1, 0]), ([MAX, MAX, 0], [MAX, 0, 0]));
        assert_eq!(div_rem(&[MAX; 3], &[0, TOP, 0]), ([MAX, 1, 0], [MAX, TOP - 1, 0]));
        assert_eq!(div_rem(&[MAX; 3], &[MAX, TOP, 0]), ([MAX - 3, 1, 0], [MAX - 4, 5, 0]));
    }

    #[test]
    fn div_rem_of_single_limbs() {
        assert_eq!(div_rem(&[7], &[2]), ([3], [1]));
        assert_eq!(div_rem(&[MAX], &[MAX]), ([1], [0]));
        assert_eq!(div_rem(&[MAX], &[TOP]), ([1], [TOP - 1]));
        assert_eq!(div_rem(&[0], &[1]), ([0], [0]));
    }

    #[test]
    #[should_panic(expected = "attempt to divide by zero")]
    fn div_rem_of_zero_by_zero_panics() { let _ = div_rem(&[0], &[0]); }

    #[test]
    #[should_panic(expected = "attempt to divide by zero")]
    fn div_rem_by_zero_panics() { let _ = div_rem(&[MAX; 3], &[0; 3]); }
}
