package io.github.eltome.satiscraft.gradle

import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.assertThrows

class RustTargetsTest {
    @Test
    fun `maps the host to its target triple`() {
        assertEquals("x86_64-unknown-linux-gnu", hostRustTarget("Linux", "amd64"))
        assertEquals("aarch64-apple-darwin", hostRustTarget("Mac OS X", "aarch64"))
        assertEquals("x86_64-pc-windows-msvc", hostRustTarget("Windows 11", "x86_64"))
    }

    @Test
    fun `rejects unknown hosts`() {
        assertThrows<IllegalStateException> { hostRustTarget("Linux", "riscv64") }
        assertThrows<IllegalStateException> { hostRustTarget("FreeBSD", "amd64") }
    }

    @Test
    fun `names the resource folder of each target`() {
        assertEquals("linux-x86_64", nativePlatform("x86_64-unknown-linux-gnu"))
        assertEquals("macos-aarch64", nativePlatform("aarch64-apple-darwin"))
        assertEquals("windows-x86_64", nativePlatform("x86_64-pc-windows-msvc"))
        assertThrows<IllegalStateException> { nativePlatform("wasm32-unknown-unknown") }
    }

    @Test
    fun `names the library file of each target`() {
        assertEquals("libfactory_jni.so", nativeLibraryFileName("x86_64-unknown-linux-gnu"))
        assertEquals("libfactory_jni.dylib", nativeLibraryFileName("aarch64-apple-darwin"))
        assertEquals("factory_jni.dll", nativeLibraryFileName("x86_64-pc-windows-msvc"))
    }
}
