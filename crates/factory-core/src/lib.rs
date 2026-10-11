//! Platform-independent factory simulation.
//!
//! This crate owns the whole simulated state. It must never depend on Minecraft,
//! JNI or networking crates, and it must keep compiling for `wasm32-unknown-unknown`.
//! Platforms talk to it only through commands in and effects out.

#![forbid(unsafe_code)]

/// Version of the core, shared with the mod (`[workspace.package]` in `Cargo.toml`).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Version of the command/effect contract spoken with the platforms.
///
/// Adding a field keeps the version; removing or changing one requires a bump,
/// mirrored on the Kotlin side (`Contract.VERSION`).
pub const CONTRACT_VERSION: u16 = 1;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_matches_cargo_manifest() {
        assert_eq!(VERSION, env!("CARGO_PKG_VERSION"));
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn contract_version_starts_at_one() {
        assert_eq!(CONTRACT_VERSION, 1);
    }
}
