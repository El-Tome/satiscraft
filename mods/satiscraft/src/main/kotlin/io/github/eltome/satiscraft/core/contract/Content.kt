package io.github.eltome.satiscraft.core.contract

// Mirror of crates/factory-core/src/contract/content.rs: the text files a simulation is
// built from.

/**
 * The engine's content and the server config, each file with the path that errors cite: the
 * embedded defaults first, then the server's overrides of files with the same name.
 */
data class ContentFiles(val files: List<ContentFile> = emptyList()) {
    companion object : Codec<ContentFiles> {
        private val fileList = ContentFile.list()

        override fun write(out: PostcardWriter, value: ContentFiles) = fileList.write(out, value.files)

        override fun read(input: PostcardReader) = ContentFiles(fileList.read(input))
    }
}

/** One text file, under the path that errors cite: `content/items.toml`, `config/satiscraft/items.toml`. */
data class ContentFile(val path: String, val text: String) {
    companion object : Codec<ContentFile> {
        override fun write(out: PostcardWriter, value: ContentFile) {
            out.string(value.path)
            out.string(value.text)
        }

        override fun read(input: PostcardReader) = ContentFile(input.string(), input.string())
    }
}
