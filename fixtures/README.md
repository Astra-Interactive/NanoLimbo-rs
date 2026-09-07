# Fixtures

Reference data the port is verified against. See `../MIGRATION_PLAN.md` section 7 for how
these fit into the testing pyramid.

Two independent oracles live here, and the distinction matters:

| Directory | Oracle | Proves |
|---|---|---|
| `text/`, `uuid/` | the Java implementation in `../../src` | the port is byte-equivalent to a build that has run in production across all 51 versions |
| `minecraft-data/` | [PrismarineJS/minecraft-data](https://github.com/PrismarineJS/minecraft-data) | the tables are right independently of Java, so a bug shared by both implementations is still caught |

Do not mistake the `packets/` ids for a third oracle: they are Java's own, and inherit
Java's mistakes.

Equivalence testing alone inherits the oracle's bugs, which is why the second oracle
exists. It has already earned its place twice, finding defects that byte-comparison
against Java can never find because the port would reproduce them exactly:

- serverbound configuration `custom_payload` mapped to `0x02` from 1.20.2, where that id
  is `finish_configuration`;
- clientbound play `Disconnect` mapped to `0x15` on 1.20.3/1.20.4, where that id is
  `set_slot` — so kicking a player on those versions sent them a malformed packet
  instead of a disconnect reason.

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

## `packets/clientbound.json`

Every clientbound packet the server can send, encoded for every version it is sent to,
grouped by shared encoding. Payloads only — the id prefix is not included, because the id
is recorded separately.

Each entry also lists the packet id per version. This is **not** an independent oracle —
those ids come out of the same Java table the Rust table was ported from, so agreement
proves only that the port is faithful, not that either is right. Only
`minecraft-data/` can say that.

It carries a known consequence: `disconnect_play` records `0x15` for protocol 765,
which is the defect described in MIGRATION_PLAN.md 3.1.6 (`0x15` is `set_slot` there;
the correct id is `0x1B`). The Rust table deliberately diverges. The *payload* bytes in
this fixture are unaffected, since a payload does not depend on the id it travels under,
so the level 1 byte comparison stands — but any test comparing ids against this file must
carry that exception explicitly.

Packets are dumped only for versions where the server actually has an id for them, so the
fixture describes what can be sent rather than what an encoder happens to produce when
asked for an impossible combination.

Values the server would draw from a random source are fixed here: entity id 1337,
teleport id 7654321, and UUIDs of the form `00000000-0000-4000-8000-00000000000N`. The
port must accept these through an injected `IdSource` rather than calling a global random,
which is why that port exists at all — see MIGRATION_PLAN.md section 7.5.

## `packets/update_tags.json`

Update tags is the one packet whose bytes are **not** comparable against Java.
`DimensionRegistry.parseUpdateTags` collects into a `HashMap`, so the order it serializes
in is an artifact of Java's hashing rather than anything the protocol specifies. Demanding
the port reproduce that order would be both impossible and pointless.

So this records content instead: per-registry tag and id counts, plus a SHA-256 over
registries and tags sorted by name. A mismatch in the digest says the content differs; the
counts say which registry to look at.

If a future test tries to byte-compare this packet against Java, it is wrong.

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
