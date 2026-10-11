package io.github.eltome.satiscraft.core

/**
 * Raised for any error or panic coming out of factory-core.
 *
 * Thrown from Rust by the `ThrowFactoryCoreException` policy of factory-jni, which
 * relies on this exact class name and on the `(String)` constructor.
 */
class FactoryCoreException(message: String) : RuntimeException(message)
