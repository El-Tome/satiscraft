package io.github.eltome.satiscraft.core

import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.assertDoesNotThrow

/** Round trip through the real native library built by cargo. */
class NativeBridgeTest {
    @Test
    fun `coreVersion is the mod version`() {
        assertEquals(System.getProperty("satiscraft.expectedCoreVersion"), NativeBridge.coreVersion())
    }

    @Test
    fun `contractVersion matches the Kotlin contract`() {
        assertEquals(Contract.VERSION, NativeBridge.contractVersion())
    }

    @Test
    fun `ensureCompatible accepts the bundled core`() {
        assertDoesNotThrow { NativeBridge.ensureCompatible() }
    }
}
