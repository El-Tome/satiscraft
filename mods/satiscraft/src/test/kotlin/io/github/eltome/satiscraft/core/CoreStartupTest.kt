package io.github.eltome.satiscraft.core

import io.github.eltome.satiscraft.core.driver.RecordingLogger
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertNotNull
import org.junit.jupiter.api.Assertions.assertNull
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Assumptions.assumeFalse
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import org.slf4j.event.Level
import java.nio.file.Path
import kotlin.io.path.createDirectories
import kotlin.io.path.isReadable
import kotlin.io.path.writeText

/** Starting the simulation of a server, through the real core. */
class CoreStartupTest {
    @TempDir
    lateinit var server: Path

    private val logger = RecordingLogger()

    private fun override(name: String, text: String): Path =
        server.resolve("config/satiscraft").createDirectories().resolve(name).apply { writeText(text) }

    /** The single error logged: why the simulation did not start. */
    private fun refusal(): String = logger.lines.single { it.level == Level.ERROR }.message

    @Test
    fun `the simulation starts on the shipped content`() {
        CoreStartup.start(server, logger).use { assertNotNull(it) }
        assertEquals(emptyList<Any>(), logger.lines)
    }

    @Test
    fun `the overrides in use are logged`() {
        override("config.toml", "[sim]\nthreads = 2\n")
        CoreStartup.start(server, logger).use { assertNotNull(it) }
        assertEquals("Satiscraft content overridden by config/satiscraft/config.toml", logger.lines.single().message)
    }

    @Test
    fun `invalid content is refused with its file and line`() {
        override("config.toml", "[sim]\nthread = 4\n")
        assertNull(CoreStartup.start(server, logger))
        val message = refusal()
        assertTrue(message.startsWith("Satiscraft cannot start its simulation, so the server stops."), message)
        assertTrue("config/satiscraft/config.toml:2: unknown field `thread`" in message, message)
        assertTrue("overrides are in config/satiscraft/" in message, message)
    }

    @Test
    fun `an unreadable override is refused`() {
        val file = override("items.toml", "")
        file.toFile().setReadable(false)
        assumeFalse(file.isReadable(), "the tests run as a user who can read anything")
        assertNull(CoreStartup.start(server, logger))
        assertTrue("cannot read config/satiscraft/items.toml" in refusal())
    }
}
