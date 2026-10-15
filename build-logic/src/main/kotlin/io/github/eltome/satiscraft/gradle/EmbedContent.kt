package io.github.eltome.satiscraft.gradle

import org.gradle.api.DefaultTask
import org.gradle.api.file.ConfigurableFileCollection
import org.gradle.api.file.DirectoryProperty
import org.gradle.api.tasks.CacheableTask
import org.gradle.api.tasks.InputFiles
import org.gradle.api.tasks.OutputDirectory
import org.gradle.api.tasks.PathSensitive
import org.gradle.api.tasks.PathSensitivity
import org.gradle.api.tasks.TaskAction

/**
 * Stages the engine's content files as jar resources under `content/`, with `content/index.txt`
 * listing their names: a jar cannot list its own resources, so the mod reads the index to know
 * which defaults to pass to the core.
 */
@CacheableTask
abstract class EmbedContent : DefaultTask() {
    /** The content files, `crates/factory-core/content/` *.toml. */
    @get:InputFiles
    @get:PathSensitive(PathSensitivity.NAME_ONLY)
    abstract val files: ConfigurableFileCollection

    /** Where the resources go: the files land in its `content/` folder. */
    @get:OutputDirectory
    abstract val outputDirectory: DirectoryProperty

    @TaskAction
    fun embed() {
        val root = outputDirectory.get().asFile
        root.deleteRecursively()
        val target = root.resolve("content")
        target.mkdirs()
        val sources = files.files
        sources.forEach { it.copyTo(target.resolve(it.name)) }
        target.resolve("index.txt").writeText(contentIndex(sources.map { it.name }))
    }
}

/** The index of the embedded content: one file name per line, sorted. */
fun contentIndex(names: Collection<String>): String {
    require(names.toSet().size == names.size) { "Two content files share a name: $names" }
    return names.sorted().joinToString("") { "$it\n" }
}
