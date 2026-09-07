#!/usr/bin/env bash
#
# Loads this server, and optionally the Java original beside it, and prints what each costs.
#
# This is the measurement in MIGRATION_PLAN.md section 9, made reproducible. Both servers
# get the same settings.yml, the same number of players and the same client, and their
# memory is read from one place at one moment rather than from two tools at two times.
#
#   ./bench/compare.sh                              this server alone
#   ./bench/compare.sh --players 1000
#   ./bench/compare.sh --protocol 47                log in as 1.8 instead
#   ./bench/compare.sh --jar ../NanoLimbo.jar       compare against the original
#
# Upstream lives in a separate repository, so its jar has to be pointed at rather than
# assumed. Build one with:
#
#   git clone https://github.com/Nan1t/NanoLimbo && cd NanoLimbo && ./gradlew shadowJar

set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
root="$(cd "$here/.." && pwd)"
settings="$root/crates/limbo-config/resources/settings.yml"

# --jar is ours; everything else is passed through to limbo-bench untouched, so its own
# options need no mirroring here.
jar="${NANOLIMBO_JAR:-}"
bench_arguments=()
while [ $# -gt 0 ]; do
    case "$1" in
        --jar)
            [ $# -ge 2 ] || { echo "--jar expects a path" >&2; exit 1; }
            jar="$2"
            shift 2
            ;;
        *)
            bench_arguments+=("$1")
            shift
            ;;
    esac
done

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

# Each server runs from its own working directory, so a path relative to the caller's
# would be resolved against the wrong one.
if [ -n "$jar" ] && [ -f "$jar" ]; then
    jar="$(cd "$(dirname "$jar")" && pwd)/$(basename "$jar")"
fi

if [ -n "$jar" ] && [ -f "$jar" ]; then
    prepare_config "$work/java" "$java_port"
    (cd "$work/java" && exec java -jar "$jar") > "$work/java.log" 2>&1 &
    java_pid=$!
    wait_for_port "$java_port" java
    targets+=("java=127.0.0.1:$java_port@$java_pid")
elif [ -n "$jar" ]; then
    echo
    echo "No jar at $jar, so only this server is measured." >&2
else
    echo
    echo "Measuring this server alone. Pass --jar <path to NanoLimbo.jar> to compare"
    echo "against the original."
fi

"$root/target/release/limbo-bench" "${bench_arguments[@]}" "${targets[@]}"

if [ -n "$java_pid" ]; then
    echo "Java heap settings shape its resident figure; this is an out-of-the-box comparison."
fi
