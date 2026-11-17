//! Integration tests for the public integer trait API.

use core::num::{
    NonZeroI8, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI128, NonZeroIsize, NonZeroU8,
    NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU128, NonZeroUsize,
};

use amplify_num::{
    AddChecked, AddOverflowing, AddSaturate, AddWrapping, BigInt, DivChecked, DivOverflowing,
    DivSaturate, DivWrapping, Integer, MulChecked, MulOverflowing, MulSaturate, MulWrapping,
    NegChecked, NegOverflowing, NegSaturate, NegWrapping, NonZero, Number, RemChecked,
    RemOverflowing, RemWrapping, ShlChecked, ShlOverflowing, ShlWrapping, ShrChecked,
    ShrOverflowing, ShrWrapping, SubChecked, SubOverflowing, SubSaturate, SubWrapping, Unsigned,
    i256, i512, i1024, u1, u2, u3, u4, u5, u6, u7, u10, u12, u14, u20, u24, u40, u48, u56, u256,
    u512, u1024,
};

fn add_checked<T, Rhs>(lhs: T, rhs: Rhs) -> Option<T>
where T: AddChecked<Rhs, Output = T> {
    lhs.checked_add(rhs)
}

fn add_overflowing<T, Rhs>(lhs: T, rhs: Rhs) -> (T, bool)
where T: AddOverflowing<Rhs, Output = T> {
    lhs.overflowing_add(rhs)
}

fn add_saturating<T, Rhs>(lhs: T, rhs: Rhs) -> T
where T: AddSaturate<Rhs, Output = T> {
    lhs.saturating_add(rhs)
}

fn add_wrapping<T, Rhs>(lhs: T, rhs: Rhs) -> T
where T: AddWrapping<Rhs, Output = T> {
    lhs.wrapping_add(rhs)
}

fn sub_checked<T, Rhs>(lhs: T, rhs: Rhs) -> Option<T>
where T: SubChecked<Rhs, Output = T> {
    lhs.checked_sub(rhs)
}

fn sub_overflowing<T, Rhs>(lhs: T, rhs: Rhs) -> (T, bool)
where T: SubOverflowing<Rhs, Output = T> {
    lhs.overflowing_sub(rhs)
}

fn sub_saturating<T, Rhs>(lhs: T, rhs: Rhs) -> T
where T: SubSaturate<Rhs, Output = T> {
    lhs.saturating_sub(rhs)
}

fn sub_wrapping<T, Rhs>(lhs: T, rhs: Rhs) -> T
where T: SubWrapping<Rhs, Output = T> {
    lhs.wrapping_sub(rhs)
}

#[test]
fn number_categories_cover_integer_types() {
    fn number<T: Number>() {}
    fn integer<T: Integer>() {}
    fn natural<T: Unsigned>() {}
    fn nonzero<T: NonZero>() {}

    macro_rules! assert_naturals {
        ($($ty:ty),+ $(,)?) => {$({
            number::<$ty>();
            natural::<$ty>();
        })+};
    }
    macro_rules! assert_integers {
        ($($ty:ty),+ $(,)?) => {$({
            number::<$ty>();
            integer::<$ty>();
        })+};
    }
    macro_rules! assert_nonzeros {
        ($($ty:ty),+ $(,)?) => {$({ nonzero::<$ty>(); })+};
    }

    assert_naturals!(u8, u16, u32, u64, u128, usize);
    assert_naturals!(u1, u2, u3, u4, u5, u6, u7, u10, u12, u14, u20, u24, u40, u48, u56);
    assert_naturals!(BigInt<2, false>, u256, u512, BigInt<12, false>, u1024);
    assert_integers!(i8, i16, i32, i64, i128, isize);
    assert_integers!(i256, BigInt<3, true>, i512, BigInt<12, true>, i1024);
    assert_nonzeros!(
        NonZeroU8,
        NonZeroU16,
        NonZeroU32,
        NonZeroU64,
        NonZeroU128,
        NonZeroUsize,
        NonZeroI8,
        NonZeroI16,
        NonZeroI32,
        NonZeroI64,
        NonZeroI128,
        NonZeroIsize
    );

    assert_eq!(<u3 as Number>::BITS, 3);
    assert_eq!(<i256 as Integer>::NEG_ONE, -i256::ONE);
    assert_eq!(<u256 as Number>::ZERO, u256::ZERO);
}

