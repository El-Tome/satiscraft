package io.github.eltome.satiscraft.core.driver

import io.github.eltome.satiscraft.core.Simulation
import io.github.eltome.satiscraft.core.contract.Command
import io.github.eltome.satiscraft.core.contract.CommandResult
import io.github.eltome.satiscraft.core.contract.Effect
import org.slf4j.Marker
import org.slf4j.event.Level
import org.slf4j.helpers.LegacyAbstractLogger
import org.slf4j.helpers.MessageFormatter

/** One logged line: its level, its formatted message, and the exception it carried. */
data class LogLine(val level: Level, val message: String, val error: Throwable?)

/** A logger that keeps what it is given, for assertions. */
class RecordingLogger : LegacyAbstractLogger() {
    val lines = mutableListOf<LogLine>()

    override fun getName() = "test"

    override fun isTraceEnabled() = true

    override fun isDebugEnabled() = true

    override fun isInfoEnabled() = true

    override fun isWarnEnabled() = true

    override fun isErrorEnabled() = true

    override fun getFullyQualifiedCallerName(): String? = null

    override fun handleNormalizedLoggingCall(
        level: Level,
        marker: Marker?,
        messagePattern: String?,
        arguments: Array<out Any?>?,
        throwable: Throwable?,
    ) {
        lines += LogLine(level, MessageFormatter.basicArrayFormat(messagePattern, arguments), throwable)
    }
}

/** A simulation scripted by the test: it records each batch and answers what it is told. */
class FakeSimulation : Simulation {
    val batches = mutableListOf<List<Command>>()
    var ticks = 0
        private set
    var closed = false
        private set

    /** The results of a batch: all accepted unless the test says otherwise. */
    var answer: (List<Command>) -> List<CommandResult> = { batch -> batch.map { CommandResult.Accepted } }

    /** The effects of each tick. */
    var effects: () -> List<Effect> = { emptyList() }

    override fun submit(commands: List<Command>): List<CommandResult> {
        batches += commands
        return answer(commands)
    }

    override fun tick(): List<Effect> {
        ticks++
        return effects()
    }

    override fun close() {
        closed = true
    }
}
