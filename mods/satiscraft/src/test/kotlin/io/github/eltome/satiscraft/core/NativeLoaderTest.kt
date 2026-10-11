package io.github.eltome.satiscraft.core

import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.assertThrows

class NativeLoaderTest {
    @Test
    fun `platform maps the V1 server target`() {
        assertEquals("linux-x86_64", NativeLoader.platform("Linux", "amd64"))
    }

    @Test
    fun `platform normalizes architecture aliases`() {
        assertEquals("macos-aarch64", NativeLoader.platform("Mac OS X", "aarch64"))
        assertEquals("macos-aarch64", NativeLoader.platform("Mac OS X", "arm64"))
        assertEquals("windows-x86_64", NativeLoader.platform("Windows 11", "x86_64"))
    }

    @Test
    fun `platform rejects unknown systems`() {
        assertThrows<FactoryCoreException> { NativeLoader.platform("Linux", "riscv64") }
        assertThrows<FactoryCoreException> { NativeLoader.platform("FreeBSD", "amd64") }
    }

    @Test
    fun `libraryFileName follows each OS convention`() {
        assertEquals("libfactory_jni.so", NativeLoader.libraryFileName("Linux"))
        assertEquals("libfactory_jni.dylib", NativeLoader.libraryFileName("Mac OS X"))
        assertEquals("factory_jni.dll", NativeLoader.libraryFileName("Windows 11"))
    }

    @Test
    fun `resourcePath points into the natives folder`() {
        assertEquals("/natives/linux-x86_64/libfactory_jni.so", NativeLoader.resourcePath("linux-x86_64", "libfactory_jni.so"))
    }

    @Test
    fun `load is idempotent`() {
        NativeLoader.load()
        NativeLoader.load()
    }
}
