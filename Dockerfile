# A statically linked binary on an empty image: nothing to patch, nothing to exec into,
# and an image that is essentially the binary plus its configuration directory.
#
#   docker build -t nanolimbo .
#   docker run --rm -p 25565:25565 -v ./docker/settings.yml:/data/settings.yml:ro nanolimbo

FROM rust:1.93-alpine AS build

# musl-dev provides the linker; every dependency in this workspace is pure Rust, so
# nothing else is needed and the result links statically.
RUN apk add --no-cache musl-dev

WORKDIR /src
COPY . .

# --locked so an image never silently builds against a different dependency tree than
# the one the tests ran against. The release profile already strips symbols, so there is
# no separate strip step - and no need for binutils in the builder.
RUN cargo build --release --locked --bin nanolimbo

# The final image has no shell to create this with, and no root to own it.
RUN mkdir -p /data && chown 65532:65532 /data

FROM scratch

COPY --from=build /src/target/release/nanolimbo /nanolimbo
COPY --from=build --chown=65532:65532 /data /data

# Where settings.yml lives. Mount over it to supply your own; the server writes a default
# here on first run if the directory is writable and empty.
WORKDIR /data
VOLUME ["/data"]

# Unprivileged, and numeric because there is no passwd file to resolve a name against.
USER 65532:65532

EXPOSE 25565

ENTRYPOINT ["/nanolimbo"]
