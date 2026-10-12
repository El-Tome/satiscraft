package io.github.eltome.satiscraft.core.contract

import io.github.eltome.satiscraft.core.FactoryCoreException
import org.junit.jupiter.api.Assertions.assertArrayEquals
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertNull
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.assertThrows

class PostcardReaderTest {
    private fun reader(vararg values: Int) = PostcardReader(bytes(*values))

    /** Asserts that reading fails with a message containing [reason]. */
    private fun assertRefused(reason: String, read: () -> Any?) {
        val error = assertThrows<FactoryCoreException> { read() }
        assertTrue(reason in error.message!!, error.message)
    }

    @Test
    fun `reads back what the writer wrote`() {
        val ulongs = listOf(0uL, 127uL, 128uL, 300uL, ULong.MAX_VALUE)
        val ints = listOf(0, -1, 1, Int.MIN_VALUE, Int.MAX_VALUE)
        val input = PostcardReader(written {
            ulongs.forEach(::u64)
            ints.forEach(::i32)
            u32(UInt.MAX_VALUE)
            string("é/🙂")
        })
        assertEquals(ulongs, ulongs.map { input.u64() })
        assertEquals(ints, ints.map { input.i32() })
        assertEquals(UInt.MAX_VALUE, input.u32())
        assertEquals("é/🙂", input.string())
        input.finish()
    }

    @Test
    fun `padded varints are accepted, as postcard does`() {
        assertEquals(0uL, reader(0x80, 0x00).u64())
    }

    @Test
    fun `varints that overflow are refused`() {
        assertRefused("invalid varint") { reader(0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x02).u64() }
        assertRefused("invalid varint") { reader(0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x81, 0x00).u64() }
        assertRefused("invalid varint") { reader(0xff, 0xff, 0xff, 0xff, 0x10).u32() }
    }

    @Test
    fun `truncated input is refused`() {
        assertRefused("unexpected end of input") { reader().byte() }
        assertRefused("unexpected end of input") { reader(0x80).u64() }
        assertRefused("unexpected end of input") { reader(0x01).fixedBytes(2) }
        assertRefused("past the end of input") { reader(0x05, 0x61).string() }
        assertRefused("past the end of input") { reader(0x03, 0x00).sequence { 0 } }
    }

    @Test
    fun `invalid UTF-8 is refused`() {
        assertRefused("invalid UTF-8") { reader(0x01, 0xff).string() }
    }

    @Test
    fun `bools and options only take 0 or 1`() {
        assertEquals(listOf(false, true), reader(0x00, 0x01).let { listOf(it.bool(), it.bool()) })
        assertRefused("invalid bool") { reader(0x02).bool() }
        assertNull(reader(0x00).option { 1 })
        assertEquals(-1, reader(0x01, 0x01).let { input -> input.option { input.i32() } })
        assertRefused("invalid bool") { reader(0x02).option { 1 } }
    }

    @Test
    fun `raw bytes, sequences and variants read their parts`() {
        val input = reader(0x07, 0x01, 0x02, 0x02, 0x02, 0x01, 0x80, 0x01)
        assertEquals(7.toByte(), input.byte())
        assertArrayEquals(bytes(0x01, 0x02), input.fixedBytes(2))
        assertEquals(listOf(1, -1), input.sequence { input.i32() })
        assertEquals(128, input.variant())
        input.finish()
    }

    @Test
    fun `a variant index past Int range is refused`() {
        assertRefused("variant index 4294967295 out of range") { reader(0xff, 0xff, 0xff, 0xff, 0x0f).variant() }
    }

    @Test
    fun `leftover bytes are refused`() {
        val input = reader(0x01, 0x02, 0x03)
        input.byte()
        assertRefused("2 bytes left after the value") { input.finish() }
    }

    @Test
    fun `validation errors become decoding errors`() {
        assertEquals(1, reader().validated { 1 })
        assertRefused("cannot decode: bad value") { reader().validated { require(false) { "bad value" } } }
    }

    @Test
    fun `unknown variants are named`() {
        assertEquals("cannot decode: unknown Command variant 9 (at byte 0)", reader().unknownVariant("Command", 9).message)
    }
}
