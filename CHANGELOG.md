Change Log
==========

## v0.6.0

### Breaking changes

- Remove the `amplify_apfloat` workspace crate (`apfloat/`) and flatten the
  repository to the `amplify_num` crate root.
- Remove the `posit` module (`Posit8`, `Posit16`, `Posit32`, `Posit64`,
  `Posit128`, `Posit256`, `Posit512`) and `error::PositDecodeError`.
- Upgrade Rust edition to `2024` and bump MSRV to `1.100.0`.
- Forbid `unsafe` code crate-wide (`#![forbid(unsafe_code)]`) and remove raw
  pointer methods `as_ptr` and `as_mut_ptr` from big integers.
- Remove the `all` Cargo feature and make `serde` independent of `std` and
  `alloc`; `hex::ToHex` and `hex::FromHex` for `Vec<u8>` now require `alloc`
  (`std` implies `alloc`).
- Replace inherent `checked_*`, `overflowing_*`, `saturating_*` and `wrapping_*`
  arithmetic methods on `BigInt` and small integers with the corresponding
  traits in `traits` (re-exported at the crate root).
- Replace separate macro-generated big integer structs with const-generic
  `BigInt<const N: usize, const SIGNED: bool>`, exposing `u256`, `u512`,
  `u1024`, `i256`, `i512` and `i1024` as type aliases:
  - `Display` and default `Debug` now format values in decimal instead of
    zero-padded `0x`-prefixed hexadecimal.
  - Remove `INNER_LEN`, `TryFrom<&[u64]>` and `core::ops::Index`
    implementations; limbs are accessed via `from_inner`, `as_inner` and
    `into_inner`.
  - Change `BYTES` from `u8` to `usize`, `bit` index from `usize` to `u32`,
    `bits_required` return type from `usize` to `u32` (including the sign bit
    for signed integers), and `Shl`/`Shr`/`ShlAssign`/`ShrAssign` shift counts
    from `usize` to `u32` (panicking when `rhs >= BITS`).
  - Restrict unary `Neg` and `abs` to signed `BigInt<N, true>` so negating
    unsigned big integers is rejected at compile time.
- Small and custom bit-sized integers (`u1`..=`u7`, `u10`, `u12`, `u14`, `u20`,
  `u24`, `u40`, `u48`, `u56`):
  - Mixing a small integer with a wider primitive integer now yields the wider
    primitive type (e.g. `u4 + u8` gives `u8`), and binary operators and
    arithmetic traits no longer accept `bool`, `char`, `NonZero*` or `&Self`
    left-hand reference operands.
  - Change `FromStr::Err` from `core::num::ParseIntError` to
    `error::ParseIntError`.
  - Delegate `Debug` to the underlying primitive integer instead of deriving
    tuple-struct formatting.
  - Remove `AsRef`, `into_u8`/`into_u16`/`into_u32`/`into_u64`, and inherent
    primitive cast helpers (`to_*`/`into_*`) on `u24`, `u40`, `u48` and `u56` in
    favor of `From`/`Into` and by-value `const fn` accessors (`to_u8`,
    `to_u16`, `to_u32`, `to_u64`).
  - Change `Shl`/`Shr`/`ShlAssign`/`ShrAssign` to take `u32` shift counts, mask
    shifted-out high bits within `BITS`, and panic when `rhs >= BITS`.

### Added

- Add the `traits` module (re-exported at the crate root) with integer category
  traits (`Number`, `Integer`, `Unsigned`, `NonZero`) and explicit
  overflow-handling traits (`*Checked`, `*Overflowing`, `*Saturate`,
  `*Wrapping` for addition, subtraction, multiplication, division, remainder,
  negation and bit shifts), implemented for primitive integers, all crate
  integer types, and `core::num::NonZero*` types (`NonZero` and `*Checked`).
- Arithmetic of integers of different types: binary arithmetic and bitwise
  operators and the binary arithmetic traits convert both operands into the
  wider type, which is the result type. This supports primitive integers on
  either side of an operator, wider right operands (e.g. `u24 + u64` gives
  `u64`), and mixing small, primitive and big integers while preserving strict
  panic-on-overflow semantics. Types where neither fits the other losslessly
  cannot be combined, matching primitive integers, and compound assignment
  operators keep the type of their left operand.
