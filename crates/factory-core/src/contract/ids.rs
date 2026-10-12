//! Identifiers and positions: how the contract names things without any Minecraft type.

use std::fmt;

use serde::{Deserialize, Serialize};

/// Why a string is not a namespaced identifier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdError {
    /// No `:` between the namespace and the path.
    MissingNamespace(String),
    /// The namespace or the path is empty.
    EmptyPart(String),
    /// A character outside `a-z 0-9 _ . -` (plus `/` in the path).
    InvalidCharacter { id: String, character: char },
}

impl fmt::Display for IdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingNamespace(id) => {
                write!(
                    f,
                    "identifier `{id}` has no namespace (expected `namespace:path`)"
                )
            }
            Self::EmptyPart(id) => write!(f, "identifier `{id}` has an empty namespace or path"),
            Self::InvalidCharacter { id, character } => {
                write!(
                    f,
                    "identifier `{id}` contains the invalid character {character:?}"
                )
            }
        }
    }
}

impl std::error::Error for IdError {}

/// Checks the Minecraft resource location rules: `namespace:path`, namespace in
/// `[a-z0-9_.-]`, path in `[a-z0-9_.-/]`, neither empty. The namespace is always explicit.
fn validate_namespaced(id: &str) -> Result<(), IdError> {
    let Some((namespace, path)) = id.split_once(':') else {
        return Err(IdError::MissingNamespace(id.to_owned()));
    };
    if namespace.is_empty() || path.is_empty() {
        return Err(IdError::EmptyPart(id.to_owned()));
    }
    let allowed = |c: char| c.is_ascii_lowercase() || c.is_ascii_digit() || "_.-".contains(c);
    let invalid = namespace
        .chars()
        .find(|&c| !allowed(c))
        .or_else(|| path.chars().find(|&c| !allowed(c) && c != '/'));
    match invalid {
        Some(character) => Err(IdError::InvalidCharacter {
            id: id.to_owned(),
            character,
        }),
        None => Ok(()),
    }
}

