package io.github.eltome.satiscraft.world

import net.neoforged.fml.common.Mod
import org.slf4j.Logger
import org.slf4j.LoggerFactory

/**
 * Mod entry point.
 *
 * Places the engine's ores and surface pieces, and fills structure chests. Declares no block
 * nor item: they all come from the engine, and placement values from worldgen.toml.
 */
@Mod(SatiscraftWorld.ID)
object SatiscraftWorld {
    const val ID = "satiscraft_world"

    val LOGGER: Logger = LoggerFactory.getLogger(ID)
}
