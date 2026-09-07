# NanoLimbo (Rust)

A lightweight Minecraft limbo server: it accepts a player, walks them through the login
sequence, parks them in empty space and keeps the connection alive while the real server
is down. It runs no game logic — there is no world, no physics, no inventory.

A port of the Java implementation in `../src`, which remains the behavioural reference.
See `MIGRATION_PLAN.md` for the architecture, the phased plan, and the testing strategy.

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
  nanolimbo        the binary: composition root and async runtime
fixtures/          reference data the port is verified against
```

The bottom four crates hold no sockets and no clock, so the whole login sequence is
replayed for all 51 versions in unit tests that run in milliseconds.

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

GPL-3.0-or-later, as the original. This is a derivative work of
[Nan1t/NanoLimbo](https://github.com/Nan1t/NanoLimbo).
