//! Integration tests of the serialization with `serde`.
//!
//! The human-readable representation is tested with JSON. A binary format is
//! modelled by a minimal serializer and deserializer defined here, since the
//! crate has no binary format among its dependencies.

#![cfg(feature = "serde")]

use amplify_num::BigInt;
use serde::de::value::Error as ValueError;
use serde::de::{DeserializeOwned, Deserializer, Visitor};
use serde::ser::{Impossible, Serializer};
use serde::{Deserialize, Serialize, forward_to_deserialize_any};

// ---------------------------------------------------------------------------
// Test harness
// ---------------------------------------------------------------------------

/// The value which a serializer was asked to write.
#[derive(Debug, PartialEq, Eq)]
enum Token {
    Bytes(Vec<u8>),
    Str(String),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
}

/// Serializer which records the only value it is asked to write.
struct Recorder {
    human_readable: bool,
}

fn unsupported() -> ValueError { serde::ser::Error::custom("unsupported by the test format") }

/// Generates the methods of the serializer rejecting the types which the
/// integers must not be serialized as.
macro_rules! unsupported {
    ($($method:ident($($arg:ty),*) -> $ret:ty;)+) => {$(
        fn $method(self $(, _: $arg)*) -> Result<$ret, ValueError> { Err(unsupported()) }
    )+};
}

impl Serializer for Recorder {
    type Ok = Token;
    type Error = ValueError;
    type SerializeSeq = Impossible<Token, ValueError>;
    type SerializeTuple = Impossible<Token, ValueError>;
    type SerializeTupleStruct = Impossible<Token, ValueError>;
    type SerializeTupleVariant = Impossible<Token, ValueError>;
    type SerializeMap = Impossible<Token, ValueError>;
    type SerializeStruct = Impossible<Token, ValueError>;
    type SerializeStructVariant = Impossible<Token, ValueError>;

    unsupported! {
        serialize_bool(bool) -> Token;
        serialize_i8(i8) -> Token;
        serialize_i16(i16) -> Token;
        serialize_i32(i32) -> Token;
        serialize_i64(i64) -> Token;
        serialize_f32(f32) -> Token;
        serialize_f64(f64) -> Token;
        serialize_char(char) -> Token;
        serialize_none() -> Token;
        serialize_unit() -> Token;
        serialize_unit_struct(&'static str) -> Token;
        serialize_unit_variant(&'static str, u32, &'static str) -> Token;
        serialize_seq(Option<usize>) -> Self::SerializeSeq;
        serialize_tuple(usize) -> Self::SerializeTuple;
        serialize_tuple_struct(&'static str, usize) -> Self::SerializeTupleStruct;
        serialize_tuple_variant(&'static str, u32, &'static str, usize)
            -> Self::SerializeTupleVariant;
        serialize_map(Option<usize>) -> Self::SerializeMap;
        serialize_struct(&'static str, usize) -> Self::SerializeStruct;
        serialize_struct_variant(&'static str, u32, &'static str, usize)
            -> Self::SerializeStructVariant;
    }

    fn is_human_readable(&self) -> bool { self.human_readable }

    fn serialize_bytes(self, v: &[u8]) -> Result<Token, ValueError> { Ok(Token::Bytes(v.to_vec())) }

    fn serialize_str(self, v: &str) -> Result<Token, ValueError> { Ok(Token::Str(v.to_owned())) }

    fn serialize_u8(self, v: u8) -> Result<Token, ValueError> { Ok(Token::U8(v)) }

    fn serialize_u16(self, v: u16) -> Result<Token, ValueError> { Ok(Token::U16(v)) }

    fn serialize_u32(self, v: u32) -> Result<Token, ValueError> { Ok(Token::U32(v)) }

    fn serialize_u64(self, v: u64) -> Result<Token, ValueError> { Ok(Token::U64(v)) }

    fn serialize_some<T: ?Sized + Serialize>(self, _: &T) -> Result<Token, ValueError> {
        Err(unsupported())
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _: &'static str,
        _: &T,
    ) -> Result<Token, ValueError> {
        Err(unsupported())
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: &T,
    ) -> Result<Token, ValueError> {
        Err(unsupported())
    }
}

/// The value held by a [`Reader`].
#[derive(Copy, Clone)]
enum Input<'de> {
    Bytes(&'de [u8]),
    Str(&'de str),
    U64(u64),
    Unit,
}

/// Deserializer which holds a single value and gives it whatever is asked.
struct Reader<'de> {
    input: Input<'de>,
    human_readable: bool,
}

impl<'de> Deserializer<'de> for Reader<'de> {
    type Error = ValueError;

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string bytes byte_buf option
        unit unit_struct newtype_struct seq tuple tuple_struct map struct enum identifier
        ignored_any
    }

    fn is_human_readable(&self) -> bool { self.human_readable }

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, ValueError> {
        match self.input {
            Input::Bytes(bytes) => visitor.visit_borrowed_bytes(bytes),
            Input::Str(s) => visitor.visit_borrowed_str(s),
            Input::U64(v) => visitor.visit_u64(v),
            Input::Unit => visitor.visit_unit(),
        }
    }
}

/// JSON representation of a value.
fn to_json<T: ?Sized + Serialize>(value: &T) -> String {
    serde_json::to_string(value).expect("serializable value")
}

/// Value parsed from JSON, or the message of the error.
fn from_json<T: DeserializeOwned>(json: &str) -> Result<T, String> {
    serde_json::from_str(json).map_err(|err| err.to_string())
}

/// The value written by a value into a binary format.
fn to_binary<T: ?Sized + Serialize>(value: &T) -> Token {
    let recorder = Recorder {
        human_readable: false,
    };
    value.serialize(recorder).expect("serializable value")
}

/// Value read from a format of the given kind, or the message of the error.
fn read<'de, T: Deserialize<'de>>(input: Input<'de>, human_readable: bool) -> Result<T, String> {
    let reader = Reader {
        input,
        human_readable,
    };
    T::deserialize(reader).map_err(|err| err.to_string())
}

/// The harness itself is checked on the standard types.
mod harness {
    use super::*;

