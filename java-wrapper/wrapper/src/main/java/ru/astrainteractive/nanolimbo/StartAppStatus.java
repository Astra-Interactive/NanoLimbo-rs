package ru.astrainteractive.nanolimbo;

/**
 * The exit codes {@code start_app} is documented to return, so that a failure reaches the console
 * as a sentence rather than as a number.
 */
public enum StartAppStatus {

    OK(0, "the server ran and was stopped"),
    NULL_TOKEN(1, "the cancellation token was null"),
    INVALID_CONFIGURATION_DIRECTORY(2, "the configuration directory was null or not valid UTF-8"),
    STARTUP_FAILED(3, "startup failed: the configuration or an embedded resource is unusable"),
    RUNTIME_UNAVAILABLE(4, "the async runtime could not be built"),
    BIND_FAILED(5, "the configured port could not be bound"),

    /**
     * A code the contract does not define, which means the jar and the cdylib disagree.
     */
    UNKNOWN(-1, "an undocumented status code");

    private final int code;
    private final String description;

    StartAppStatus(int code, String description) {
        this.code = code;
        this.description = description;
    }

    /**
     * Maps a raw return value onto this contract, answering {@link #UNKNOWN} for anything else.
     */
    public static StartAppStatus fromCode(int code) {
        for (StartAppStatus status : values()) {
            if (status != UNKNOWN && status.code == code) {
                return status;
            }
        }
        return UNKNOWN;
    }

    public String description() {
        return description;
    }

    public boolean isFailure() {
        return this != OK;
    }
}
