package io.github.eltome.satiscraft.core.contract

// Mirror of crates/factory-core/src/contract/messages.rs: commands in, results and effects
// out, queries for display. Variant indices are positional: append, never reorder, and keep
// write and read in step. The golden tests check them against the core.

/** A request to the core, validated and applied by `submit`. */
sealed interface Command {
    /** Diagnostic round trip: always accepted, answered by [Effect.Pong] on the next tick. */
    data class Ping(val nonce: ULong) : Command

    /** Diagnostic fault injection: always accepted, then the next tick panics. */
    data object PanicOnNextStep : Command

    companion object : Codec<Command> {
        override fun write(out: PostcardWriter, value: Command) = when (value) {
            is Ping -> {
                out.variant(0)
                out.u64(value.nonce)
            }
            PanicOnNextStep -> out.variant(1)
        }

        override fun read(input: PostcardReader): Command = when (val index = input.variant()) {
            0 -> Ping(input.u64())
            1 -> PanicOnNextStep
            else -> throw input.unknownVariant("Command", index)
        }
    }
}

/** The core's answer to one command, at the same index as the command in the batch. */
sealed interface CommandResult {
    /** Applied: its consequences come out as effects of the next tick. */
    data object Accepted : CommandResult

    /** Refused, with nothing changed. */
    data class Rejected(val reason: Rejection) : CommandResult

    companion object : Codec<CommandResult> {
        override fun write(out: PostcardWriter, value: CommandResult) = when (value) {
            Accepted -> out.variant(0)
            is Rejected -> {
                out.variant(1)
                Rejection.write(out, value.reason)
            }
        }

        override fun read(input: PostcardReader): CommandResult = when (val index = input.variant()) {
            0 -> Accepted
            1 -> Rejected(Rejection.read(input))
            else -> throw input.unknownVariant("CommandResult", index)
        }
    }
}

/** Why the core refused a command; the platform turns it into player-facing text. Encoded by ordinal. */
enum class Rejection {
    /** The player lacks the right: block of another team, role too low. */
    NotAllowed,

    /** The team has not unlocked the tier, blueprint or recipe yet. */
    NotUnlocked,

    /** No room at the destination. */
    NoSpace,

    /** The source does not hold the items. */
    MissingItems,

    /** The simulation faulted while handling the batch; also answered by Kotlin when a core call fails. */
    Faulted,
    ;

    companion object : Codec<Rejection> {
        override fun write(out: PostcardWriter, value: Rejection) = out.variant(value.ordinal)

        override fun read(input: PostcardReader): Rejection {
            val index = input.variant()
            return entries.getOrNull(index) ?: throw input.unknownVariant("Rejection", index)
        }
    }
}

/** A change the platform must apply to the world or show to players. */
sealed interface Effect {
    /** Answer to [Command.Ping]. */
    data class Pong(val nonce: ULong) : Effect

    companion object : Codec<Effect> {
        override fun write(out: PostcardWriter, value: Effect) = when (value) {
            is Pong -> {
                out.variant(0)
                out.u64(value.nonce)
            }
        }

        override fun read(input: PostcardReader): Effect = when (val index = input.variant()) {
            0 -> Pong(input.u64())
            else -> throw input.unknownVariant("Effect", index)
        }
    }
}

/** A read-only question, for GUIs and rendering. */
sealed interface Query {
    /** How many ticks the simulation has stepped since it was created. */
    data object Tick : Query

    companion object : Codec<Query> {
        override fun write(out: PostcardWriter, value: Query) = when (value) {
            Tick -> out.variant(0)
        }

        override fun read(input: PostcardReader): Query = when (val index = input.variant()) {
            0 -> Tick
            else -> throw input.unknownVariant("Query", index)
        }
    }
}

/** The answer to a [Query], in the variant of the same name. */
sealed interface QueryResult {
    /** Answer to [Query.Tick]. */
    data class Tick(val ticks: ULong) : QueryResult

    companion object : Codec<QueryResult> {
        override fun write(out: PostcardWriter, value: QueryResult) = when (value) {
            is Tick -> {
                out.variant(0)
                out.u64(value.ticks)
            }
        }

        override fun read(input: PostcardReader): QueryResult = when (val index = input.variant()) {
            0 -> Tick(input.u64())
            else -> throw input.unknownVariant("QueryResult", index)
        }
    }
}
