//! Integration tests of the integer traits implemented for the primitive
//! integers and for the nonzero integers of the standard library.
//!
//! The traits of the primitive integers must do exactly the same as their
//! eponymous inherent methods, which is checked for each type on the operands
//! including the bounds of the type.

/// Shift counts around the bit size of a type.
fn shifts(bits: u32) -> [u32; 7] { [0, 1, bits - 1, bits, bits + 1, 2 * bits, u32::MAX] }

/// Generates the tests common for the signed and unsigned primitive integers.
macro_rules! primitive {
    ($prim:ident, $values:expr) => {
        use amplify_num::*;

        /// Operands of the tested operations.
        const VALUES: &[$prim] = &$values;

        // -- Number category ------------------------------------------------

        #[test]
        fn number_constants_match_the_type() {
            assert_eq!(<$prim as Number>::BITS, $prim::BITS);
            assert_eq!(<$prim as Number>::MIN, $prim::MIN);
            assert_eq!(<$prim as Number>::MAX, $prim::MAX);
            assert_eq!(<$prim as Number>::ZERO, 0);
            assert_eq!(<$prim as Number>::ONE, 1);
        }

        #[test]
        fn is_zero_holds_only_for_zero() {
            for &value in VALUES {
                assert_eq!(Number::is_zero(value), value == 0, "{value}");
            }
        }

        #[test]
        fn is_one_holds_only_for_one() {
            for &value in VALUES {
                assert_eq!(Number::is_one(value), value == 1, "{value}");
            }
        }

        // -- Arithmetic -----------------------------------------------------

        #[test]
        fn addition_matches_inherent_methods() {
            for &a in VALUES {
                for &b in VALUES {
                    assert_eq!(<$prim as AddChecked>::checked_add(a, b), a.checked_add(b));
                    assert_eq!(
                        <$prim as AddOverflowing>::overflowing_add(a, b),
                        a.overflowing_add(b)
                    );
                    assert_eq!(<$prim as AddSaturate>::saturating_add(a, b), a.saturating_add(b));
                    assert_eq!(<$prim as AddWrapping>::wrapping_add(a, b), a.wrapping_add(b));
                }
            }
        }

        #[test]
        fn subtraction_matches_inherent_methods() {
            for &a in VALUES {
                for &b in VALUES {
                    assert_eq!(<$prim as SubChecked>::checked_sub(a, b), a.checked_sub(b));
                    assert_eq!(
                        <$prim as SubOverflowing>::overflowing_sub(a, b),
                        a.overflowing_sub(b)
                    );
                    assert_eq!(<$prim as SubSaturate>::saturating_sub(a, b), a.saturating_sub(b));
                    assert_eq!(<$prim as SubWrapping>::wrapping_sub(a, b), a.wrapping_sub(b));
                }
            }
        }

        #[test]
        fn multiplication_matches_inherent_methods() {
            for &a in VALUES {
                for &b in VALUES {
                    assert_eq!(<$prim as MulChecked>::checked_mul(a, b), a.checked_mul(b));
                    assert_eq!(
                        <$prim as MulOverflowing>::overflowing_mul(a, b),
                        a.overflowing_mul(b)
                    );
                    assert_eq!(<$prim as MulSaturate>::saturating_mul(a, b), a.saturating_mul(b));
                    assert_eq!(<$prim as MulWrapping>::wrapping_mul(a, b), a.wrapping_mul(b));
                }
            }
        }

        #[test]
        fn division_matches_inherent_methods() {
            for &a in VALUES {
                for &b in VALUES {
                    assert_eq!(<$prim as DivChecked>::checked_div(a, b), a.checked_div(b));
                    if b == 0 {
                        continue;
                    }
                    assert_eq!(
                        <$prim as DivOverflowing>::overflowing_div(a, b),
                        a.overflowing_div(b)
                    );
                    assert_eq!(<$prim as DivSaturate>::saturating_div(a, b), a.saturating_div(b));
                    assert_eq!(<$prim as DivWrapping>::wrapping_div(a, b), a.wrapping_div(b));
                }
            }
        }

        #[test]
        fn remainder_matches_inherent_methods() {
            for &a in VALUES {
                for &b in VALUES {
                    assert_eq!(<$prim as RemChecked>::checked_rem(a, b), a.checked_rem(b));
                    if b == 0 {
                        continue;
                    }
                    assert_eq!(
                        <$prim as RemOverflowing>::overflowing_rem(a, b),
                        a.overflowing_rem(b)
                    );
                    assert_eq!(<$prim as RemWrapping>::wrapping_rem(a, b), a.wrapping_rem(b));
                }
            }
        }

        #[test]
        fn wrapping_negation_matches_inherent_method() {
            for &value in VALUES {
                assert_eq!(<$prim as NegWrapping>::wrapping_neg(value), value.wrapping_neg());
            }
        }

        // -- Shifts ---------------------------------------------------------

        #[test]
        fn left_shift_matches_inherent_methods() {
            for &value in VALUES {
                for shift in crate::shifts($prim::BITS) {
                    assert_eq!(
                        <$prim as ShlChecked>::checked_shl(value, shift),
                        value.checked_shl(shift)
                    );
                    assert_eq!(
                        <$prim as ShlOverflowing>::overflowing_shl(value, shift),
                        value.overflowing_shl(shift)
                    );
                    assert_eq!(
                        <$prim as ShlWrapping>::wrapping_shl(value, shift),
                        value.wrapping_shl(shift)
                    );
                }
            }
        }

        #[test]
        fn right_shift_matches_inherent_methods() {
            for &value in VALUES {
                for shift in crate::shifts($prim::BITS) {
                    assert_eq!(
                        <$prim as ShrChecked>::checked_shr(value, shift),
                        value.checked_shr(shift)
                    );
                    assert_eq!(
                        <$prim as ShrOverflowing>::overflowing_shr(value, shift),
                        value.overflowing_shr(shift)
                    );
                    assert_eq!(
                        <$prim as ShrWrapping>::wrapping_shr(value, shift),
                        value.wrapping_shr(shift)
                    );
                }
            }
        }
    };
}

