//! Asks serde for the variants of an enum, so that a test can check the golden samples
//! cover every one of them.

use std::fmt;

use serde::de::{self, DeserializeOwned, Deserializer, Visitor};

/// The variant names of the enum `T`, in declaration order: a variant's position in this
/// list is its postcard index.
pub(super) fn variants<T: DeserializeOwned>() -> &'static [&'static str] {
    match T::deserialize(Probe) {
        Err(Found::Variants(variants)) => variants,
        _ => panic!("{} is not an enum", std::any::type_name::<T>()),
    }
}

/// A deserializer that answers nothing, but records the variants an enum asks for.
struct Probe;

/// What the probe learned, carried out as the deserialization error.
#[derive(Debug)]
enum Found {
    Variants(&'static [&'static str]),
    NotAnEnum,
}

impl fmt::Display for Found {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for Found {}

impl de::Error for Found {
    fn custom<T: fmt::Display>(_msg: T) -> Self {
        Self::NotAnEnum
    }
}

impl<'de> Deserializer<'de> for Probe {
    type Error = Found;

    fn deserialize_any<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value, Found> {
        Err(Found::NotAnEnum)
    }

    fn deserialize_enum<V: Visitor<'de>>(
        self,
        _name: &'static str,
        variants: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Found> {
        Err(Found::Variants(variants))
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string bytes byte_buf
        option unit unit_struct newtype_struct seq tuple tuple_struct map struct identifier
        ignored_any
    }
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    use super::*;

    #[derive(Deserialize)]
    #[allow(dead_code, reason = "only its shape matters")]
    enum Shape {
        Unit,
        Newtype(u8),
        Struct { field: u8 },
    }

    #[derive(Deserialize)]
    #[allow(dead_code, reason = "only its shape matters")]
    struct NotAnEnum(u8);

    #[test]
    fn variants_lists_an_enum_in_declaration_order() {
        assert_eq!(variants::<Shape>(), ["Unit", "Newtype", "Struct"]);
    }

    #[test]
    #[should_panic(expected = "is not an enum")]
    fn variants_refuses_a_struct() {
        variants::<NotAnEnum>();
    }

    #[test]
    fn the_probe_error_describes_itself() {
        assert_eq!(<Found as de::Error>::custom("x").to_string(), "NotAnEnum");
    }
}
