package io.github.eltome.satiscraft.core

import io.github.eltome.satiscraft.core.contract.ContentFile
import io.github.eltome.satiscraft.core.contract.ContentFiles
import java.io.IOException
import java.nio.file.Path
import kotlin.io.path.isRegularFile
import kotlin.io.path.readText

/**
 * Gathers the files the engine's simulation runs on: the defaults embedded in the jar under
 * `content/`, then the server's overrides of the same files from `config/satiscraft/`.
 *
 * The core merges and checks them; its errors cite the paths given here. Other files in
 * `config/satiscraft/` belong to the other mods (`worldgen.toml`, `vanilla.toml`).
 */
object EngineContent {
    /** Folder of the embedded defaults in the jar, listed by its `index.txt` (built by `embedContent`). */
    private const val EMBEDDED = "content"

    /** Folder of a server's overrides, relative to the server folder. */
    const val OVERRIDES = "config/satiscraft"

    /** The embedded defaults, then the overrides found in [serverDirectory]. */
    fun read(serverDirectory: Path): ContentFiles = gather(::resource, serverDirectory)

    /**
     * The defaults read with [embedded], which returns the text of a jar resource or null, then
     * the overrides found in [serverDirectory].
     *
     * @throws IOException when an override cannot be read.
     */
    internal fun gather(embedded: (String) -> String?, serverDirectory: Path): ContentFiles {
        val index = "$EMBEDDED/index.txt"
        val names = checkNotNull(embedded(index)) { "$index is missing from the jar" }.lines().filter { it.isNotBlank() }
        val defaults = names.map { name ->
            val path = "$EMBEDDED/$name"
            ContentFile(path, checkNotNull(embedded(path)) { "$path is listed in $index but missing from the jar" })
        }
        val overrides = names.mapNotNull { name ->
            val path = "$OVERRIDES/$name"
            val file = serverDirectory.resolve(path)
            if (file.isRegularFile()) ContentFile(path, readOverride(file, path)) else null
        }
        return ContentFiles(defaults + overrides)
    }

    /** Reads an override as UTF-8; the error names it by [path]. */
    private fun readOverride(file: Path, path: String): String = try {
        file.readText()
    } catch (e: IOException) {
        throw IOException("cannot read $path: $e", e)
    }

    /** Text of a resource of this jar, or null. */
    private fun resource(path: String): String? =
        EngineContent::class.java.getResourceAsStream("/$path")?.use { it.readBytes().decodeToString() }
}
