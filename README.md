# Amplifying rust number types

![Build](https://github.com/rust-amplify/amplify-num/workflows/Build/badge.svg)
![Tests](https://github.com/rust-amplify/amplify-num/workflows/Tests/badge.svg)
![Lints](https://github.com/rust-amplify/amplify-num/workflows/Lints/badge.svg)
[![codecov](https://codecov.io/gh/rust-amplify/amplify-num/branch/master/graph/badge.svg)](https://codecov.io/gh/rust-amplify/amplify-num)

[![crates.io](https://img.shields.io/crates/v/amplify_num)](https://crates.io/crates/amplify_num)
[![Docs](https://docs.rs/amplify_num/badge.svg)](https://docs.rs/amplify_num)
[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance/)
[![Apache-2 licensed](https://img.shields.io/crates/l/amplify_num)](./LICENSE)

Crate adding more number types to rust:
- big signed and unsigned integers (`u256`, `u512`, `u1024`, `i256`, `i512`,
  `i1024`), which are aliases of `BigInt<N, SIGNED>` supporting any number of
  64-bit limbs;
- small-bit unsigned integers (`u1` to `u7`);
- custom bit-sized unsigned integers (`u10`, `u12`, `u14`, `u20`, `u24`, `u40`,
  `u48`, `u56`).

Unlike the primitive integers, all the types panic on an arithmetic overflow in
both debug and release builds; the cases when an overflow is expected must use
`checked_*`, `overflowing_*`, `saturating_*` or `wrapping_*` methods.

The crate contains no unsafe code, has no mandatory dependencies and requires
neither the standard library nor an allocator, so it works on bare-metal
targets (Cortex-M etc.).

| Feature | Default | Description                                                      |
|---------|---------|------------------------------------------------------------------|
| `hex`   | yes     | Hex encoding and decoding (`amplify_num::hex`)                   |
| `alloc` | no      | Hex conversions of the allocated types (`String`, `Vec<u8>`)     |
| `std`   | no      | Enables `alloc` and the standard library support in dependencies |
| `serde` | no      | Serialization of all the types; works without an allocator       |

This is a part of Rust language amplification library [`amplify`].

Minimum supported rust compiler version (MSRV): 1.100.0. Rust edition 2024.

[`amplify`]: https://crates.io/crate/amplify
