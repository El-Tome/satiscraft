//! Wire format of the contract: postcard, the same bytes on both sides of JNI.

use std::fmt;

use serde::Serialize;
use serde::de::DeserializeOwned;

/// A value that could not be turned into bytes, or bytes that are not a value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CodecError {
    /// The value uses a serde feature postcard does not support.
    Encode(String),
    /// The bytes are not an encoding of the expected type.
    Decode(String),
    /// The value decoded, but bytes were left after it.
    TrailingBytes(usize),
}

impl fmt::Display for CodecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Encode(reason) => write!(f, "cannot encode: {reason}"),
            Self::Decode(reason) => write!(f, "cannot decode: {reason}"),
            Self::TrailingBytes(count) => {
                write!(f, "cannot decode: {count} bytes left after the value")
            }
        }
    }
}

impl std::error::Error for CodecError {}

/// Encodes a contract value, or a batch of them, in postcard.
pub fn encode<T: Serialize + ?Sized>(value: &T) -> Result<Vec<u8>, CodecError> {
    postcard::to_allocvec(value).map_err(|err| CodecError::Encode(err.to_string()))
}

/// Decodes exactly one value from `bytes`: leftover bytes mean the two sides disagree on
/// the format, so they are an error rather than ignored.
pub fn decode<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, CodecError> {
    let (value, rest) =
        postcard::take_from_bytes(bytes).map_err(|err| CodecError::Decode(err.to_string()))?;
    if rest.is_empty() {
        Ok(value)
    } else {
        Err(CodecError::TrailingBytes(rest.len()))
    }
}

#[cfg(test)]
mod tests {
    use serde::Serializer;
    use serde::ser::SerializeSeq;

    use super::*;

    /// A sequence that does not announce its length, which postcard cannot encode.
    struct UnknownLength;

    impl Serialize for UnknownLength {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            serializer.serialize_seq(None)?.end()
        }
    }

    #[test]
    fn encode_then_decode_gives_the_value_back() {
        let value = (42_u64, String::from("satiscraft:bauxite"), -3_i32);
        assert_eq!(
            decode::<(u64, String, i32)>(&encode(&value).unwrap()),
            Ok(value)
        );
    }

    #[test]
    fn encode_reports_unsupported_values() {
        assert!(matches!(encode(&UnknownLength), Err(CodecError::Encode(_))));
    }

    #[test]
    fn decode_rejects_truncated_bytes() {
        let bytes = encode("satiscraft").unwrap();
        assert!(matches!(
            decode::<String>(&bytes[..4]),
            Err(CodecError::Decode(_))
        ));
    }

    #[test]
    fn decode_rejects_leftover_bytes() {
        let mut bytes = encode(&7_u32).unwrap();
        bytes.extend([1, 2]);
        assert_eq!(decode::<u32>(&bytes), Err(CodecError::TrailingBytes(2)));
    }

    #[test]
    fn error_messages_say_what_failed() {
        assert_eq!(
            CodecError::Encode("x".into()).to_string(),
            "cannot encode: x"
        );
        assert_eq!(
            CodecError::Decode("y".into()).to_string(),
            "cannot decode: y"
        );
        assert_eq!(
            CodecError::TrailingBytes(3).to_string(),
            "cannot decode: 3 bytes left after the value"
        );
    }
}
