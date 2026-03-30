#!/usr/bin/env bash
set -euo pipefail

SMOKE_NAME="dev-install-world-agent-staging-linux-smoke"

log() { printf '[%s] %s\n' "${SMOKE_NAME}" "$1"; }
fatal() { printf '[%s][ERROR] %s\n' "${SMOKE_NAME}" "$1" >&2; exit 1; }

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    fatal "Missing required command: $1"
  fi
}

require_cmd bash
require_cmd cargo
require_cmd git
require_cmd jq
require_cmd mktemp
require_cmd readlink

REPO_ROOT="${REPO_ROOT:-}"
if [[ -z "${REPO_ROOT}" ]]; then
  REPO_ROOT="$(git rev-parse --show-toplevel)"
fi

if [[ ! -d "${REPO_ROOT}/scripts/substrate" ]]; then
  fatal "Expected repo root at ${REPO_ROOT}, but scripts/substrate is missing"
fi

tmp_root="$(mktemp -d)"
cleanup() { rm -rf "${tmp_root}"; }
trap cleanup EXIT

export SUBSTRATE_HOME="${tmp_root}/substrate-home"
mkdir -p "${SUBSTRATE_HOME}"

log "Repo root: ${REPO_ROOT}"
log "Temp SUBSTRATE_HOME: ${SUBSTRATE_HOME}"

cd "${REPO_ROOT}"

assert_link_points_to() {
  local link_path="$1"
  local expected_target="$2"

  if [[ ! -L "${link_path}" ]]; then
    fatal "Expected symlink at ${link_path}"
  fi

  local actual
  actual="$(readlink "${link_path}")"
  if [[ "${actual}" != "${expected_target}" ]]; then
    fatal "Expected ${link_path} -> ${expected_target}, got ${actual}"
  fi
}

assert_contains() {
  local haystack="$1"
  local needle="$2"
  if [[ "${haystack}" != *"${needle}"* ]]; then
    fatal "Expected output to contain ${needle}"
  fi
}

log "Case 1: dev-install --no-world stages world-agent for debug profile"
scripts/substrate/dev-install-substrate.sh --prefix "${SUBSTRATE_HOME}" --profile debug --no-world --no-shims

expected_debug_world_agent="${REPO_ROOT}/target/debug/world-agent"
assert_link_points_to "target/bin/world-agent" "${expected_debug_world_agent}"
assert_link_points_to "target/bin/linux/world-agent" "${expected_debug_world_agent}"

if ! grep -qE '^[[:space:]]+enabled:[[:space:]]+false[[:space:]]*$' "${SUBSTRATE_HOME}/config.yaml"; then
  fatal "Expected ${SUBSTRATE_HOME}/config.yaml to keep world.enabled: false after --no-world"
fi

log "Case 2: repeated dev-install refreshes staged world-agent bridge (debug -> release)"
scripts/substrate/dev-install-substrate.sh --prefix "${SUBSTRATE_HOME}" --profile release --no-world --no-shims
expected_release_world_agent="${REPO_ROOT}/target/release/world-agent"
assert_link_points_to "target/bin/world-agent" "${expected_release_world_agent}"
assert_link_points_to "target/bin/linux/world-agent" "${expected_release_world_agent}"

log "Case 3: missing-artifact dry-run fails early with exit code 3 and remediation"
rm -f target/bin/world-agent target/bin/linux/world-agent

set +e
dry_run_output="$("${SUBSTRATE_HOME}/bin/substrate" world enable --home "${SUBSTRATE_HOME}" --dry-run 2>&1)"
dry_run_status=$?
set -e

if [[ "${dry_run_status}" -ne 3 ]]; then
  fatal "Expected world enable --dry-run to exit 3 when staged world-agent is missing (got ${dry_run_status})"
fi

assert_contains "${dry_run_output}" "target/bin/world-agent"
assert_contains "${dry_run_output}" "target/bin/linux/world-agent"
assert_contains "${dry_run_output}" "scripts/substrate/dev-install-substrate.sh --no-world"
assert_contains "${dry_run_output}" "cargo build -p world-agent"

log "Smoke complete"

