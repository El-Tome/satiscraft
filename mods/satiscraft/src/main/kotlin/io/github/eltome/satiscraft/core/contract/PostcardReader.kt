package io.github.eltome.satiscraft.core.contract

import io.github.eltome.satiscraft.core.FactoryCoreException
import java.nio.charset.CharacterCodingException

/**
 * Reads postcard values written by factory-core, as strictly as the core reads them:
 * truncated input, an overflowing varint, a bad bool or option tag, invalid UTF-8, an
 * unknown variant or leftover bytes all throw [FactoryCoreException].
 */
class PostcardReader(private val bytes: ByteArray) {
    private var position = 0

    /** One raw byte, as Rust reads a `u8`. */
    fun byte(): Byte {
        if (position >= bytes.size) throw fail("unexpected end of input")
        return bytes[position++]
    }

    /** The next [size] bytes as they are: a Rust `[u8; N]`. */
    fun fixedBytes(size: Int): ByteArray {
        if (size > bytes.size - position) throw fail("unexpected end of input")
        return bytes.copyOfRange(position, position + size).also { position += size }
    }

    fun bool(): Boolean = when (byte().toInt()) {
        0 -> false
        1 -> true
        else -> throw fail("invalid bool")
    }

    fun u64(): ULong = varint(ULong.SIZE_BITS)

    fun u32(): UInt = varint(UInt.SIZE_BITS).toUInt()

    /** A zigzag varint back to an `i32`. */
    fun i32(): Int {
        val zigzag = u32().toInt()
        return zigzag ushr 1 xor -(zigzag and 1)
    }

    fun string(): String {
        val utf8 = fixedBytes(length())
        return try {
            utf8.decodeToString(throwOnInvalidSequence = true)
        } catch (e: CharacterCodingException) {
            throw fail("invalid UTF-8")
        }
    }

    /** The variant index of an enum value; its fields follow. */
    fun variant(): Int {
        val index = u32()
        if (index > Int.MAX_VALUE.toUInt()) throw fail("variant index $index out of range")
        return index.toInt()
    }

    /** A length, then that many items read by [readItem]: a Rust `Vec`. */
    fun <T> sequence(readItem: () -> T): List<T> {
        val size = length()
        return buildList { repeat(size) { add(readItem()) } }
    }

    /** Null for 0, otherwise 1 then the value read by [readValue]: a Rust `Option`. */
    fun <T : Any> option(readValue: () -> T): T? = if (bool()) readValue() else null

    /** The value built by [build], with its validation errors reported as decoding errors. */
    fun <T> validated(build: () -> T): T = try {
        build()
    } catch (e: IllegalArgumentException) {
        throw fail(e.message ?: "invalid value")
    }

    /** The error for a variant index that [type] does not have. */
    fun unknownVariant(type: String, index: Int): FactoryCoreException = fail("unknown $type variant $index")

    /** Throws unless every byte was read: leftovers mean the two sides disagree on the format. */
    fun finish() {
        if (position != bytes.size) throw fail("${bytes.size - position} bytes left after the value")
    }

    /**
     * A varint of at most [bits] bits. Like postcard, it accepts padded encodings but refuses
     * one that is too long or overflows in its last byte.
     */
    private fun varint(bits: Int): ULong {
        val maxBytes = (bits + 6) / 7
        val maxLastByte = (1 shl bits - 7 * (maxBytes - 1)) - 1
        var value = 0uL
        for (i in 0 until maxBytes) {
            val byte = byte().toInt() and 0xFF
            value = value or ((byte and 0x7F).toULong() shl 7 * i)
            if (byte and 0x80 == 0) {
                if (i == maxBytes - 1 && byte > maxLastByte) break
                return value
            }
        }
        throw fail("invalid varint")
    }

    /**
     * A string length or an item count. Every contract value takes at least one byte, so
     * either one past the remaining input is malformed.
     */
    private fun length(): Int {
        val size = u64()
        if (size > (bytes.size - position).toULong()) throw fail("length $size past the end of input")
        return size.toInt()
    }

    private fun fail(reason: String) = FactoryCoreException("cannot decode: $reason (at byte $position)")
}
