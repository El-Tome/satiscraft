// World generation of the Stone Age: places the engine's ores and surface pieces, and fills
// structure chests. Declares no block nor item; depends on the engine, never the reverse.

plugins {
    id("satiscraft.mod")
}

dependencies {
    implementation(project(":mods:satiscraft"))
}
