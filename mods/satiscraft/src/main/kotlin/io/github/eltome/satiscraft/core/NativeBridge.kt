package io.github.eltome.satiscraft.core

/**
 * JNI entry points exported by factory-jni (`crates/factory-jni/src/exports.rs`).
 *
 * Call only from the server thread. Every call may throw [FactoryCoreException].
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

    /** Throws [FactoryCoreException] if the loaded core speaks another contract version. */
    fun ensureCompatible() = Contract.check(contractVersion())
}
