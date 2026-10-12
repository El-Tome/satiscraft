package io.github.eltome.satiscraft.core.contract

import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.assertDoesNotThrow
import org.junit.jupiter.api.assertThrows

/** The same cases as the validation tests of `ids.rs`, so both sides accept the same ids. */
class IdsTest {
    private fun refusal(id: String) = assertThrows<IllegalArgumentException> { requireNamespaced(id) }.message

    @Test
    fun `resource locations are accepted`() {
        for (id in listOf("satiscraft:bauxite", "minecraft:overworld", "my_mod-2.0:block/stone_axe.head", "a:b")) {
            assertDoesNotThrow(id) { requireNamespaced(id) }
        }
    }

    @Test
    fun `a namespace is required`() {
        assertEquals("identifier `bauxite` has no namespace (expected `namespace:path`)", refusal("bauxite"))
    }

    @Test
    fun `empty parts are refused`() {
        for (id in listOf(":bauxite", "satiscraft:", ":")) {
            assertEquals("identifier `$id` has an empty namespace or path", refusal(id))
        }
    }

    @Test
    fun `invalid characters are refused`() {
        val cases = listOf(
            "Satiscraft:bauxite" to 'S',
            "satiscraft:Bauxite" to 'B',
            "satiscraft:raw bauxite" to ' ',
            "satis/craft:bauxite" to '/',
            "satiscraft:ore:bauxite" to ':',
            "satiscraft:bauxité" to 'é',
        )
        for ((id, character) in cases) {
            assertEquals("identifier `$id` contains the invalid character '$character'", refusal(id))
        }
    }

    @Test
    fun `item and dimension ids validate on construction`() {
        assertThrows<IllegalArgumentException> { ItemId("minecraft:Stick") }
        assertThrows<IllegalArgumentException> { DimensionId("overworld") }
    }
}
