//! What crosses the boundary: commands in, results and effects out, queries for display.
//!
//! Each enum starts with the variants the plumbing needs; gameplay issues append theirs.

use serde::{Deserialize, Serialize};

/// A request from the platform, validated and applied by [`Simulation::submit`].
///
/// [`Simulation::submit`]: crate::Simulation::submit
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Command {
    /// Diagnostic round trip: always accepted, answered by [`Effect::Pong`] with the same
    /// nonce on the next step. Lets a platform check the command → effect path end to end.
    Ping { nonce: u64 },
}

/// The core's answer to one command, at the same index as the command in the batch.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommandResult {
    /// Applied: its consequences come out as effects of the next step.
    Accepted,
    /// Refused, with nothing changed.
    Rejected(Rejection),
}

/// Why the core refused a command. The platform turns it into player-facing text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Rejection {
    /// The player lacks the right: block of another team, role too low.
    NotAllowed,
    /// The team has not unlocked the tier, blueprint or recipe yet.
    NotUnlocked,
    /// No room at the destination.
    NoSpace,
    /// The source does not hold the items.
    MissingItems,
}

/// A change the platform must apply to the world or show to players.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Effect {
    /// Answer to [`Command::Ping`].
    Pong { nonce: u64 },
}

/// A read-only question, for GUIs and rendering.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Query {
    /// How many ticks the simulation has stepped since it was created.
    Tick,
}

/// The answer to a [`Query`], in the variant of the same name.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum QueryResult {
    /// Answer to [`Query::Tick`].
    Tick(u64),
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;
    use crate::contract::arbitrary::{command, command_result, effect, query, query_result};
    use crate::contract::{decode, encode};

    #[test]
    fn every_variant_round_trips() {
        let commands = vec![Command::Ping { nonce: u64::MAX }];
        let results = vec![
            CommandResult::Accepted,
            CommandResult::Rejected(Rejection::NotAllowed),
            CommandResult::Rejected(Rejection::NotUnlocked),
            CommandResult::Rejected(Rejection::NoSpace),
            CommandResult::Rejected(Rejection::MissingItems),
        ];
        let effects = vec![Effect::Pong { nonce: 0 }];
        assert_eq!(
            decode::<Vec<Command>>(&encode(&commands).unwrap()),
            Ok(commands)
        );
        assert_eq!(
            decode::<Vec<CommandResult>>(&encode(&results).unwrap()),
            Ok(results)
        );
        assert_eq!(
            decode::<Vec<Effect>>(&encode(&effects).unwrap()),
            Ok(effects)
        );
        assert_eq!(
            decode::<Query>(&encode(&Query::Tick).unwrap()),
            Ok(Query::Tick)
        );
        assert_eq!(
            decode::<QueryResult>(&encode(&QueryResult::Tick(7)).unwrap()),
            Ok(QueryResult::Tick(7))
        );
    }

    #[test]
    fn variants_are_encoded_by_their_position() {
        // Kotlin mirrors these indices: append new variants, never reorder them.
        assert_eq!(encode(&Command::Ping { nonce: 1 }).unwrap(), [0, 1]);
        assert_eq!(encode(&CommandResult::Accepted).unwrap(), [0]);
        assert_eq!(
            encode(&CommandResult::Rejected(Rejection::MissingItems)).unwrap(),
            [1, 3]
        );
        assert_eq!(encode(&Effect::Pong { nonce: 2 }).unwrap(), [0, 2]);
        assert_eq!(encode(&Query::Tick).unwrap(), [0]);
        assert_eq!(encode(&QueryResult::Tick(3)).unwrap(), [0, 3]);
    }

    #[test]
    fn batches_are_prefixed_by_their_length() {
        let commands = [Command::Ping { nonce: 4 }, Command::Ping { nonce: 5 }];
        assert_eq!(encode(&commands[..]).unwrap(), [2, 0, 4, 0, 5]);
    }

    #[test]
    fn unknown_variant_fails_to_decode() {
        assert!(decode::<Command>(&[9, 0]).is_err());
        assert!(decode::<Rejection>(&[4]).is_err());
    }

    proptest! {
        #[test]
        fn generated_messages_round_trip(
            commands in prop::collection::vec(command(), 0..16),
            results in prop::collection::vec(command_result(), 0..16),
            effects in prop::collection::vec(effect(), 0..16),
            query in query(),
            answer in query_result(),
        ) {
            prop_assert_eq!(decode::<Vec<Command>>(&encode(&commands).unwrap()), Ok(commands));
            prop_assert_eq!(decode::<Vec<CommandResult>>(&encode(&results).unwrap()), Ok(results));
            prop_assert_eq!(decode::<Vec<Effect>>(&encode(&effects).unwrap()), Ok(effects));
            prop_assert_eq!(decode::<Query>(&encode(&query).unwrap()), Ok(query));
            prop_assert_eq!(decode::<QueryResult>(&encode(&answer).unwrap()), Ok(answer));
        }
    }
}