    #[test]
    fn recorder_records_the_written_value() {
        assert_eq!(to_binary(&7u8), Token::U8(7));
        assert_eq!(to_binary(&7u64), Token::U64(7));
        assert_eq!(to_binary("text"), Token::Str("text".to_owned()));
    }

    #[test]
    fn recorder_rejects_the_unsupported_types() {
        let recorder = Recorder {
            human_readable: false,
        };
        let err = true.serialize(recorder).expect_err("unsupported type");
        assert_eq!(err.to_string(), "unsupported by the test format");
    }

    #[test]
    fn reader_gives_the_held_value() {
        assert_eq!(read::<u64>(Input::U64(7), false), Ok(7));
        assert_eq!(read::<&str>(Input::Str("text"), true), Ok("text"));
        assert_eq!(read::<&[u8]>(Input::Bytes(&[1, 2]), false), Ok(&[1u8, 2][..]));
        assert_eq!(read::<()>(Input::Unit, false), Ok(()));
        assert!(read::<u8>(Input::Unit, false).is_err());
    }
}

// ---------------------------------------------------------------------------
// Big integers
// ---------------------------------------------------------------------------

/// Generates the test suites of the big integers of the given sizes.
macro_rules! big_suite {
    ($($module:ident: $limbs:literal limbs, $signed:literal;)+) => {$(
        mod $module {
            use crate::*;

            type T = BigInt<$limbs, $signed>;

            const BYTES: usize = 8 * $limbs;

            /// Bytes with the values counting from one.
            fn pattern_bytes() -> Vec<u8> { (1..=BYTES).map(|byte| byte as u8).collect() }

            /// The integer made of the big-endian [`pattern_bytes`].
            fn pattern() -> T { T::from_be_slice(&pattern_bytes()).expect("slice of the right length") }

            /// Hex digits of [`pattern_bytes`].
            fn pattern_hex() -> String {
                pattern_bytes().iter().map(|byte| format!("{byte:02x}")).collect()
            }

            fn values() -> [T; 6] { [T::ZERO, T::ONE, T::MIN, T::MAX, !T::ZERO, pattern()] }

            fn quoted(hex: &str) -> String { format!("\"{hex}\"") }

            /// What the errors tell about the accepted data.
            fn expectation() -> String {
                format!("{BYTES} bytes or a hex string of {} characters", 2 * BYTES)
            }

            // -- Human-readable formats --------------------------------------

            #[test]
            fn zero_is_written_as_a_string_of_zeros() {
                assert_eq!(to_json(&T::ZERO), quoted(&"0".repeat(2 * BYTES)));
            }

            #[test]
            fn one_is_written_with_the_leading_zeros() {
                assert_eq!(to_json(&T::ONE), quoted(&format!("{}1", "0".repeat(2 * BYTES - 1))));
            }

            #[test]
            fn all_ones_are_written_in_lower_case() {
                assert_eq!(to_json(&!T::ZERO), quoted(&"f".repeat(2 * BYTES)));
            }

            #[test]
            fn bytes_are_written_in_the_big_endian_order() {
                assert_eq!(to_json(&pattern()), quoted(&pattern_hex()));
            }

            #[test]
            fn top_bit_is_written_first() {
                let top = T::ONE << (T::BITS - 1);
                assert_eq!(to_json(&top), quoted(&format!("8{}", "0".repeat(2 * BYTES - 1))));
            }

            #[test]
            fn string_has_a_fixed_length() {
                for value in values() {
                    assert_eq!(to_json(&value).len(), 2 * BYTES + 2, "{value}");
                }
            }

            #[test]
            fn written_string_is_read_back() {
                for value in values() {
                    assert_eq!(from_json(&to_json(&value)), Ok(value), "{value}");
                }
            }

            #[test]
            fn string_is_read_in_the_big_endian_order() {
                assert_eq!(from_json(&quoted(&pattern_hex())), Ok(pattern()));
            }

            #[test]
            fn upper_case_is_read() {
                assert_eq!(from_json(&quoted(&"F".repeat(2 * BYTES))), Ok(!T::ZERO));
                assert_eq!(from_json(&quoted(&pattern_hex().to_uppercase())), Ok(pattern()));
            }

            #[test]
            fn human_readable_serializer_gets_a_string() {
                let recorder = Recorder { human_readable: true };
                assert_eq!(pattern().serialize(recorder), Ok(Token::Str(pattern_hex())));
            }

            #[test]
            fn strings_of_a_wrong_length_are_rejected() {
                let hex = pattern_hex();
                let (shorter, longer) = (&hex[2..], format!("00{hex}"));
                for (src, len) in [
                    ("", 0),
                    ("00", 2),
                    (shorter, 2 * BYTES - 2),
                    (longer.as_str(), 2 * BYTES + 2),
                ] {
                    let err = from_json::<T>(&quoted(src)).expect_err("string of a wrong length");
                    let expected = format!("invalid length {len}, expected {}", expectation());
                    assert!(err.starts_with(&expected), "{err}");
                }
            }

            #[test]
            fn strings_of_an_odd_length_are_rejected() {
                let hex = pattern_hex();
                let longer = format!("0{hex}");
                for src in [&hex[1..], longer.as_str()] {
                    let err = from_json::<T>(&quoted(src)).expect_err("string of an odd length");
                    let expected = format!("invalid length {}, expected {}", src.len(), expectation());
                    assert!(err.starts_with(&expected), "{err}");
                }
            }

            #[test]
            fn invalid_characters_are_rejected() {
                let hex = pattern_hex();
                let invalid = [
                    format!("g{}", &hex[1..]),
                    format!("{}g", &hex[1..]),
                    format!("0x{}", &hex[2..]),
                    format!("+{}", &hex[1..]),
                    format!("-{}", &hex[1..]),
                    format!(" {}", &hex[1..]),
                    format!("{} ", &hex[1..]),
                    // A character of two bytes keeps the length in bytes.
                    format!("é{}", &hex[2..]),
                ];
                for src in invalid {
                    let err = from_json::<T>(&quoted(&src)).expect_err("invalid character");
                    let expected = format!("invalid value: string {src:?}, expected {}", expectation());
                    assert!(err.starts_with(&expected), "{err}");
                }
            }

            #[test]
            fn other_json_types_are_rejected() {
                for src in ["12", "-1", "1.5", "null", "true", "[]", "[1, 2]", "{}"] {
                    let err = from_json::<T>(src).expect_err("value of a wrong type");
                    assert!(err.starts_with("invalid type: "), "{err}");
                    assert!(err.contains(&expectation()), "{err}");
                }
            }

            // -- Binary formats ----------------------------------------------

            #[test]
            fn binary_serializer_gets_the_big_endian_bytes() {
                assert_eq!(to_binary(&pattern()), Token::Bytes(pattern_bytes()));
                assert_eq!(to_binary(&T::ZERO), Token::Bytes(vec![0; BYTES]));
                assert_eq!(to_binary(&!T::ZERO), Token::Bytes(vec![0xFF; BYTES]));
            }

            #[test]
            fn one_is_written_into_the_last_byte() {
                let mut bytes = vec![0; BYTES];
                bytes[BYTES - 1] = 1;
                assert_eq!(to_binary(&T::ONE), Token::Bytes(bytes));
            }

            #[test]
            fn written_bytes_are_read_back() {
                for value in values() {
                    let Token::Bytes(bytes) = to_binary(&value) else {
                        panic!("{value} is not written as bytes");
                    };
                    assert_eq!(bytes.len(), BYTES);
                    assert_eq!(read(Input::Bytes(&bytes), false), Ok(value), "{value}");
                }
            }

            #[test]
            fn bytes_are_read_in_the_big_endian_order() {
                assert_eq!(read(Input::Bytes(&pattern_bytes()), false), Ok(pattern()));
            }

            #[test]
            fn bytes_of_a_wrong_length_are_rejected() {
                let bytes = vec![0xA5; 2 * BYTES];
                for len in [0, 1, BYTES - 8, BYTES - 1, BYTES + 1, BYTES + 8, 2 * BYTES] {
                    let err = read::<T>(Input::Bytes(&bytes[..len]), false)
                        .expect_err("bytes of a wrong length");
                    assert_eq!(err, format!("invalid length {len}, expected {}", expectation()));
                }
            }

            #[test]
            fn other_types_are_rejected_by_the_binary_formats() {
                for input in [Input::U64(1), Input::Unit] {
                    let err = read::<T>(input, false).expect_err("value of a wrong type");
                    assert!(err.starts_with("invalid type: "), "{err}");
                    assert!(err.ends_with(&expectation()), "{err}");
                }
            }

            // -- Formats giving the other representation ---------------------

            /// A format may ignore the hint of the type and give the data in
            /// its own representation.
            #[test]
            fn string_is_read_from_a_binary_format() {
                assert_eq!(read(Input::Str(&pattern_hex()), false), Ok(pattern()));
            }

            #[test]
            fn bytes_are_read_from_a_human_readable_format() {
                assert_eq!(read(Input::Bytes(&pattern_bytes()), true), Ok(pattern()));
            }

            #[test]
            fn wrong_data_is_rejected_in_the_other_representation() {
                let err = read::<T>(Input::Str("00"), false).expect_err("string of a wrong length");
                assert_eq!(err, format!("invalid length 2, expected {}", expectation()));
                let err = read::<T>(Input::Bytes(&[0]), true).expect_err("bytes of a wrong length");
                assert_eq!(err, format!("invalid length 1, expected {}", expectation()));
            }
        }
    )+};
}

mod big {
    big_suite! {
        u64_sized: 1 limbs, false;
        i64_sized: 1 limbs, true;
        u192: 3 limbs, false;
        i192: 3 limbs, true;
        u256: 4 limbs, false;
        i256: 4 limbs, true;
        u512: 8 limbs, false;
        i512: 8 limbs, true;
        u1024: 16 limbs, false;
        i1024: 16 limbs, true;
    }
}

/// The representations of the negative numbers and of the numbers nested into
/// other data.
mod big_values {
    use amplify_num::{i256, u256};

