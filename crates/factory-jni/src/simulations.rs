//! The core simulations of this process, driven with postcard bytes: one function per
//! native method of `NativeBridge`.
//!
//! Each call decodes its input first, so malformed bytes never reach a simulation, then runs
//! the core under the registry lock, and encodes the output. The server runs one simulation
//! for all dimensions; tests run several side by side.

use std::sync::{Mutex, MutexGuard, PoisonError};

use factory_core::contract::{Command, ContentFiles, Query, decode, encode};
use factory_core::{Content, Core, Simulation};

use crate::error::BridgeError;
use crate::registry::Registry;

static SIMULATIONS: Mutex<Registry<Core>> = Mutex::new(Registry::new());

/// Locks the registry. A panic in a simulation unwinds through the lock and poisons it, but
/// leaves the registry consistent, since the simulation is already marked faulted: the
/// poison is ignored.
fn simulations() -> MutexGuard<'static, Registry<Core>> {
    SIMULATIONS.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Creates a simulation running on `content`, once checked, and returns its id.
pub(crate) fn init(content: &[u8]) -> Result<i64, BridgeError> {
    let content: ContentFiles = decode(content).map_err(BridgeError::codec("content"))?;
    let core = Core::new(Content::load(&content)?);
    Ok(simulations().insert(core))
}

/// Rebuilds a simulation from `save`, running on `content`, and returns its id.
pub(crate) fn load(content: &[u8], save: &[u8]) -> Result<i64, BridgeError> {
    let content: ContentFiles = decode(content).map_err(BridgeError::codec("content"))?;
    let core = Core::load(content, save)?;
    Ok(simulations().insert(core))
}

/// Applies a batch of commands and returns one result per command. A panic faults the
/// simulation.
pub(crate) fn submit(id: i64, commands: &[u8]) -> Result<Vec<u8>, BridgeError> {
    let commands: Vec<Command> = decode(commands).map_err(BridgeError::codec("commands"))?;
    let results = simulations().modify(id, |core| core.submit(&commands))?;
    encode(&results).map_err(BridgeError::codec("command results"))
}

/// Steps the simulation once and returns its effects. A panic faults the simulation.
pub(crate) fn tick(id: i64) -> Result<Vec<u8>, BridgeError> {
    let effects = simulations().modify(id, Core::step)?;
    encode(&effects).map_err(BridgeError::codec("effects"))
}

/// Answers a read-only query. A panic leaves the simulation running.
pub(crate) fn query(id: i64, query: &[u8]) -> Result<Vec<u8>, BridgeError> {
    let query: Query = decode(query).map_err(BridgeError::codec("query"))?;
    let answer = simulations().read(id, |core| core.query(&query))?;
    encode(&answer).map_err(BridgeError::codec("query result"))
}

/// The whole state of a healthy simulation: a faulted one is never saved.
pub(crate) fn save(id: i64) -> Result<Vec<u8>, BridgeError> {
    Ok(simulations().read(id, Core::save)?)
}

