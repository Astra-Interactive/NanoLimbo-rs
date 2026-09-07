package ru.astrainteractive.nanolimbo;

import com.sun.jna.Library;
import com.sun.jna.Native;
import com.sun.jna.Platform;
import com.sun.jna.Pointer;
import java.io.IOException;
import java.io.InputStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardCopyOption;
import java.util.Locale;

/**
 * The NanoLimbo-rs cdylib, as C sees it.
 *
 * <p>The method names are the exported symbols, so they keep the snake_case spelling of the Rust
 * side; JNA binds by name and a Java-style rename would not resolve.
 *
 * <p>The cdylib is not on the library path of a proxy JVM, so {@link #load()} unpacks the build for
 * the running platform out of this jar first.
 */
public interface NativeLibrary extends Library {

    /**
     * {@code CancellationToken* get_cancellation_token(void);}
     *
     * @return the token that ties {@code start_app} to {@code stop_app}, or {@code null} when the
     *     allocation failed.
     */
    Pointer get_cancellation_token();

    /**
     * {@code int32_t start_app(CancellationToken* ptr, const char* config_dir);}
     *
     * <p>Blocks until the token is cancelled. The server reads, and on a first run writes,
     * {@code settings.yml} inside {@code config_dir}.
     *
     * @return one of the codes {@link StartAppStatus} lists.
     */
    int start_app(Pointer token, Pointer configurationDirectory);

    /**
     * {@code void stop_app(CancellationToken* ptr);} Cancels the token and returns at once.
     */
    void stop_app(Pointer token);

    /**
     * {@code void cleanup_token(CancellationToken* ptr);} Frees the token. Calling it while
     * {@code start_app} still holds the token is a use-after-free.
     */
    void cleanup_token(Pointer token);

    private static String normalizeArchitecture(String reported) {
        String architecture = reported.toLowerCase(Locale.ROOT);
        return switch (architecture) {
            case "amd64", "x86-64", "x64" -> "x86_64";
            case "arm64" -> "aarch64";
            default -> architecture;
        };
    }

    private static String resourcePath(String libraryName) {
        String architecture = normalizeArchitecture(System.getProperty("os.arch", ""));
        if (Platform.isWindows()) {
            if (!"x86_64".equals(architecture)) {
                throw new UnsupportedOperationException("Unsupported Windows architecture: " + architecture);
            }
            return "/windows/x86_64/" + libraryName + ".dll";
        }
        if (Platform.isMac()) {
            if (!"x86_64".equals(architecture) && !"aarch64".equals(architecture)) {
                throw new UnsupportedOperationException("Unsupported macOS architecture: " + architecture);
            }
            return "/macos/" + architecture + "/lib" + libraryName + ".dylib";
        }
        if (Platform.isLinux()) {
            if (!"x86_64".equals(architecture) && !"aarch64".equals(architecture)) {
                throw new UnsupportedOperationException("Unsupported Linux architecture: " + architecture);
            }
            return "/linux/" + architecture + "/lib" + libraryName + ".so";
        }
        throw new UnsupportedOperationException("Unsupported operating system: " + System.getProperty("os.name"));
    }

    private static Path extract(String resourcePath, String libraryName) throws IOException {
        String extension = resourcePath.substring(resourcePath.lastIndexOf('.'));
        Path extracted = Files.createTempFile(libraryName, extension);
        extracted.toFile().deleteOnExit();
        try (InputStream packaged = NativeLibrary.class.getResourceAsStream(resourcePath)) {
            if (packaged == null) {
                throw new IOException("This jar carries no native library at " + resourcePath);
            }
            Files.copy(packaged, extracted, StandardCopyOption.REPLACE_EXISTING);
        }
        return extracted;
    }

    /**
     * Unpacks the cdylib built for the running platform and binds to it.
     *
     * @throws IOException when the jar carries no build for this platform, or unpacking it fails.
     * @throws UnsupportedOperationException when no build for this platform exists at all.
     */
    static NativeLibrary load() throws IOException {
        String libraryName = BuildConstants.LIB_NAME;
        Path extracted = extract(resourcePath(libraryName), libraryName);
        return Native.load(extracted.toAbsolutePath().toString(), NativeLibrary.class);
    }
}
