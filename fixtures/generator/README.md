# Fixture generator

The two Java files beside this one produce everything in `../text`, `../uuid` and
`../packets`. They are kept here, in this repository, because the fixtures are worthless
without a way to reproduce them and the upstream project has no reason to carry a tool
that only this port needs.

They are not part of the server and are never started by it.

## Running them

```sh
git clone https://github.com/Nan1t/NanoLimbo upstream
cp FixtureDumper.java PacketFixture.java upstream/src/main/java/ua/nanit/limbo/tools/
cd upstream
./gradlew shadowJar
java -cp build/libs/NanoLimbo.jar ua.nanit.limbo.tools.FixtureDumper /path/to/this/repo/fixtures
```

Requires JDK 21, which is what upstream builds against.

## What determinism to expect

Values the server would draw from a random source or a clock are passed in fixed, so
`text/`, `uuid/` and `packets/update_tags.json` reproduce byte for byte. A diff there
without a deliberate change upstream is a bug.

`packets/clientbound.json` does **not** reproduce exactly, and cannot: two of its entries
carry NBT whose compound key order Java randomises per JVM run. `../README.md` explains
which and why.

## If upstream moves on

The dumper reaches into `ByteMessage`, `ComponentUtils`, `DimensionRegistry`, `State` and
the packet classes. A refactor upstream will break the compile rather than silently change
the output, which is the failure mode to prefer — but it does mean these files need
adjusting alongside, and the fixtures regenerating, before a new Minecraft version can be
verified against anything.

At that point it is worth asking whether the fixtures still need regenerating at all. They
were the oracle for the port; once every version they cover is verified, they work as
self-hosted regression snapshots and only a genuinely new version needs new reference
bytes. See `MIGRATION_PLAN.md` section 7.8.
