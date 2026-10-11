package io.github.eltome.satiscraft.primitive

import net.neoforged.fml.common.Mod
import org.slf4j.Logger
import org.slf4j.LoggerFactory

/**
 * Mod entry point.
 *
 * Shapes the Stone Age by restriction only: what is blocked, and until which age, comes from
 * vanilla.toml, and the age a team reached is asked to the engine. Declares no block nor item.
 */
@Mod(SatiscraftPrimitive.ID)
object SatiscraftPrimitive {
    const val ID = "satiscraft_primitive"

    val LOGGER: Logger = LoggerFactory.getLogger(ID)
}
