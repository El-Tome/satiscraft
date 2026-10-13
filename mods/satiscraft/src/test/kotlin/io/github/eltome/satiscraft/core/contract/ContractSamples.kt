package io.github.eltome.satiscraft.core.contract

import java.util.UUID

/**
 * The values of the golden files, in the same order as the core writes them
 * (`crates/factory-core/src/contract/golden/samples.rs`): change both together.
 */
object ContractSamples {
    val commands = listOf(
        Command.Ping(0u),
        Command.Ping(127u),
        Command.Ping(128u),
        Command.Ping(ULong.MAX_VALUE),
        Command.PanicOnNextStep,
    )

    val rejections = listOf(
        Rejection.NotAllowed,
        Rejection.NotUnlocked,
        Rejection.NoSpace,
        Rejection.MissingItems,
        Rejection.Faulted,
    )

    val commandResults = listOf(CommandResult.Accepted) + rejections.map { CommandResult.Rejected(it) }

    val effects = listOf(Effect.Pong(0u), Effect.Pong(300u), Effect.Pong(ULong.MAX_VALUE))

    val queries = listOf(Query.Tick)

    val queryResults = listOf(QueryResult.Tick(0u), QueryResult.Tick(16_384u), QueryResult.Tick(ULong.MAX_VALUE))

    val contentFile = listOf(
        ContentFile("config.toml", ""),
        ContentFile("content/items.toml", "[[items]]\nid = \"satiscraft:flint\"\nname = \"Silex taillé\"\n"),
        ContentFile("é/🙂.toml", "x".repeat(130)),
    )

    val contentFiles = listOf(ContentFiles(), ContentFiles(contentFile))

    val itemIds = listOf("satiscraft:bauxite", "minecraft:stick", "my_mod-2.0:block/stone_axe.head").map(::ItemId)

    private val overworld = DimensionId("minecraft:overworld")
    private val nether = DimensionId("minecraft:the_nether")

    val dimensionIds = listOf(overworld, nether)

    val positions = listOf(
        Position(overworld, 0, 0, 0),
        Position(nether, -1, 64, 1),
        Position(overworld, Int.MIN_VALUE, -64, Int.MAX_VALUE),
    )

    val playerIds = listOf(
        "00000000-0000-0000-0000-000000000000",
        "069a79f4-44e9-4026-bf4b-8f2c4137000b",
        "ffffffff-ffff-ffff-ffff-ffffffffffff",
    ).map { PlayerId(UUID.fromString(it)) }

    val teamIds = listOf(TeamId(0u), TeamId(300u), TeamId(ULong.MAX_VALUE))

    val machineIds = listOf(MachineId(1u), MachineId(16_384u), MachineId(ULong.MAX_VALUE))
}
