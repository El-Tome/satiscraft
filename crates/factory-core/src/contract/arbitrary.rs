//! `proptest` strategies generating every contract type, shared by the crate's tests.

use proptest::prelude::*;

use super::{
    Command, CommandResult, ContentFile, ContentFiles, DimensionId, Effect, ItemId, MachineId,
    PlayerId, Position, Query, QueryResult, Rejection, TeamId,
};

/// Valid `namespace:path` text.
fn namespaced() -> impl Strategy<Value = String> {
    ("[a-z0-9_.-]{1,12}", "[a-z0-9_./-]{1,24}").prop_map(|(ns, path)| format!("{ns}:{path}"))
}

pub(crate) fn item_id() -> impl Strategy<Value = ItemId> {
    namespaced().prop_map(|id| ItemId::new(id).unwrap())
}

pub(crate) fn dimension_id() -> impl Strategy<Value = DimensionId> {
    namespaced().prop_map(|id| DimensionId::new(id).unwrap())
}

pub(crate) fn position() -> impl Strategy<Value = Position> {
    (dimension_id(), any::<i32>(), any::<i32>(), any::<i32>())
        .prop_map(|(dimension, x, y, z)| Position { dimension, x, y, z })
}

pub(crate) fn player_id() -> impl Strategy<Value = PlayerId> {
    any::<[u8; 16]>().prop_map(PlayerId::from_bytes)
}

pub(crate) fn team_id() -> impl Strategy<Value = TeamId> {
    any::<u64>().prop_map(TeamId)
}

pub(crate) fn machine_id() -> impl Strategy<Value = MachineId> {
    any::<u64>().prop_map(MachineId)
}

pub(crate) fn command() -> impl Strategy<Value = Command> {
    prop_oneof![
        any::<u64>().prop_map(|nonce| Command::Ping { nonce }),
        Just(Command::PanicOnNextStep),
    ]
}

fn rejection() -> impl Strategy<Value = Rejection> {
    prop_oneof![
        Just(Rejection::NotAllowed),
        Just(Rejection::NotUnlocked),
        Just(Rejection::NoSpace),
        Just(Rejection::MissingItems),
        Just(Rejection::Faulted),
    ]
}

pub(crate) fn command_result() -> impl Strategy<Value = CommandResult> {
    prop_oneof![
        Just(CommandResult::Accepted),
        rejection().prop_map(CommandResult::Rejected),
    ]
}

pub(crate) fn effect() -> impl Strategy<Value = Effect> {
    any::<u64>().prop_map(|nonce| Effect::Pong { nonce })
}

pub(crate) fn query() -> impl Strategy<Value = Query> {
    Just(Query::Tick)
}

pub(crate) fn query_result() -> impl Strategy<Value = QueryResult> {
    any::<u64>().prop_map(QueryResult::Tick)
}

pub(crate) fn content_files() -> impl Strategy<Value = ContentFiles> {
    prop::collection::vec(
        (".{0,24}", ".{0,64}").prop_map(|(path, text)| ContentFile { path, text }),
        0..4,
    )
    .prop_map(|files| ContentFiles { files })
}
