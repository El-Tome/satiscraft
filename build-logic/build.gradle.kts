import java.util.Properties

plugins {
    `kotlin-dsl`
}

// Versions stay pinned in the root gradle.properties, next to all the others.
val rootProperties: Properties = providers.fileContents(layout.projectDirectory.file("../gradle.properties"))
    .asText.map { text -> Properties().apply { load(text.reader()) } }.get()

/** Reads a pinned version from the root gradle.properties. */
fun pinned(name: String): String = rootProperties.getProperty(name) ?: error("gradle.properties has no $name")

repositories {
    gradlePluginPortal()
    mavenCentral()
}

dependencies {
    implementation("net.neoforged:moddev-gradle:${pinned("moddev_version")}")
    implementation("org.jetbrains.kotlin:kotlin-gradle-plugin:${pinned("kotlin_version")}")

    testImplementation(platform("org.junit:junit-bom:${pinned("junit_version")}"))
    testImplementation("org.junit.jupiter:junit-jupiter")
    testRuntimeOnly("org.junit.platform:junit-platform-launcher")
}

tasks.test {
    useJUnitPlatform()
}
