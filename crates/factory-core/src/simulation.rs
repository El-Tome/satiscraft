//! The [`Simulation`] trait every platform drives, and [`Core`], the simulation it drives.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::content::{Content, ContentErrors};
use crate::contract::{
    CodecError, Command, CommandResult, ContentFiles, Effect, Query, QueryResult, decode, encode,
};

/// What a platform sees of the core: it submits commands, steps once per tick, applies the
/// effects, and queries for display.
pub trait Simulation {
    /// Applies the commands received since the last call, in order. Returns one result
    /// per command, at the same index.
    fn submit(&mut self, commands: &[Command]) -> Vec<CommandResult>;

    /// Advances one tick and returns what changed, including the consequences of the
    /// commands submitted since the previous step.
    fn step(&mut self) -> Vec<Effect>;

    /// Read-only, for GUIs and rendering.
    fn query(&self, query: &Query) -> QueryResult;

    /// The whole simulated state, as bytes that [`Simulation::load`] reads back.
    fn save(&self) -> Vec<u8>;

    /// Rebuilds a simulation from the output of [`Simulation::save`], with the files of the
    /// content it runs on: the save holds the state, never the content.
    fn load(content: ContentFiles, bytes: &[u8]) -> Result<Self, LoadError>
    where
        Self: Sized;
}

/// Why [`Simulation::load`] cannot rebuild a simulation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadError {
    /// The bytes do not decode to a saved state.
    Malformed(CodecError),
    /// The content to run on is invalid.
    Content(ContentErrors),
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Malformed(err) => write!(f, "malformed save: {err}"),
            Self::Content(err) => err.fmt(f),
        }
    }
}

impl std::error::Error for LoadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Malformed(err) => Some(err),
            Self::Content(err) => Some(err),
        }
    }
}

/// The factory simulation. For now it only keeps time and answers pings; each gameplay
/// system adds its state here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Core {
    state: State,
    /// What the simulation runs on.
    content: Content,
}

/// Everything [`Simulation::save`] writes. The versioned save format (`FCTR` header, migrations)
/// wraps it in its own issue.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct State {
    /// Ticks stepped since creation.
    tick: u64,
    /// Nonces of the pings accepted since the last step, in order.
    pending_pongs: Vec<u64>,
    /// Set by [`Command::PanicOnNextStep`].
    panic_on_next_step: bool,
}

impl Core {
    /// An empty world at tick 0, running on `content`.
    pub fn new(content: Content) -> Self {
        Self {
            state: State::default(),
            content,
        }
    }

    /// What this simulation runs on.
    pub fn content(&self) -> &Content {
        &self.content
    }
}

impl Simulation for Core {
    fn submit(&mut self, commands: &[Command]) -> Vec<CommandResult> {
        commands
            .iter()
            .map(|command| match command {
                Command::Ping { nonce } => {
                    self.state.pending_pongs.push(*nonce);
                    CommandResult::Accepted
                }
                Command::PanicOnNextStep => {
                    self.state.panic_on_next_step = true;
                    CommandResult::Accepted
                }
            })
            .collect()
    }

    fn step(&mut self) -> Vec<Effect> {
        self.state.tick += 1;
        if self.state.panic_on_next_step {
            panic!(
                "panic requested by Command::PanicOnNextStep, at tick {}",
                self.state.tick
            );
        }
        self.state
            .pending_pongs
            .drain(..)
            .map(|nonce| Effect::Pong { nonce })
            .collect()
    }

    fn query(&self, query: &Query) -> QueryResult {
        match query {
            Query::Tick => QueryResult::Tick(self.state.tick),
        }
    }

    fn save(&self) -> Vec<u8> {
        // Only fields with a known length, which postcard always encodes.
        encode(&self.state).expect("the core state is always encodable")
    }

