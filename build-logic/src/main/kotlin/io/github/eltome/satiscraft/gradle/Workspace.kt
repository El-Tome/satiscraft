package io.github.eltome.satiscraft.gradle

/** Reads `version` from the `[workspace.package]` section of a Cargo manifest. */
fun workspaceVersion(manifest: String): String {
    val section = Regex("""(?ms)^\[workspace\.package]\s*$(.*?)(?=^\[|\z)""").find(manifest)
        ?: error("Cargo.toml has no [workspace.package] section")
    return Regex("""(?m)^version\s*=\s*"([^"]+)"""").find(section.groupValues[1])?.groupValues?.get(1)
        ?: error("Cargo.toml [workspace.package] has no version")
}

/** Mod id of a Gradle project: its name with dashes turned into underscores (`satiscraft-world` -> `satiscraft_world`). */
fun modIdOf(projectName: String): String {
    val modId = projectName.replace('-', '_')
    require(Regex("^[a-z][a-z0-9_]{1,63}$").matches(modId)) { "$projectName does not make a valid mod id" }
    return modId
}
