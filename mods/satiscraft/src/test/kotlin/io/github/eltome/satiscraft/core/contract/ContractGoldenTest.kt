package io.github.eltome.satiscraft.core.contract

import io.github.eltome.satiscraft.core.NativeBridge
import org.junit.jupiter.api.Assertions.assertArrayEquals
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.DynamicTest
import org.junit.jupiter.api.DynamicTest.dynamicTest
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.TestFactory
import java.io.File

/**
 * Checks the Kotlin codec against the core's, byte for byte: against the golden files the
 * core writes (`crates/factory-core/golden/`), then live through JNI.
 *
 * A contract change on the Rust side regenerates the golden files, and these tests fail
 * until Kotlin mirrors it.
 */
class ContractGoldenTest {
    /** A contract type: the name of its golden file, its codec, and the values in that file. */
    private class Case<T>(val name: String, val codec: Codec<T>, val samples: List<T>) {
        fun checkGolden(golden: List<ByteArray>) {
            assertEquals(samples.size, golden.size, "$name: one sample per line")
            samples.zip(golden).forEachIndexed { index, (sample, bytes) ->
                assertEquals(sample, codec.decode(bytes), "$name, line $index, decoded")
                assertArrayEquals(bytes, codec.encode(sample), "$name, line $index, encoded")
            }
        }

        fun checkThroughCore() = samples.forEachIndexed { index, sample ->
            val bytes = codec.encode(sample)
            val again = NativeBridge.reencode(name, bytes)
            assertArrayEquals(bytes, again, "$name, sample $index, as the core writes it back")
            assertEquals(sample, codec.decode(again), "$name, sample $index, read back")
        }
    }

    private val cases: List<Case<*>> = with(ContractSamples) {
        listOf(
            Case("command", Command, commands),
            Case("command_batch", Command.list(), listOf(emptyList(), commands)),
            Case("command_result", CommandResult, commandResults),
            Case("command_result_batch", CommandResult.list(), listOf(emptyList(), commandResults)),
            Case("rejection", Rejection, rejections),
            Case("effect", Effect, effects),
            Case("effect_batch", Effect.list(), listOf(emptyList(), effects)),
            Case("query", Query, queries),
            Case("query_result", QueryResult, queryResults),
            Case("content_file", ContentFile, contentFile),
            Case("content_files", ContentFiles, contentFiles),
            Case("item_id", ItemId, itemIds),
            Case("dimension_id", DimensionId, dimensionIds),
            Case("position", Position, positions),
            Case("player_id", PlayerId, playerIds),
            Case("team_id", TeamId, teamIds),
            Case("machine_id", MachineId, machineIds),
        )
    }

    private val goldenDir = File(System.getProperty("satiscraft.goldenDir"))

    /** The bytes of each sample line of a golden file: hex pairs, then a tab and the Rust value. */
    private fun golden(name: String): List<ByteArray> = File(goldenDir, "$name.txt").readLines()
        .filter { it.isNotBlank() && !it.startsWith("#") }
        .map { line ->
            val hex = line.substringBefore('\t').split(' ').filter(String::isNotEmpty)
            ByteArray(hex.size) { hex[it].toInt(16).toByte() }
        }

    @Test
    fun `every golden file has a Kotlin mirror`() {
        val files = goldenDir.list { _, file -> file.endsWith(".txt") }!!.map { it.removeSuffix(".txt") }
        assertEquals(files.toSortedSet(), cases.map { it.name }.toSortedSet())
    }

    @TestFactory
    fun `Kotlin reads and writes the core's golden bytes`(): List<DynamicTest> =
        cases.map { case -> dynamicTest(case.name) { case.checkGolden(golden(case.name)) } }

    @TestFactory
    fun `the core reads Kotlin's bytes and writes them back identical`(): List<DynamicTest> =
        cases.map { case -> dynamicTest(case.name) { case.checkThroughCore() } }
}
