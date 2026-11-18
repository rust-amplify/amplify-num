# Amplifying Rust number types

[![Build](https://github.com/rust-amplify/amplify-num/actions/workflows/build.yml/badge.svg)](https://github.com/rust-amplify/amplify-num/actions/workflows/build.yml)
[![Tests](https://github.com/rust-amplify/amplify-num/actions/workflows/test.yml/badge.svg)](https://github.com/rust-amplify/amplify-num/actions/workflows/test.yml)
[![Lints](https://github.com/rust-amplify/amplify-num/actions/workflows/lint.yml/badge.svg)](https://github.com/rust-amplify/amplify-num/actions/workflows/lint.yml)
[![Audit](https://github.com/rust-amplify/amplify-num/actions/workflows/audit.yml/badge.svg)](https://github.com/rust-amplify/amplify-num/actions/workflows/audit.yml)
[![codecov](https://codecov.io/gh/rust-amplify/amplify-num/branch/master/graph/badge.svg)](https://codecov.io/gh/rust-amplify/amplify-num)

[![crates.io](https://img.shields.io/crates/v/amplify_num)](https://crates.io/crates/amplify_num)
[![Docs](https://docs.rs/amplify_num/badge.svg)](https://docs.rs/amplify_num)
[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance/)
[![Apache-2.0 licensed](https://img.shields.io/crates/l/amplify_num)](./LICENSE)

`amplify_num` extends Rust's integer system with fixed-width big integers,
sub-byte and custom bit-sized unsigned integers, and unified traits for integer
categories and explicit overflow-handling arithmetic. It is part of the
[`amplify`] Rust language amplification library.

- **Strict overflow semantics**: unlike primitive integers, arithmetic operators
  (`+`, `-`, `*`, `/`, `%`, `<<`, `>>`, and unary `-`) panic on overflow in
  **both debug and release** builds instead of silently wrapping. Expected
  overflow is handled explicitly through `checked_*`, `overflowing_*`,
  `saturating_*`, and `wrapping_*` trait methods.
- **`#![forbid(unsafe_code)]`**: implemented entirely in safe Rust and verified
  under Miri in CI.
- **`no_std` and allocation-free by default**: zero mandatory dependencies;
  requires neither the standard library nor an allocator, so it works on
  bare-metal targets (ARM Cortex-M, WebAssembly, etc.).

## Integer Types

- **Big signed and unsigned integers (`BigInt<N, SIGNED>`)**:
  - Fixed-size integers composed of `N` little-endian 64-bit limbs (`64 * N`
    bits), using two's complement representation for signed types.
  - Type aliases for common widths: `u256`, `u512`, `u1024`, `i256`, `i512`,
    and `i1024`; arbitrary multiples of 64 bits can be instantiated directly via
    `BigInt<N, SIGNED>`.
  - Supports lossless mixed-width primitive operands, standard formatting traits
    (`Display`, `Debug`, `Binary`, `Octal`, `LowerHex`, `UpperHex`), `FromStr`
    and `from_str_radix` (`2..=36`), big- and little-endian byte array/slice
    conversions, `div_rem` and Euclidean division, bit-inspection helpers, and
    compile-time rejection of lossy `From` conversions.
- **Sub-byte unsigned integers (`u1` to `u7`)**:
  - `u1`, `u2`, `u3`, `u4`, `u5`, `u6`, and `u7` backed by `u8`, enforcing the
    range `0..2^BITS` across construction (`with`, `TryFrom`, `FromStr`,
    `from_str_radix`), bitwise operations, shifts, and arithmetic.
- **Custom bit-sized unsigned integers (`u10` to `u56`)**:
  - `u10`, `u12`, `u14` (backed by `u16`), `u20`, `u24` (backed by `u32`), and
    `u40`, `u48`, `u56` (backed by `u64`).
  - Whole-byte widths (`u24`, `u40`, `u48`, `u56`) provide `const` big- and
    little-endian byte-array conversions (`[u8; 3]`, `[u8; 5]`, `[u8; 6]`,
    `[u8; 7]`).
  - All sub-byte and custom bit-sized integers implement lossless `From`
    widening into every larger custom and primitive integer type.

## Numeric & Arithmetic Traits

The [`traits`] module (re-exported at the crate root) defines category and
overflow-explicit arithmetic traits implemented for Rust's primitive integers,
all custom-width integers in this crate, and `core::num::NonZero*` types
(floating-point types are intentionally excluded):

- **Category traits**:
  - `Number`: fixed-width zero-inclusive integers (`BITS`, `MIN`, `MAX`, `ZERO`,
    `ONE`, `is_zero`, `is_one`).
  - `Integer`: signed integers (`NEG_ONE`, `is_negative`).
  - `Unsigned`: zero-inclusive unsigned integers.
  - `NonZero`: nonzero integer wrappers (`Inner: Number`, `get`), kept separate
    from `Number` because nonzero types have no zero identity.
- **Overflow-handling arithmetic traits**:
  - Checked (`Option<Output>`): `AddChecked`, `SubChecked`, `MulChecked`,
    `DivChecked`, `RemChecked`, `NegChecked`, `ShlChecked`, `ShrChecked` (also
    implemented for `NonZero*`, returning `None` if a result would be zero or
    overflow).
  - Overflowing (`(Output, bool)`): `AddOverflowing`, `SubOverflowing`,
    `MulOverflowing`, `DivOverflowing`, `RemOverflowing`, `NegOverflowing`,
    `ShlOverflowing`, `ShrOverflowing`.
  - Saturating (`Output`): `AddSaturate`, `SubSaturate`, `MulSaturate`,
    `DivSaturate`, `NegSaturate`.
  - Wrapping (`Output`): `AddWrapping`, `SubWrapping`, `MulWrapping`,
    `DivWrapping`, `RemWrapping`, `NegWrapping`, `ShlWrapping`, `ShrWrapping`.

For `BigInt` and the small/custom-width integers, these traits provide their
`checked_*`, `overflowing_*`, `saturating_*`, and `wrapping_*` methods directly
(without duplicate inherent methods) while supporting generic right-hand
operands.

## Usage

Add `amplify_num` to `Cargo.toml`:

```toml
[dependencies]
amplify_num = "0.6"
```

For minimal `no_std` builds without default features, or with `serde` enabled:

```toml
[dependencies]
amplify_num = { version = "0.6", default-features = false, features = ["serde"] }
```

### Big integers

```rust
use amplify_num::{AddChecked, BigInt, SubSaturate, i256, u256};

type U384 = BigInt<6, false>;
assert_eq!(U384::BITS, 384);

let x = u256::from(u128::MAX) * 3u8 + 2u8;
assert_eq!(x.to_string(), "1020847100762815390390123822295304634367");
assert_eq!(format!("{x:#x}"), "0x2ffffffffffffffffffffffffffffffff");
assert_eq!("1020847100762815390390123822295304634367".parse(), Ok(x));
assert_eq!(u256::from_str_radix("2ffffffffffffffffffffffffffffffff", 16), Ok(x));

assert_eq!(x.div_rem(u128::MAX), Ok((u256::from(3u8), u256::from(2u8))));
assert_eq!(i256::from(-7).rem_euclid(4), i256::from(1));
assert_eq!(u256::from_be_bytes(x.to_be_bytes()), x);

assert_eq!(u256::MAX.checked_add(1u8), None);
assert_eq!(i256::MIN.saturating_sub(1), i256::MIN);
```

### Sub-byte and custom bit-sized integers

```rust
use amplify_num::{AddSaturate, AddWrapping, u4, u24};

let nibble = u4::with(0b1010);
assert_eq!(nibble.to_u8(), 10);
assert!(u4::try_from(16u8).is_err());

assert_eq!(nibble + 5u8, u4::MAX);
assert_eq!(nibble.wrapping_add(7u8), u4::ONE);
assert_eq!(nibble.saturating_add(10u8), u4::MAX);

let tag = u24::from_be_bytes([0x12, 0x34, 0x56]);
assert_eq!(u32::from(tag), 0x0012_3456);
assert_eq!(u24::from(nibble), u24::with(10));
```

### Generic integer bounds

```rust
use core::num::NonZeroU32;

use amplify_num::{AddChecked, NonZero, Number, SubChecked, u12, u256};

fn checked_add_one<T>(value: T) -> Option<T>
where T: Number + AddChecked<Output = T> {
    value.checked_add(T::ONE)
}

fn checked_diff<T>(lhs: T, rhs: T) -> Option<T>
where T: SubChecked<Output = T> {
    lhs.checked_sub(rhs)
}

assert_eq!(checked_add_one(254u8), Some(255));
assert_eq!(checked_add_one(u8::MAX), None);
assert_eq!(checked_add_one(u12::MAX), None);
assert_eq!(checked_add_one(u256::ONE), Some(u256::from(2u8)));

let five = NonZeroU32::new(5).unwrap();
assert_eq!(checked_diff(five, NonZeroU32::MIN).map(NonZero::get), Some(4));
assert_eq!(checked_diff(five, five), None);
```

## Feature Flags

| Feature | Default | Description                                                                                                  |
|---------|---------|--------------------------------------------------------------------------------------------------------------|
| `hex`   | yes     | Hexadecimal decoding and byte iteration (`amplify_num::hex::FromHex`, `HexIterator`); works without `alloc`  |
| `alloc` | no      | Enables `alloc` support (`amplify_num::hex::ToHex` and `FromHex` for `Vec<u8>` with `hex`, plus `serde/alloc`) |
| `std`   | no      | Enables `alloc` and standard library support in optional dependencies (`serde/std`)                          |
| `serde` | no      | Serialization and deserialization of all integer types (enables `hex`; works without an allocator)           |

With `serde`, `BigInt` uses zero-padded lowercase hex strings in human-readable
formats and raw big-endian bytes in binary formats; sub-byte and custom
bit-sized integers serialize as their underlying primitive integer and validate
the bit range on deserialization.

## Minimum Supported Rust Version & License

- **MSRV**: Rust **1.100.0** (Rust edition **2024**).
- **Targets**: tested in CI on Linux, macOS, Windows, WebAssembly
  (`wasm32-unknown-unknown`, `wasm32-wasip1`), and bare-metal ARM Cortex-M
  (`thumbv6m-none-eabi`, `thumbv7em-none-eabi`, `thumbv8m.main-none-eabi`).
- **License**: [Apache-2.0](./LICENSE). See [CONTRIBUTING.md](./CONTRIBUTING.md)
  for contribution guidelines and [SECURITY.md](./SECURITY.md) for vulnerability
  reporting.

[`amplify`]: https://crates.io/crates/amplify
[`traits`]: https://docs.rs/amplify_num/latest/amplify_num/traits/index.html
