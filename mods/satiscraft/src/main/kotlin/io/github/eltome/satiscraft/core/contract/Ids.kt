package io.github.eltome.satiscraft.core.contract

import java.nio.ByteBuffer
import java.util.UUID

// Mirror of crates/factory-core/src/contract/ids.rs: how the contract names things without
// any Minecraft type.

/**
 * Checks the resource location rules the core enforces: `namespace:path`, namespace in
 * `[a-z0-9_.-]`, path in `[a-z0-9_.-/]`, neither empty. Throws [IllegalArgumentException].
 */
internal fun requireNamespaced(id: String) {
    require(':' in id) { "identifier `$id` has no namespace (expected `namespace:path`)" }
    val namespace = id.substringBefore(':')
    val path = id.substringAfter(':')
    require(namespace.isNotEmpty() && path.isNotEmpty()) { "identifier `$id` has an empty namespace or path" }
    val invalid = namespace.firstOrNull { !it.isIdChar() } ?: path.firstOrNull { !it.isIdChar() && it != '/' }
    require(invalid == null) { "identifier `$id` contains the invalid character '$invalid'" }
}

private fun Char.isIdChar() = this in 'a'..'z' || this in '0'..'9' || this in "_.-"

/** An item, vanilla or modded: `satiscraft:bauxite`, `minecraft:stick`. */
@JvmInline
value class ItemId(val id: String) {
    init {
        requireNamespaced(id)
    }

    override fun toString() = id

    companion object : Codec<ItemId> {
        override fun write(out: PostcardWriter, value: ItemId) = out.string(value.id)

        override fun read(input: PostcardReader): ItemId {
            val id = input.string()
            return input.validated { ItemId(id) }
        }
    }
}

/** A dimension: `minecraft:overworld`, `minecraft:the_nether`. */
@JvmInline
value class DimensionId(val id: String) {
    init {
        requireNamespaced(id)
    }

    override fun toString() = id

    companion object : Codec<DimensionId> {
        override fun write(out: PostcardWriter, value: DimensionId) = out.string(value.id)

        override fun read(input: PostcardReader): DimensionId {
            val id = input.string()
            return input.validated { DimensionId(id) }
        }
    }
}

/** A block position: a dimension and integer coordinates. */
data class Position(val dimension: DimensionId, val x: Int, val y: Int, val z: Int) {
    companion object : Codec<Position> {
        override fun write(out: PostcardWriter, value: Position) {
            DimensionId.write(out, value.dimension)
            out.i32(value.x)
            out.i32(value.y)
            out.i32(value.z)
        }

        override fun read(input: PostcardReader) =
            Position(DimensionId.read(input), input.i32(), input.i32(), input.i32())
    }
}

/** A player, by UUID: its 16 bytes, most significant first (RFC 4122 order). */
@JvmInline
value class PlayerId(val uuid: UUID) {
    override fun toString() = uuid.toString()

    companion object : Codec<PlayerId> {
        private const val SIZE = 16

        override fun write(out: PostcardWriter, value: PlayerId) {
            val bytes = ByteBuffer.allocate(SIZE)
                .putLong(value.uuid.mostSignificantBits)
                .putLong(value.uuid.leastSignificantBits)
            out.fixedBytes(bytes.array())
        }

        override fun read(input: PostcardReader): PlayerId {
            val bytes = ByteBuffer.wrap(input.fixedBytes(SIZE))
            return PlayerId(UUID(bytes.getLong(), bytes.getLong()))
        }
    }
}

/** A team, numbered by the core. */
@JvmInline
value class TeamId(val value: ULong) {
    companion object : Codec<TeamId> {
        override fun write(out: PostcardWriter, value: TeamId) = out.u64(value.value)

        override fun read(input: PostcardReader) = TeamId(input.u64())
    }
}

/** A machine, numbered by the core: the only handle the platform keeps on simulated state. */
@JvmInline
value class MachineId(val value: ULong) {
    companion object : Codec<MachineId> {
        override fun write(out: PostcardWriter, value: MachineId) = out.u64(value.value)

        override fun read(input: PostcardReader) = MachineId(input.u64())
    }
}
