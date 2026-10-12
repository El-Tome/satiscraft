package io.github.eltome.satiscraft

import io.github.eltome.satiscraft.core.FactoryCore
import io.github.eltome.satiscraft.core.NativeBridge
import io.github.eltome.satiscraft.core.contract.ContentFiles
import net.neoforged.fml.common.Mod
import net.neoforged.neoforge.common.NeoForge
import net.neoforged.neoforge.event.server.ServerStartingEvent
import net.neoforged.neoforge.event.server.ServerStoppedEvent
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

    /** The simulation of the running server, for all its dimensions; null while none runs. */
    private var core: FactoryCore? = null

    init {
        NeoForge.EVENT_BUS.addListener(ServerStartingEvent::class.java, ::onServerStarting)
        NeoForge.EVENT_BUS.addListener(ServerStoppedEvent::class.java, ::onServerStopped)
    }

    /** Loads factory-core, refuses to start on a contract mismatch, and creates the simulation. */
    private fun onServerStarting(@Suppress("UNUSED_PARAMETER") event: ServerStartingEvent) {
        NativeBridge.ensureCompatible()
        // The engine's TOML content is not loaded yet: the core runs on no files.
        core = FactoryCore.create(ContentFiles())
        LOGGER.info("factory-core {} loaded (contract v{})", NativeBridge.coreVersion(), NativeBridge.contractVersion())
    }

    /** Frees the simulation once the server has stopped, after its last save. */
    private fun onServerStopped(@Suppress("UNUSED_PARAMETER") event: ServerStoppedEvent) {
        core?.close()
        core = null
    }
}
