# NanoLimbo (Rust)

A lightweight Minecraft limbo server: it accepts a player, walks them through the login
sequence, parks them in empty space and keeps the connection alive while the real server
is down. It runs no game logic — there is no world, no physics, no inventory.

A port of [Nan1t/NanoLimbo](https://github.com/Nan1t/NanoLimbo), which remains the
behavioural reference: this project is verified against byte-level dumps taken from it.
See `MIGRATION_PLAN.md` for the architecture, the phased plan, and the testing strategy.

This repository stands alone. The reference dumps it is checked against are committed in
`fixtures/`, together with the tool that produces them, so nothing here depends on having
the upstream project checked out.

## Status

Working. The server binds, serves clients and stops cleanly. Every protocol version from
**1.7.2 (protocol 4) to 26.2 (protocol 776)** — 51 in all — reaches the play phase.

| | |
|---|---|
| Protocol versions | 51, from 1.7.2 to 26.2 |
| Info forwarding | `NONE`, `LEGACY`, `MODERN` (Velocity), `BUNGEE_GUARD` |
| Configuration | byte-compatible with the Java `settings.yml` |
| Tests | 300+, including byte-level parity against dumps from the Java build |

## Building and running

```sh
cargo build --release
./target/release/nanolimbo
```

The server writes a `settings.yml` on first run and reads it from the working directory
thereafter. An existing deployment's file works unchanged.

`RUST_LOG` overrides the configured `debugLevel` when you need detail from one module
without editing the file.

### Console

`help`, `conn`, `mem`, `version` (`ver`), `stop`. `Ctrl-C` and `SIGTERM` also stop it
cleanly.

## Measuring it against the Java build

The figures below are reproducible rather than something to take on trust. `limbo-bench`
logs a crowd of players into one or more servers and reports what each one cost; it speaks
the protocol through the same version tables the server does, so a new Minecraft release
cannot leave it silently measuring a failed login.

```sh
./bench/compare.sh --players 300                 # this server alone
./bench/compare.sh --protocol 47                 # join as 1.8 instead
./bench/compare.sh --jar path/to/NanoLimbo.jar   # against the original
```

Given a jar it starts both on the same configuration, logs the players in and reads both
servers' memory from one place at one moment:

```
target       asked  joined      time       idle      loaded   mem/player  sent/player
------------------------------------------------------------------------------------
rust           200     200     0.07s    22.7 MB     34.9 MB      62.5 KB     153.4 KB
java           200     200     0.19s   118.6 MB    159.3 MB     208.3 KB     153.4 KB
```

Upstream lives in its own repository, so its jar is pointed at rather than assumed:

```sh
git clone https://github.com/Nan1t/NanoLimbo && (cd NanoLimbo && ./gradlew shadowJar)
```

Java's resident figure is shaped by its heap settings, so this is an out-of-the-box
comparison rather than the best either runtime can do. Below a couple of hundred players
the per-player column is mostly startup cost spread thin, and the tool says so.

`sent/player` matching to the byte is worth noticing: it says both servers put the same
thing on the wire, which is the whole point of the parity work.

`limbo-bench` can also be pointed at anything already running:

```sh
cargo run --release --bin limbo-bench -- --players 500 rust=127.0.0.1:25565@$(pgrep nanolimbo)
```

The `@pid` is optional — without it the server is still load-tested, only its memory goes
unreported.

## Containers

```sh
docker compose up --build
```

The image is a statically linked binary on `scratch`: nothing to patch, nothing to exec
into, and no shell for an attacker to find. It runs unprivileged as uid 65532 and reads
its configuration from `/data`.

`docker/settings.yml.example` is the shipped configuration with the two changes a
container needs — an empty `bind.ip` so it listens on every interface rather than on its
own loopback, and the conventional port. Compose mounts it read-only; edit it on the host.
Remove the mount and the server writes its own default into the volume instead.

## Layout

```
crates/
  limbo-protocol   versions, connection states, packet id tables, buffer primitives
  limbo-text       chat components, MiniMessage, per-version JSON and NBT encoding
  limbo-world      dimension codecs and registry tags, embedded in the binary
  limbo-packet     clientbound packets and their per-version pre-encoding
  limbo-net        framing, traffic limits, identity, proxy forwarding
  limbo-config     settings.yml
  limbo-server     connection lifecycle and decision making, with no I/O
  limbo-bench      load generator and the side-by-side comparison
  nanolimbo        the binary: composition root and async runtime
fixtures/          reference data the port is verified against
```

Everything except `nanolimbo` and `limbo-bench` holds no sockets and no clock, so the
whole login sequence is replayed for all 51 versions in unit tests that run in
milliseconds.

## Differences from the Java implementation

Deliberate, and each documented at the site and in `MIGRATION_PLAN.md`:

- **Fixed:** the clientbound disconnect id on 1.20.3/1.20.4 (§3.1.6), the serverbound
  configuration plugin message id from 1.20.2 (§3.1.5), text silently dropped from
  components on 1.20.3+ (§3.1.10), and the connection registry keying that lost track of
  two players sharing a name (§3.1.2).
- **Hardened:** no path reachable from network input can panic; BungeeGuard tokens are
  compared in constant time; the traffic limiter cannot overflow into disabling itself.
- **Not carried over:** `netty.transportType: IO_URING` and `netty.threads.bossGroup`
  have no equivalent under this runtime. Both are accepted and reported as warnings
  rather than ignored or rejected.
- `mem` reports the operating system's resident figure instead of JVM heap statistics,
  and says so plainly where the platform cannot tell.

## Licence

GPL-3.0-or-later, as the original — see `LICENSE`. This is a derivative work of
[Nan1t/NanoLimbo](https://github.com/Nan1t/NanoLimbo) by Nan1t and contributors, ported
rather than rewritten from scratch, and the licence follows accordingly.
