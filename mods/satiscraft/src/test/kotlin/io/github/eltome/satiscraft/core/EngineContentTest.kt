package io.github.eltome.satiscraft.core

import io.github.eltome.satiscraft.core.contract.ContentFile
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Assumptions.assumeFalse
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.assertThrows
import org.junit.jupiter.api.io.TempDir
import java.io.IOException
import java.nio.file.Path
import kotlin.io.path.createDirectories
import kotlin.io.path.isReadable
import kotlin.io.path.writeText

/** Gathering the engine's content: embedded defaults, then a server's overrides. */
class EngineContentTest {
    @TempDir
    lateinit var server: Path

    /** Writes [text] as the server's `config/satiscraft/<name>`. */
    private fun override(name: String, text: String): Path =
        server.resolve("config/satiscraft").createDirectories().resolve(name).apply { writeText(text) }

    /** A jar holding [resources], by path. */
    private fun jar(vararg resources: Pair<String, String>): (String) -> String? = resources.toMap()::get

    @Test
    fun `the jar embeds every content file, which the core loads`() {
        val content = EngineContent.read(server)
        assertEquals(listOf("content/config.toml", "content/items.toml"), content.files.map { it.path })
        FactoryCore.create(content).close()
    }

    @Test
    fun `overrides of the engine's files follow the defaults`() {
        override("items.toml", "# none")
        override("config.toml", "[sim]\nthreads = 2\n")
        override("worldgen.toml", "# satiscraft-world's")
        val content = EngineContent.read(server)
        val overrides = content.files.drop(2)
        assertEquals(
            listOf(
                ContentFile("config/satiscraft/config.toml", "[sim]\nthreads = 2\n"),
                ContentFile("config/satiscraft/items.toml", "# none"),
            ),
            overrides,
        )
        FactoryCore.create(content).close()
    }

    @Test
    fun `the defaults are the files the index lists`() {
        val content = EngineContent.gather(jar("content/index.txt" to "b.toml\n\na.toml\n", "content/a.toml" to "A", "content/b.toml" to "B"), server)
        assertEquals(listOf(ContentFile("content/b.toml", "B"), ContentFile("content/a.toml", "A")), content.files)
    }

    @Test
    fun `a jar without its index or a listed file is broken`() {
        val noIndex = assertThrows<IllegalStateException> { EngineContent.gather(jar(), server) }
        assertEquals("content/index.txt is missing from the jar", noIndex.message)
        val noFile = assertThrows<IllegalStateException> { EngineContent.gather(jar("content/index.txt" to "a.toml"), server) }
        assertEquals("content/a.toml is listed in content/index.txt but missing from the jar", noFile.message)
    }

    @Test
    fun `an unreadable override is named`() {
        val file = override("config.toml", "")
        file.toFile().setReadable(false)
        assumeFalse(file.isReadable(), "the tests run as a user who can read anything")
        val error = assertThrows<IOException> { EngineContent.read(server) }
        assertTrue(error.message!!.startsWith("cannot read config/satiscraft/config.toml: "), error.message)
    }
}