/// Frees a simulation, faulted or not. Its id is never handed out again.
pub(crate) fn shutdown(id: i64) -> Result<(), BridgeError> {
    let core = simulations().remove(id)?;
    // Dropped outside the lock.
    drop(core);
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use std::fs;
    use std::panic::catch_unwind;
    use std::path::Path;

    use factory_core::contract::{CommandResult, ContentFile, Effect, QueryResult};

    use super::*;
    use crate::registry::RegistryError;

    /// The engine's content as the platform passes it: each `.toml` file of
    /// `crates/factory-core/content/` as `content/<name>`, then `overrides`.
    fn content_with(overrides: &[ContentFile]) -> Vec<u8> {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../factory-core/content");
        let mut names: Vec<String> = fs::read_dir(&dir)
            .unwrap()
            .filter_map(|entry| entry.unwrap().file_name().into_string().ok())
            .filter(|name| name.ends_with(".toml"))
            .collect();
        names.sort();
        let mut files: Vec<ContentFile> = names
            .into_iter()
            .map(|name| ContentFile {
                text: fs::read_to_string(dir.join(&name)).unwrap(),
                path: format!("content/{name}"),
            })
            .collect();
        files.extend_from_slice(overrides);
        encode(&ContentFiles { files }).unwrap()
    }

    fn no_content() -> Vec<u8> {
        content_with(&[])
    }

    fn batch(commands: &[Command]) -> Vec<u8> {
        encode(commands).unwrap()
    }

    fn current_tick(id: i64) -> u64 {
        let QueryResult::Tick(tick) =
            decode(&query(id, &encode(&Query::Tick).unwrap()).unwrap()).expect("a query result");
        tick
    }

    fn registry_error(result: Result<impl Debug, BridgeError>) -> RegistryError {
        match result {
            Err(BridgeError::Registry(err)) => err,
            other => panic!("expected a registry error, got {other:?}"),
        }
    }

    fn codec_error(result: Result<impl Debug, BridgeError>) -> &'static str {
        match result {
            Err(BridgeError::Codec { what, .. }) => what,
            other => panic!("expected a codec error, got {other:?}"),
        }
    }

    #[test]
    fn a_command_comes_back_as_an_effect_on_the_next_tick() {
        let id = init(&no_content()).unwrap();
        let results = submit(id, &batch(&[Command::Ping { nonce: 42 }])).unwrap();
        assert_eq!(decode(&results), Ok(vec![CommandResult::Accepted]));
        let effects = tick(id).unwrap();
        assert_eq!(decode(&effects), Ok(vec![Effect::Pong { nonce: 42 }]));
        assert_eq!(current_tick(id), 1);
        shutdown(id).unwrap();
    }

    #[test]
    fn load_resumes_a_saved_simulation_under_a_new_id() {
        let id = init(&no_content()).unwrap();
        tick(id).unwrap();
        submit(id, &batch(&[Command::Ping { nonce: 7 }])).unwrap();
        let loaded = load(&no_content(), &save(id).unwrap()).unwrap();
        assert!(loaded > id);
        assert_eq!(current_tick(loaded), 1);
        assert_eq!(
            decode(&tick(loaded).unwrap()),
            Ok(vec![Effect::Pong { nonce: 7 }])
        );
        shutdown(id).unwrap();
        shutdown(loaded).unwrap();
    }

    #[test]
    fn invalid_content_is_refused_with_its_file_and_line() {
        let typo = ContentFile {
            path: "config/satiscraft/config.toml".into(),
            text: "[sim]\nthread = 4\n".into(),
        };
        let expected = "invalid content (1 error):\n  config/satiscraft/config.toml:2: unknown \
                        field `thread`, expected `threads` or `tick_budget_ms`";
        for result in [
            init(&content_with(std::slice::from_ref(&typo))),
            load(&content_with(&[typo]), &[]),
        ] {
            assert_eq!(result.unwrap_err().to_string(), expected);
        }
    }

    #[test]
    fn a_shut_down_id_is_never_reused() {
        let id = init(&no_content()).unwrap();
        shutdown(id).unwrap();
        let next = init(&no_content()).unwrap();
        assert!(next > id);
        assert_eq!(registry_error(tick(id)), RegistryError::Unknown(id));
        assert_eq!(registry_error(shutdown(id)), RegistryError::Unknown(id));
        shutdown(next).unwrap();
    }

    #[test]
    fn every_call_refuses_an_unknown_id() {
        let unknown = RegistryError::Unknown(-1);
        assert_eq!(registry_error(submit(-1, &batch(&[]))), unknown);
        assert_eq!(registry_error(tick(-1)), unknown);
        let tick_query = encode(&Query::Tick).unwrap();
        assert_eq!(registry_error(query(-1, &tick_query)), unknown);
        assert_eq!(registry_error(save(-1)), unknown);
        assert_eq!(registry_error(shutdown(-1)), unknown);
    }

    #[test]
    fn malformed_inputs_are_refused_without_faulting() {
        assert_eq!(codec_error(init(&[])), "content");
        assert_eq!(codec_error(load(&[], &[])), "content");
        assert!(matches!(
            load(&no_content(), &[0xff]),
            Err(BridgeError::Load(_))
        ));
        let id = init(&no_content()).unwrap();
        assert_eq!(codec_error(submit(id, &[1])), "commands");
        assert_eq!(codec_error(query(id, &[9])), "query");
        assert_eq!(decode(&tick(id).unwrap()), Ok(Vec::<Effect>::new()));
        shutdown(id).unwrap();
    }

    #[test]
    fn a_panic_in_tick_faults_the_simulation_until_shut_down() {
        let id = init(&no_content()).unwrap();
        let other = init(&no_content()).unwrap();
        submit(id, &batch(&[Command::PanicOnNextStep])).unwrap();
        assert!(catch_unwind(|| tick(id)).is_err());
        let faulted = RegistryError::Faulted(id);
        assert_eq!(registry_error(submit(id, &batch(&[]))), faulted);
        assert_eq!(registry_error(tick(id)), faulted);
        let tick_query = encode(&Query::Tick).unwrap();
        assert_eq!(registry_error(query(id, &tick_query)), faulted);
        assert_eq!(registry_error(save(id)), faulted);
        tick(other).unwrap();
        assert_eq!(current_tick(other), 1);
        shutdown(id).unwrap();
        assert_eq!(registry_error(save(id)), RegistryError::Unknown(id));
        shutdown(other).unwrap();
    }
}
