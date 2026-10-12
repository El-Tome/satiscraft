package io.github.eltome.satiscraft.core.contract

import org.junit.jupiter.api.Assertions.assertArrayEquals
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.assertThrows
import java.nio.charset.CharacterCodingException

/** Bytes as `0x..` literals, for readable expectations. */
internal fun bytes(vararg values: Int) = ByteArray(values.size) { values[it].toByte() }

internal fun written(write: PostcardWriter.() -> Unit): ByteArray = PostcardWriter().apply(write).toByteArray()

class PostcardWriterTest {
    @Test
    fun `u64 uses seven bits per byte, low bits first`() {
        assertArrayEquals(bytes(0x00), written { u64(0u) })
        assertArrayEquals(bytes(0x7f), written { u64(127u) })
        assertArrayEquals(bytes(0x80, 0x01), written { u64(128u) })
        assertArrayEquals(bytes(0xac, 0x02), written { u64(300u) })
        assertArrayEquals(bytes(0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x01), written { u64(ULong.MAX_VALUE) })
    }

    @Test
    fun `u32 and variant indices are varints too`() {
        assertArrayEquals(bytes(0xff, 0xff, 0xff, 0xff, 0x0f), written { u32(UInt.MAX_VALUE) })
        assertArrayEquals(bytes(0x80, 0x01), written { variant(128) })
    }

    @Test
    fun `i32 is zigzag encoded`() {
        assertArrayEquals(bytes(0x00, 0x01, 0x02, 0x7f, 0x80, 0x01), written { listOf(0, -1, 1, -64, 64).forEach(::i32) })
        assertArrayEquals(bytes(0xff, 0xff, 0xff, 0xff, 0x0f), written { i32(Int.MIN_VALUE) })
        assertArrayEquals(bytes(0xfe, 0xff, 0xff, 0xff, 0x0f), written { i32(Int.MAX_VALUE) })
    }

    @Test
    fun `strings are their UTF-8 length then their bytes`() {
        assertArrayEquals(bytes(0x00), written { string("") })
        assertArrayEquals(bytes(0x03, 0x61, 0xc3, 0xa9), written { string("aé") })
        assertArrayEquals(bytes(0x80, 0x01) + ByteArray(128) { 0x78 }, written { string("x".repeat(128)) })
    }

    @Test
    fun `a lone surrogate cannot be written`() {
        assertThrows<CharacterCodingException> { written { string("\uD800") } }
    }

    @Test
    fun `raw bytes and bools are written as they are`() {
        assertArrayEquals(bytes(0xfe, 0x01, 0x02, 0x00, 0x01), written {
            byte(0xfe.toByte())
            fixedBytes(bytes(0x01, 0x02))
            bool(false)
            bool(true)
        })
    }

    @Test
    fun `sequences are their item count then their items`() {
        assertArrayEquals(bytes(0x00), written { sequence(emptyList<Int>(), ::i32) })
        assertArrayEquals(bytes(0x02, 0x02, 0x01), written { sequence(listOf(1, -1), ::i32) })
    }

    @Test
    fun `options are a tag then the value`() {
        assertArrayEquals(bytes(0x00), written { option(null as Int?, ::i32) })
        assertArrayEquals(bytes(0x01, 0x02), written { option(1, ::i32) })
    }
}
