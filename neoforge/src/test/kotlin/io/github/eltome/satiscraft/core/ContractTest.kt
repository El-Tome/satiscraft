package io.github.eltome.satiscraft.core

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.assertDoesNotThrow
import org.junit.jupiter.api.assertThrows

class ContractTest {
    @Test
    fun `check accepts the expected version`() {
        assertDoesNotThrow { Contract.check(Contract.VERSION) }
    }

    @Test
    fun `check rejects any other version`() {
        assertThrows<FactoryCoreException> { Contract.check(Contract.VERSION + 1) }
    }
}
