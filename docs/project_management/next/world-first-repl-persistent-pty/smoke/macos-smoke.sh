#!/usr/bin/env bash
set -euo pipefail

FEATURE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SLICE_ID="${SUBSTRATE_SMOKE_SLICE_ID:-}"

if ! command -v jq >/dev/null 2>&1; then
  echo "ERR: jq is required for smoke scripts" >&2
  exit 3
fi

echo "INFO: feature_dir=$FEATURE_DIR"
echo "INFO: slice_id=${SLICE_ID:-<full>}"

echo "INFO: checking world backend (macOS via Lima)"
substrate world doctor --json | jq -e '.ok == true' >/dev/null
echo "OK: world backend reachable"

echo "INFO: delegating to Linux-equivalent checks on macOS host"
SUBSTRATE_SMOKE_SLICE_ID="${SLICE_ID}" bash "$FEATURE_DIR/smoke/linux-smoke.sh"

