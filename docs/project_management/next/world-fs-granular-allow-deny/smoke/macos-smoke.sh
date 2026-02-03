#!/usr/bin/env bash
set -euo pipefail

SUBSTRATE_BIN="${SUBSTRATE_BIN:-substrate}"
SMOKE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SMOKE_DIR}/../../../../.." && pwd)"

if [[ "${OSTYPE:-}" != darwin* ]]; then
  echo "world-fs-granular-allow-deny: macOS smoke is supported only on macOS (OSTYPE=${OSTYPE:-unknown})" >&2
  exit 4
fi

if ! command -v limactl >/dev/null 2>&1; then
  PATH="/opt/homebrew/opt/lima/bin:/opt/homebrew/bin:$PATH"
fi

if ! command -v limactl >/dev/null 2>&1; then
  echo "world-fs-granular-allow-deny: limactl not found on PATH (install with: brew install lima)" >&2
  exit 4
fi

echo "== Preflight: warm Lima world (macOS -> Linux VM) =="
bash "$REPO_ROOT/scripts/mac/lima-warm.sh" "$REPO_ROOT"

if ! command -v "$SUBSTRATE_BIN" >/dev/null 2>&1; then
  echo "world-fs-granular-allow-deny: substrate binary not found (SUBSTRATE_BIN=$SUBSTRATE_BIN)" >&2
  exit 3
fi

smoke_root="$REPO_ROOT/.smoke/world-fs-granular-allow-deny.$RANDOM.$RANDOM"
mkdir -p "$smoke_root"
export SUBSTRATE_SMOKE_ROOT="$smoke_root"

exec bash "$SMOKE_DIR/_core.sh"
