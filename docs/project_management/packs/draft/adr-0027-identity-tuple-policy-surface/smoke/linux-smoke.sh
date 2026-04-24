#!/usr/bin/env bash
set -euo pipefail

SMOKE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

if [[ "${OSTYPE:-}" != linux* ]]; then
  echo "adr-0027-identity-tuple-policy-surface: linux smoke is supported only on Linux (OSTYPE=${OSTYPE:-unknown})" >&2
  exit 4
fi

ITPS_SMOKE_PLATFORM=linux exec bash "$SMOKE_DIR/_core.sh"