#[test]
fn add_sub_overflow_flavors_match_integer_bounds() {
    assert_eq!(add_checked(u8::MAX, 1), None);
    assert_eq!(add_overflowing(u8::MAX, 1), (0, true));
    assert_eq!(add_saturating(u8::MAX, 1), u8::MAX);
    assert_eq!(add_wrapping(u8::MAX, 1), 0);

    assert_eq!(sub_checked(i8::MIN, 1), None);
    assert_eq!(sub_overflowing(i8::MIN, 1), (i8::MAX, true));
    assert_eq!(sub_saturating(i8::MIN, 1), i8::MIN);
    assert_eq!(sub_wrapping(i8::MIN, 1), i8::MAX);

    assert_eq!(add_checked(u3::MAX, 1u8), None);
    assert_eq!(add_overflowing(u3::MAX, 1u8), (u3::ZERO, true));
    assert_eq!(add_saturating(u3::MAX, 1u8), u3::MAX);
    assert_eq!(add_wrapping(u3::MAX, 1u8), u3::ZERO);
    assert_eq!(sub_checked(u3::ZERO, 1u8), None);
    assert_eq!(sub_overflowing(u3::ZERO, 1u8), (u3::MAX, true));
    assert_eq!(sub_saturating(u3::ZERO, 1u8), u3::ZERO);
    assert_eq!(sub_wrapping(u3::ZERO, 1u8), u3::MAX);

    assert_eq!(add_checked(i256::MAX, i256::ONE), None);
    assert_eq!(add_checked(i256::MAX, 1i8), None);
    assert_eq!(add_saturating(i256::MAX, i256::ONE), i256::MAX);
    assert_eq!(sub_checked(i256::MIN, i256::ONE), None);
    assert_eq!(sub_saturating(i256::MIN, i256::ONE), i256::MIN);

    assert_eq!(<u8 as MulChecked>::checked_mul(u8::MAX, 2), None);
    assert_eq!(<u8 as MulOverflowing>::overflowing_mul(u8::MAX, 2), (254, true));
    assert_eq!(<u8 as MulSaturate>::saturating_mul(u8::MAX, 2), u8::MAX);
    assert_eq!(<u8 as MulWrapping>::wrapping_mul(u8::MAX, 2), 254);
}

#[test]
fn division_and_negation_cover_signed_extremes() {
    assert_eq!(<i8 as DivChecked>::checked_div(i8::MIN, -1), None);
    assert_eq!(<i8 as DivOverflowing>::overflowing_div(i8::MIN, -1), (i8::MIN, true));
    assert_eq!(<i8 as DivChecked>::checked_div(7, 0), None);
    assert_eq!(<i8 as RemChecked>::checked_rem(7, 0), None);
    assert_eq!(<i8 as RemChecked>::checked_rem(i8::MIN, -1), None);
    assert_eq!(<i8 as RemOverflowing>::overflowing_rem(i8::MIN, -1), (0, true));
    assert_eq!(<i8 as RemWrapping>::wrapping_rem(i8::MIN, -1), 0);
    assert_eq!(<i256 as DivChecked>::checked_div(i256::MIN, -i256::ONE), None);

    assert_eq!(<i8 as NegChecked>::checked_neg(i8::MIN), None);
    assert_eq!(<i8 as NegOverflowing>::overflowing_neg(i8::MIN), (i8::MIN, true));
    assert_eq!(<i8 as NegSaturate>::saturating_neg(i8::MIN), i8::MAX);
    assert_eq!(<i8 as NegWrapping>::wrapping_neg(i8::MIN), i8::MIN);
    assert_eq!(<i256 as NegChecked>::checked_neg(i256::MIN), None);
    assert!(std::panic::catch_unwind(|| <u8 as DivWrapping>::wrapping_div(1, 0)).is_err());
}

