package io.github.eltome.satiscraft.core

/**
 * JNI entry points exported by factory-jni (`crates/factory-jni/src/exports.rs`).
 *
 * Call only from the server thread. Every call may throw [FactoryCoreException].
 * Simulations are reached by a numeric id, never a pointer: prefer the [FactoryCore] handle,
 * which encodes and decodes the bytes with the contract codecs of
 * [io.github.eltome.satiscraft.core.contract].
 */
object NativeBridge {
    init {
        NativeLoader.load()
    }

    /** Version of the loaded factory-core, equal to the mod version. */
    @JvmStatic
    external fun coreVersion(): String

    /** Contract version spoken by the loaded factory-core. */
    @JvmStatic
    external fun contractVersion(): Int

    /** Creates a simulation running on [content] (`ContentFiles`) and returns its id. */
    @JvmStatic
    external fun init(content: ByteArray): Long

    /** Rebuilds a simulation from [save], running on [content], and returns its id. */
    @JvmStatic
    external fun load(content: ByteArray, save: ByteArray): Long

    /** Applies a batch of commands (`Vec<Command>`) and returns one `CommandResult` per command. */
    @JvmStatic
    external fun submit(id: Long, commands: ByteArray): ByteArray

    /** Steps the simulation once and returns its effects (`Vec<Effect>`). */
    @JvmStatic
    external fun tick(id: Long): ByteArray

    /** Answers a read-only `Query` with a `QueryResult`. */
    @JvmStatic
    external fun query(id: Long, query: ByteArray): ByteArray

    /** The whole simulated state; throws once the simulation is faulted. */
    @JvmStatic
    external fun save(id: Long): ByteArray

    /** Frees the simulation, faulted or not. Its id is never handed out again. */
    @JvmStatic
    external fun shutdown(id: Long)

    /**
     * Decodes [bytes] as the contract type named [type] (`command`, `command_batch`,
     * `position`...) and encodes it again, for checking the Kotlin codec against the core's.
     */
    @JvmStatic
    external fun reencode(type: String, bytes: ByteArray): ByteArray

    /** Throws [FactoryCoreException] if the loaded core speaks another contract version. */
    fun ensureCompatible() = Contract.check(contractVersion())
}
