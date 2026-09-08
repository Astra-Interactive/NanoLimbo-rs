package ru.astrainteractive.nanolimbo;

import com.google.inject.Inject;
import com.velocitypowered.api.event.Subscribe;
import com.velocitypowered.api.event.proxy.ProxyInitializeEvent;
import com.velocitypowered.api.event.proxy.ProxyShutdownEvent;
import com.velocitypowered.api.plugin.Plugin;
import com.velocitypowered.api.plugin.annotation.DataDirectory;
import com.velocitypowered.api.proxy.ProxyServer;
import com.velocitypowered.api.scheduler.ScheduledTask;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.logging.Level;
import java.util.logging.Logger;

@Plugin(
        id = BuildConstants.PLUGIN_ID,
        name = "NanoLimbo-rs",
        version = BuildConstants.VERSION,
        description = "Runs the NanoLimbo-rs limbo server inside the proxy",
        authors = {"makeevrserg"}
)
public final class VelocityPlugin {

    /**
     * The runner is shared with the BungeeCord and standalone entry points, so it logs through the
     * JDK rather than Velocity's SLF4J binding.
     */
    private static final Logger LOGGER = Logger.getLogger(VelocityPlugin.class.getName());

    private final ProxyServer proxy;
    private final Path dataDirectory;

    private NanoLimboRunner runner;
    private ScheduledTask task;

    @Inject
    public VelocityPlugin(ProxyServer proxy, @DataDirectory Path dataDirectory) {
        this.proxy = proxy;
        this.dataDirectory = dataDirectory;
    }

    @Subscribe
    public void onProxyInitialize(ProxyInitializeEvent event) {
        try {
            // The Rust side writes settings.yml into this directory but will not create it.
            Files.createDirectories(dataDirectory);
            runner = new NanoLimboRunner(NativeLibrary.load(), dataDirectory, LOGGER);
            task = proxy.getScheduler()
                    .buildTask(this, runner)
                    .schedule();
        } catch (IOException | UnsupportedOperationException failure) {
            LOGGER.log(Level.SEVERE, "NanoLimbo could not be started", failure);
        }
    }

    @Subscribe
    public void onProxyShutdown(ProxyShutdownEvent event) {
        if (runner != null) {
            runner.stop();
        }
        if (task != null) {
            task.cancel();
        }
    }
}
