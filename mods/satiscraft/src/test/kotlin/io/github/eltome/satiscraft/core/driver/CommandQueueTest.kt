package io.github.eltome.satiscraft.core.driver

import io.github.eltome.satiscraft.core.contract.Command
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.assertThrows
import kotlin.concurrent.thread

class CommandQueueTest {
    @Test
    fun `drain hands commands over in order and empties the queue`() {
        val queue = CommandQueue()
        queue.enqueue(Command.Ping(1u)) {}
        queue.enqueue(Command.Ping(2u)) {}
        assertEquals(listOf(Command.Ping(1u), Command.Ping(2u)), queue.drain().map { it.command })
        assertEquals(emptyList<PendingCommand>(), queue.drain())
    }

    @Test
    fun `only the thread that created the queue may fill it`() {
        val queue = CommandQueue()
        var error: Throwable? = null
        thread { error = runCatching { queue.enqueue(Command.Ping(1u)) {} }.exceptionOrNull() }.join()
        assertThrows<IllegalStateException> { throw error!! }
        assertEquals(emptyList<PendingCommand>(), queue.drain())
    }
}
