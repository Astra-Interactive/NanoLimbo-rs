# minecraft-data packet id fixture

`packet_ids.json` is a distilled snapshot of the packet id tables published by
[`PrismarineJS/minecraft-data`](https://github.com/PrismarineJS/minecraft-data). It backs
`crates/limbo-protocol/tests/minecraft_data_cross_check.rs`, the level-0 independent-oracle
check described in `MIGRATION_PLAN.md` §7.2 and §7.4.

## Why it exists

Every other level of the test suite compares the port against the Java implementation. That
proves equivalence, not correctness: a bug already present in Java is reproduced byte for
byte and the comparison stays green. `minecraft-data` is extracted from the vanilla client
and owes nothing to NanoLimbo, so a disagreement between the two tables is evidence that one
of them is wrong.

It has paid for itself twice so far:

- serverbound configuration `custom_payload` is `0x01` on 1.20.2–1.20.3 and `0x02` from
  1.20.5; the Java table started `0x02` at 1.20.2, where `0x02` is `finish_configuration`
  (`MIGRATION_PLAN.md` §3.1.5);
- clientbound play `Disconnect` stays at `0x1B` on 1.20.3; the Java table drops it to `0x15`,
  which is `set_slot` there — a copy of the serverbound `KeepAlive` line above it.

## Provenance

| | |
|---|---|
| Repository | `PrismarineJS/minecraft-data` |
| Commit | `3d75720098b35bdacaa9a6ba56bb9f53e93a9a66` |
| Generated | 2026-09-07 |
| Versions covered | 48 of the 51 the server supports |

The `source` object inside `packet_ids.json` repeats the commit and date, so the file is
self-describing even when read on its own.

## Versions the snapshot does not cover

`minecraft-data` publishes no `protocol.json` for these, so the cross-check skips and reports
them rather than passing silently:

| Release | Protocol | Why |
|---|---|---|
| 1.7.2 | 4 | upstream ships only 1.7.10 for the 1.7 line (protocol 5) |
| 1.14.2 | 485 | upstream jumps from 1.14.1 (480) to 1.14.3 (490) |
| 26.2 | 776 | not released upstream yet |

Where upstream names a release differently from us but shares the protocol number, the
number wins: `pc/1.7` (1.7.10) covers protocol 5, `pc/1.19.2` covers 760, `pc/1.21.3` covers
768, `pc/1.21.8` covers 772. Each entry records the folder it came from in
`minecraft_data_path`, so any mapping can be traced back to its upstream source.

## Shape

Only the id → name tables survive the distillation; the field layouts, which make each
upstream `protocol.json` roughly 200 KB, are dropped. The result is one file of about 270 KB
instead of roughly 9 MB.

```json
{
  "source": { "repository": "...", "commit": "...", "generated_at": "...", "generator": "..." },
  "protocols": {
    "764": {
      "release": "1.20.2",
      "minecraft_data_release": "1.20.2",
      "minecraft_data_path": "pc/1.20.2",
      "states": {
        "configuration": {
          "toServer": { "0x00": "settings", "0x01": "custom_payload" },
          "toClient": { "0x00": "custom_payload" }
        }
      }
    }
  }
}
```

Keys of `protocols` are protocol numbers as strings; state keys are `handshaking`, `status`,
`login`, `configuration`, `play`; direction keys are `toServer` and `toClient`.

## Refreshing it

```sh
python3 fixtures/minecraft-data/distil_packet_ids.py
```

The script needs network access and nothing else — no third-party Python packages. It

1. resolves `master` to a commit sha and downloads everything from that sha, so a push
   mid-run cannot mix two snapshots;
2. reads the supported-version table straight out of
   `crates/limbo-protocol/src/version/protocol_version.rs`, so the fixture cannot drift away
   from the versions the server claims to support;
3. resolves each protocol number to an upstream folder through `data/dataPaths.json` and
   `data/pc/common/protocolVersions.json`, preferring the release we name ourselves and a
   plain release over a pre-release, so re-running it is stable;
4. keeps only
   `json[<state>][<direction>]["types"]["packet"][1][0]["type"][1]["mappings"]`;
5. prints the commit, the coverage count and every version it could not cover.

After refreshing, update the commit and date in the table above, then run
`cargo test -p limbo-protocol --test minecraft_data_cross_check -- --nocapture`. A new
disagreement is a question to investigate against the Java source and the wiki, not a number
to overwrite: `minecraft-data` is occasionally wrong too, and the test carries a small,
individually justified allowlist for the cases where it is.
