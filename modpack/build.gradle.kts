// The modpack assembles the three mods: here, dev runs of the whole Stone Age experience.
// The mods load from their jars, as on a real server.

plugins {
    java
    id("net.neoforged.moddev")
}

java.toolchain.languageVersion = JavaLanguageVersion.of(providers.gradleProperty("java_version").get().toInt())

// Dependency mods a server needs next to the three jars, staged for scripts/deploy-test.sh.
val serverMods = configurations.create("serverMods") {
    isTransitive = false
}

repositories {
    maven("https://thedarkcolour.github.io/KotlinForForge/") {
        name = "Kotlin for Forge"
        content { includeGroup("thedarkcolour") }
    }
    mavenCentral()
}

dependencies {
    runtimeOnly(project(":mods:satiscraft"))
    runtimeOnly(project(":mods:satiscraft-world"))
    runtimeOnly(project(":mods:satiscraft-primitive"))

    serverMods("thedarkcolour:kotlinforforge-neoforge:${providers.gradleProperty("kff_version").get()}")
}

neoForge {
    version = providers.gradleProperty("neo_version").get()

    runs {
        register("client") {
            client()
        }
        register("server") {
            server()
            programArgument("--nogui")
        }
        configureEach {
            logLevel = org.slf4j.event.Level.INFO
        }
    }
}

// The modpack ships data, not code.
tasks.jar {
    enabled = false
}

val stageServerMods = tasks.register<Sync>("serverMods") {
    description = "Stages the dependency mods of the server in build/serverMods."
    from(serverMods)
    into(layout.buildDirectory.dir("serverMods"))
}

tasks.assemble {
    dependsOn(stageServerMods)
}
