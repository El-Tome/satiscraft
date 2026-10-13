package io.github.eltome.satiscraft.core.driver

import io.github.eltome.satiscraft.core.FactoryCoreException
import io.github.eltome.satiscraft.core.Simulation
import io.github.eltome.satiscraft.core.contract.CommandResult
import io.github.eltome.satiscraft.core.contract.Effect
import io.github.eltome.satiscraft.core.contract.Rejection
import org.slf4j.Logger

/**
 * Drives a simulation once per server tick: one `submit` of the commands queued during the
 * tick, one `tick` of the simulation, then each command's result to its handler, then the
 * effects to the [effects] dispatcher.
 *
 * Results wait for the step. When either call fails, the simulation is faulted and the batch
 * is lost with its state, so every command of the tick is answered `Rejected(Faulted)` and
 * its handler reverts it. That first failure also halts the driver: it no longer calls the
 * simulation, and answers every later command `Rejected(Faulted)` until the server restarts.
 */
class CoreDriver(
    private val simulation: Simulation,
    private val effects: EffectDispatcher,
    private val logger: Logger,
) : AutoCloseable {
    /** Where event handlers queue their commands during the tick. */
    val commands = CommandQueue()

    /** Whether a failure stopped the simulation for good. */
    var halted = false
        private set

    /** Runs at the end of each server tick, on the server thread; [serverTick] is for the logs. */
    fun tick(serverTick: Int) {
        val batch = commands.drain()
        if (halted) {
            batch.forEach { answer(it, FAULTED) }
            return
        }
        val (results, produced) = try {
            step(batch)
        } catch (e: FactoryCoreException) {
            halted = true
            logger.error(
                "factory-core failed at server tick {} with {} queued commands {}. The simulation is halted " +
                    "until the server restarts: every command is refused and its world change reverted.",
                serverTick, batch.size, batch.map { it.command }, e,
            )
            batch.forEach { answer(it, FAULTED) }
            return
        }
        batch.zip(results).forEach { (pending, result) -> answer(pending, result) }
        effects.dispatch(produced)
    }

    /** Frees the simulation. */
    override fun close() = simulation.close()

    /** Submits [batch], unless it is empty, then steps: the results and the effects. */
    private fun step(batch: List<PendingCommand>): Pair<List<CommandResult>, List<Effect>> {
        val results = if (batch.isEmpty()) emptyList() else simulation.submit(batch.map { it.command })
        if (results.size != batch.size) {
            throw FactoryCoreException("factory-core answered ${results.size} results to ${batch.size} commands")
        }
        return results to simulation.tick()
    }

    /** Hands [result] to the handler of [pending]; a failing handler is logged, not propagated. */
    private fun answer(pending: PendingCommand, result: CommandResult) {
        try {
            pending.onResult(result)
        } catch (e: Exception) {
            logger.error("The handler of factory-core command {} failed on {}", pending.command, result, e)
        }
    }

    private companion object {
        val FAULTED = CommandResult.Rejected(Rejection.Faulted)
    }
}
