package ru.astrainteractive.nanolimbo;

/**
 * Values the build knows and the code cannot: the plugin identifier, the name the Rust cdylib was
 * compiled under, and the version of the Rust workspace this jar ships.
 */
public final class BuildConstants {

    /**
     * Base name of the cdylib, without the {@code lib} prefix or the platform file extension.
     */
    public static final String LIB_NAME = "${libName}";

    /**
     * The plugin identifier both proxy descriptors carry, and so the name of the data directory
     * {@code settings.yml} lands in.
     */
    public static final String PLUGIN_ID = "${pluginId}";

    /**
     * The {@code [workspace.package] version} of the Rust sources this wrapper was built against.
     */
    public static final String VERSION = "${version}";

    private BuildConstants() {
    }
}
