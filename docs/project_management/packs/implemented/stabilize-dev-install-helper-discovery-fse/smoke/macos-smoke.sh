#!/usr/bin/env bash
set -euo pipefail

SMOKE_NAME="stabilize-dev-install-helper-discovery-macos-smoke"

log() {
  printf '[%s] %s\n' "${SMOKE_NAME}" "$1"
}

fatal() {
  printf '[%s][ERROR] %s\n' "${SMOKE_NAME}" "$1" >&2
  exit 1
}

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    fatal "Missing required command: $1"
  fi
}

run_step() {
  local label="$1"
  shift
  log "RUN ${label}"
  "$@"
}

require_cmd bash
require_cmd cargo
require_cmd git

REPO_ROOT="${REPO_ROOT:-}"
if [[ -z "${REPO_ROOT}" ]]; then
  REPO_ROOT="$(git rev-parse --show-toplevel)"
fi

if [[ ! -d "${REPO_ROOT}/scripts/substrate" ]]; then
  fatal "Expected repo root at ${REPO_ROOT}, but scripts/substrate is missing"
fi

cd "${REPO_ROOT}"

log "macOS smoke is limited to helper discovery, validation, and managed cleanup."
log "It does not claim full provisioning parity or release-root staging parity."

run_step \
  "world_enable_prefers_prefix_runtime_bundle_without_override" \
  cargo test -p shell world_enable_prefers_prefix_runtime_bundle_without_override -- --exact --nocapture

run_step \
  "world_enable_uses_prefix_runtime_bundle_when_version_binary_is_missing" \
  cargo test -p shell world_enable_uses_prefix_runtime_bundle_when_version_binary_is_missing -- --exact --nocapture

run_step \
  "dev-runtime-bundle" \
  tests/mac/installer_parity_fixture.sh --scenario dev-runtime-bundle

run_step \
  "dev-runtime-bundle-self-contained" \
  tests/mac/installer_parity_fixture.sh --scenario dev-runtime-bundle-self-contained

run_step \
  "dev-runtime-bundle-protected-path-conflicts" \
  tests/mac/installer_parity_fixture.sh --scenario dev-runtime-bundle-protected-path-conflicts

log "macOS smoke complete"
