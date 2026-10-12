package io.github.eltome.satiscraft.core.contract

import io.github.eltome.satiscraft.core.FactoryCoreException
import org.junit.jupiter.api.Assertions.assertArrayEquals
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.assertThrows
import java.util.UUID

class CodecTest {
    @Test
    fun `encode and decode go through write and read`() {
        val ping = Command.Ping(300u)
        assertArrayEquals(bytes(0x00, 0xac, 0x02), Command.encode(ping))
        assertEquals(ping, Command.decode(bytes(0x00, 0xac, 0x02)))
    }

    @Test
    fun `decode refuses leftover bytes`() {
        val error = assertThrows<FactoryCoreException> { Query.decode(bytes(0x00, 0x00)) }
        assertTrue("1 bytes left" in error.message!!, error.message)
    }

    @Test
    fun `a list is its item count then its items`() {
        val batch = listOf(Command.PanicOnNextStep, Command.Ping(1u))
        assertArrayEquals(bytes(0x02, 0x01, 0x00, 0x01), Command.list().encode(batch))
        assertEquals(batch, Command.list().decode(bytes(0x02, 0x01, 0x00, 0x01)))
    }

    @Test
    fun `every enum refuses a variant it does not have`() {
        val codecs = mapOf<String, Codec<*>>(
            "Command" to Command,
            "CommandResult" to CommandResult,
            "Rejection" to Rejection,
            "Effect" to Effect,
            "Query" to Query,
            "QueryResult" to QueryResult,
        )
        for ((name, codec) in codecs) {
            val error = assertThrows<FactoryCoreException> { codec.decode(bytes(0x7f)) }
            assertTrue("unknown $name variant 127" in error.message!!, error.message)
        }
    }

    @Test
    fun `an invalid identifier in the bytes is a decoding error`() {
        val noNamespace = written { string("stick") }
        assertThrows<FactoryCoreException> { ItemId.decode(noNamespace) }
        assertThrows<FactoryCoreException> { DimensionId.decode(noNamespace) }
    }

    @Test
    fun `ids print as Minecraft writes them`() {
        assertEquals("satiscraft:bauxite", ItemId("satiscraft:bauxite").toString())
        assertEquals("minecraft:overworld", DimensionId("minecraft:overworld").toString())
        val uuid = "069a79f4-44e9-4026-bf4b-8f2c4137000b"
        assertEquals(uuid, PlayerId(UUID.fromString(uuid)).toString())
    }
}
