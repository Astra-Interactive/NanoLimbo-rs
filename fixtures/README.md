# Fixtures

Reference data the port is verified against. See `../MIGRATION_PLAN.md` section 7 for how
these fit into the testing pyramid.

Two independent oracles live here, and the distinction matters:

| Directory | Oracle | Proves |
|---|---|---|
| `text/`, `uuid/` | the Java implementation in `../../src` | the port is byte-equivalent to a build that has run in production across all 51 versions |
| `minecraft-data/` | [PrismarineJS/minecraft-data](https://github.com/PrismarineJS/minecraft-data) | the tables are right independently of Java, so a bug shared by both implementations is still caught |

Equivalence testing alone inherits the oracle's bugs, which is why the second oracle
exists. It has already earned its place: it confirmed that the Java packet table mapped
serverbound configuration `custom_payload` to `0x02` from 1.20.2, where that id is
`finish_configuration`.

## `text/components.json`

Output of `ByteMessage.writeComponent` for every text input `settings.yml` can carry,
across all 51 supported versions. Encodings are grouped by the versions that share them,
which keeps the file small and makes the profile boundaries legible — for example a
component carrying a hover event splits four ways, at 1.16, 1.20.3 and 1.21.5.

This is the oracle for the `limbo-text` crate, the highest-risk part of the port: no Rust
crate implements Adventure's per-version JSON profiles or its NBT component encoding, so
that code is written from scratch and has nothing but these bytes to check it.

Each entry also records `legacy` and `plain` renderings, used for the server brand, the
ping version string and log output.

## `uuid/offline.json`

`UUID.nameUUIDFromBytes("OfflinePlayer:" + name)` for a range of usernames including
non-ASCII and astral-plane characters. The port must reproduce these exactly: the offline
identity keys the connection registry, so a mismatch changes who a player *is*.

## Regenerating

The dumper lives in the Java tree at `src/main/java/ua/nanit/limbo/tools/FixtureDumper.java`.
It is not part of the server and is never started by it.

```sh
cd ..            # repository root, the Java project
./gradlew shadowJar
java -cp build/libs/NanoLimbo.jar ua.nanit.limbo.tools.FixtureDumper rust/fixtures
```

Everything it writes is deterministic: values the server would normally draw from a random
source or a clock are passed in fixed. Regenerating on an unchanged Java tree must produce
an identical file, and a diff that appears without a deliberate Java change is a bug.

Once the Java tree is retired these files stop being derived from it and become
self-hosted regression snapshots — see MIGRATION_PLAN.md section 7.8.
