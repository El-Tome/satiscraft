package io.github.eltome.satiscraft.core

import io.github.eltome.satiscraft.core.contract.ContentFile
import io.github.eltome.satiscraft.core.contract.ContentFiles
import java.nio.file.Path

/** The engine's content embedded in the jar, without any server override: what tests run on. */
val shippedContent: ContentFiles by lazy { EngineContent.read(Path.of("no-server-folder")) }

/** The shipped content with [overrides] on top, each as `config/satiscraft/<name>`. */
fun shippedContentWith(vararg overrides: Pair<String, String>): ContentFiles =
    ContentFiles(shippedContent.files + overrides.map { (name, text) -> ContentFile("config/satiscraft/$name", text) })
