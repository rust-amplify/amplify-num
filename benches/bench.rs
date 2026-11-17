//! Benchmarks of the big integer arithmetic.

use std::hint::black_box;

use amplify_num::{i1024, u1024};
use criterion::{Criterion, criterion_group, criterion_main};

const X: [u64; 16] = [
    0xe4, 0xf7, 0xab, 0x7f, 0xdd, 0xaa, 0xbb, 0xcc, 0x77, 0x66, 0xf5, 0x06, 0x39, 0xa2, 0xb8, 0xcc,
];
const Y: [u64; 16] = [
    0xd2, 0x77, 0x66, 0xf5, 0x06, 0x39, 0xee, 0xa6, 0xf7, 0xab, 0x7f, 0xc2, 0x55, 0x75, 0x90, 0x21,
];
/// The same as `Y`, but negative when read as a signed integer.
const Z: [u64; 16] = {
    let mut limbs = Y;
    limbs[15] |= 0xffff_ffff_ffff_ff00;
    limbs
};

fn criterion_u1024(c: &mut Criterion) {
    const X: u1024 = u1024::from_inner(self::X);
    const Y: u1024 = u1024::from_inner(self::Y);

    c.bench_function("u1024_add", |c| c.iter(|| black_box(X).overflowing_add(black_box(Y))));
    c.bench_function("u1024_sub", |c| c.iter(|| black_box(X).overflowing_sub(black_box(Y))));
    c.bench_function("u1024_mul", |c| c.iter(|| black_box(X).overflowing_mul(black_box(Y))));
    c.bench_function("u1024_div", |c| c.iter(|| black_box(X).overflowing_div(black_box(Y))));
    // A divisor of a single limb gives the longest quotient.
    c.bench_function("u1024_div_limb", |c| {
        c.iter(|| black_box(X).overflowing_div(black_box(u64::MAX / 3)))
    });
    c.bench_function("u1024_cmp_eq", |c| c.iter(|| black_box(X).cmp(black_box(&X))));
    c.bench_function("u1024_to_string", |c| c.iter(|| black_box(X).to_string()));
}

fn criterion_i1024(c: &mut Criterion) {
    const X: i1024 = i1024::from_inner(self::X);
    const Y: i1024 = i1024::from_inner(self::Y);
    const Z: i1024 = i1024::from_inner(self::Z);
    assert!(X.is_positive());
    assert!(Y.is_positive());
    assert!(Z.is_negative());

    c.bench_function("i1024_add", |c| c.iter(|| black_box(X).overflowing_add(black_box(Y))));
    c.bench_function("i1024_mul", |c| c.iter(|| black_box(X).overflowing_mul(black_box(Z))));
    c.bench_function("i1024_div", |c| c.iter(|| black_box(X).overflowing_div(black_box(Z))));
    c.bench_function("i1024_is_positive", |c| c.iter(|| black_box(X).is_positive()));
    c.bench_function("i1024_is_negative", |c| c.iter(|| black_box(X).is_negative()));
    c.bench_function("i1024_zero_is_positive", |c| c.iter(|| black_box(i1024::ZERO).is_positive()));
    c.bench_function("i1024_zero_is_negative", |c| c.iter(|| black_box(i1024::ZERO).is_negative()));
    c.bench_function("i1024_cmp_pos", |c| c.iter(|| black_box(X).cmp(black_box(&Y))));
    c.bench_function("i1024_cmp_mixed", |c| c.iter(|| black_box(X).cmp(black_box(&Z))));
    c.bench_function("i1024_cmp_eq", |c| c.iter(|| black_box(X).cmp(black_box(&X))));
}

criterion_group!(benches, criterion_i1024, criterion_u1024);
criterion_main!(benches);
