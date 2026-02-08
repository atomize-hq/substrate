#!/usr/bin/env bash
set -euo pipefail

SUBSTRATE_BIN="${SUBSTRATE_BIN:-substrate}"
SMOKE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

if [[ "${OSTYPE:-}" != linux* ]]; then
  echo "llm_and_agent_config_policy_surface: linux smoke is supported only on Linux (OSTYPE=${OSTYPE:-unknown})" >&2
  exit 4
fi

if ! command -v "$SUBSTRATE_BIN" >/dev/null 2>&1; then
  echo "llm_and_agent_config_policy_surface: substrate binary not found (SUBSTRATE_BIN=$SUBSTRATE_BIN)" >&2
  exit 3
fi

exec bash "$SMOKE_DIR/_core.sh"

