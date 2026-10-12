package io.github.eltome.satiscraft.core

/**
 * Handle on one simulation of factory-core, which the JVM only knows by its numeric [id].
 *
 * Inputs and outputs are postcard bytes of the contract types. Call only from the server
 * thread. Every call may throw [FactoryCoreException]. After a panic in [submit] or [tick]
 * the simulation is faulted: every later call throws, [save] included, so a half-modified
 * state is never written. Only [close] still works.
 */
class FactoryCore private constructor(val id: Long) : AutoCloseable {
    private var closed = false

    /** Applies a batch of commands and returns one result per command. */
    fun submit(commands: ByteArray): ByteArray = NativeBridge.submit(id, commands)

    /** Steps the simulation once and returns its effects. */
    fun tick(): ByteArray = NativeBridge.tick(id)

    /** Answers a read-only query. */
    fun query(query: ByteArray): ByteArray = NativeBridge.query(id, query)

    /** The whole simulated state, for [load]. */
    fun save(): ByteArray = NativeBridge.save(id)

    /** Frees the simulation, faulted or not. Later calls throw; closing again does nothing. */
    override fun close() {
        if (closed) return
        closed = true
        NativeBridge.shutdown(id)
    }

    companion object {
        /** An empty file list in postcard: the content until the engine loads its TOML. */
        fun noContent(): ByteArray = byteArrayOf(0)

        /** A new simulation running on [content], the postcard `ContentFiles`. */
        fun create(content: ByteArray): FactoryCore = FactoryCore(NativeBridge.init(content))

        /** The simulation saved in [save], running on [content]. */
        fun load(content: ByteArray, save: ByteArray): FactoryCore =
            FactoryCore(NativeBridge.load(content, save))
    }
}
