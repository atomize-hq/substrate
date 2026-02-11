#!/usr/bin/env bash
set -euo pipefail

# Feature smoke (macOS) for:
# - world-fs-granular-allow-deny-APPENDIX-addon-v3-alignment
#
# This smoke is host-only (no Lima/world backend required). It validates:
# - `substrate policy show` output is V3-shaped (Appendix A.6)
# - Legacy V2 policy keys are rejected (no-backcompat; exit 2)

SUBSTRATE_BIN="${SUBSTRATE_BIN:-substrate}"
SMOKE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

if [[ "${OSTYPE:-}" != darwin* ]]; then
  echo "wfgad-appendix-addon-v3-alignment: macOS smoke is supported only on macOS (OSTYPE=${OSTYPE:-unknown})" >&2
  exit 4
fi

if ! command -v "$SUBSTRATE_BIN" >/dev/null 2>&1; then
  echo "wfgad-appendix-addon-v3-alignment: substrate binary not found (SUBSTRATE_BIN=$SUBSTRATE_BIN)" >&2
  exit 3
fi

exec bash "$SMOKE_DIR/_core.sh"

