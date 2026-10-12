package io.github.eltome.satiscraft.core.contract

import java.io.ByteArrayOutputStream

/**
 * Writes values in postcard, the wire format factory-core speaks (`factory_core::contract`).
 *
 * Unsigned integers are LEB128 varints and signed ones zigzag varints. Strings and sequences
 * are their length as a varint, then their content. An enum is its variant index as a
 * varint, then the variant's fields. A struct is its fields in order, with no framing.
 */
class PostcardWriter {
    private val out = ByteArrayOutputStream()

    /** The bytes written so far. */
    fun toByteArray(): ByteArray = out.toByteArray()

    /** One raw byte, as Rust writes a `u8`. */
    fun byte(value: Byte) = out.write(value.toInt())

    /** Bytes as they are, with no length: a Rust `[u8; N]`. */
    fun fixedBytes(value: ByteArray) = out.write(value)

    fun bool(value: Boolean) = out.write(if (value) 1 else 0)

    /** A `u64` as a varint: seven bits per byte, low bits first, high bit set on all but the last. */
    fun u64(value: ULong) {
        var rest = value
        while (rest >= 0x80u) {
            out.write((rest and 0x7Fu or 0x80u).toInt())
            rest = rest shr 7
        }
        out.write(rest.toInt())
    }

    fun u32(value: UInt) = u64(value.toULong())

    /** An `i32` zigzag-encoded (0, -1, 1, -2... become 0, 1, 2, 3...), then as a varint. */
    fun i32(value: Int) = u32((value shl 1 xor (value shr 31)).toUInt())

    /** UTF-8 bytes after their length. A lone surrogate, which Rust cannot hold, throws. */
    fun string(value: String) {
        val bytes = value.encodeToByteArray(throwOnInvalidSequence = true)
        length(bytes.size)
        out.write(bytes)
    }

    /** The variant index of an enum value; its fields follow. */
    fun variant(index: Int) = u32(index.toUInt())

    /** The number of [items], then each one written by [writeItem]: a Rust `Vec`. */
    fun <T> sequence(items: List<T>, writeItem: (T) -> Unit) {
        length(items.size)
        items.forEach(writeItem)
    }

    /** 0 for null, otherwise 1 then the value written by [writeValue]: a Rust `Option`. */
    fun <T : Any> option(value: T?, writeValue: (T) -> Unit) {
        bool(value != null)
        value?.let(writeValue)
    }

    private fun length(size: Int) = u64(size.toULong())
}
