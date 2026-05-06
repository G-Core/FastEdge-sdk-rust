#!/usr/bin/env bash
# Build all FastEdge SDK examples.
#
# Each example is a standalone crate (its own [workspace] root), so a single
# cargo build from the repo root won't pick them up. This script iterates over
# each example directory and runs `cargo build --release` with the right target.
#
# Targets:
#   examples/cdn/*          -> wasm32-wasip1
#   examples/http/basic/*   -> wasm32-wasip1
#   examples/http/wasi/*    -> wasm32-wasip2
#
# Usage:
#   ./build-examples.sh                # build everything
#   ./build-examples.sh cdn            # build only CDN examples
#   ./build-examples.sh http-basic     # build only http/basic examples
#   ./build-examples.sh http-wasi      # build only http/wasi examples
#   ./build-examples.sh clean          # cargo clean every example

set -u

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$REPO_ROOT"

GROUP="${1:-all}"

PASSED=()
FAILED=()
SKIPPED=()

build_group() {
    local label="$1"
    local target="$2"
    local glob="$3"

    shopt -s nullglob
    local dirs=($glob)
    shopt -u nullglob

    if [ ${#dirs[@]} -eq 0 ]; then
        echo "[$label] no example directories matched: $glob"
        return
    fi

    echo
    echo "=========================================="
    echo "  Building $label (target: $target)"
    echo "=========================================="

    for dir in "${dirs[@]}"; do
        [ -f "$dir/Cargo.toml" ] || { SKIPPED+=("$dir (no Cargo.toml)"); continue; }
        local name="${dir#$REPO_ROOT/}"
        echo
        echo "--- $name ---"
        if (cd "$dir" && cargo build --release --target "$target"); then
            PASSED+=("$name")
        else
            FAILED+=("$name")
        fi
    done
}

clean_group() {
    local label="$1"
    local glob="$2"

    shopt -s nullglob
    local dirs=($glob)
    shopt -u nullglob

    if [ ${#dirs[@]} -eq 0 ]; then
        echo "[$label] no example directories matched: $glob"
        return
    fi

    echo
    echo "=========================================="
    echo "  Cleaning $label"
    echo "=========================================="

    for dir in "${dirs[@]}"; do
        [ -f "$dir/Cargo.toml" ] || { SKIPPED+=("$dir (no Cargo.toml)"); continue; }
        local name="${dir#$REPO_ROOT/}"
        echo
        echo "--- $name ---"
        if (cd "$dir" && cargo clean); then
            PASSED+=("$name")
        else
            FAILED+=("$name")
        fi
    done
}

case "$GROUP" in
    cdn)
        build_group "cdn" "wasm32-wasip1" "$REPO_ROOT/examples/cdn/*/"
        ;;
    http-basic)
        build_group "http/basic" "wasm32-wasip1" "$REPO_ROOT/examples/http/basic/*/"
        ;;
    http-wasi)
        build_group "http/wasi" "wasm32-wasip2" "$REPO_ROOT/examples/http/wasi/*/"
        ;;
    all)
        build_group "cdn"        "wasm32-wasip1" "$REPO_ROOT/examples/cdn/*/"
        build_group "http/basic" "wasm32-wasip1" "$REPO_ROOT/examples/http/basic/*/"
        build_group "http/wasi"  "wasm32-wasip2" "$REPO_ROOT/examples/http/wasi/*/"
        ;;
    clean)
        clean_group "cdn"        "$REPO_ROOT/examples/cdn/*/"
        clean_group "http/basic" "$REPO_ROOT/examples/http/basic/*/"
        clean_group "http/wasi"  "$REPO_ROOT/examples/http/wasi/*/"
        ;;
    *)
        echo "Unknown group: $GROUP" >&2
        echo "Usage: $0 [all|cdn|http-basic|http-wasi|clean]" >&2
        exit 2
        ;;
esac

echo
echo "=========================================="
echo "  Summary"
echo "=========================================="
echo "  passed:  ${#PASSED[@]}"
echo "  failed:  ${#FAILED[@]}"
echo "  skipped: ${#SKIPPED[@]}"

if [ ${#FAILED[@]} -gt 0 ]; then
    echo
    echo "Failed examples:"
    for f in "${FAILED[@]}"; do echo "  - $f"; done
    exit 1
fi
