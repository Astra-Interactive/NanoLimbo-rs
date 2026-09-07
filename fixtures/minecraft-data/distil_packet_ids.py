#!/usr/bin/env python3
"""Distil PrismarineJS/minecraft-data packet id tables into one compact fixture.

Each upstream ``protocol.json`` is ~200 KB and describes the full wire layout of every
packet. The cross-check test needs only the id -> name table for each state/direction, so
this script downloads the upstream files, keeps those tables and discards everything else.

The set of protocol versions to cover is read from the Rust source, so the fixture cannot
drift away from the versions the server claims to support.

Usage:
    python3 fixtures/minecraft-data/distil_packet_ids.py

Writes ``fixtures/minecraft-data/packet_ids.json`` and prints a coverage summary.
"""

import datetime
import json
import os
import re
import sys
import urllib.error
import urllib.request

REPOSITORY = "PrismarineJS/minecraft-data"
STATES = ["handshaking", "status", "login", "configuration", "play"]
DIRECTIONS = ["toServer", "toClient"]
PLAIN_RELEASE = re.compile(r"^\d+(\.\d+)*$")

RUST_ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
VERSION_SOURCE = os.path.join(
    RUST_ROOT, "crates", "limbo-protocol", "src", "version", "protocol_version.rs"
)
OUTPUT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "packet_ids.json")


def fetch(url):
    request = urllib.request.Request(url, headers={"User-Agent": "nanolimbo-fixture-generator"})
    with urllib.request.urlopen(request, timeout=120) as response:
        return response.read()


def fetch_json(url):
    return json.loads(fetch(url))


def resolve_commit():
    """Pin the whole run to one commit so a mid-run push cannot mix two snapshots."""
    head = fetch_json(f"https://api.github.com/repos/{REPOSITORY}/commits/master")
    return head["sha"]


def supported_versions():
    """The (protocol number, release name) pairs the Rust server claims to support."""
    source = open(VERSION_SOURCE, encoding="utf-8").read()
    numbers = dict(re.findall(r"pub const (V[\w]+): Self = Self\((\d+)\);", source))
    descriptors = re.findall(
        r"VersionDescriptor::new\(ProtocolVersion::(V[\w]+), \"([^\"]+)\"\)", source
    )
    if len(descriptors) < 2:
        sys.exit(f"could not parse the supported-version table out of {VERSION_SOURCE}")

    versions = []
    for constant, release in descriptors:
        if constant not in numbers:
            sys.exit(f"{constant} appears in the table but has no protocol number")
        versions.append({"protocol": int(numbers[constant]), "release": release})
    return versions


def folder_protocol_number(base, path):
    """Protocol number declared by a folder itself, for releases missing from the index.

    ``data/pc/1.7`` is the only protocol folder minecraft-data does not name after a
    release listed in ``protocolVersions.json``; its ``version.json`` says 1.7.10 / 5.
    """
    try:
        return fetch_json(f"{base}/data/{path}/version.json").get("version")
    except urllib.error.HTTPError:
        return None


def candidate_folders(base):
    """Protocol number -> the minecraft-data releases that publish a protocol.json for it."""
    protocol_paths = fetch_json(f"{base}/data/dataPaths.json")["pc"]
    protocol_numbers = {}
    for entry in fetch_json(f"{base}/data/pc/common/protocolVersions.json"):
        protocol_numbers.setdefault(entry["minecraftVersion"], entry["version"])

    candidates = {}
    for release, paths in protocol_paths.items():
        path = paths.get("protocol")
        if path is None:
            continue
        number = protocol_numbers.get(release)
        if number is None:
            number = folder_protocol_number(base, path)
        if number is None:
            continue
        candidates.setdefault(number, []).append({"release": release, "path": path})
    return candidates


def choose_folder(candidates, release):
    """Pick one upstream folder deterministically.

    A protocol number can be shared by several releases (1.20.5 and 1.20.6 are both 766).
    Prefer the release we name ourselves, then a plain release over a pre-release or
    snapshot, then the lowest version number, so re-running the script is stable.
    """
    if not candidates:
        return None

    def rank(candidate):
        name = candidate["release"]
        parts = tuple(int(part) for part in name.split(".")) if PLAIN_RELEASE.match(name) else ()
        return (name != release, not PLAIN_RELEASE.match(name), parts, name)

    return sorted(candidates, key=rank)[0]


def packet_id_tables(protocol):
    """The id -> name table of every state/direction present in one protocol.json."""
    tables = {}
    for state in STATES:
        directions = protocol.get(state)
        if not isinstance(directions, dict):
            continue
        for direction in DIRECTIONS:
            flow = directions.get(direction)
            if not isinstance(flow, dict):
                continue
            try:
                mappings = flow["types"]["packet"][1][0]["type"][1]["mappings"]
            except (KeyError, IndexError, TypeError):
                continue
            if isinstance(mappings, dict) and mappings:
                tables.setdefault(state, {})[direction] = mappings
    return tables


def main():
    commit = resolve_commit()
    base = f"https://raw.githubusercontent.com/{REPOSITORY}/{commit}"
    candidates = candidate_folders(base)

    protocols = {}
    skipped = []
    downloaded = {}
    for version in supported_versions():
        number = version["protocol"]
        folder = choose_folder(candidates.get(number, []), version["release"])
        if folder is None:
            skipped.append(version)
            continue

        path = folder["path"]
        if path not in downloaded:
            downloaded[path] = json.loads(fetch(f"{base}/data/{path}/protocol.json"))
            print(f"  downloaded {path}", file=sys.stderr)

        protocols[str(number)] = {
            "release": version["release"],
            "minecraft_data_release": folder["release"],
            "minecraft_data_path": path,
            "states": packet_id_tables(downloaded[path]),
        }

    fixture = {
        "source": {
            "repository": REPOSITORY,
            "commit": commit,
            "generated_at": datetime.date.today().isoformat(),
            "generator": "fixtures/minecraft-data/distil_packet_ids.py",
        },
        "protocols": protocols,
    }
    with open(OUTPUT, "w", encoding="utf-8") as output:
        json.dump(fixture, output, indent=1, sort_keys=True)
        output.write("\n")

    print(f"commit {commit}")
    print(f"covered {len(protocols)} protocol versions -> {OUTPUT}")
    print(f"{os.path.getsize(OUTPUT) / 1024:.0f} KiB")
    for version in skipped:
        print(f"not published upstream: {version['release']} (protocol {version['protocol']})")


if __name__ == "__main__":
    main()
