#!/usr/bin/env bash
set -euo pipefail

slice="${SUBSTRATE_SMOKE_SLICE_ID:-}"
if [[ -n "${slice}" && "${slice}" != "C0" && "${slice}" != "C1" ]]; then
  echo "SKIP: SUBSTRATE_SMOKE_SLICE_ID=${slice} (supported: C0,C1)"
  exit 0
fi

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "missing dependency: $1" >&2
    exit 3
  fi
}

require_cmd cargo
require_cmd python3
require_cmd git

REPO_ROOT="$(git rev-parse --show-toplevel)"
DRIVER="${REPO_ROOT}/scripts/dev/substrate_shell_driver"

tmp="$(mktemp -d)"
cleanup() { rm -rf "${tmp}"; }
trap cleanup EXIT

export SUBSTRATE_HOME="${tmp}/home"
mkdir -p "${SUBSTRATE_HOME}"

workspace="${tmp}/ws"
mkdir -p "${workspace}"
cd "${workspace}"

cd "${REPO_ROOT}"
cargo build --bin substrate

cd "${workspace}"
"${DRIVER}" workspace init --force

run_c0() {
  mkdir -p .substrate
  cat >"${SUBSTRATE_HOME}/policy.yaml" <<'YAML'
world_fs:
  require_world: true
YAML

  cat >.substrate/policy.yaml <<'YAML'
world_fs:
  require_world: false
YAML

  effective_json="$("${DRIVER}" policy current show --json 2>/dev/null)"
  python3 - <<'PY' "${effective_json}"
import json, sys
data = json.loads(sys.argv[1])
assert data["world_fs_require_world"] is False, data
PY

  touch .substrate/workspace.disabled
  effective_json="$("${DRIVER}" policy current show --json 2>/dev/null)"
  python3 - <<'PY' "${effective_json}"
import json, sys
data = json.loads(sys.argv[1])
assert data["world_fs_require_world"] is True, data
PY
  rm -f .substrate/workspace.disabled
}

run_c1() {
  rm -f .substrate/workspace.disabled || true
  cat >.substrate/policy.yaml <<'YAML'
world_fs: [
YAML

  set +e
  out="$("${DRIVER}" --command "echo SMOKE_POLICY_INVALID_YAML_RAN" 2>&1)"
  rc=$?
  set -e

  if [[ "${rc}" -ne 2 ]]; then
    echo "FAIL: expected exit 2 for invalid policy patch, got ${rc}" >&2
    echo "${out}" >&2
    exit 1
  fi

  if [[ "${out}" == *"SMOKE_POLICY_INVALID_YAML_RAN"* ]]; then
    echo "FAIL: command executed despite invalid policy patch" >&2
    echo "${out}" >&2
    exit 1
  fi
}

if [[ -z "${slice}" || "${slice}" == "C0" ]]; then
  run_c0
fi
if [[ -z "${slice}" || "${slice}" == "C1" ]]; then
  run_c1
fi

echo "OK: linux smoke passed"