    fn load(content: ContentFiles, bytes: &[u8]) -> Result<Self, LoadError> {
        let content = Content::load(&content).map_err(LoadError::Content)?;
        let state = decode(bytes).map_err(LoadError::Malformed)?;
        Ok(Self { state, content })
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use proptest::prelude::*;

    use super::*;
    use crate::content::testing::{shipped, shipped_files, with_overrides};

    /// A simulation on the shipped content, as every test starts.
    fn new_core() -> Core {
        Core::new(shipped())
    }

    fn pings(nonces: &[u64]) -> Vec<Command> {
        nonces
            .iter()
            .map(|&nonce| Command::Ping { nonce })
            .collect()
    }

    fn pongs(nonces: &[u64]) -> Vec<Effect> {
        nonces.iter().map(|&nonce| Effect::Pong { nonce }).collect()
    }

    #[test]
    fn new_core_starts_at_tick_zero_with_nothing_pending() {
        let mut core = new_core();
        assert_eq!(core.query(&Query::Tick), QueryResult::Tick(0));
        assert_eq!(core.step(), []);
    }

    #[test]
    fn submit_answers_each_command_in_order() {
        let mut core = new_core();
        assert_eq!(core.submit(&[]), []);
        assert_eq!(
            core.submit(&pings(&[1, 2, 3])),
            vec![CommandResult::Accepted; 3]
        );
    }

    #[test]
    fn ping_is_answered_by_the_next_step_only() {
        let mut core = new_core();
        core.submit(&pings(&[5, 6]));
        core.submit(&pings(&[7]));
        assert_eq!(core.step(), pongs(&[5, 6, 7]));
        assert_eq!(core.step(), []);
    }

    #[test]
    fn step_advances_the_tick() {
        let mut core = new_core();
        for _ in 0..3 {
            core.step();
        }
        assert_eq!(core.query(&Query::Tick), QueryResult::Tick(3));
    }

    #[test]
    fn save_then_load_restores_the_state() {
        let mut core = new_core();
        core.step();
        core.submit(&pings(&[9]));
        let mut loaded = Core::load(shipped_files(), &core.save()).unwrap();
        assert_eq!(loaded, core);
        assert_eq!(loaded.step(), pongs(&[9]));
        assert_eq!(loaded.query(&Query::Tick), QueryResult::Tick(2));
    }

    #[test]
    fn load_runs_on_the_given_content_not_the_saved_one() {
        let mut core = new_core();
        assert_eq!(core.content(), &shipped());
        core.step();
        let threads = with_overrides(&[("config.toml", "[sim]\nthreads = 3\n")]);
        let loaded = Core::load(threads, &core.save()).unwrap();
        assert_eq!(loaded.content().config().sim.threads, 3);
        assert_eq!(loaded.query(&Query::Tick), QueryResult::Tick(1));
    }

    #[test]
    #[should_panic(expected = "panic requested by Command::PanicOnNextStep, at tick 1")]
    fn panic_on_next_step_is_accepted_then_the_next_step_panics() {
        let mut core = new_core();
        assert_eq!(
            core.submit(&[Command::PanicOnNextStep]),
            [CommandResult::Accepted]
        );
        let mut loaded = Core::load(shipped_files(), &core.save()).unwrap();
        loaded.step();
    }

    #[test]
    fn load_rejects_malformed_bytes() {
        assert!(matches!(
            Core::load(shipped_files(), &[0xff]),
            Err(LoadError::Malformed(CodecError::Decode(_)))
        ));
        let mut bytes = new_core().save();
        bytes.push(0);
        assert_eq!(
            Core::load(shipped_files(), &bytes),
            Err(LoadError::Malformed(CodecError::TrailingBytes(1)))
        );
    }

    #[test]
    fn load_rejects_invalid_content_before_reading_the_save() {
        let files = with_overrides(&[("config.toml", "[sim]\nthread = 4\n")]);
        let Err(LoadError::Content(errors)) = Core::load(files, &[0xff]) else {
            panic!("a content error");
        };
        assert_eq!(errors.errors()[0].path, "config/satiscraft/config.toml");
    }

    #[test]
    fn load_error_explains_and_keeps_its_cause() {
        let err = LoadError::Malformed(CodecError::TrailingBytes(1));
        assert_eq!(
            err.to_string(),
            "malformed save: cannot decode: 1 bytes left after the value"
        );
        assert_eq!(
            err.source().map(ToString::to_string),
            Some(CodecError::TrailingBytes(1).to_string())
        );
        let files = with_overrides(&[("quests.toml", "")]);
        let Err(err) = Core::load(files, &[]) else {
            panic!("a content error");
        };
        assert!(
            err.to_string().starts_with("invalid content (1 error):"),
            "{err}"
        );
        assert_eq!(err.source().map(ToString::to_string), Some(err.to_string()));
    }

    /// One platform action, for scenario properties.
    #[derive(Debug, Clone)]
    enum Action {
        Submit(Vec<u64>),
        Step,
        SaveAndLoad,
    }

    fn action() -> impl Strategy<Value = Action> {
        prop_oneof![
            prop::collection::vec(any::<u64>(), 0..4).prop_map(Action::Submit),
            Just(Action::Step),
            Just(Action::SaveAndLoad),
        ]
    }

    proptest! {
        /// Every ping comes back exactly once, in order, and saving never changes the
        /// outcome of a run.
        #[test]
        fn save_and_load_are_invisible_to_the_platform(
            actions in prop::collection::vec(action(), 0..32),
        ) {
            let mut reference = new_core();
            let mut reloaded = new_core();
            let (mut sent, mut received) = (Vec::new(), Vec::new());
            for action in actions {
                match action {
                    Action::Submit(nonces) => {
                        let commands = pings(&nonces);
                        prop_assert_eq!(reference.submit(&commands), reloaded.submit(&commands));
                        sent.extend(nonces);
                    }
                    Action::Step => {
                        let effects = reference.step();
                        prop_assert_eq!(&effects, &reloaded.step());
                        received.extend(effects.into_iter().map(|Effect::Pong { nonce }| nonce));
                    }
                    Action::SaveAndLoad => reloaded = Core::load(shipped_files(), &reloaded.save()).unwrap(),
                }
            }
            prop_assert_eq!(&reference, &reloaded);
            received.extend(reference.step().into_iter().map(|Effect::Pong { nonce }| nonce));
            prop_assert_eq!(received, sent);
        }
    }
}
