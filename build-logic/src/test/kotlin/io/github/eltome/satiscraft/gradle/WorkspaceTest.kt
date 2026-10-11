package io.github.eltome.satiscraft.gradle

import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.assertThrows

class WorkspaceTest {
    @Test
    fun `reads the version of the workspace package`() {
        val manifest = """
            [package]
            version = "9.9.9"

            [workspace.package]
            edition = "2024"
            version = "0.1.2"

            [workspace.dependencies]
            version = "1.0.0"
        """.trimIndent()

        assertEquals("0.1.2", workspaceVersion(manifest))
    }

    @Test
    fun `fails without a workspace package section`() {
        assertThrows<IllegalStateException> { workspaceVersion("[package]\nversion = \"1.0.0\"\n") }
    }

    @Test
    fun `fails when the workspace package has no version`() {
        assertThrows<IllegalStateException> { workspaceVersion("[workspace.package]\nedition = \"2024\"\n") }
    }

    @Test
    fun `turns project names into mod ids`() {
        assertEquals("satiscraft", modIdOf("satiscraft"))
        assertEquals("satiscraft_world", modIdOf("satiscraft-world"))
    }

    @Test
    fun `rejects names that make no valid mod id`() {
        assertThrows<IllegalArgumentException> { modIdOf("Satiscraft") }
        assertThrows<IllegalArgumentException> { modIdOf("1mod") }
    }
}
