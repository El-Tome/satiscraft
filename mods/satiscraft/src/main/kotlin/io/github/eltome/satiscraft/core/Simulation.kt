package io.github.eltome.satiscraft.core

import io.github.eltome.satiscraft.core.contract.Command
import io.github.eltome.satiscraft.core.contract.CommandResult
import io.github.eltome.satiscraft.core.contract.Effect

/** What the server tick needs of a simulation: [FactoryCore] in game, a fake in tests. */
interface Simulation : AutoCloseable {
    /** Applies a batch of commands and returns one result per command, in order. */
    fun submit(commands: List<Command>): List<CommandResult>

    /** Steps the simulation once and returns its effects. */
    fun tick(): List<Effect>
}
