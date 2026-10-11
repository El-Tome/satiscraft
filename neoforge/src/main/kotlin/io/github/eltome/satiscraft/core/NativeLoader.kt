package io.github.eltome.satiscraft.core

import java.nio.file.Files
import java.nio.file.StandardCopyOption

/**
 * Loads the factory-jni library that the build embeds under `natives/<platform>/`.
 *
 * The library is extracted to a temporary file first, since the JVM cannot load
 * a native library from inside a jar.
 */
object NativeLoader {
    @Volatile
    private var loaded = false

    /** Extracts and loads the library for the current platform, once. */
    @Synchronized
    fun load() {
        if (loaded) return
        val resource = resourcePath(platform(), libraryFileName())
        val stream = NativeLoader::class.java.getResourceAsStream(resource)
            ?: throw FactoryCoreException("No factory-core build for this platform: $resource is missing from the mod jar")
        val file = Files.createTempFile("satiscraft-", "-" + resource.substringAfterLast('/'))
        file.toFile().deleteOnExit()
        stream.use { Files.copy(it, file, StandardCopyOption.REPLACE_EXISTING) }
        System.load(file.toAbsolutePath().toString())
        loaded = true
    }

    /** Resource folder name of a platform, e.g. `linux-x86_64`; must match `nativePlatform` in the build script. */
    fun platform(
        osName: String = System.getProperty("os.name"),
        osArch: String = System.getProperty("os.arch"),
    ): String {
        val arch = when (osArch) {
            "amd64", "x86_64" -> "x86_64"
            "aarch64", "arm64" -> "aarch64"
            else -> throw FactoryCoreException("Unsupported architecture: $osArch")
        }
        val os = when {
            osName.startsWith("Linux") -> "linux"
            osName.startsWith("Mac") -> "macos"
            osName.startsWith("Windows") -> "windows"
            else -> throw FactoryCoreException("Unsupported OS: $osName")
        }
        return "$os-$arch"
    }

    /** File name of the factory-jni library on an OS. */
    fun libraryFileName(osName: String = System.getProperty("os.name")): String = when {
        osName.startsWith("Windows") -> "factory_jni.dll"
        osName.startsWith("Mac") -> "libfactory_jni.dylib"
        else -> "libfactory_jni.so"
    }

    /** Classpath location of the library for a platform. */
    fun resourcePath(platform: String, fileName: String): String = "/natives/$platform/$fileName"
}