#[test]
fn shifts_use_logical_width_for_custom_integers() {
    assert_eq!(<u3 as ShlChecked>::checked_shl(u3::ONE, 3), None);
    assert_eq!(<u3 as ShlOverflowing>::overflowing_shl(u3::ONE, 3), (u3::ONE, true));
    assert_eq!(<u3 as ShlWrapping>::wrapping_shl(u3::ONE, 3), u3::ONE);
    assert_eq!(<u3 as ShrChecked>::checked_shr(u3::MAX, 3), None);
    assert_eq!(<u3 as ShrOverflowing>::overflowing_shr(u3::MAX, 3), (u3::MAX, true));
    assert_eq!(<u3 as ShrWrapping>::wrapping_shr(u3::MAX, 3), u3::MAX);

    assert_eq!(<u8 as ShlChecked>::checked_shl(1, 8), None);
    assert_eq!(<u8 as ShlOverflowing>::overflowing_shl(1, 8), (1, true));
    assert_eq!(<u8 as ShlWrapping>::wrapping_shl(1, 8), 1);
}

#[test]
fn custom_integer_arithmetic_uses_traits_directly() {
    let small = u3::with(6);
    let sum = small.checked_add(1u8).unwrap();
    assert_eq!(sum, u3::with(7));
    assert_eq!(small + 1u8, sum);
    assert_eq!(u3::ONE.wrapping_neg(), u3::MAX);
    assert!(std::panic::catch_unwind(|| u3::MAX + 1u8).is_err());
    assert!(std::panic::catch_unwind(|| u3::ONE.wrapping_div(0u8)).is_err());
    assert!(std::panic::catch_unwind(|| u3::ONE.wrapping_rem(0u8)).is_err());

    let big_signed = BigInt::<1, true>::MAX;
    assert_eq!(big_signed.overflowing_add(1i8), (BigInt::<1, true>::MIN, true));
    assert_eq!(u256::ONE.wrapping_neg(), u256::MAX);

    let lhs = BigInt::<1, true>::from(123i64);
    let rhs = BigInt::<1, true>::from(-7i64);
    let (model, overflow) = 123i64.overflowing_mul(-7);
    let (actual, trait_overflow) = lhs.overflowing_mul(rhs);
    assert_eq!(actual, BigInt::<1, true>::from(model));
    assert_eq!(trait_overflow, overflow);
    assert_eq!(lhs.saturating_mul(rhs), lhs.checked_mul(rhs).unwrap());
    assert_eq!(i256::ONE + 1i8, i256::from(2i8));

    let signed_min = BigInt::<1, true>::MIN;
    let negative_one = BigInt::<1, true>::from(-1i8);
    assert_eq!(signed_min.overflowing_div(negative_one), (signed_min, true));
    assert_eq!(signed_min.saturating_div(negative_one), BigInt::<1, true>::MAX);
    assert_eq!(signed_min.wrapping_rem(negative_one), BigInt::<1, true>::ZERO);
    assert_eq!(BigInt::<1, true>::ONE.checked_div(BigInt::ZERO), None);
    assert!(
        std::panic::catch_unwind(|| BigInt::<1, true>::ONE.wrapping_div(BigInt::ZERO)).is_err()
    );
}

#[test]
fn nonzero_traits_preserve_the_nonzero_invariant() {
    let one = NonZeroU8::new(1).unwrap();
    let two = NonZeroU8::new(2).unwrap();
    assert_eq!(<NonZeroU8 as NonZero>::get(two), 2);
    assert_eq!(
        <NonZeroU8 as AddChecked>::checked_add(two, one)
            .unwrap()
            .get(),
        3
    );
    assert_eq!(<NonZeroU8 as SubChecked>::checked_sub(two, two), None);
    assert_eq!(
        <NonZeroU8 as MulChecked>::checked_mul(two, one)
            .unwrap()
            .get(),
        2
    );

    let negative_min = NonZeroI8::new(i8::MIN).unwrap();
    assert_eq!(<NonZeroI8 as NegChecked>::checked_neg(negative_min), None);
}