- `BigInt<N, SIGNED>` enhancements:
  - Support arbitrary non-zero counts of 64-bit limbs (`64 * N` bits).
  - Implement `FromStr` and `from_str_radix` (`2..=36`), allocation-free
    `Display`, `Binary`, `Octal`, `LowerHex` and `UpperHex`, and hexadecimal
    `Debug` flag support (`{:x?}` and `{:X?}`).
  - Expose `div_rem` and `div_rem_euclid`, and add `unbounded_shl`,
    `unbounded_shr`, `cast_signed`, `cast_unsigned` and `unsigned_abs`.
  - Add lossless `From` conversions from `usize`, `isize` and all small integers
    into `BigInt`, widening `From` conversions for signed big integers
    (`i256` into `i512`/`i1024`, `i512` into `i1024`), and compile-time
    rejection of lossy primitive-to-`BigInt` conversions.
  - Support allocation-free `serde` serialization and deserialization in
    `no_std` builds without `alloc`.
- Small and custom bit-sized integer enhancements:
  - Add named constants covering all values of `u2` (`TWO`, `THREE`) and `u3`
    (`TWO`, `THREE`, `FOUR`, `FIVE`, `SIX`, `SEVEN`).
  - Add `from_str_radix` (`2..=36`), bitwise `Not` (`!`), and `div_rem`
    accepting `impl Into<Inner>`.
  - Make `from_le_bytes`, `from_be_bytes`, `to_le_bytes` and `to_be_bytes` on
    `u24`, `u40`, `u48` and `u56` `const fn`.
  - Support generic right-hand operands convertible into `Self`, lossless `From`
    conversions from compatible unsigned primitives (`u8`, `u16`, `u32`), and
    complete lossless `From` widening across all larger small, primitive
    (pointer-width aware for `usize`/`isize`) and big integer types.
- `error` and `hex` enhancements:
  - Add `error::ParseIntError` wrapping `core::num::IntErrorKind`, implement
    `core::error::Error` for all crate error types in `no_std` builds, and
    generalize `Display` and `Error` on `OverflowError<T>` over `T`.
  - Implement `hex::FromHex` generically for byte arrays `[u8; LEN]` of any
    length.

### Fixed

- Fix `BigInt` multi-limb arithmetic and edge cases:
  - Fix `saturating_sub` saturating to `MAX` on underflow and signed
    `saturating_add`/`saturating_mul` clamping negative overflow to `MAX`
    instead of `MIN`.
  - Fix signed `overflowing_add` and `overflowing_sub` failing to detect
    overflow when an operand is `MIN`.
  - Fix signed `MIN.abs()` silently wrapping instead of panicking on overflow.
  - Fix arithmetic right shifts (`>>`) on negative signed `BigInt` values not
    sign-extending all vacated bits.
  - Replace slow bit-by-bit division with normalized multi-limb division.
- Fix small integer arithmetic and deserialization:
  - Fix `wrapping_*` and `overflowing_*` addition, subtraction and
    multiplication reducing modulo `MAX` (`2^BITS - 1`) instead of `2^BITS`.
  - Enforce strict panic-on-overflow checks in release builds for all binary and
    compound-assignment arithmetic operators instead of relying on primitive
    operators that wrap in release mode.
  - Validate bit-width bounds during `serde::Deserialize` instead of accepting
    out-of-range underlying primitives via `#[serde(transparent)]`.

### Documentation, testing & CI

- Rewrite and expand `README.md` with doctested examples, trait overview and
  feature flag documentation.
- Add comprehensive test suites covering `BigInt`, limb arithmetic, small
  integers, traits, primitive implementations, mixed-type promotion, `hex`,
  `serde` and security-audit findings, and update Criterion benchmarks.
- Standardize repository configuration and CI workflows with `cargo-deny`
  (`deny.toml`), Miri, `cargo-hack` feature-powerset checks, bare-metal ARM
  Cortex-M and `wasm32-wasip1` targets, `docs.rs` rustdoc verification, and
  Dependabot.
