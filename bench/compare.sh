#!/usr/bin/env bash
#
# Runs the Rust and Java servers side by side and prints what each one costs.
#
# This is the measurement in MIGRATION_PLAN.md section 9, made reproducible. Both servers
# get the same settings.yml, the same number of players and the same client, and their
# memory is read from one place at one moment rather than from two tools at two times.
#
#   ./bench/compare.sh                 300 players, newest protocol
#   ./bench/compare.sh --players 1000
#   ./bench/compare.sh --protocol 47   log in as 1.8 instead
#
# The Java side is skipped, with a note, when no jar has been built.

set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
root="$(cd "$here/.." && pwd)"
java_root="$(cd "$root/.." && pwd)"
jar="$java_root/build/libs/NanoLimbo.jar"
settings="$root/crates/limbo-config/resources/settings.yml"

rust_port=25578
java_port=25577
work="$(mktemp -d)"
rust_pid=""
java_pid=""

cleanup() {
    [ -n "$rust_pid" ] && kill "$rust_pid" 2>/dev/null || true
    [ -n "$java_pid" ] && kill "$java_pid" 2>/dev/null || true
    rm -rf "$work"
}
trap cleanup EXIT

# The shipped file binds to localhost and caps players at 100, neither of which suits a
# benchmark. Everything else is left exactly as a real deployment would have it.
prepare_config() {
    local dir="$1" port="$2"
    mkdir -p "$dir"
    sed -e "s/^  ip: .*/  ip: \"127.0.0.1\"/" \
        -e "s/^  port: .*/  port: $port/" \
        -e "s/^maxPlayers: .*/maxPlayers: 100000/" \
        "$settings" > "$dir/settings.yml"
}

wait_for_port() {
    local port="$1" name="$2"
    for _ in $(seq 1 100); do
        if (exec 3<>"/dev/tcp/127.0.0.1/$port") 2>/dev/null; then
            exec 3>&- 3<&-
            return 0
        fi
        sleep 0.2
    done
    echo "$name did not start listening on port $port; see $work/$name.log" >&2
    return 1
}

echo "building the release binaries..."
cargo build --release --quiet --manifest-path "$root/Cargo.toml"

prepare_config "$work/rust" "$rust_port"
(cd "$work/rust" && exec "$root/target/release/nanolimbo") > "$work/rust.log" 2>&1 &
rust_pid=$!
wait_for_port "$rust_port" rust

targets=("rust=127.0.0.1:$rust_port@$rust_pid")

if [ -f "$jar" ]; then
    prepare_config "$work/java" "$java_port"
    (cd "$work/java" && exec java -jar "$jar") > "$work/java.log" 2>&1 &
    java_pid=$!
    wait_for_port "$java_port" java
    targets+=("java=127.0.0.1:$java_port@$java_pid")
else
    echo
    echo "No Java jar at $jar, so only the Rust server is measured."
    echo "Build it with:  (cd $java_root && ./gradlew shadowJar)"
fi

"$root/target/release/limbo-bench" "$@" "${targets[@]}"

echo "Java heap settings shape its resident figure; this is an out-of-the-box comparison."
