# Satiscraft

A progression-driven factory mod for NeoForge multiplayer servers: start with flint tools in
the Stone Age, master heat and metals, then build powered, automated factories as a team
around a HUB.

The whole simulation runs in a Rust core (`factory-core`) that knows nothing about Minecraft.
The NeoForge mod, written in Kotlin, only translates game events into commands and displays
the effects the core sends back.

> Status: early development (0.0.x). 0.1.0 = the Stone Age, from bare hands to the first iron bloom.

## Requirements

- [rustup](https://rustup.rs): the pinned toolchain and targets install automatically.
- A JDK 17+ to launch Gradle. Gradle then provisions the JDK 25 that Minecraft 26.2 needs.

## Getting started

```sh
./gradlew build                  # Rust core + JNI bridge + mod jar, all tests
./gradlew :neoforge:runServer    # dev server with the mod
./gradlew :neoforge:runClient    # dev client
```

The jar lands in `neoforge/build/libs/`. It embeds the native library of the machine that
built it: a server jar must be built on Linux x86_64 (CI does it on every merge).

## Repository layout

| Path | Content |
| --- | --- |
| `crates/factory-core/` | Simulation and game content (TOML), platform-independent |
| `crates/factory-jni/` | JNI bridge between the JVM and the core |
| `neoforge/` | Kotlin NeoForge mod |
| `fixtures/` | Reference saves, one per released version |
| `scripts/deploy-test.sh` | Deployment to the test server |
| `docs/` | Design documents |

Contribution rules (architecture, tests, versioning) are in [CLAUDE.md](CLAUDE.md).

## License

[PolyForm Noncommercial 1.0.0](LICENSE.md): free to use, modify and share for any
noncommercial purpose, as long as the license and the copyright notice come along.
Commercial use requires the author's agreement.
