import io.github.eltome.satiscraft.gradle.hostRustTarget
import io.github.eltome.satiscraft.gradle.nativeLibraryFileName
import io.github.eltome.satiscraft.gradle.nativePlatform

// The engine: drives factory-core through JNI and declares every block and item of the game.
// It runs on its own; satiscraft-world and satiscraft-primitive build on it.

plugins {
    id("satiscraft.mod")
}

val modId = "satiscraft"
val repoRoot: Directory = rootProject.layout.projectDirectory

neoForge {
    runs {
        register("client") {
            client()
            systemProperty("neoforge.enabledGameTestNamespaces", modId)
        }
        register("server") {
            server()
            programArgument("--nogui")
            systemProperty("neoforge.enabledGameTestNamespaces", modId)
        }
        register("gameTestServer") {
            type = "gameTestServer"
            systemProperty("neoforge.enabledGameTestNamespaces", modId)
        }
        register("data") {
            clientData()
            programArguments.addAll(
                "--mod", modId, "--all",
                "--output", file("src/generated/resources/").absolutePath,
                "--existing", file("src/main/resources/").absolutePath,
            )
        }
    }
}

// ---------------------------------------------------------------------------
// Rust native library: Gradle drives cargo, then embeds the library in the jar
// under natives/<platform>/ where NativeLoader looks for it.
// Defaults to the host target; CI (Linux x86_64) therefore produces the server jar.
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

sourceSets.main {
    resources.srcDir(bundleNatives)
}

neoForge.ideSyncTask(bundleNatives)

tasks.test {
    systemProperty("satiscraft.expectedCoreVersion", version.toString())
    // Java 24+ warns on System.load without it (JEP 472).
    jvmArgs("--enable-native-access=ALL-UNNAMED")
}

tasks.check {
    dependsOn(cargoTest)
}
