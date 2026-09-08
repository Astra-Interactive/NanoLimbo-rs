package ru.astrainteractive.nanolimbo;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.logging.Level;
import java.util.logging.Logger;

public final class Standalone {

    private static final Logger LOGGER = Logger.getLogger(Standalone.class.getName());
    private static final int FAILURE_EXIT_CODE = 1;

    private Standalone() {
    }

    /**
     * @param arguments the configuration directory, or nothing for the working directory
     */
    public static void main(String[] arguments) {
        Path configurationDirectory = Path.of(arguments.length > 0 ? arguments[0] : ".");
        try {
            Files.createDirectories(configurationDirectory);
            NanoLimboRunner runner = new NanoLimboRunner(NativeLibrary.load(), configurationDirectory, LOGGER);
            Runtime.getRuntime().addShutdownHook(new Thread(runner::stop, "nanolimbo-shutdown"));
            runner.run();
        } catch (IOException | UnsupportedOperationException failure) {
            LOGGER.log(Level.SEVERE, "NanoLimbo could not be started", failure);
            System.exit(FAILURE_EXIT_CODE);
        }
    }
}
