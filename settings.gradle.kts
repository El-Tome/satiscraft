pluginManagement {
    includeBuild("build-logic")
    repositories {
        gradlePluginPortal()
    }
}

plugins {
    id("org.gradle.toolchains.foojay-resolver-convention") version "1.0.0"
}

rootProject.name = "satiscraft"

// One Gradle project per mod, all at the version of the Cargo workspace.
include(":mods:satiscraft")
include(":mods:satiscraft-world")
include(":mods:satiscraft-primitive")
include(":modpack")
