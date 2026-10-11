# Satiscraft

A progression-driven factory mod for NeoForge multiplayer servers: start with flint tools in
the Stone Age, master heat and metals, then build powered, automated factories as a team
around a HUB.

The whole simulation runs in a Rust core (`factory-core`) that knows nothing about Minecraft.
It ships as three NeoForge mods written in Kotlin, at one common version:

- `satiscraft`, the engine: translates game events into commands for the core, displays the
  effects it sends back, and declares every block and item. It runs on its own.
- `satiscraft_world`: world generation of the Stone Age (ores, surface pieces, structure loot).
- `satiscraft_primitive`: the Stone Age rules (neutralized vanilla, animals that fight back).

> Status: early development (0.0.x). 0.1.0 = the Stone Age, from bare hands to the first iron bloom.

## Requirements

- [rustup](https://rustup.rs): the pinned toolchain and targets install automatically.
- A JDK 17+ to launch Gradle. Gradle then provisions the JDK 25 that Minecraft 26.2 needs.

## Getting started

```sh
./gradlew build                  # Rust core + JNI bridge + the three jars, all tests
./gradlew :modpack:runServer     # dev server with the three mods
./gradlew :modpack:runClient     # dev client with the three mods
./gradlew :mods:satiscraft:runServer   # the engine alone
```

The jars land in `mods/*/build/libs/`. The engine jar embeds the native library of the
machine that built it: a server jar must be built on Linux x86_64 (CI does it on every merge).

## Repository layout

| Path | Content |
| --- | --- |
| `crates/factory-core/` | Simulation and game content (TOML), platform-independent |
| `crates/factory-jni/` | JNI bridge between the JVM and the core |
| `mods/satiscraft/` | The engine mod (Kotlin), embeds the native library |
| `mods/satiscraft-world/` | World generation mod |
| `mods/satiscraft-primitive/` | Stone Age rules mod |
| `modpack/` | Config and content overrides, dev runs of the whole pack |
| `build-logic/` | Gradle setup shared by the mods |
| `fixtures/` | Reference saves, one per released version |
| `scripts/deploy-test.sh` | Deployment to the test server (SFTP, see `.env.deploy.example`) |
| `docs/` | Design documents |

Contribution rules (architecture, tests, versioning) are in [CLAUDE.md](CLAUDE.md).

## License

[PolyForm Noncommercial 1.0.0](LICENSE.md): free to use, modify and share for any
noncommercial purpose, as long as the license and the copyright notice come along.
Commercial use requires the author's agreement.