/// Generates the tests of the unsigned primitive integers.
macro_rules! unsigned {
    ($($prim:ident)+) => {$(
        mod $prim {
            primitive!($prim, [0, 1, 2, 3, $prim::MAX / 2, $prim::MAX - 1, $prim::MAX]);

            #[test]
            fn belongs_to_the_unsigned_category() {
                fn assert_unsigned<T: Unsigned>() {}
                assert_unsigned::<$prim>();
            }
        }
    )+};
}

/// Generates the tests of the signed primitive integers.
macro_rules! signed {
    ($($prim:ident)+) => {$(
        mod $prim {
            primitive!($prim, [
                $prim::MIN,
                $prim::MIN + 1,
                -2,
                -1,
                0,
                1,
                2,
                $prim::MAX - 1,
                $prim::MAX
            ]);

            #[test]
            fn belongs_to_the_signed_category() {
                fn assert_signed<T: Integer>() {}
                assert_signed::<$prim>();
                assert_eq!(<$prim as Integer>::NEG_ONE, -1);
            }

            #[test]
            fn is_negative_holds_only_below_zero() {
                for &value in VALUES {
                    assert_eq!(Integer::is_negative(value), value < 0, "{value}");
                }
            }

            #[test]
            fn negation_matches_inherent_methods() {
                for &value in VALUES {
                    assert_eq!(<$prim as NegChecked>::checked_neg(value), value.checked_neg());
                    assert_eq!(
                        <$prim as NegOverflowing>::overflowing_neg(value),
                        value.overflowing_neg()
                    );
                    assert_eq!(
                        <$prim as NegSaturate>::saturating_neg(value),
                        value.saturating_neg()
                    );
                }
            }
        }
    )+};
}

// ---------------------------------------------------------------------------
// Primitive integers
// ---------------------------------------------------------------------------

/// Traits of the unsigned primitive integers.
mod unsigned_primitive {
    unsigned!(u8 u16 u32 u64 u128 usize);
}

/// Traits of the signed primitive integers.
mod signed_primitive {
    signed!(i8 i16 i32 i64 i128 isize);
}

// ---------------------------------------------------------------------------
// Nonzero integers
// ---------------------------------------------------------------------------

/// Generates the tests common for the signed and unsigned nonzero integers.
/// The checked operations must return `None` both on overflow and when the
/// result is zero.
macro_rules! nonzero {
    ($nonzero:ident, $prim:ident) => {
        use core::num::$nonzero;

        use amplify_num::*;

        fn nz(value: $prim) -> $nonzero { $nonzero::new(value).expect("nonzero test value") }

        #[test]
        fn get_returns_the_wrapped_value() {
            let value: $prim = NonZero::get(nz(5));
            assert_eq!(value, 5);
            assert_eq!(NonZero::get($nonzero::MAX), $prim::MAX);
        }

        #[test]
        fn checked_add_returns_a_nonzero_sum() {
            assert_eq!(<$nonzero as AddChecked>::checked_add(nz(2), nz(3)), Some(nz(5)));
        }

        #[test]
        fn checked_add_rejects_an_overflow() {
            assert_eq!(<$nonzero as AddChecked>::checked_add($nonzero::MAX, nz(1)), None);
        }

        #[test]
        fn checked_sub_returns_a_nonzero_difference() {
            assert_eq!(<$nonzero as SubChecked>::checked_sub(nz(5), nz(2)), Some(nz(3)));
        }

        #[test]
        fn checked_sub_rejects_a_zero_difference() {
            assert_eq!(<$nonzero as SubChecked>::checked_sub(nz(5), nz(5)), None);
        }

        #[test]
        fn checked_mul_returns_a_nonzero_product() {
            assert_eq!(<$nonzero as MulChecked>::checked_mul(nz(2), nz(3)), Some(nz(6)));
        }

        #[test]
        fn checked_mul_rejects_an_overflow() {
            assert_eq!(<$nonzero as MulChecked>::checked_mul($nonzero::MAX, nz(2)), None);
        }

        #[test]
        fn checked_div_returns_a_nonzero_quotient() {
            assert_eq!(<$nonzero as DivChecked>::checked_div(nz(7), nz(3)), Some(nz(2)));
        }

        #[test]
        fn checked_div_rejects_a_zero_quotient() {
            assert_eq!(<$nonzero as DivChecked>::checked_div(nz(1), nz(2)), None);
        }

        #[test]
        fn checked_rem_returns_a_nonzero_remainder() {
            assert_eq!(<$nonzero as RemChecked>::checked_rem(nz(7), nz(3)), Some(nz(1)));
        }

        #[test]
        fn checked_rem_rejects_a_zero_remainder() {
            assert_eq!(<$nonzero as RemChecked>::checked_rem(nz(6), nz(3)), None);
        }
    };
}

