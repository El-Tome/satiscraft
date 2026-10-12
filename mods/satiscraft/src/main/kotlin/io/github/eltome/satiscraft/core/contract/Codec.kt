package io.github.eltome.satiscraft.core.contract

/**
 * How one contract type is written and read in postcard. The companion object of each
 * contract type implements it, so that `Command.encode(command)` reads like the Rust side.
 */
interface Codec<T> {
    fun write(out: PostcardWriter, value: T)

    fun read(input: PostcardReader): T

    /** [value] in postcard. */
    fun encode(value: T): ByteArray = PostcardWriter().also { write(it, value) }.toByteArray()

    /** Exactly one value: leftover bytes are an error, as in the core. */
    fun decode(bytes: ByteArray): T {
        val input = PostcardReader(bytes)
        return read(input).also { input.finish() }
    }
}

/** A list of [T], such as the batches that cross JNI: the item count, then each item. */
fun <T> Codec<T>.list(): Codec<List<T>> = ListCodec(this)

private class ListCodec<T>(private val item: Codec<T>) : Codec<List<T>> {
    override fun write(out: PostcardWriter, value: List<T>) = out.sequence(value) { item.write(out, it) }

    override fun read(input: PostcardReader): List<T> = input.sequence { item.read(input) }
}
