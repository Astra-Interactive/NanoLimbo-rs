package ru.astrainteractive.nanolimbo;

import com.sun.jna.Memory;
import com.sun.jna.Pointer;
import java.nio.charset.StandardCharsets;
import java.nio.file.Path;
import java.time.Duration;
import java.util.concurrent.CountDownLatch;
import java.util.concurrent.TimeUnit;
import java.util.logging.Level;
import java.util.logging.Logger;

/**
 * {@link #run()} blocks for as long as the server lives and {@link #stop()} ends it from another
 * thread. Both reach the same native token, so a stop arriving before {@code run} acquired it, or
 * after it freed it, would dereference a stale pointer - hence the lock on every access.
 */
public final class NanoLimboRunner implements Runnable {

    private static final Duration SHUTDOWN_TIMEOUT = Duration.ofSeconds(10);

    private final NativeLibrary library;
    private final Path configurationDirectory;
    private final Logger logger;
    private final Object lifecycle = new Object();
    private final CountDownLatch terminated = new CountDownLatch(1);

    /** Guarded by {@link #lifecycle}; non-null only between acquiring and freeing. */
    private Pointer token;

    /** Guarded by {@link #lifecycle}; keeps a stop that arrived before the start from being lost. */
    private boolean stopRequested;

    public NanoLimboRunner(NativeLibrary library, Path configurationDirectory, Logger logger) {
        this.library = library;
        this.configurationDirectory = configurationDirectory;
        this.logger = logger;
    }

    /**
     * A Java {@code String} would leave the encoding to JNA's {@code jna.encoding} property, and
     * setting that inside a proxy JVM changes how every other plugin marshals its strings.
     */
    private static Memory encodeUtf8(String value) {
        byte[] encoded = value.getBytes(StandardCharsets.UTF_8);
        Memory memory = new Memory(encoded.length + 1L);
        memory.write(0L, encoded, 0, encoded.length);
        memory.setByte(encoded.length, (byte) 0);
        return memory;
    }

    /**
     * @return {@code null} when the server must not start, having been stopped already
     */
    private Pointer acquireToken() {
        synchronized (lifecycle) {
            if (stopRequested) {
                return null;
            }
            Pointer acquired = library.get_cancellation_token();
            if (acquired == null) {
                logger.severe("NanoLimbo could not allocate a cancellation token");
                return null;
            }
            token = acquired;
            return acquired;
        }
    }

    private void releaseToken(Pointer acquired) {
        if (acquired == null) {
            return;
        }
        synchronized (lifecycle) {
            library.cleanup_token(acquired);
            token = null;
        }
    }

    private void startBlocking(Pointer acquired) {
        try (Memory directory = encodeUtf8(configurationDirectory.toAbsolutePath().toString())) {
            int code = library.start_app(acquired, directory);
            StartAppStatus status = StartAppStatus.fromCode(code);
            if (status.isFailure()) {
                logger.severe("NanoLimbo stopped with code " + code + ": " + status.description());
            }
        }
    }

    private void awaitTermination() {
        try {
            if (!terminated.await(SHUTDOWN_TIMEOUT.toMillis(), TimeUnit.MILLISECONDS)) {
                logger.warning("NanoLimbo did not stop within " + SHUTDOWN_TIMEOUT.toSeconds() + "s");
            }
        } catch (InterruptedException interruption) {
            Thread.currentThread().interrupt();
            logger.log(Level.FINE, "Interrupted while waiting for NanoLimbo to stop", interruption);
        }
    }

    @Override
    public void run() {
        Pointer acquired = acquireToken();
        try {
            if (acquired != null) {
                startBlocking(acquired);
            }
        } finally {
            releaseToken(acquired);
            terminated.countDown();
        }
    }

    /**
     * Waits for {@link #run()} to return, so a proxy shutting down does not race the teardown.
     * Safe before {@code run}, after it, and more than once.
     */
    public void stop() {
        boolean cancelled;
        synchronized (lifecycle) {
            stopRequested = true;
            cancelled = token != null;
            if (cancelled) {
                library.stop_app(token);
            }
        }
        if (cancelled) {
            awaitTermination();
        }
    }
}
