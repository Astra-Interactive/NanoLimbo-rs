#!/usr/bin/env bash
#
# The same comparison as compare.sh, against the two containers instead of two local
# processes. Memory is read with `docker stats`, so both figures still come from one
# source at one moment.
#
#   ./bench/compare-docker.sh [--players N] [--protocol N]
#
# Requires a Java jar to compare against:  (cd .. && ./gradlew shadowJar)

set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
root="$(cd "$here/.." && pwd)"
cd "$root"

rust_service=nanolimbo
java_service=nanolimbo-java

memory_of() {
    # `docker stats` prints "12.3MiB / 7.7GiB"; the part before the slash is what is used.
    docker stats --no-stream --format '{{.MemUsage}}' "$1" 2>/dev/null | cut -d/ -f1 | tr -d ' '
}

echo "starting both servers..."
docker compose --profile compare up --build -d

# Compose names containers <project>_<service>_1 or <project>-<service>-1 depending on
# version, so ask compose rather than guessing.
rust_container="$(docker compose ps -q "$rust_service")"
java_container="$(docker compose ps -q "$java_service")"

cleanup() { docker compose --profile compare down --remove-orphans >/dev/null 2>&1 || true; }
trap cleanup EXIT

echo "waiting for both to listen..."
for _ in $(seq 1 100); do
    if (exec 3<>/dev/tcp/127.0.0.1/25565) 2>/dev/null && (exec 3<>/dev/tcp/127.0.0.1/25577) 2>/dev/null; then
        break
    fi
    sleep 0.3
done

echo "idle:  rust $(memory_of "$rust_container")   java $(memory_of "$java_container")"

cargo build --release --quiet --bin limbo-bench
./target/release/limbo-bench "$@" rust=127.0.0.1:25565 java=127.0.0.1:25577

echo "loaded: rust $(memory_of "$rust_container")   java $(memory_of "$java_container")"
echo
echo "Memory comes from docker stats, so it counts the whole container."
echo "Java heap settings shape its figure; this is an out-of-the-box comparison."
