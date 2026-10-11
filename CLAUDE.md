# Satiscraft: rules for the agent

Factory-building mod for NeoForge (Minecraft 26.2, dedicated Linux servers) whose whole
simulation lives in a Rust core, so that it can later be plugged into Pumpkin.
Design references (French), to read before any non-trivial change:

- `docs/cadrage-technique-v1.md`: architecture, contract, persistence, build.
- `docs/cadrage-gameplay-age-de-pierre.md`: gameplay of 0.1.0, the Stone Age.

Roadmap: 0.1 Stone Age, 0.2 Iron Age and kinetics, 0.3 electricity and conveyors,
0.4 electronics. The gameplay doc says what 0.1 contains, the technical doc how to build it.
The docs call the project "factory": its name is Satiscraft. If code and docs disagree, ask.

## Layout

```
Cargo.toml                 Rust workspace; [workspace.package] version = THE version of all mods
rust-toolchain.toml        pinned Rust toolchain and targets
crates/factory-core/       pure game logic + default content/ (TOML); no MC, JNI or network
crates/factory-jni/        JNI bridge: translation, panic handling, threading only
mods/satiscraft/           the engine (Kotlin): embeds the native lib, declares all blocks and items
mods/satiscraft-world/     world generation: places the engine's ores and surface pieces, loot
mods/satiscraft-primitive/ Stone Age rules: neutralized vanilla, bare-hand limits, animals strike back
modpack/                   config and content overrides; dev runs of the three mods together
build-logic/               Gradle convention plugin `satiscraft.mod`, shared by the mods
fixtures/                  reference saves, one per released version
docs/                      design documents (French)
scripts/deploy-test.sh     deploys to the TEST server only
gradle.properties          pinned MC / NeoForge / Kotlin / KFF versions, shared metadata
```

## Commands

```sh
./gradlew build                      # everything: cargo build, Rust + Kotlin + JNI tests, 3 jars
cargo test --workspace               # Rust tests only
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all
cargo build -p factory-core --target wasm32-unknown-unknown   # portability guard
./gradlew :modpack:runServer         # dev server with the three mods (also runClient)
./gradlew :mods:satiscraft:runServer # the engine alone (also runClient, runGameTestServer, runData)
scripts/deploy-test.sh [jar...]      # the jars (from CI) -> test server, restart, check the core loaded
scripts/deploy-test.sh --check       # read-only: mods folder, server state, API key scope
scripts/deploy-test.sh --power stop  # or start, restart, kill; waits for the resulting state
scripts/deploy-test.sh --log 100     # end of the test server console
```

The Gradle build compiles `factory-jni` for the host and embeds it under
`natives/<platform>/` in the engine jar. Only a jar built on Linux x86_64 (CI) runs on the
server; `-Prust_target=<triple>` overrides the target.

## Architecture rules

- Dependencies only point down: `satiscraft-world` / `satiscraft-primitive` → engine mod
  `satiscraft` → `factory-jni` → `factory-core`.
- Three mods, one version, one release: the engine (`satiscraft`), `satiscraft_world` and
  `satiscraft_primitive`. Only the engine declares blocks and items, from its TOML.
  `satiscraft_world` only places the engine's ores, `satiscraft_primitive` only restricts.
  Neither depends on the other, and the engine runs alone in a vanilla world. They query the
  engine through its small Kotlin API (age reached by a team, whether something is unlocked).
- `factory-core`: `#![forbid(unsafe_code)]`, no JNI/Minecraft/network crate, must keep
  building for `wasm32-unknown-unknown`. Rust owns all simulated state.
- `factory-jni`: no game rule. `#![deny(unsafe_code)]`; the only exemption is
  `src/exports.rs`, for `#[unsafe(no_mangle)]`. Each export is a thin wrapper:
  `with_env(...)` then `.resolve::<ThrowFactoryCoreException>()`. Any new `unsafe` needs
  a justification comment and explicit maintainer approval in the PR.
- Core vs platform (decided 2026-10-11): the core owns everything that progresses or is
  simulated: teams, camp levels, quests, blueprints, machines and heat, station recipes,
  mini-game validation and luck draws. Rules native to the Minecraft world (mob AI, harvest
  rules, loot, worldgen, removal of vanilla recipes) are enforced by Kotlin and datapacks in
  `satiscraft-world` and `satiscraft-primitive`, with values from their TOML, and unlocks
  are asked to the engine.
- The engine's Kotlin registers blocks/items/menus and displays what the core returns. No
  Kotlin code stores simulated state or validates core gameplay: doing so is an architecture bug.
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
- Every balancing value lives in TOML, never in code: `crates/factory-core/content/` for the
  engine, `worldgen.toml` for `satiscraft-world`, `vanilla.toml` for `satiscraft-primitive`.
  The modpack and servers override them from `config/satiscraft/`.
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

Only `scripts/deploy-test.sh`: it uploads the jars with the test server's SFTP account (which
Pterodactyl scopes to that server), along with the dependency mods staged by
`./gradlew :modpack:serverMods`, then restarts the server through the client API. Its account
is non-admin and reaches only the test server. Never touch production: it is updated by hand.
Never open `.env.deploy`: only the script reads it.

## Not wired yet

NeoForge GameTests in CI, the frozen-identifier check and the fixture-loading check arrive
with the first blocks, content files and save format.
