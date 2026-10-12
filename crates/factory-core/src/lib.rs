//! Platform-independent factory simulation.
//!
//! This crate owns the whole simulated state. It must never depend on Minecraft,
//! JNI or networking crates, and it must keep compiling for `wasm32-unknown-unknown`.
//! Platforms talk to it only through commands in and effects out: see [`contract`] and
//! [`Simulation`].

#![forbid(unsafe_code)]

pub mod contract;
mod simulation;

pub use contract::CONTRACT_VERSION;
pub use simulation::{Core, LoadError, Simulation};

/// Version of the core, shared with the mod (`[workspace.package]` in `Cargo.toml`).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_matches_cargo_manifest() {
        assert_eq!(VERSION, env!("CARGO_PKG_VERSION"));
        assert!(!VERSION.is_empty());
    }
}
