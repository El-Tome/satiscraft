package io.github.eltome.satiscraft.gradle

import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.assertThrows

class EmbedContentTest {
    @Test
    fun `the index lists one name per line, sorted`() {
        assertEquals("config.toml\nitems.toml\n", contentIndex(listOf("items.toml", "config.toml")))
        assertEquals("", contentIndex(emptyList()))
    }

    @Test
    fun `two files of the same name are refused`() {
        assertThrows<IllegalArgumentException> { contentIndex(listOf("items.toml", "items.toml")) }
    }
}
