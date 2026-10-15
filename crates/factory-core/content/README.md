# Game content

The engine's game content lives here as TOML, embedded in the mod under `content/` and
overridable by the modpack or a server from `config/satiscraft/`. The core validates it all at
startup and refuses to start on an error, giving the file and line. Every balancing value
belongs in these files, never in code.

## Files

| File | Content |
| --- | --- |
| `config.toml` | Server settings: simulation threads, tick budget, activity policies |
| `items.toml` | Items, resources, tools, ore blocks |

Planned, each with the issue that needs it (see `docs/cadrage-technique-v1.md`):

| File | Content |
| --- | --- |
| `stations.toml` | Custom crafting stations and their recipes |
| `minigames.toml` | Knapping (flint, stone) and shaping (clay) patterns |
| `assembly.toml` | In-hand assemblies |
| `heat.toml` | Furnaces, fuels, temperatures |
| `multiblocks.toml` | Multiblock templates |
| `quests.toml` | Main and optional quests |
| `progression.toml` | Camp levels and revealed blueprints |
| `teams.toml` | Roles and permissions |
| `machines.toml` | Machines (from 0.2) |
| `recipes.toml` | Machine recipes (from 0.2) |
| `logistics.toml` | Conveyors, cables (from 0.3) |

A new file is listed in `FILES` (`crates/factory-core/src/content/mod.rs`); the build embeds
every `*.toml` of this folder and the mod passes them all to the core.

The other mods keep their own content: `worldgen.toml` (ore placement, surface pieces) in
`satiscraft-world`, `vanilla.toml` (what is blocked, and until which age) in
`satiscraft-primitive`. Their overrides share `config/satiscraft/`.

## Overrides

A file in `config/satiscraft/` overrides the file of the same name here, and only states what
it changes:

- a table merges key by key: `[sim]` then `threads = 4` changes the thread count alone;
- `[[entries]]` with an `id` merge by `id`: an override entry changes the entry of the same
  `id`, so it needs the `id` and the keys it changes;
- any other value, an array included, replaces the default one.

Minecraft registers items and blocks from the defaults embedded in the jar, on servers and
clients alike: an override can change an item's values but cannot add an item. Unknown keys,
unknown files, values of the wrong type and broken references are all errors, each reported
as `file:line`.

## Units and identifiers

Durations in ticks, energy in milli-units, chances as decimals between 0 and 1 with at most 6
decimal places (read as exact fractions). Identifiers (`satiscraft:bauxite`) live in the
`satiscraft` namespace and are frozen once released: deprecate, never rename or delete.
