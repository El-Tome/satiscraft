# Modpack

Assembles the three Satiscraft mods into the Stone Age experience. Data only: no code.

- `./gradlew :modpack:runServer` / `runClient` start a dev game with the three mods, loaded
  from their jars as on a real server. The game folder is `modpack/run/` (git-ignored).
- `config/satiscraft/` holds the modpack's config and content overrides, copied into the
  game folder before each dev run: edit them here, not in `run/`. An override only states
  what it changes from the engine's defaults (`crates/factory-core/content/`); see
  `crates/factory-core/content/README.md`. A server takes the same files in its own
  `config/satiscraft/`.
