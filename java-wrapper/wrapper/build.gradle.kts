import ru.astrainteractive.gradle.property.api.gradleProperty
import ru.astrainteractive.gradle.property.api.klibsGradleProperty
import ru.astrainteractive.gradleplugin.property.util.requireString

plugins {
    application
    id("ru.astrainteractive.gradleplugin.java.version")
}

// The version is scraped from the Rust workspace manifest rather than taken from
// `klibs.project.version.string`. `.github/workflows/call-version-changed.yml` refuses a pull
// request that does not bump `[workspace.package] version` in Cargo.toml, and the release job
// tags from the same field, so a second copy of the number in gradle.properties would only ever
// drift away from the one CI enforces.
val cargoManifest = rootProject.file("../Cargo.toml")
val workspaceVersion = Regex("(?ms)^\\[workspace\\.package].*?^version\\s*=\\s*\"([^\"]+)\"")
    .find(cargoManifest.readText())
    ?.groupValues
    ?.get(1)
    ?: error("Could not read [workspace.package] version from ${cargoManifest.path}")

group = klibsGradleProperty("project.group").requireString
version = workspaceVersion

dependencies {
    implementation(libs.nativeaccess.jna)
    compileOnly(libs.minecraft.bungee.api)
    compileOnly(libs.minecraft.velocity.api)
    annotationProcessor(libs.minecraft.velocity.api)
}

application {
    mainClass = "ru.astrainteractive.nanolimbo.Standalone"
}

val pluginId = gradleProperty("pluginId").requireString
val templateProperties = mapOf(
    "libName" to gradleProperty("libName").requireString,
    "pluginId" to pluginId,
    "version" to workspaceVersion
)

val generateTemplates = tasks.register<Copy>("generateTemplates") {
    inputs.properties(templateProperties)
    from(layout.projectDirectory.dir("src/main/templates"))
    into(layout.buildDirectory.dir("generated/sources/templates"))
    expand(templateProperties)
}

sourceSets {
    main {
        java {
            srcDir(generateTemplates.map { it.outputs })
        }
    }
}

tasks.named<ProcessResources>("processResources").configure {
    filteringCharset = "UTF-8"
    val descriptorProperties = mapOf(
        "name" to pluginId,
        "main" to "ru.astrainteractive.nanolimbo.BungeeCordPlugin",
        "version" to workspaceVersion,
        "description" to klibsGradleProperty("project.description").requireString,
        "author" to klibsGradleProperty("project.developers").requireString.substringBefore('|')
    )
    inputs.properties(descriptorProperties)
    filesMatching("plugin.yml") {
        expand(descriptorProperties)
    }
}

// A plain fat jar rather than a shaded one: the single runtime dependency is JNA, and relocating
// it would break the native stub `Native` looks up by its own package name.
tasks.jar {
    archiveFileName = "NanoLimbo-rs-Wrapper-$version.jar"
    duplicatesStrategy = DuplicatesStrategy.EXCLUDE
    exclude("META-INF/*.SF", "META-INF/*.DSA", "META-INF/*.RSA")

    manifest {
        attributes(
            // Java 24 warns on every restricted native call unless the executable jar opts in.
            "Enable-Native-Access" to "ALL-UNNAMED",
            "Main-Class" to application.mainClass.get()
        )
    }

    from(sourceSets.main.get().output)
    dependsOn(configurations.runtimeClasspath)
    from({
        configurations.runtimeClasspath.get()
            .filter { dependencyJar -> dependencyJar.name.endsWith("jar") }
            .map(::zipTree)
    })
}
