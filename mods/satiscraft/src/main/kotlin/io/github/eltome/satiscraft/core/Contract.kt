package io.github.eltome.satiscraft.core

/** Command/effect contract spoken with factory-core. */
object Contract {
    /** Mirrors `factory_core::CONTRACT_VERSION`: bump both sides together. */
    const val VERSION: Int = 1

    /** Throws [FactoryCoreException] unless [nativeVersion] is the version this mod speaks. */
    fun check(nativeVersion: Int) {
        if (nativeVersion != VERSION) {
            throw FactoryCoreException(
                "factory-core speaks contract v$nativeVersion but this mod expects v$VERSION",
            )
        }
    }
}
