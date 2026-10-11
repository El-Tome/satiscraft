# Satiscraft: rules for the agent

Factory-building mod for NeoForge (Minecraft 26.2, dedicated Linux servers) whose whole
simulation lives in a Rust core, so that it can later be plugged into Pumpkin.
Design references (French), to read before any non-trivial change:

- `docs/cadrage-technique-v1.md`: architecture, contract, persistence, build.
- `docs/cadrage-gameplay-age-de-pierre.md`: gameplay of 0.1.0, the Stone Age.

Roadmap: 0.1 Stone Age, 0.2 Iron Age and kinetics, 0.3 electricity and conveyors,
0.4 electronics. The technical framing predates this split: for version scope, the
gameplay doc wins. If code and docs disagree on anything else, ask.

## Layout

```
Cargo.toml                 Rust workspace; [workspace.package] version = THE mod version
rust-toolchain.toml        pinned Rust toolchain and targets
crates/factory-core/       pure game logic + content/ (TOML); no MC, JNI or network
crates/factory-jni/        JNI bridge: translation, panic handling, threading only
neoforge/                  Kotlin mod (ModDevGradle + Kotlin for Forge)
fixtures/                  reference saves, one per released version
docs/                      design documents (French)
scripts/deploy-test.sh     deploys to the TEST server only
gradle.properties          pinned MC / NeoForge / Kotlin / KFF versions, mod metadata
```

## Commands

```sh
./gradlew build                      # everything: cargo build, Rust tests, Kotlin + JNI tests, jar
cargo test --workspace               # Rust tests only
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all
cargo build -p factory-core --target wasm32-unknown-unknown   # portability guard
./gradlew :neoforge:runServer        # dev server (also runClient, runGameTestServer)
scripts/deploy-test.sh               # jar -> test server, then restart
```

The Gradle build compiles `factory-jni` for the host and embeds it under
`natives/<platform>/` in the jar. Only a jar built on Linux x86_64 (CI) runs on the server;
`-Prust_target=<triple>` overrides the target.

## Architecture rules

- Dependencies only point down: Kotlin mod → `factory-jni` → `factory-core`.
- `factory-core`: `#![forbid(unsafe_code)]`, no JNI/Minecraft/network crate, must keep
  building for `wasm32-unknown-unknown`. Rust owns all simulated state.
- `factory-jni`: no game rule. `#![deny(unsafe_code)]`; the only exemption is
  `src/exports.rs`, for `#[unsafe(no_mangle)]`. Each export is a thin wrapper:
  `with_env(...)` then `.resolve::<ThrowFactoryCoreException>()`. Any new `unsafe` needs
  a justification comment and explicit maintainer approval in the PR.
- Kotlin: registers blocks/items/menus and displays what the core returns. It never stores
  simulated state nor validates gameplay. A gameplay rule in Kotlin is an architecture bug.
- Contract: commands in, effects out (`submit`, `step`/`tick`, `query`, `save`, `load`).
  Positions are `(dimension, x, y, z)` integers, items string ids (`satiscraft:bauxite`),
  players UUIDs. No Minecraft type crosses. Adding a field is fine; removing or changing one
  bumps `factory_core::CONTRACT_VERSION` and Kotlin `Contract.VERSION` together.
- JNI: one call per batch, never per machine; byte arrays (postcard) in and out; the JVM only
  holds numeric simulation ids. Called from the server thread only; the core's thread pool
  never calls back into the JVM. Panics always unwind (`panic = "unwind"`) and become
  `FactoryCoreException`; a panicking island goes `Faulted`, the rest keeps running.
- Simulation: exact integer fractions, no floats. Parallel and sequential runs must produce
  the same state. Durations in ticks, energy in milli-units.
- Every balancing value lives in `crates/factory-core/content/*.toml`, never in code.
- Identifiers are frozen once released: deprecate, never rename or delete.
- Save format: versioned binary blob (`FCTR` header); every structure change bumps the format
  version and adds `migrate_vN_to_vN+1` plus a fixture.

## Conventions

- English for code, comments, commits and repo docs. Player-facing text goes through the
  `en_us` / `fr_fr` lang files, never hard-coded.
- Every function has at least one test, non-trivial private ones included. Rust: unit tests
  per module, command → effect scenarios, `proptest` invariants (no item created or lost,
  energy conserved, parallel == sequential). Kotlin: JUnit for translation, one NeoForge
  GameTest per block and per menu.
- Versions: bump the patch of `[workspace.package] version` in every PR that closes an issue,
  the minor at each milestone (0.1.0 = Stone Age). CI fails otherwise; the `no-version-bump` label
  exempts PRs that ship nothing (CI, docs). Merging to main tags `vX.Y.Z` and publishes the
  jar, with release notes from PR titles.
- Toolchain and dependency versions are pinned; upgrading one is a separate task.

## Deployment

Only `scripts/deploy-test.sh`, which targets the test server through a Pterodactyl key
restricted to it. Never touch production: it is updated by hand.

## Not wired yet

NeoForge GameTests in CI, the frozen-identifier check and the fixture-loading check arrive
with the first blocks, content files and save format.
