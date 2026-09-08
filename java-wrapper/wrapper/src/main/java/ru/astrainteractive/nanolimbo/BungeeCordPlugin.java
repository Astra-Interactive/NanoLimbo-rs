package ru.astrainteractive.nanolimbo;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.logging.Level;
import net.md_5.bungee.api.plugin.Plugin;

public final class BungeeCordPlugin extends Plugin {

    private NanoLimboRunner runner;

    @Override
    public void onEnable() {
        Path dataDirectory = getDataFolder().toPath();
        try {
            // The Rust side writes settings.yml into this directory but will not create it.
            Files.createDirectories(dataDirectory);
            runner = new NanoLimboRunner(NativeLibrary.load(), dataDirectory, getLogger());
            getProxy().getScheduler().runAsync(this, runner);
        } catch (IOException | UnsupportedOperationException failure) {
            getLogger().log(Level.SEVERE, "NanoLimbo could not be started", failure);
        }
    }

    @Override
    public void onDisable() {
        if (runner != null) {
            runner.stop();
        }
    }
}
