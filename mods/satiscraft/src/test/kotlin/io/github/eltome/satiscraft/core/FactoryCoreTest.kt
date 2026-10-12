package io.github.eltome.satiscraft.core

import org.junit.jupiter.api.Assertions.assertArrayEquals
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.assertDoesNotThrow
import org.junit.jupiter.api.assertThrows

/**
 * Lifecycle of simulations through the real native library, with raw postcard bytes:
 * a batch is its length then its items, an enum its variant index then its fields.
 */
class FactoryCoreTest {
    private val pingBatch = bytes(1, 0, 42) // [Command::Ping { nonce: 42 }]
    private val panicBatch = bytes(1, 1) // [Command::PanicOnNextStep]
    private val oneAccepted = bytes(1, 0) // [CommandResult::Accepted]
    private val onePong = bytes(1, 0, 42) // [Effect::Pong { nonce: 42 }]
    private val noEffect = bytes(0)
    private val tickQuery = bytes(0) // Query::Tick

    private fun bytes(vararg values: Int) = ByteArray(values.size) { values[it].toByte() }

    /** QueryResult::Tick(ticks), for a tick count under 128 (one varint byte). */
    private fun tickAnswer(ticks: Int) = bytes(0, ticks)

    @Test
    fun `a command comes back as an effect on the next tick`() {
        FactoryCore.create(FactoryCore.noContent()).use { core ->
            assertArrayEquals(oneAccepted, core.submit(pingBatch))
            assertArrayEquals(onePong, core.tick())
            assertArrayEquals(noEffect, core.tick())
            assertArrayEquals(tickAnswer(2), core.query(tickQuery))
        }
    }

    @Test
    fun `load resumes a saved simulation`() {
        val saved = FactoryCore.create(FactoryCore.noContent()).use { core ->
            core.tick()
            core.submit(pingBatch)
            core.save()
        }
        FactoryCore.load(FactoryCore.noContent(), saved).use { core ->
            assertArrayEquals(tickAnswer(1), core.query(tickQuery))
            assertArrayEquals(onePong, core.tick())
        }
    }

    @Test
    fun `a closed simulation is gone and its id never comes back`() {
        val first = FactoryCore.create(FactoryCore.noContent())
        first.close()
        assertDoesNotThrow { first.close() }
        FactoryCore.create(FactoryCore.noContent()).use { second ->
            assertTrue(second.id > first.id)
            val error = assertThrows<FactoryCoreException> { first.tick() }
            assertTrue("no simulation ${first.id}" in error.message!!, error.message)
        }
    }

    @Test
    fun `unknown ids raise`() {
        assertThrows<FactoryCoreException> { NativeBridge.tick(-1) }
        assertThrows<FactoryCoreException> { NativeBridge.save(0) }
        assertThrows<FactoryCoreException> { NativeBridge.shutdown(Long.MAX_VALUE) }
    }

    @Test
    fun `malformed bytes raise without faulting`() {
        assertThrows<FactoryCoreException> { FactoryCore.create(bytes()) }
        assertThrows<FactoryCoreException> { FactoryCore.load(FactoryCore.noContent(), bytes(0xff)) }
        FactoryCore.create(FactoryCore.noContent()).use { core ->
            val error = assertThrows<FactoryCoreException> { core.submit(bytes(1)) }
            assertTrue("commands" in error.message!!, error.message)
            assertArrayEquals(noEffect, core.tick())
        }
    }

    @Test
    fun `a panic in tick faults the simulation and spares the others`() {
        FactoryCore.create(FactoryCore.noContent()).use { other ->
            FactoryCore.create(FactoryCore.noContent()).use { core ->
                assertArrayEquals(oneAccepted, core.submit(panicBatch))
                val panic = assertThrows<FactoryCoreException> { core.tick() }
                assertTrue("factory-core panic" in panic.message!!, panic.message)
                val calls = listOf({ core.submit(pingBatch) }, { core.tick() }, { core.query(tickQuery) }, { core.save() })
                for (call in calls) {
                    val error = assertThrows<FactoryCoreException> { call() }
                    assertTrue("simulation ${core.id} is faulted" in error.message!!, error.message)
                }
                assertArrayEquals(noEffect, other.tick())
            }
        }
    }
}
