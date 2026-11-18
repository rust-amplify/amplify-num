Change Log
==========

## _Unreleased_

- Arithmetic of the integers of different types: the operands are converted
  into the wider of their types, which is the type of the result, both in the
  binary operators and in the arithmetic traits. This includes the primitive
  integers as the left operands, wider right operands (e.g. `u24 + u64` gives
  `u64`), and mixing the small, primitive and big integers. The types of which
  none fits the other losslessly can't be combined, as for the primitive
  integers. The compound assignment operators keep the type of their left
  operand.
- Breaking: an operation of a small integer with a wider primitive one now gives
  the primitive type (e.g. `u4 + u8` gives `u8`), and the binary operators and
  the arithmetic traits of the small integers no longer accept `bool`, `char`
  and `NonZero*` operands.
- Small integers support generic right-hand operands convertible into their own
  type again, and provide lossless `From` conversions from compatible unsigned
  primitive types (`u8`, `u16`, and `u32`).
- Lossless conversions of `usize`, `isize` and the small integers into `BigInt`.
