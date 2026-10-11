pluginManagement {
    repositories {
        gradlePluginPortal()
    }
    plugins {
        id("net.neoforged.moddev") version providers.gradleProperty("moddev_version").get()
        kotlin("jvm") version providers.gradleProperty("kotlin_version").get()
    }
}

plugins {
    id("org.gradle.toolchains.foojay-resolver-convention") version "1.0.0"
}

rootProject.name = "satiscraft"

include("neoforge")
