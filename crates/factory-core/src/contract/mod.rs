//! The contract between the core and the platforms: commands in, effects out.
//!
//! No Minecraft, JNI or network type appears here. Positions are a dimension id and integer
//! coordinates, items are `namespace:path` ids, players are UUIDs, and machines and teams
//! are numbers handed out by the core.

#[cfg(test)]
pub(crate) mod arbitrary;
mod codec;
mod content;
#[cfg(test)]
mod golden;
mod ids;
mod messages;
mod reencode;

pub use codec::{CodecError, decode, encode};
pub use content::{ContentFile, ContentFiles};
pub use ids::{DimensionId, IdError, ItemId, MachineId, PlayerId, Position, TeamId};
pub use messages::{Command, CommandResult, Effect, Query, QueryResult, Rejection};
pub use reencode::reencode;

/// Version of the contract types: [`Command`], [`CommandResult`], [`Rejection`],
/// [`Effect`], [`Query`], [`QueryResult`], the ids and positions they carry, and the
/// [`ContentFiles`] a simulation is built from.
///
/// The encoding is positional (postcard): fields in declaration order, enum variants by
/// index. Kotlin mirrors it and ships in the same jar, so adding a field or appending a
/// variant keeps the version; removing, reordering or changing one bumps it, together with
/// Kotlin's `Contract.VERSION`. Every change is mirrored in Kotlin and checked against the
/// golden files in `crates/factory-core/golden/` (see the `golden` test module).
pub const CONTRACT_VERSION: u16 = 1;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contract_version_starts_at_one() {
        assert_eq!(CONTRACT_VERSION, 1);
    }
}
