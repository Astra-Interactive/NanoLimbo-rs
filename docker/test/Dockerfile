# Builds one image: the wrapper jar and our two binaries dropped into itzg/bungeecord, which
# already knows how to fetch and launch Velocity or BungeeCord. The context is the repository
# root, because the jar needs the Rust workspace and java-wrapper together:
#
#   docker build -f docker/test/Dockerfile -t nanolimbo-test:local .

# glibc rather than the musl base the published image uses: the cdylib is dlopened by a JVM
# that has already loaded glibc, and a musl build of it will not resolve there.
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

# `uname -m` already spells the architecture the way NativeLibrary.resourcePath does
# (x86_64, aarch64), so the same file builds on an ARM laptop and an x86 runner.
RUN set -eu; \
    destination="/src/java-wrapper/wrapper/src/main/resources/linux/$(uname -m)"; \
    mkdir -p "$destination"; \
    cp /tmp/libnanolimbo_lib.so "$destination/"

WORKDIR /src/java-wrapper

# Gradle state that arrived with the build context was produced by another JDK at another
# path, and a stale configuration cache fails in a way that reads like a broken build script.
RUN rm -rf .gradle build wrapper/build

RUN chmod +x gradlew && ./gradlew --no-daemon build

# java25 because Velocity 4.x publishes java.version.minimum = 25, and that is also what the
# wrapper jar now targets, so one base serves both proxies. Ubuntu-based, so the glibc
# binaries copied in below run as they are.
FROM itzg/bungeecord:java25 AS runtime

# /config is the tree this image syncs into /server on startup, so the jar arrives in
# /server/plugins the same way a mounted configuration file arrives in /server. Syncing is
# idempotent, which its own /plugins mechanism is not: that one copies rather than syncs and
# nests a second plugins directory when the server volume already has one.
COPY --from=jar-build /src/java-wrapper/wrapper/build/libs/nanolimbo_java_wrapper.jar /config/plugins/nanolimbo_java_wrapper.jar

COPY --from=rust-build /src/target/release/nanolimbo /usr/local/bin/nanolimbo
COPY --from=rust-build /src/target/release/limbo-bench /usr/local/bin/limbo-bench