    use super::*;

    #[test]
    fn negative_numbers_are_written_as_the_two_complement() {
        assert_eq!(to_json(&i256::from(-1)), format!("\"{}\"", "f".repeat(64)));
        assert_eq!(to_json(&i256::from(-2)), format!("\"{}e\"", "f".repeat(63)));
        assert_eq!(to_json(&i256::MIN), format!("\"8{}\"", "0".repeat(63)));
        assert_eq!(to_json(&i256::MAX), format!("\"7{}\"", "f".repeat(63)));
    }

    #[test]
    fn negative_numbers_are_read_back() {
        assert_eq!(from_json(&format!("\"{}\"", "f".repeat(64))), Ok(i256::from(-1)));
        assert_eq!(from_json(&format!("\"8{}\"", "0".repeat(63))), Ok(i256::MIN));
        assert_eq!(read(Input::Bytes(&[0xFF; 32]), false), Ok(i256::from(-1)));
    }

    /// The signed and unsigned integers of the same bits have the same
    /// representation.
    #[test]
    fn signedness_does_not_change_the_representation() {
        let unsigned = u256::MAX - 5u8;
        assert_eq!(to_json(&unsigned), to_json(&unsigned.cast_signed()));
        assert_eq!(to_binary(&unsigned), to_binary(&unsigned.cast_signed()));
    }

