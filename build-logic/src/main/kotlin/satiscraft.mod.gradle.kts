import io.github.eltome.satiscraft.gradle.modIdOf
import io.github.eltome.satiscraft.gradle.workspaceVersion
import org.jetbrains.kotlin.gradle.dsl.JvmTarget

// Setup shared by every Satiscraft mod: toolchains, Kotlin for Forge, NeoForge, metadata
// expanded from src/main/templates, and the single version read from the Cargo workspace.
// The mod id is the project name with underscores (mods/satiscraft-world -> satiscraft_world).

plugins {
    `java-library`
    idea
    id("net.neoforged.moddev")
    kotlin("jvm")
}

/** Reads a required entry of the root gradle.properties. */
fun requiredProperty(name: String): String = providers.gradleProperty(name).get()

val javaVersion = requiredProperty("java_version")
val neoVersion = requiredProperty("neo_version")
val kffVersion = requiredProperty("kff_version")
val modId = modIdOf(project.name)
val repoRoot: Directory = rootProject.layout.projectDirectory

// The Cargo workspace owns the version, common to all mods: one bump per PR, in Cargo.toml only.
val modVersion: String = providers.fileContents(repoRoot.file("Cargo.toml")).asText.map(::workspaceVersion).get()

version = modVersion
group = requiredProperty("mod_group_id")

base {
    archivesName = project.name
}

java.toolchain.languageVersion = JavaLanguageVersion.of(javaVersion.toInt())

kotlin {
    jvmToolchain(javaVersion.toInt())
    compilerOptions.jvmTarget = JvmTarget.fromTarget(javaVersion)
}

repositories {
    maven("https://thedarkcolour.github.io/KotlinForForge/") {
        name = "Kotlin for Forge"
        content { includeGroup("thedarkcolour") }
    }
    mavenCentral()
}

dependencies {
    implementation("thedarkcolour:kotlinforforge-neoforge:$kffVersion")

    testImplementation(platform("org.junit:junit-bom:${requiredProperty("junit_version")}"))
    testImplementation("org.junit.jupiter:junit-jupiter")
    testRuntimeOnly("org.junit.platform:junit-platform-launcher")
}

neoForge {
    version = neoVersion

    mods {
        register(modId) {
            sourceSet(sourceSets.main.get())
        }
    }

    runs.configureEach {
        systemProperty("forge.logging.markers", "REGISTRIES")
        logLevel = org.slf4j.event.Level.DEBUG
    }
}

// neoforge.mods.toml is expanded from src/main/templates.
val generateModMetadata = tasks.register<ProcessResources>("generateModMetadata") {
    val replaceProperties = mapOf(
        "minecraft_version" to requiredProperty("minecraft_version"),
        "minecraft_version_range" to requiredProperty("minecraft_version_range"),
        "neo_version" to neoVersion,
        "kff_version" to kffVersion,
        "mod_id" to modId,
        "mod_license" to requiredProperty("mod_license"),
        "mod_version" to modVersion,
        "mod_authors" to requiredProperty("mod_authors"),
    )
    inputs.properties(replaceProperties)
    expand(replaceProperties)
    from("src/main/templates")
    into(layout.buildDirectory.dir("generated/sources/modMetadata"))
}

sourceSets.main {
    resources {
        srcDir("src/generated/resources")
        srcDir(generateModMetadata)
        exclude("**/*.bbmodel")
        exclude("src/generated/**/.cache")
    }
}

neoForge.ideSyncTask(generateModMetadata)

tasks.test {
    useJUnitPlatform()
}

// The license requires its terms and notice to travel with every copy.
tasks.jar {
    from(repoRoot.file("LICENSE.md"))
}

tasks.withType<JavaCompile>().configureEach {
    options.encoding = "UTF-8"
}

idea {
    module {
        isDownloadSources = true
        isDownloadJavadoc = true
    }
}
