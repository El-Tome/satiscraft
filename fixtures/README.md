# Save fixtures

One reference save per released version, produced by `factory-core` and committed
here (`fixtures/<version>/...`). CI loads every fixture and runs the migration chain
up to the current format: a release must never break an older world.

Never edit or delete a fixture. A newer save format adds a new fixture and a
`migrate_vN_to_vN+1` function.
