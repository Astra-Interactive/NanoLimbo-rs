plugins {
    // Neither is applied here: the root only puts them on the build script classpath the wrapper
    // module inherits. The Kotlin plugin has to be among them because the klibs java.version
    // convention plugin configures Kotlin compile tasks, and so loads Kotlin Gradle plugin classes
    // even in a build that compiles no Kotlin.
    alias(libs.plugins.kotlin.jvm) apply false
    alias(libs.plugins.klibs.gradle.java.version) apply false
}
