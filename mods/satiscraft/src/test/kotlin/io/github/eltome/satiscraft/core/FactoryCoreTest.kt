package io.github.eltome.satiscraft.core

import io.github.eltome.satiscraft.core.contract.Command
import io.github.eltome.satiscraft.core.contract.CommandResult
import io.github.eltome.satiscraft.core.contract.ContentFile
import io.github.eltome.satiscraft.core.contract.ContentFiles
import io.github.eltome.satiscraft.core.contract.Effect
import io.github.eltome.satiscraft.core.contract.Query
import io.github.eltome.satiscraft.core.contract.QueryResult
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.assertDoesNotThrow
import org.junit.jupiter.api.assertThrows

/** Lifecycle of simulations through the real native library. */
class FactoryCoreTest {
    private val ping = listOf(Command.Ping(42u))
    private val pong = listOf(Effect.Pong(42u))

    private fun create() = FactoryCore.create(ContentFiles())

    @Test
    fun `a command comes back as an effect on the next tick`() {
        create().use { core ->
            assertEquals(listOf(CommandResult.Accepted), core.submit(ping))
            assertEquals(pong, core.tick())
            assertEquals(emptyList<Effect>(), core.tick())
            assertEquals(QueryResult.Tick(2u), core.query(Query.Tick))
        }
    }

    @Test
    fun `load resumes a saved simulation`() {
        val saved = create().use { core ->
            core.tick()
            core.submit(ping)
            core.save()
        }
        val content = ContentFiles(listOf(ContentFile("config.toml", "threads = 2")))
        FactoryCore.load(content, saved).use { core ->
            assertEquals(QueryResult.Tick(1u), core.query(Query.Tick))
            assertEquals(pong, core.tick())
        }
    }

    @Test
    fun `a closed simulation is gone and its id never comes back`() {
        val first = create()
        first.close()
        assertDoesNotThrow { first.close() }
        create().use { second ->
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
        assertThrows<FactoryCoreException> { NativeBridge.init(ByteArray(0)) }
        assertThrows<FactoryCoreException> { FactoryCore.load(ContentFiles(), byteArrayOf(-1)) }
        create().use { core ->
            val error = assertThrows<FactoryCoreException> { NativeBridge.submit(core.id, byteArrayOf(1)) }
            assertTrue("commands" in error.message!!, error.message)
            assertEquals(emptyList<Effect>(), core.tick())
        }
    }

    @Test
    fun `a panic in tick faults the simulation and spares the others`() {
        create().use { other ->
            create().use { core ->
                assertEquals(listOf(CommandResult.Accepted), core.submit(listOf(Command.PanicOnNextStep)))
                val panic = assertThrows<FactoryCoreException> { core.tick() }
                assertTrue("factory-core panic" in panic.message!!, panic.message)
                val calls = listOf({ core.submit(ping) }, { core.tick() }, { core.query(Query.Tick) }, { core.save() })
                for (call in calls) {
                    val error = assertThrows<FactoryCoreException> { call() }
                    assertTrue("simulation ${core.id} is faulted" in error.message!!, error.message)
                }
                assertEquals(emptyList<Effect>(), other.tick())
            }
        }
    }
}
