#!/usr/bin/env bash
set -euo pipefail

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "SKIP: world-sync macOS smoke (not macOS)"
  exit 0
fi

if ! command -v substrate >/dev/null 2>&1; then
  echo "FAIL: substrate not found on PATH" >&2
  exit 1
fi

SLICE_ID="${SUBSTRATE_SMOKE_SLICE_ID:-WS7}"

WS_DIR="$(mktemp -d)"
cleanup() { rm -rf "$WS_DIR"; }
trap cleanup EXIT

cd "$WS_DIR"
substrate workspace init . >/dev/null

case "$SLICE_ID" in
  WS2)
    substrate --world -c "sh -lc 'echo hello > hello-from-world.txt'"
    substrate workspace sync --direction from_world --verbose >/dev/null
    test -f hello-from-world.txt
    substrate workspace sync --direction from_world >/dev/null
    echo "OK: world-sync macOS smoke ($SLICE_ID)"
    ;;
  WS5)
    echo "host" > host-only.txt
    substrate workspace sync --dry-run --direction from_host --verbose >/dev/null
    substrate workspace sync --direction from_host --verbose >/dev/null
    substrate --world -c "sh -lc 'echo w > hello-both.txt'"
    substrate workspace sync --direction both --verbose >/dev/null
    test -f hello-both.txt
    echo "OK: world-sync macOS smoke ($SLICE_ID)"
    ;;
  WS7)
    substrate workspace checkpoint --message "smoke" >/dev/null
    echo "mutated" > mutation.txt
    set +e
    substrate workspace rollback last >/dev/null 2>&1
    code=$?
    set -e
    test "$code" -eq 5
    substrate workspace rollback last --force >/dev/null
    test ! -f mutation.txt
    echo "OK: world-sync macOS smoke ($SLICE_ID)"
    ;;
  *)
    echo "FAIL: unsupported SUBSTRATE_SMOKE_SLICE_ID=$SLICE_ID" >&2
    exit 1
    ;;
esac
