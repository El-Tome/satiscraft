import org.jetbrains.kotlin.gradle.dsl.JvmTarget

plugins {
    `java-library`
    idea
    id("net.neoforged.moddev")
    kotlin("jvm")
}

/** Reads a required entry of gradle.properties. */
fun requiredProperty(name: String): String = providers.gradleProperty(name).get()

val minecraft_version = requiredProperty("minecraft_version")
val minecraft_version_range = requiredProperty("minecraft_version_range")
val neo_version = requiredProperty("neo_version")
val kff_version = requiredProperty("kff_version")
val junit_version = requiredProperty("junit_version")
val java_version = requiredProperty("java_version")
val mod_id = requiredProperty("mod_id")
val mod_name = requiredProperty("mod_name")
val mod_license = requiredProperty("mod_license")
val mod_authors = requiredProperty("mod_authors")
val mod_group_id = requiredProperty("mod_group_id")

val repoRoot: Directory = rootProject.layout.projectDirectory

// The Cargo workspace owns the version: one bump per PR, in Cargo.toml only.
val modVersion: String = providers.fileContents(repoRoot.file("Cargo.toml")).asText.map(::workspaceVersion).get()

version = modVersion
group = mod_group_id

base {
    archivesName = mod_id
}

java.toolchain.languageVersion = JavaLanguageVersion.of(java_version.toInt())

kotlin {
    jvmToolchain(java_version.toInt())
    compilerOptions.jvmTarget = JvmTarget.fromTarget(java_version)
}

repositories {
    maven("https://thedarkcolour.github.io/KotlinForForge/") {
        name = "Kotlin for Forge"
        content { includeGroup("thedarkcolour") }
    }
    mavenCentral()
}

dependencies {
    implementation("thedarkcolour:kotlinforforge-neoforge:$kff_version")

    testImplementation(platform("org.junit:junit-bom:$junit_version"))
    testImplementation("org.junit.jupiter:junit-jupiter")
    testRuntimeOnly("org.junit.platform:junit-platform-launcher")
}

neoForge {
    version = neo_version

    runs {
        register("client") {
            client()
            systemProperty("neoforge.enabledGameTestNamespaces", mod_id)
        }
        register("server") {
            server()
            programArgument("--nogui")
            systemProperty("neoforge.enabledGameTestNamespaces", mod_id)
        }
        register("gameTestServer") {
            type = "gameTestServer"
            systemProperty("neoforge.enabledGameTestNamespaces", mod_id)
        }
        register("data") {
            clientData()
            programArguments.addAll(
                "--mod", mod_id, "--all",
                "--output", file("src/generated/resources/").absolutePath,
                "--existing", file("src/main/resources/").absolutePath,
            )
        }
        configureEach {
            systemProperty("forge.logging.markers", "REGISTRIES")
            logLevel = org.slf4j.event.Level.DEBUG
        }
    }

    mods {
        register(mod_id) {
            sourceSet(sourceSets.main.get())
        }
    }
}

// ---------------------------------------------------------------------------
// Rust native library: Gradle drives cargo, then embeds the library in the jar
// under natives/<platform>/ where NativeLoader looks for it.
// Defaults to the host target; CI (Linux x86_64) therefore produces the V1 server jar.
// Override with -Prust_target=<triple>.
// ---------------------------------------------------------------------------

val rustTarget: String = providers.gradleProperty("rust_target").getOrElse(
    hostRustTarget(
        providers.systemProperty("os.name").get(),
        providers.systemProperty("os.arch").get(),
    )
)

val cargoBuild = tasks.register<Exec>("cargoBuild") {
    group = "rust"
    description = "Builds the factory-jni native library for $rustTarget."
    workingDir = repoRoot.asFile
    commandLine("cargo", "build", "--release", "-p", "factory-jni", "--target", rustTarget)
    outputs.file(repoRoot.file("target/$rustTarget/release/${nativeLibraryFileName(rustTarget)}"))
    // Cargo tracks its own inputs and is a no-op when nothing changed.
    outputs.upToDateWhen { false }
}

val cargoTest = tasks.register<Exec>("cargoTest") {
    group = "rust"
    description = "Runs every Rust test of the workspace."
    workingDir = repoRoot.asFile
    commandLine("cargo", "test", "--workspace")
}

val bundleNatives = tasks.register<Sync>("bundleNatives") {
    group = "rust"
    description = "Stages the native library as a jar resource."
    from(cargoBuild) {
        into("natives/${nativePlatform(rustTarget)}")
    }
    into(layout.buildDirectory.dir("generated/natives"))
}

// ---------------------------------------------------------------------------
// neoforge.mods.toml is expanded from src/main/templates.
// ---------------------------------------------------------------------------

val generateModMetadata = tasks.register<ProcessResources>("generateModMetadata") {
    val replaceProperties = mapOf(
        "minecraft_version" to minecraft_version,
        "minecraft_version_range" to minecraft_version_range,
        "neo_version" to neo_version,
        "kff_version" to kff_version,
        "mod_id" to mod_id,
        "mod_name" to mod_name,
        "mod_license" to mod_license,
        "mod_version" to modVersion,
        "mod_authors" to mod_authors,
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
        srcDir(bundleNatives)
        exclude("**/*.bbmodel")
        exclude("src/generated/**/.cache")
    }
}

neoForge.ideSyncTask(generateModMetadata)
neoForge.ideSyncTask(bundleNatives)

tasks.test {
    useJUnitPlatform()
    systemProperty("satiscraft.expectedCoreVersion", modVersion)
    // Java 24+ warns on System.load without it (JEP 472).
    jvmArgs("--enable-native-access=ALL-UNNAMED")
}

// The license requires its terms and notice to travel with every copy.
tasks.jar {
    from(repoRoot.file("LICENSE.md"))
}

tasks.check {
    dependsOn(cargoTest)
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

/** Reads `version` from the `[workspace.package]` section of a Cargo manifest. */
fun workspaceVersion(manifest: String): String {
    val section = Regex("""(?ms)^\[workspace\.package]\s*$(.*?)(?=^\[|\z)""").find(manifest)
        ?: error("Cargo.toml has no [workspace.package] section")
    return Regex("""(?m)^version\s*=\s*"([^"]+)"""").find(section.groupValues[1])?.groupValues?.get(1)
        ?: error("Cargo.toml [workspace.package] has no version")
}

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
