# Game content

All game content lives here as TOML, embedded in the mod and overridable by a server
from `config/satiscraft/`. Every balancing value belongs in these files, never in code.

Planned files (see `docs/cadrage-technique-v1.md`):

| File | Content |
| --- | --- |
| `items.toml` | Items and resources |
| `machines.toml` | Machines, slots, power draw |
| `recipes.toml` | Recipes per machine |
| `generators.toml` | Generators and fuels |
| `logistics.toml` | Conveyors, cables, speeds, capacities |
| `progression.toml` | HUB tiers and unlocks |
| `worldgen.toml` | Ore veins and resource nodes |
| `teams.toml` | Roles and permissions |

Units: durations in ticks, energy in milli-units, chances as decimals (converted to
exact fractions at load time). Identifiers (`satiscraft:bauxite`) are frozen once
released: deprecate, never rename or delete.
