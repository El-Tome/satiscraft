package io.github.eltome.satiscraft

import io.github.eltome.satiscraft.core.NativeBridge
import net.neoforged.fml.common.Mod
import net.neoforged.neoforge.common.NeoForge
import net.neoforged.neoforge.event.server.ServerStartingEvent
import org.slf4j.Logger
import org.slf4j.LoggerFactory

/**
 * Mod entry point.
 *
 * The Kotlin layer registers what Minecraft must know and displays what factory-core
 * returns. It never stores simulated state nor takes a gameplay decision.
 */
@Mod(Satiscraft.ID)
object Satiscraft {
    const val ID = "satiscraft"

    val LOGGER: Logger = LoggerFactory.getLogger(ID)

    init {
        NeoForge.EVENT_BUS.addListener(ServerStartingEvent::class.java, ::onServerStarting)
    }

    /** Loads factory-core on the server and refuses to start on a contract mismatch. */
    private fun onServerStarting(@Suppress("UNUSED_PARAMETER") event: ServerStartingEvent) {
        NativeBridge.ensureCompatible()
        LOGGER.info("factory-core {} loaded (contract v{})", NativeBridge.coreVersion(), NativeBridge.contractVersion())
    }
}
