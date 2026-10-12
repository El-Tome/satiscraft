//! Any contract type decoded and encoded again by name, so that a platform can check its
//! own codec against the core's, byte for byte.

use serde::Serialize;
use serde::de::DeserializeOwned;

use super::{
    CodecError, Command, CommandResult, ContentFile, ContentFiles, DimensionId, Effect, ItemId,
    MachineId, PlayerId, Position, Query, QueryResult, Rejection, TeamId, decode, encode,
};

/// Decodes `bytes` as the contract type named `type_name`, then encodes the value again.
///
/// Names are those of the golden files in `crates/factory-core/golden/`: `command`,
/// `command_batch` (the `Vec<Command>` that crosses JNI), `position`...
pub fn reencode(type_name: &str, bytes: &[u8]) -> Result<Vec<u8>, CodecError> {
    match type_name {
        "command" => again::<Command>(bytes),
        "command_batch" => again::<Vec<Command>>(bytes),
        "command_result" => again::<CommandResult>(bytes),
        "command_result_batch" => again::<Vec<CommandResult>>(bytes),
        "rejection" => again::<Rejection>(bytes),
        "effect" => again::<Effect>(bytes),
        "effect_batch" => again::<Vec<Effect>>(bytes),
        "query" => again::<Query>(bytes),
        "query_result" => again::<QueryResult>(bytes),
        "content_file" => again::<ContentFile>(bytes),
        "content_files" => again::<ContentFiles>(bytes),
        "item_id" => again::<ItemId>(bytes),
        "dimension_id" => again::<DimensionId>(bytes),
        "position" => again::<Position>(bytes),
        "player_id" => again::<PlayerId>(bytes),
        "team_id" => again::<TeamId>(bytes),
        "machine_id" => again::<MachineId>(bytes),
        _ => Err(CodecError::UnknownType(type_name.to_owned())),
    }
}

fn again<T: Serialize + DeserializeOwned>(bytes: &[u8]) -> Result<Vec<u8>, CodecError> {
    encode(&decode::<T>(bytes)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_known_type_comes_back_identical() {
        let bytes = encode(&Command::Ping { nonce: 300 }).unwrap();
        assert_eq!(reencode("command", &bytes), Ok(bytes));
    }

    #[test]
    fn bytes_of_another_type_are_refused() {
        assert!(matches!(
            reencode("rejection", &[9]),
            Err(CodecError::Decode(_))
        ));
        assert_eq!(
            reencode("query", &[0, 0]),
            Err(CodecError::TrailingBytes(1))
        );
    }

    #[test]
    fn an_unknown_type_is_refused() {
        assert_eq!(
            reencode("Command", &[0, 0]),
            Err(CodecError::UnknownType("Command".into()))
        );
    }
}
