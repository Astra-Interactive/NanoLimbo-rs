# glibc, not the musl base the published image uses: the cdylib is dlopened by a JVM that has
# already loaded glibc, and a musl build will not resolve there.
FROM rust:1.93-bookworm AS rust-build

WORKDIR /src
COPY . .

RUN cargo build --release --locked \
    --package limbo-ffi \
    --package nanolimbo \
    --package limbo-bench

FROM eclipse-temurin:25-jdk AS jar-build

WORKDIR /src
COPY . .
COPY --from=rust-build /src/target/release/libnanolimbo_lib.so /tmp/libnanolimbo_lib.so

RUN set -eu; \
    destination="/src/java-wrapper/wrapper/src/main/resources/linux/$(uname -m)"; \
    mkdir -p "$destination"; \
    cp /tmp/libnanolimbo_lib.so "$destination/"

WORKDIR /src/java-wrapper

# Host Gradle state came from another JDK at another path, and a stale configuration cache
# fails as if the build script were broken.
RUN rm -rf .gradle build wrapper/build

RUN chmod +x gradlew && ./gradlew --no-daemon build

# java25 because Velocity 4 requires it, which is also what the jar targets.
FROM itzg/bungeecord:java25 AS runtime

# /config is synced into /server on startup. The image's own /plugins is copied rather than
# synced, and nests a second plugins directory on a reused volume. Renamed without its version
# so the compose entrypoint does not have to know it.
COPY --from=jar-build /src/java-wrapper/wrapper/build/libs/NanoLimbo-rs-Wrapper-*.jar /config/plugins/nanolimbo_java_wrapper.jar

COPY --from=rust-build /src/target/release/nanolimbo /usr/local/bin/nanolimbo
COPY --from=rust-build /src/target/release/limbo-bench /usr/local/bin/limbo-bench
