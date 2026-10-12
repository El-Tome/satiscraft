package io.github.eltome.satiscraft.core.driver

import io.github.eltome.satiscraft.core.contract.Effect
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.assertThrows
import org.slf4j.event.Level

class EffectDispatcherTest {
    private val logger = RecordingLogger()
    private val dispatcher = EffectDispatcher(logger)

    @Test
    fun `each effect goes to the handler of its type, in order`() {
        val received = mutableListOf<Effect.Pong>()
        dispatcher.register<Effect.Pong> { received += it }
        dispatcher.dispatch(listOf(Effect.Pong(1u), Effect.Pong(2u)))
        assertEquals(listOf(Effect.Pong(1u), Effect.Pong(2u)), received)
        assertEquals(emptyList<LogLine>(), logger.lines)
    }

    @Test
    fun `a type takes only one handler`() {
        dispatcher.register<Effect.Pong> {}
        assertThrows<IllegalStateException> { dispatcher.register<Effect.Pong> {} }
    }

    @Test
    fun `an effect without a handler is logged once per type`() {
        dispatcher.dispatch(listOf(Effect.Pong(1u), Effect.Pong(2u)))
        dispatcher.dispatch(listOf(Effect.Pong(3u)))
        assertEquals(1, logger.lines.size)
        assertEquals(Level.WARN, logger.lines[0].level)
        assertEquals("No handler for factory-core effect Pong(nonce=1): dropped, and logged once per type", logger.lines[0].message)
    }

    @Test
    fun `a failing handler is logged and the next effects still go through`() {
        val received = mutableListOf<Effect.Pong>()
        dispatcher.register<Effect.Pong> { pong ->
            check(pong.nonce != 1uL) { "boom" }
            received += pong
        }
        dispatcher.dispatch(listOf(Effect.Pong(1u), Effect.Pong(2u)))
        assertEquals(listOf(Effect.Pong(2u)), received)
        val line = logger.lines.single()
        assertEquals(Level.ERROR, line.level)
        assertEquals("The handler of factory-core effect Pong(nonce=1) failed", line.message)
        assertEquals("boom", line.error?.message)
    }
}