    #[test]
    fn decimal_strings_are_not_accepted() {
        let err = from_json::<u256>("\"12\"").expect_err("decimal string");
        assert!(err.starts_with("invalid length 2, "), "{err}");
        let err = from_json::<u256>(&format!("\"{}\"", u256::MAX)).expect_err("decimal string");
        assert!(err.starts_with("invalid length 78, "), "{err}");
    }

    #[test]
    fn numbers_are_serialized_inside_of_collections() {
        let values = vec![u256::ZERO, u256::ONE, u256::MAX];
        let json = to_json(&values);
        assert_eq!(json.len(), 3 * 66 + 4);
        assert_eq!(from_json(&json), Ok(values));
        assert_eq!(from_json(&to_json(&Some(u256::ONE))), Ok(Some(u256::ONE)));
        assert_eq!(from_json("null"), Ok(None::<u256>));
    }
}

// ---------------------------------------------------------------------------
// Small integers
// ---------------------------------------------------------------------------

/// Generates the test suites of the small integers. The arguments are the
/// same as of the macro defining the types in the library, followed by the
/// token which a serializer must get.
macro_rules! small_suite {
    ($($name:ident: $bits:literal bits in $inner:ident as $token:ident;)+) => {$(
        mod $name {
            use amplify_num::$name as T;

            use crate::*;

            /// The largest value of the type.
            const MAX: u64 = (1 << $bits) - 1;

            /// The largest value of the underlying type.
            fn inner_max() -> u64 {
                u64::try_from(u128::from($inner::MAX)).expect("underlying type fits 64 bits")
            }

            fn inner(value: u64) -> $inner {
                $inner::try_from(value).expect("value fits the underlying type")
            }

            #[test]
            fn serializer_gets_the_underlying_integer() {
                assert_eq!(to_binary(&T::MAX), Token::$token(inner(MAX)));
                assert_eq!(to_binary(&T::ZERO), Token::$token(0));
                assert_eq!(to_binary(&T::ONE), Token::$token(1));
            }

            #[test]
            fn kind_of_the_format_does_not_matter() {
                let recorder = Recorder { human_readable: true };
                assert_eq!(T::MAX.serialize(recorder), Ok(Token::$token(inner(MAX))));
            }

            #[test]
            fn number_is_written_into_json() {
                assert_eq!(to_json(&T::MAX), MAX.to_string());
                assert_eq!(to_json(&T::ZERO), "0");
            }

            #[test]
            fn bounds_are_read_back() {
                for value in [T::MIN, T::ONE, T::MAX] {
                    assert_eq!(from_json(&to_json(&value)), Ok(value));
                }
                assert_eq!(from_json(&MAX.to_string()), Ok(T::MAX));
                assert_eq!(read(Input::U64(MAX), false), Ok(T::MAX));
                assert_eq!(read(Input::U64(0), true), Ok(T::ZERO));
            }

            #[test]
            fn value_above_the_maximum_is_rejected() {
                let message =
                    format!("value {} overflows the maximum {MAX} of the integer type", MAX + 1);
                let err = from_json::<T>(&(MAX + 1).to_string()).expect_err("value out of range");
                assert!(err.starts_with(&message), "{err}");
                assert_eq!(read::<T>(Input::U64(MAX + 1), false), Err(message));
            }

            #[test]
            fn maximum_of_the_underlying_type_is_rejected() {
                let max = inner_max();
                let message = format!("value {max} overflows the maximum {MAX} of the integer type");
                assert_eq!(read::<T>(Input::U64(max), false), Err(message));
            }

            /// Such a value is rejected by the underlying type itself.
            #[test]
            fn value_above_the_underlying_type_is_rejected() {
                let src = (u128::from(inner_max()) + 1).to_string();
                let err = from_json::<T>(&src).expect_err("value out of range");
                assert!(err.starts_with("invalid "), "{err}");
                assert!(!err.contains("overflows the maximum"), "{err}");
            }

            #[test]
            fn other_json_types_are_rejected() {
                for src in ["-1", "0.5", "1e0", "\"1\"", "null", "true", "[1]"] {
                    assert!(from_json::<T>(src).is_err(), "{src}");
                }
            }

            #[test]
            fn other_types_are_rejected_by_the_binary_formats() {
                for input in [Input::Bytes(&[1]), Input::Str("1"), Input::Unit] {
                    let err = read::<T>(input, false).expect_err("value of a wrong type");
                    assert!(err.starts_with("invalid type: "), "{err}");
                }
            }
        }
    )+};
}

mod small {
    small_suite! {
        u1: 1 bits in u8 as U8;
        u2: 2 bits in u8 as U8;
        u3: 3 bits in u8 as U8;
        u4: 4 bits in u8 as U8;
        u5: 5 bits in u8 as U8;
        u6: 6 bits in u8 as U8;
        u7: 7 bits in u8 as U8;
        u10: 10 bits in u16 as U16;
        u12: 12 bits in u16 as U16;
        u14: 14 bits in u16 as U16;
        u20: 20 bits in u32 as U32;
        u24: 24 bits in u32 as U32;
        u40: 40 bits in u64 as U64;
        u48: 48 bits in u64 as U64;
        u56: 56 bits in u64 as U64;
    }
}

/// Exhaustively checks every `u8` value against the human-readable and binary
/// formats for `u1` through `u7`.
mod subbyte_exhaustive {
    macro_rules! subbyte_serde {
        ($($name:ident: $bits:literal;)+) => {$(
            #[test]
            fn $name() {
                use amplify_num::$name as T;

                use crate::*;

                const MAX: u8 = (1 << $bits) - 1;

                for raw in 0..=u8::MAX {
                    let json = raw.to_string();
                    let u64_val = u64::from(raw);
                    if raw <= MAX {
                        let value = T::with(raw);
                        assert_eq!(to_binary(&value), Token::U8(raw));
                        assert_eq!(
                            value.serialize(Recorder { human_readable: true }),
                            Ok(Token::U8(raw))
                        );
                        assert_eq!(to_json(&value), json);
                        assert_eq!(from_json::<T>(&json), Ok(value));
                        assert_eq!(read::<T>(Input::U64(u64_val), false), Ok(value));
                        assert_eq!(read::<T>(Input::U64(u64_val), true), Ok(value));
                    } else {
                        let message = format!(
                            "value {raw} overflows the maximum {MAX} of the integer type"
                        );
                        let err = from_json::<T>(&json).expect_err("value out of range");
                        assert!(err.starts_with(&message), "{err}");
                        assert_eq!(read::<T>(Input::U64(u64_val), false), Err(message.clone()));
                        assert_eq!(read::<T>(Input::U64(u64_val), true), Err(message));
                    }
                }
            }
        )+};
    }

    subbyte_serde! {
        u1: 1;
        u2: 2;
        u3: 3;
        u4: 4;
        u5: 5;
        u6: 6;
        u7: 7;
    }
}