/// Generates the tests of the unsigned nonzero integers.
macro_rules! nonzero_unsigned {
    ($($module:ident: $nonzero:ident($prim:ident);)+) => {$(
        mod $module {
            nonzero!($nonzero, $prim);

            #[test]
            fn checked_sub_rejects_a_negative_difference() {
                assert_eq!(<$nonzero as SubChecked>::checked_sub(nz(1), nz(2)), None);
            }
        }
    )+};
}

/// Generates the tests of the signed nonzero integers.
macro_rules! nonzero_signed {
    ($($module:ident: $nonzero:ident($prim:ident);)+) => {$(
        mod $module {
            nonzero!($nonzero, $prim);

            #[test]
            fn checked_add_rejects_a_zero_sum() {
                assert_eq!(<$nonzero as AddChecked>::checked_add(nz(-5), nz(5)), None);
            }

            #[test]
            fn checked_sub_returns_a_negative_difference() {
                assert_eq!(<$nonzero as SubChecked>::checked_sub(nz(1), nz(2)), Some(nz(-1)));
            }

            #[test]
            fn checked_sub_rejects_an_overflow() {
                assert_eq!(<$nonzero as SubChecked>::checked_sub($nonzero::MIN, nz(1)), None);
            }

            #[test]
            fn checked_mul_rejects_a_negative_overflow() {
                assert_eq!(<$nonzero as MulChecked>::checked_mul($nonzero::MIN, nz(2)), None);
            }

            #[test]
            fn checked_div_returns_a_negative_quotient() {
                assert_eq!(<$nonzero as DivChecked>::checked_div(nz(-7), nz(3)), Some(nz(-2)));
            }

            #[test]
            fn checked_div_rejects_an_overflow() {
                assert_eq!(<$nonzero as DivChecked>::checked_div($nonzero::MIN, nz(-1)), None);
            }

            #[test]
            fn checked_rem_keeps_the_sign_of_the_dividend() {
                assert_eq!(<$nonzero as RemChecked>::checked_rem(nz(-7), nz(3)), Some(nz(-1)));
            }

            #[test]
            fn checked_rem_rejects_an_overflow() {
                assert_eq!(<$nonzero as RemChecked>::checked_rem($nonzero::MIN, nz(-1)), None);
            }

            #[test]
            fn checked_neg_returns_the_opposite_value() {
                assert_eq!(<$nonzero as NegChecked>::checked_neg(nz(5)), Some(nz(-5)));
                assert_eq!(<$nonzero as NegChecked>::checked_neg(nz(-5)), Some(nz(5)));
                assert_eq!(<$nonzero as NegChecked>::checked_neg($nonzero::MAX), Some(nz(-$prim::MAX)));
            }

            #[test]
            fn checked_neg_rejects_the_minimal_value() {
                assert_eq!(<$nonzero as NegChecked>::checked_neg($nonzero::MIN), None);
            }
        }
    )+};
}

nonzero_unsigned! {
    nonzero_u8: NonZeroU8(u8);
    nonzero_u16: NonZeroU16(u16);
    nonzero_u32: NonZeroU32(u32);
    nonzero_u64: NonZeroU64(u64);
    nonzero_u128: NonZeroU128(u128);
    nonzero_usize: NonZeroUsize(usize);
}

nonzero_signed! {
    nonzero_i8: NonZeroI8(i8);
    nonzero_i16: NonZeroI16(i16);
    nonzero_i32: NonZeroI32(i32);
    nonzero_i64: NonZeroI64(i64);
    nonzero_i128: NonZeroI128(i128);
    nonzero_isize: NonZeroIsize(isize);
}
