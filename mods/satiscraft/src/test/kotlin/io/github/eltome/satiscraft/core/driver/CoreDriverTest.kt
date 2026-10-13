package io.github.eltome.satiscraft.core.driver

import io.github.eltome.satiscraft.core.FactoryCoreException
import io.github.eltome.satiscraft.core.contract.Command
import io.github.eltome.satiscraft.core.contract.CommandResult
import io.github.eltome.satiscraft.core.contract.Effect
import io.github.eltome.satiscraft.core.contract.Rejection
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertFalse
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Test
import org.slf4j.event.Level

/** The driver against a fake simulation. */
class CoreDriverTest {
    private val simulation = FakeSimulation()
    private val logger = RecordingLogger()
    private val dispatcher = EffectDispatcher(logger)
    private val driver = CoreDriver(simulation, dispatcher, logger)

    /** What happened during a tick, in order: results as `command -> result`, then effects. */
    private val events = mutableListOf<String>()

    private val faulted = CommandResult.Rejected(Rejection.Faulted)

    private fun enqueue(nonce: Int) = driver.commands.enqueue(Command.Ping(nonce.toULong())) { events += "$nonce -> $it" }

    @Test
    fun `the commands of a tick go in one submit, then one step`() {
        enqueue(1)
        enqueue(2)
        driver.tick(1)
        assertEquals(listOf(listOf(Command.Ping(1u), Command.Ping(2u))), simulation.batches)
        assertEquals(1, simulation.ticks)
    }

    @Test
    fun `a tick without commands skips submit but still steps`() {
        driver.tick(1)
        assertEquals(emptyList<List<Command>>(), simulation.batches)
        assertEquals(1, simulation.ticks)
    }

    @Test
    fun `each handler gets its own result, then the effects are dispatched`() {
        dispatcher.register<Effect.Pong> { events += "effect $it" }
        simulation.answer = { listOf(CommandResult.Accepted, CommandResult.Rejected(Rejection.NoSpace)) }
        simulation.effects = { listOf(Effect.Pong(1u)) }
        enqueue(1)
        enqueue(2)
        driver.tick(1)
        assertEquals(listOf("1 -> Accepted", "2 -> Rejected(reason=NoSpace)", "effect Pong(nonce=1)"), events)
    }

    @Test
    fun `a failing submit refuses the whole batch and halts the simulation`() {
        simulation.answer = { throw FactoryCoreException("simulation 1 is faulted") }
        enqueue(1)
        enqueue(2)
        driver.tick(42)
        assertTrue(driver.halted)
        assertEquals(listOf("1 -> $faulted", "2 -> $faulted"), events)
        assertEquals(0, simulation.ticks)
        val line = logger.lines.single()
        assertEquals(Level.ERROR, line.level)
        assertTrue(line.message.startsWith("factory-core failed at server tick 42 with 2 queued commands [Ping(nonce=1), Ping(nonce=2)]"))
        assertEquals("simulation 1 is faulted", line.error?.message)
    }

    @Test
    fun `a failing step refuses the commands it had accepted`() {
        dispatcher.register<Effect.Pong> { events += "effect $it" }
        simulation.effects = { throw FactoryCoreException("factory-core panic: boom") }
        enqueue(1)
        driver.tick(1)
        assertTrue(driver.halted)
        assertEquals(listOf("1 -> $faulted"), events)
    }

    @Test
    fun `a halted driver no longer calls the simulation and refuses every command`() {
        simulation.answer = { throw FactoryCoreException("boom") }
        enqueue(1)
        driver.tick(1)
        enqueue(2)
        driver.tick(2)
        driver.tick(3)
        assertEquals(1, simulation.batches.size)
        assertEquals(0, simulation.ticks)
        assertEquals(listOf("1 -> $faulted", "2 -> $faulted"), events)
        assertEquals(1, logger.lines.size)
    }

    @Test
    fun `a wrong number of results halts the simulation`() {
        simulation.answer = { emptyList() }
        enqueue(1)
        driver.tick(1)
        assertTrue(driver.halted)
        assertEquals(listOf("1 -> $faulted"), events)
        assertEquals("factory-core answered 0 results to 1 commands", logger.lines.single().error?.message)
    }

    @Test
    fun `a failing result handler is logged and the tick goes on`() {
        dispatcher.register<Effect.Pong> { events += "effect $it" }
        simulation.effects = { listOf(Effect.Pong(9u)) }
        driver.commands.enqueue(Command.Ping(1u)) { error("boom") }
        enqueue(2)
        driver.tick(1)
        assertFalse(driver.halted)
        assertEquals(listOf("2 -> Accepted", "effect Pong(nonce=9)"), events)
        val line = logger.lines.single()
        assertEquals("The handler of factory-core command Ping(nonce=1) failed on Accepted", line.message)
    }

    @Test
    fun `close frees the simulation`() {
        driver.close()
        assertTrue(simulation.closed)
    }
}
