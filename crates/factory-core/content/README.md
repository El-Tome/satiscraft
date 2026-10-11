# Game content

The engine's game content lives here as TOML, embedded in the mod and overridable by the
modpack or a server from `config/satiscraft/`. The core validates it all at startup and
refuses to start on an error, giving the file and line. Every balancing value belongs in
these files, never in code.

Planned files (see `docs/cadrage-technique-v1.md`):

| File | Content |
| --- | --- |
| `items.toml` | Items, resources, tools, ore blocks |
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

The other mods keep their own content: `worldgen.toml` (ore placement, surface pieces) in
`satiscraft-world`, `vanilla.toml` (what is blocked, and until which age) in
`satiscraft-primitive`. Server settings (threads, tick budget, activity policies) go in
`config.toml`.

Units: durations in ticks, energy in milli-units, chances as decimals (converted to
exact fractions at load time). Identifiers (`satiscraft:bauxite`) are frozen once
released: deprecate, never rename or delete.
