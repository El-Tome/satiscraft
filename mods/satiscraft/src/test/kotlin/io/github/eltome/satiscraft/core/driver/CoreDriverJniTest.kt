package io.github.eltome.satiscraft.core.driver

import io.github.eltome.satiscraft.core.FactoryCore
import io.github.eltome.satiscraft.core.contract.Command
import io.github.eltome.satiscraft.core.contract.CommandResult
import io.github.eltome.satiscraft.core.contract.ContentFiles
import io.github.eltome.satiscraft.core.contract.Effect
import io.github.eltome.satiscraft.core.contract.Rejection
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Test

/** The driver on the real core, through JNI. */
class CoreDriverJniTest {
    private val logger = RecordingLogger()
    private val dispatcher = EffectDispatcher(logger)
    private val results = mutableListOf<CommandResult>()
    private val pongs = mutableListOf<Effect.Pong>()

    init {
        dispatcher.register<Effect.Pong> { pongs += it }
    }

    @Test
    fun `a submitted command produces its effect on the next tick`() {
        CoreDriver(FactoryCore.create(ContentFiles()), dispatcher, logger).use { driver ->
            driver.commands.enqueue(Command.Ping(7u)) { results += it }
            driver.tick(1)
            assertEquals(listOf(CommandResult.Accepted), results)
            assertEquals(listOf(Effect.Pong(7u)), pongs)
            driver.tick(2)
            assertEquals(listOf(Effect.Pong(7u)), pongs)
        }
    }

    @Test
    fun `a panic in the core refuses the tick's commands and halts the driver`() {
        CoreDriver(FactoryCore.create(ContentFiles()), dispatcher, logger).use { driver ->
            driver.commands.enqueue(Command.PanicOnNextStep) { results += it }
            driver.commands.enqueue(Command.Ping(7u)) { results += it }
            driver.tick(1)
            assertTrue(driver.halted)
            assertEquals(List(2) { CommandResult.Rejected(Rejection.Faulted) }, results)
            assertEquals(emptyList<Effect.Pong>(), pongs)
            assertTrue("factory-core panic" in logger.lines.single().error?.message.orEmpty())
        }
    }
}
