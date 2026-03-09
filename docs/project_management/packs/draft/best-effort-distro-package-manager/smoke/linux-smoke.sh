#!/usr/bin/env bash
set -euo pipefail

if [[ "$(uname -s)" != "Linux" ]]; then
  echo "SKIP: linux-smoke.sh intended for Linux (uname=$(uname -s))"
  exit 0
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../../../../.." && pwd)"
HARNESS_PATH="${REPO_ROOT}/tests/installers/pkg_manager_detection_smoke.sh"

if [[ ! -f "${HARNESS_PATH}" ]]; then
  echo "FAIL: missing authoritative BEDPM repo harness at ${HARNESS_PATH}" >&2
  exit 2
fi

exec bash "${HARNESS_PATH}"
