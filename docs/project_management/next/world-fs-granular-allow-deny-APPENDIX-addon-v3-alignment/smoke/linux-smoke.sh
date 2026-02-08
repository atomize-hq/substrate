#!/usr/bin/env bash
set -euo pipefail

# Feature smoke (linux) for:
# - world-fs-granular-allow-deny-APPENDIX-addon-v3-alignment
#
# Validates the add-on contract closures:
# - `substrate policy show` output is V3-shaped (Appendix A.6)
# - Legacy V2 policy keys are rejected (no-backcompat; exit 2)

SUBSTRATE_BIN="${SUBSTRATE_BIN:-substrate}"
SMOKE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

if [[ "${OSTYPE:-}" != linux* ]]; then
  echo "wfgad-appendix-addon-v3-alignment: linux smoke is supported only on Linux (OSTYPE=${OSTYPE:-unknown})" >&2
  exit 4
fi

if ! command -v "$SUBSTRATE_BIN" >/dev/null 2>&1; then
  echo "wfgad-appendix-addon-v3-alignment: substrate binary not found (SUBSTRATE_BIN=$SUBSTRATE_BIN)" >&2
  exit 3
fi

exec bash "$SMOKE_DIR/_core.sh"
