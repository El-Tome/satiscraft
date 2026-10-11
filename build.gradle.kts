plugins {
    base
    // Puts ModDevGradle and the Kotlin plugin on one classpath shared by every project.
    id("satiscraft.mod") apply false
}

// `./gradlew build` also tests the build logic.
tasks.check {
    dependsOn(gradle.includedBuild("build-logic").task(":check"))
}
