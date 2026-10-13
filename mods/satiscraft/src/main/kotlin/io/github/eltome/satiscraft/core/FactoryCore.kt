package io.github.eltome.satiscraft.core

import io.github.eltome.satiscraft.core.contract.Command
import io.github.eltome.satiscraft.core.contract.CommandResult
import io.github.eltome.satiscraft.core.contract.ContentFiles
import io.github.eltome.satiscraft.core.contract.Effect
import io.github.eltome.satiscraft.core.contract.Query
import io.github.eltome.satiscraft.core.contract.QueryResult
import io.github.eltome.satiscraft.core.contract.list

/**
 * Handle on one simulation of factory-core, which the JVM only knows by its numeric [id].
 *
 * Call only from the server thread. Every call may throw [FactoryCoreException]. After a
 * panic in [submit] or [tick] the simulation is faulted: every later call throws, [save]
 * included, so a half-modified state is never written. Only [close] still works.
 */
class FactoryCore private constructor(val id: Long) : Simulation {
    private var closed = false

    override fun submit(commands: List<Command>): List<CommandResult> =
        resultBatch.decode(NativeBridge.submit(id, commandBatch.encode(commands)))

    override fun tick(): List<Effect> = effectBatch.decode(NativeBridge.tick(id))

    /** Answers a read-only query. */
    fun query(query: Query): QueryResult = QueryResult.decode(NativeBridge.query(id, Query.encode(query)))

    /** The whole simulated state, opaque to Kotlin, for [load]. */
    fun save(): ByteArray = NativeBridge.save(id)

    /** Frees the simulation, faulted or not. Later calls throw; closing again does nothing. */
    override fun close() {
        if (closed) return
        closed = true
        NativeBridge.shutdown(id)
    }

    companion object {
        private val commandBatch = Command.list()
        private val resultBatch = CommandResult.list()
        private val effectBatch = Effect.list()

        /** A new simulation running on [content]. */
        fun create(content: ContentFiles): FactoryCore = FactoryCore(NativeBridge.init(ContentFiles.encode(content)))

        /** The simulation saved in [save], running on [content]. */
        fun load(content: ContentFiles, save: ByteArray): FactoryCore =
            FactoryCore(NativeBridge.load(ContentFiles.encode(content), save))
    }
}
