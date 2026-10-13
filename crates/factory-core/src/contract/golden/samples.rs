//! The values written to the golden files. Kotlin's `ContractSamples` lists the same values
//! in the same order: change both together.
//!
//! Each enum has a sample per variant (checked by the golden tests), and the numbers and
//! strings sit on the encoding boundaries: one and two byte varints, negative zigzag
//! values, multi-byte UTF-8, lengths past 127.

use crate::contract::{
    Command, CommandResult, ContentFile, ContentFiles, DimensionId, Effect, ItemId, MachineId,
    PlayerId, Position, Query, QueryResult, Rejection, TeamId,
};

pub(super) fn commands() -> Vec<Command> {
    vec![
        Command::Ping { nonce: 0 },
        Command::Ping { nonce: 127 },
        Command::Ping { nonce: 128 },
        Command::Ping { nonce: u64::MAX },
        Command::PanicOnNextStep,
    ]
}

pub(super) fn rejections() -> Vec<Rejection> {
    vec![
        Rejection::NotAllowed,
        Rejection::NotUnlocked,
        Rejection::NoSpace,
        Rejection::MissingItems,
        Rejection::Faulted,
    ]
}

pub(super) fn command_results() -> Vec<CommandResult> {
    let mut results = vec![CommandResult::Accepted];
    results.extend(rejections().into_iter().map(CommandResult::Rejected));
    results
}

pub(super) fn effects() -> Vec<Effect> {
    vec![
        Effect::Pong { nonce: 0 },
        Effect::Pong { nonce: 300 },
        Effect::Pong { nonce: u64::MAX },
    ]
}

pub(super) fn queries() -> Vec<Query> {
    vec![Query::Tick]
}

pub(super) fn query_results() -> Vec<QueryResult> {
    vec![
        QueryResult::Tick(0),
        QueryResult::Tick(16_384),
        QueryResult::Tick(u64::MAX),
    ]
}

pub(super) fn content_file() -> Vec<ContentFile> {
    let file = |path: &str, text: &str| ContentFile {
        path: path.into(),
        text: text.into(),
    };
    vec![
        file("config.toml", ""),
        file(
            "content/items.toml",
            "[[items]]\nid = \"satiscraft:flint\"\nname = \"Silex taillé\"\n",
        ),
        file("é/🙂.toml", &"x".repeat(130)),
    ]
}

pub(super) fn content_files() -> Vec<ContentFiles> {
    vec![
        ContentFiles::default(),
        ContentFiles {
            files: content_file(),
        },
    ]
}

pub(super) fn item_ids() -> Vec<ItemId> {
    [
        "satiscraft:bauxite",
        "minecraft:stick",
        "my_mod-2.0:block/stone_axe.head",
    ]
    .into_iter()
    .map(|id| ItemId::new(id).unwrap())
    .collect()
}

pub(super) fn dimension_ids() -> Vec<DimensionId> {
    ["minecraft:overworld", "minecraft:the_nether"]
        .into_iter()
        .map(|id| DimensionId::new(id).unwrap())
        .collect()
}

pub(super) fn positions() -> Vec<Position> {
    let [overworld, nether] = <[DimensionId; 2]>::try_from(dimension_ids()).unwrap();
    vec![
        Position {
            dimension: overworld.clone(),
            x: 0,
            y: 0,
            z: 0,
        },
        Position {
            dimension: nether,
            x: -1,
            y: 64,
            z: 1,
        },
        Position {
            dimension: overworld,
            x: i32::MIN,
            y: -64,
            z: i32::MAX,
        },
    ]
}

pub(super) fn player_ids() -> Vec<PlayerId> {
    vec![
        PlayerId::from_bytes([0; 16]),
        PlayerId::from_bytes([
            0x06, 0x9a, 0x79, 0xf4, 0x44, 0xe9, 0x40, 0x26, 0xbf, 0x4b, 0x8f, 0x2c, 0x41, 0x37,
            0x00, 0x0b,
        ]),
        PlayerId::from_bytes([0xff; 16]),
    ]
}

pub(super) fn team_ids() -> Vec<TeamId> {
    vec![TeamId(0), TeamId(300), TeamId(u64::MAX)]
}

pub(super) fn machine_ids() -> Vec<MachineId> {
    vec![MachineId(1), MachineId(16_384), MachineId(u64::MAX)]
}
