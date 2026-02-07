#!/usr/bin/env bash
set -euo pipefail

SUBSTRATE_BIN="${SUBSTRATE_BIN:-substrate}"
SMOKE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

if [[ "${OSTYPE:-}" != darwin* ]]; then
  echo "world_process_exec_tracing_parity: macos smoke is supported only on macOS (OSTYPE=${OSTYPE:-unknown})" >&2
  exit 4
fi

if ! command -v "$SUBSTRATE_BIN" >/dev/null 2>&1; then
  echo "world_process_exec_tracing_parity: substrate binary not found (SUBSTRATE_BIN=$SUBSTRATE_BIN)" >&2
  exit 3
fi

exec bash "$SMOKE_DIR/_core.sh"

