package io.github.eltome.satiscraft.core

import org.slf4j.Logger
import java.io.IOException
import java.nio.file.Path

/**
 * Starts the server's simulation on the engine's content, or says why it cannot: invalid
 * content, cited by `file:line`, or an override that cannot be read. The server must not run
 * without its core, so the caller stops it.
 */
object CoreStartup {
    /** The simulation running on the content of [serverDirectory]; null once [logger] has the reason. */
    fun start(serverDirectory: Path, logger: Logger): FactoryCore? {
        val content = try {
            EngineContent.read(serverDirectory)
        } catch (e: IOException) {
            return refuse(logger, e.message)
        }
        val overrides = content.files.map { it.path }.filter { it.startsWith(EngineContent.OVERRIDES) }
        if (overrides.isNotEmpty()) {
            logger.info("Satiscraft content overridden by {}", overrides.joinToString())
        }
        return try {
            FactoryCore.create(content)
        } catch (e: FactoryCoreException) {
            refuse(logger, e.message)
        }
    }

    private fun refuse(logger: Logger, reason: String?): FactoryCore? {
        logger.error(
            "Satiscraft cannot start its simulation, so the server stops.\n{}\n" +
                "Fix the files above (a server's overrides are in {}/), then start the server again.",
            reason, EngineContent.OVERRIDES,
        )
        return null
    }
}
