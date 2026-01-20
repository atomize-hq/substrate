#!/usr/bin/env bash
set -euo pipefail

fail() {
  echo "FAIL: $*" >&2
  exit 1
}

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || fail "missing dependency: $1"
}

if [[ "$(uname -s)" != "Linux" ]]; then
  echo "SKIP: full-isolation-landlock-overlayfs-compat smoke (not Linux)"
  exit 0
fi

need_cmd cargo
need_cmd git
need_cmd jq
need_cmd mktemp

if [[ "${EUID}" -eq 0 ]]; then
  fail "do not run as root"
fi

REPO_ROOT="$(git rev-parse --show-toplevel)"
cd "${REPO_ROOT}"
cargo build --bin substrate >/dev/null
export PATH="${REPO_ROOT}/target/debug:${PATH}"
need_cmd substrate

TMP_HOME="$(mktemp -d)"
TMP_WS="$(mktemp -d)"
cleanup() { rm -rf "${TMP_HOME}" "${TMP_WS}"; }
trap cleanup EXIT

export SUBSTRATE_HOME="${TMP_HOME}"
export HOME="${TMP_HOME}"

substrate config global init --force >/dev/null
substrate policy global init --force >/dev/null
substrate config global set policy.mode=enforce >/dev/null
substrate config global set world.enabled=true >/dev/null
substrate config global set world.anchor_mode=follow-cwd >/dev/null

substrate workspace init "${TMP_WS}" >/dev/null
cd "${TMP_WS}"
mkdir -p writable

substrate policy workspace init --force >/dev/null
substrate policy workspace set \
  world_fs.mode=writable \
  world_fs.isolation=full \
  world_fs.require_world=true \
  world_fs.write_allowlist='["./writable/*"]' \
  >/dev/null

doctor_json="$(substrate world doctor --json 2>/dev/null)" || {
  echo "${doctor_json:-}" >&2
  exit 3
}

doctor_ok="$(printf '%s' "${doctor_json}" | jq -r '.ok // false' 2>/dev/null || printf 'false')"
if [[ "${doctor_ok}" != "true" ]]; then
  echo "${doctor_json}" >&2
  exit 3
fi

landlock_supported="$(printf '%s' "${doctor_json}" | jq -r '.world.landlock.supported // false' 2>/dev/null || printf 'false')"
if [[ "${landlock_supported}" != "true" ]]; then
  echo "${doctor_json}" >&2
  exit 4
fi

fs_primary="$(printf '%s' "${doctor_json}" | jq -r '.world.world_fs_strategy.primary // ""' 2>/dev/null || printf '')"
if [[ "${fs_primary}" != "overlay" ]]; then
  echo "${doctor_json}" >&2
  exit 4
fi

set +e
allow_out="$(substrate --world --ci --command "sh -lc 'set -eu\nmkdir -p writable/sub\necho OK > writable/sub/ok.txt\ncat writable/sub/ok.txt\n' " 2>&1)"
allow_rc=$?
set -e
if [[ "${allow_rc}" -ne 0 ]]; then
  echo "${allow_out}" >&2
  fail "allowlisted write failed (expected exit 0)"
fi

if [[ -e "${TMP_WS}/writable/sub/ok.txt" ]]; then
  fail "allowlisted write mutated host project directory"
fi
echo "OK: allowlisted write succeeded"

set +e
deny_out="$(substrate --world --ci --command "sh -lc 'set -eu\nif echo NOPE > denied.txt 2>/dev/null; then\n  echo UNEXPECTED_WRITE\n  exit 41\nelse\n  echo DENIED_WRITE\nfi\n' " 2>&1)"
deny_rc=$?
set -e
if [[ "${deny_rc}" -ne 0 ]]; then
  echo "${deny_out}" >&2
  fail "denied-write check failed (expected exit 0)"
fi
if [[ "${deny_out}" != *"DENIED_WRITE"* ]]; then
  echo "${deny_out}" >&2
  fail "missing DENIED_WRITE marker"
fi
if [[ "${deny_out}" == *"UNEXPECTED_WRITE"* ]]; then
  echo "${deny_out}" >&2
  fail "unexpected write succeeded outside allowlist"
fi
if [[ -e "${TMP_WS}/denied.txt" ]]; then
  fail "denied write mutated host project directory"
fi
echo "OK: denied write remained denied"

echo "OK: linux smoke passed"