/// Declares a validated `namespace:path` identifier, encoded as its string.
macro_rules! namespaced_id {
    ($(#[$doc:meta])* $name:ident) => {
        $(#[$doc])*
        #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
        #[serde(try_from = "String", into = "String")]
        pub struct $name(String);

        impl $name {
            /// Validates `id` against the resource location rules.
            pub fn new(id: impl Into<String>) -> Result<Self, IdError> {
                let id = id.into();
                validate_namespaced(&id)?;
                Ok(Self(id))
            }

            /// The identifier as written, `namespace:path`.
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl TryFrom<String> for $name {
            type Error = IdError;

            fn try_from(id: String) -> Result<Self, IdError> {
                Self::new(id)
            }
        }

        impl From<$name> for String {
            fn from(id: $name) -> String {
                id.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }
    };
}

namespaced_id! {
    /// An item, vanilla or modded: `satiscraft:bauxite`, `minecraft:stick`.
    ItemId
}

namespaced_id! {
    /// A dimension: `minecraft:overworld`, `minecraft:the_nether`.
    DimensionId
}

/// A block position: a dimension and integer coordinates.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Position {
    pub dimension: DimensionId,
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

/// A player, by UUID. Encoded as its 16 bytes in big-endian (RFC 4122) order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PlayerId([u8; 16]);

impl PlayerId {
    /// The UUID whose bytes, most significant first, are `bytes`.
    pub const fn from_bytes(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }

    /// The UUID bytes, most significant first.
    pub const fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }
}

impl fmt::Display for PlayerId {
    /// Hyphenated lowercase form, as Minecraft logs it.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, byte) in self.0.iter().enumerate() {
            if matches!(i, 4 | 6 | 8 | 10) {
                f.write_str("-")?;
            }
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}

/// A team, numbered by the core.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TeamId(pub u64);

/// A machine, numbered by the core: the only handle the platform keeps on simulated state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct MachineId(pub u64);

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;
    use crate::contract::arbitrary::{
        dimension_id, item_id, machine_id, player_id, position, team_id,
    };
    use crate::contract::{decode, encode};

    fn round_trip<T>(value: &T) -> T
    where
        T: Serialize + serde::de::DeserializeOwned,
    {
        decode(&encode(value).unwrap()).unwrap()
    }

    #[test]
    fn validate_accepts_resource_locations() {
        for id in [
            "satiscraft:bauxite",
            "minecraft:overworld",
            "my_mod-2.0:block/stone_axe.head",
            "a:b",
        ] {
            assert_eq!(validate_namespaced(id), Ok(()), "{id}");
        }
    }

    #[test]
    fn validate_requires_a_namespace() {
        assert_eq!(
            validate_namespaced("bauxite"),
            Err(IdError::MissingNamespace("bauxite".into()))
        );
    }

    #[test]
    fn validate_rejects_empty_parts() {
        for id in [":bauxite", "satiscraft:", ":"] {
            assert_eq!(validate_namespaced(id), Err(IdError::EmptyPart(id.into())));
        }
    }

    #[test]
    fn validate_rejects_invalid_characters() {
        for (id, character) in [
            ("Satiscraft:bauxite", 'S'),
            ("satiscraft:Bauxite", 'B'),
            ("satiscraft:raw bauxite", ' '),
            ("satis/craft:bauxite", '/'),
            ("satiscraft:ore:bauxite", ':'),
            ("satiscraft:bauxité", 'é'),
        ] {
            assert_eq!(
                validate_namespaced(id),
                Err(IdError::InvalidCharacter {
                    id: id.into(),
                    character
                }),
                "{id}"
            );
        }
    }

    #[test]
    fn id_error_messages_name_the_identifier() {
        assert_eq!(
            IdError::MissingNamespace("stick".into()).to_string(),
            "identifier `stick` has no namespace (expected `namespace:path`)"
        );
        assert_eq!(
            IdError::EmptyPart("a:".into()).to_string(),
            "identifier `a:` has an empty namespace or path"
        );
        assert_eq!(
            IdError::InvalidCharacter {
                id: "a:B".into(),
                character: 'B'
            }
            .to_string(),
            "identifier `a:B` contains the invalid character 'B'"
        );
    }

    #[test]
    fn item_id_keeps_the_text_it_was_built_from() {
        let id = ItemId::new("satiscraft:bauxite").unwrap();
        assert_eq!(id.as_str(), "satiscraft:bauxite");
        assert_eq!(id.to_string(), "satiscraft:bauxite");
        assert_eq!(String::from(id), "satiscraft:bauxite");
    }

    #[test]
    fn dimension_id_validates_like_item_id() {
        let id = DimensionId::try_from("minecraft:the_nether".to_owned()).unwrap();
        assert_eq!(id.as_str(), "minecraft:the_nether");
        assert_eq!(id.to_string(), "minecraft:the_nether");
        assert_eq!(String::from(id), "minecraft:the_nether");
        assert_eq!(
            DimensionId::new("overworld"),
            Err(IdError::MissingNamespace("overworld".into()))
        );
        assert!(ItemId::try_from("minecraft:Stick".to_owned()).is_err());
    }

    #[test]
    fn decoding_an_invalid_id_fails() {
        let bytes = encode("no namespace").unwrap();
        assert!(decode::<ItemId>(&bytes).is_err());
        assert!(decode::<DimensionId>(&bytes).is_err());
    }

    #[test]
    fn item_id_is_encoded_as_its_string() {
        let id = ItemId::new("a:b").unwrap();
        assert_eq!(encode(&id).unwrap(), [3, b'a', b':', b'b']);
    }

    #[test]
    fn position_is_encoded_field_by_field() {
        let position = Position {
            dimension: DimensionId::new("a:b").unwrap(),
            x: 1,
            y: -1,
            z: 64,
        };
        // String with its length, then each coordinate as a zigzag varint.
        assert_eq!(
            encode(&position).unwrap(),
            [3, b'a', b':', b'b', 2, 1, 128, 1]
        );
    }

    #[test]
    fn player_id_round_trips_its_bytes() {
        let bytes = [7; 16];
        assert_eq!(PlayerId::from_bytes(bytes).as_bytes(), &bytes);
    }

    #[test]
    fn player_id_displays_as_a_hyphenated_uuid() {
        let id = PlayerId::from_bytes([
            0x06, 0x9a, 0x79, 0xf4, 0x44, 0xe9, 0x40, 0x26, 0xbf, 0x4b, 0x8f, 0x2c, 0x41, 0x37,
            0x00, 0x0b,
        ]);
        assert_eq!(id.to_string(), "069a79f4-44e9-4026-bf4b-8f2c4137000b");
    }

    #[test]
    fn player_id_is_encoded_as_sixteen_raw_bytes() {
        let bytes: [u8; 16] = std::array::from_fn(|i| 0xf0 + i as u8);
        assert_eq!(encode(&PlayerId::from_bytes(bytes)).unwrap(), bytes);
    }

    #[test]
    fn numeric_ids_are_encoded_as_varints() {
        assert_eq!(encode(&TeamId(300)).unwrap(), [0xac, 0x02]);
        assert_eq!(encode(&MachineId(5)).unwrap(), [5]);
    }

    proptest! {
        #[test]
        fn generated_ids_are_valid(id in item_id(), dimension in dimension_id()) {
            prop_assert!(validate_namespaced(id.as_str()).is_ok());
            prop_assert!(validate_namespaced(dimension.as_str()).is_ok());
        }

        #[test]
        fn ids_and_positions_round_trip(
            item in item_id(),
            position in position(),
            player in player_id(),
            team in team_id(),
            machine in machine_id(),
        ) {
            prop_assert_eq!(round_trip(&item), item);
            prop_assert_eq!(round_trip(&position), position);
            prop_assert_eq!(round_trip(&player), player);
            prop_assert_eq!(round_trip(&team), team);
            prop_assert_eq!(round_trip(&machine), machine);
        }
    }
}
