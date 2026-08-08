#!/usr/bin/env bash
# Focused cross-compile regression for the R3 Lima guest executor. It performs no lifecycle action.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
TARGET="aarch64-unknown-linux-gnu"
BASE_COMMIT="3d6b2eb1b02b1a24a1e055d12e5cbdb9b0312774"
LINKER="/Users/spensermcconnell/.codex/evidence/substrate-a1-1d-5r3-mac-20260806-41f97e1c570d/AUX-R3-MAC-AARCH64-TOOLCHAIN-PREP/d514db6915877692c799086ce6c96daad6bcd0417d87a1bdf9ba44c2c578da53/linker/aarch64-linux-gnu-zig-cc"
MODE="${1:-check}"

if [[ "$MODE" != "check" && "$MODE" != "expect-base-failure" ]]; then
    echo "usage: $0 [check|expect-base-failure]" >&2
    exit 2
fi

TARGET_DIR="$(mktemp -d "${TMPDIR:-/tmp}/substrate-aarch64-guest-compile-r3-target.XXXXXX")"
LOG_DIR="$(mktemp -d "${TMPDIR:-/tmp}/substrate-aarch64-guest-compile-r3-log.XXXXXX")"
SOURCE_DIR="$REPO_ROOT"
trap 'rm -rf "$TARGET_DIR" "$LOG_DIR" "${BASE_SOURCE_DIR:-}"' EXIT

if [[ ! -x "$LINKER" ]]; then
    echo "missing required aarch64 Linux linker wrapper: $LINKER" >&2
    exit 1
fi

command=(
    cargo check
    --locked
    --offline
    --target "$TARGET"
    --bin substrate-lifecycle-linux
)

run_check() {
    (
        cd "$SOURCE_DIR"
        CARGO_TARGET_DIR="$TARGET_DIR" \
            CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER="$LINKER" \
            CC_aarch64_unknown_linux_gnu="$LINKER" \
            "${command[@]}"
    )
}

if [[ "$MODE" == "expect-base-failure" ]]; then
    BASE_SOURCE_DIR="$(mktemp -d "${TMPDIR:-/tmp}/substrate-aarch64-guest-compile-r3-base.XXXXXX")"
    git -C "$REPO_ROOT" archive "$BASE_COMMIT" | tar -x -C "$BASE_SOURCE_DIR"
    SOURCE_DIR="$BASE_SOURCE_DIR"
    set +e
    run_check >"$LOG_DIR/cargo-check.log" 2>&1
    status=$?
    set -e

    cat "$LOG_DIR/cargo-check.log"
    if [[ "$status" -eq 0 ]]; then
        echo "expected the base aarch64 guest compile to fail" >&2
        exit 1
    fi
    error_count="$(grep -c 'error\[E0308\]' "$LOG_DIR/cargo-check.log" || true)"
    if [[ "$error_count" -ne 4 ]]; then
        echo "expected exactly four E0308 errors, got $error_count" >&2
        exit 1
    fi
    echo "reproduced the four expected aarch64 guest pointer-type E0308 errors"
    exit 0
fi

run_check
