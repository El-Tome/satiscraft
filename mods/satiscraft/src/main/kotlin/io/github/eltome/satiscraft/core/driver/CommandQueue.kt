package io.github.eltome.satiscraft.core.driver

import io.github.eltome.satiscraft.core.contract.Command
import io.github.eltome.satiscraft.core.contract.CommandResult

/** A command waiting for the end of the server tick, with the handler that wants its result. */
class PendingCommand(val command: Command, val onResult: (CommandResult) -> Unit)

/**
 * The commands that event handlers gather during a server tick, in order, for one `submit`
 * at its end. Only the thread that created the queue, the server thread, may fill it.
 */
class CommandQueue(private val owner: Thread = Thread.currentThread()) {
    private val pending = ArrayList<PendingCommand>()

    /**
     * Queues [command]. At the end of the tick, [onResult] gets the core's answer. A handler
     * that already changed the world reverts that change when the command is rejected.
     */
    fun enqueue(command: Command, onResult: (CommandResult) -> Unit) {
        check(Thread.currentThread() === owner) { "factory-core commands are queued from the server thread only" }
        pending += PendingCommand(command, onResult)
    }

    /** Everything queued since the last call, in order; the queue is left empty. */
    fun drain(): List<PendingCommand> = pending.toList().also { pending.clear() }
}
