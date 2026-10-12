//! The simulations alive in this process, each behind the numeric id the JVM holds.
//!
//! The JVM never gets a pointer: it passes the id back on every call, and the registry looks
//! the simulation up. Ids grow and are never reused, so a stale id cannot reach another
//! simulation.

use std::collections::BTreeMap;
use std::fmt;

/// Why a call cannot reach a simulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RegistryError {
    /// No simulation has this id: never created, or already shut down.
    Unknown(i64),
    /// A panic interrupted a call that changes the simulation, which may be half-modified.
    Faulted(i64),
}

impl fmt::Display for RegistryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unknown(id) => {
                write!(f, "no simulation {id}: never created or already shut down")
            }
            Self::Faulted(id) => write!(
                f,
                "simulation {id} is faulted: a panic may have left it half-modified, \
                 so it can no longer run nor be saved, only shut down"
            ),
        }
    }
}

impl std::error::Error for RegistryError {}

/// Simulations by id, each with whether a panic faulted it.
#[derive(Debug)]
pub(crate) struct Registry<S> {
    /// Id of the next simulation: ids start at 1 and only grow.
    next_id: i64,
    slots: BTreeMap<i64, Slot<S>>,
}

#[derive(Debug)]
struct Slot<S> {
    simulation: S,
    /// Set while a call that changes the simulation runs, cleared when it returns: a panic
    /// leaves it set.
    faulted: bool,
}

impl<S> Registry<S> {
    /// An empty registry, usable in a `static`.
    pub(crate) const fn new() -> Self {
        Self {
            next_id: 1,
            slots: BTreeMap::new(),
        }
    }

    /// Adds a simulation under an id never handed out before, and returns that id.
    pub(crate) fn insert(&mut self, simulation: S) -> i64 {
        let id = self.next_id;
        self.next_id = id.checked_add(1).expect("simulation ids exhausted");
        let slot = Slot {
            simulation,
            faulted: false,
        };
        self.slots.insert(id, slot);
        id
    }

    /// Runs `f` on a healthy simulation, for a call that changes it. If `f` panics, the
    /// simulation stays faulted and the panic goes on unwinding.
    pub(crate) fn modify<T>(
        &mut self,
        id: i64,
        f: impl FnOnce(&mut S) -> T,
    ) -> Result<T, RegistryError> {
        let slot = self.slots.get_mut(&id).ok_or(RegistryError::Unknown(id))?;
        if slot.faulted {
            return Err(RegistryError::Faulted(id));
        }
        slot.faulted = true;
        let value = f(&mut slot.simulation);
        slot.faulted = false;
        Ok(value)
    }

    /// Runs `f` on a healthy simulation, for a call that only reads it. A panic in `f` goes
    /// on unwinding but leaves the simulation as it was, so it does not fault it.
    pub(crate) fn read<T>(&self, id: i64, f: impl FnOnce(&S) -> T) -> Result<T, RegistryError> {
        let slot = self.slots.get(&id).ok_or(RegistryError::Unknown(id))?;
        if slot.faulted {
            return Err(RegistryError::Faulted(id));
        }
        Ok(f(&slot.simulation))
    }

    /// Takes a simulation out of the registry, faulted or not.
    pub(crate) fn remove(&mut self, id: i64) -> Result<S, RegistryError> {
        self.slots
            .remove(&id)
            .map(|slot| slot.simulation)
            .ok_or(RegistryError::Unknown(id))
    }
}

#[cfg(test)]
mod tests {
    use std::panic::{AssertUnwindSafe, catch_unwind};

    use super::*;

    /// A registry holding one counter per simulation.
    fn registry_of(counters: &[u32]) -> (Registry<u32>, Vec<i64>) {
        let mut registry = Registry::new();
        let ids = counters.iter().map(|&c| registry.insert(c)).collect();
        (registry, ids)
    }

    fn panic_while_modifying(registry: &mut Registry<u32>, id: i64) {
        let outcome = catch_unwind(AssertUnwindSafe(|| {
            registry.modify(id, |counter| {
                *counter += 1;
                panic!("boom");
            })
        }));
        assert!(outcome.is_err());
    }

    #[test]
    fn ids_start_at_one_and_grow() {
        let (_, ids) = registry_of(&[10, 20, 30]);
        assert_eq!(ids, [1, 2, 3]);
    }

    #[test]
    fn each_id_reaches_its_own_simulation() {
        let (mut registry, ids) = registry_of(&[10, 20]);
        assert_eq!(registry.modify(ids[0], |c| *c += 1), Ok(()));
        assert_eq!(registry.read(ids[0], |c| *c), Ok(11));
        assert_eq!(registry.read(ids[1], |c| *c), Ok(20));
    }

    #[test]
    fn unknown_ids_are_refused() {
        let (mut registry, _) = registry_of(&[10]);
        for id in [0, -1, 2, i64::MAX] {
            assert_eq!(registry.read(id, |c| *c), Err(RegistryError::Unknown(id)));
            assert_eq!(registry.modify(id, |_| ()), Err(RegistryError::Unknown(id)));
            assert_eq!(registry.remove(id), Err(RegistryError::Unknown(id)));
        }
    }

    #[test]
    fn remove_shuts_a_simulation_down_once() {
        let (mut registry, ids) = registry_of(&[10, 20]);
        assert_eq!(registry.remove(ids[0]), Ok(10));
        assert_eq!(registry.remove(ids[0]), Err(RegistryError::Unknown(ids[0])));
        assert_eq!(
            registry.read(ids[0], |c| *c),
            Err(RegistryError::Unknown(ids[0]))
        );
        assert_eq!(registry.read(ids[1], |c| *c), Ok(20));
    }

    #[test]
    fn ids_are_never_reused() {
        let (mut registry, ids) = registry_of(&[10, 20]);
        registry.remove(ids[1]).unwrap();
        let next = registry.insert(30);
        assert_eq!(next, 3);
        assert_eq!(
            registry.read(ids[1], |c| *c),
            Err(RegistryError::Unknown(ids[1]))
        );
    }

    #[test]
    fn a_panic_while_modifying_faults_only_that_simulation() {
        let (mut registry, ids) = registry_of(&[10, 20]);
        panic_while_modifying(&mut registry, ids[0]);
        let faulted = Err(RegistryError::Faulted(ids[0]));
        assert_eq!(registry.read(ids[0], |c| *c), faulted);
        assert_eq!(registry.modify(ids[0], |c| *c), faulted);
        assert_eq!(registry.modify(ids[1], |c| *c), Ok(20));
    }

    #[test]
    fn a_faulted_simulation_can_still_be_shut_down() {
        let (mut registry, ids) = registry_of(&[10]);
        panic_while_modifying(&mut registry, ids[0]);
        assert_eq!(registry.remove(ids[0]), Ok(11));
    }

    #[test]
    fn a_panic_while_reading_does_not_fault() {
        let (registry, ids) = registry_of(&[10]);
        let outcome = catch_unwind(|| registry.read(ids[0], |_| panic!("boom")));
        assert!(outcome.is_err());
        assert_eq!(registry.read(ids[0], |c| *c), Ok(10));
    }

    #[test]
    fn errors_say_why_the_call_failed() {
        assert_eq!(
            RegistryError::Unknown(4).to_string(),
            "no simulation 4: never created or already shut down"
        );
        assert_eq!(
            RegistryError::Faulted(4).to_string(),
            "simulation 4 is faulted: a panic may have left it half-modified, \
             so it can no longer run nor be saved, only shut down"
        );
    }
}
