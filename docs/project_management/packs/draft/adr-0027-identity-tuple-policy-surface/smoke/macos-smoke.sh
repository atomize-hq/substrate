#!/usr/bin/env bash
set -euo pipefail

SMOKE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

if [[ "${OSTYPE:-}" != darwin* ]]; then
  echo "adr-0027-identity-tuple-policy-surface: macos smoke is supported only on macOS (OSTYPE=${OSTYPE:-unknown})" >&2
  exit 4
fi

ITPS_SMOKE_PLATFORM=macos exec bash "$SMOKE_DIR/_core.sh"
