package io.github.eltome.satiscraft.gradle

/** Rust target triple of the machine running Gradle. */
fun hostRustTarget(osName: String, osArch: String): String {
    val arch = when (osArch) {
        "amd64", "x86_64" -> "x86_64"
        "aarch64", "arm64" -> "aarch64"
        else -> error("Unsupported architecture: $osArch")
    }
    return when {
        osName.startsWith("Linux") -> "$arch-unknown-linux-gnu"
        osName.startsWith("Mac") -> "$arch-apple-darwin"
        osName.startsWith("Windows") -> "$arch-pc-windows-msvc"
        else -> error("Unsupported OS: $osName")
    }
}

/** Resource folder name for a target triple; must match NativeLoader.platform(). */
fun nativePlatform(target: String): String {
    val arch = target.substringBefore('-')
    return when {
        "linux" in target -> "linux-$arch"
        "apple-darwin" in target -> "macos-$arch"
        "windows" in target -> "windows-$arch"
        else -> error("Unsupported Rust target: $target")
    }
}

/** File name cargo gives the factory-jni cdylib on a target triple. */
fun nativeLibraryFileName(target: String): String = when {
    "windows" in target -> "factory_jni.dll"
    "apple-darwin" in target -> "libfactory_jni.dylib"
    else -> "libfactory_jni.so"
}
