#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
substrate_bin="${repo_root}/target/debug/substrate"

echo "[verify_profile_fail_fast] Building substrate (debug)..."
(cd "${repo_root}" && cargo build -p substrate)

tmpdir="$(mktemp -d)"
trap 'rm -rf "${tmpdir}"' EXIT

mkdir -p "${tmpdir}/proj" "${tmpdir}/home"

echo "[verify_profile_fail_fast] Using temp dir: ${tmpdir}"

echo "[verify_profile_fail_fast] Writing invalid profile..."
printf 'not: [valid: yaml' >"${tmpdir}/proj/.substrate-profile"

echo "[verify_profile_fail_fast] Running with invalid profile (expect failure)..."
invalid_out="${tmpdir}/invalid.out"
set +e
(
  cd "${tmpdir}/proj" || exit 1
  HOME="${tmpdir}/home" USERPROFILE="${tmpdir}/home" "${substrate_bin}" --no-world -c 'true'
) >"${invalid_out}" 2>&1
status=$?
set -e

if [[ ${status} -eq 0 ]]; then
  echo "[verify_profile_fail_fast][FAIL] Expected non-zero exit with invalid profile, got exit=0"
  echo "[verify_profile_fail_fast][FAIL] Output:"
  cat "${invalid_out}"
  exit 1
fi
if ! grep -q "failed to load Substrate profile" "${invalid_out}"; then
  echo "[verify_profile_fail_fast][FAIL] Expected profile load error in output"
  cat "${invalid_out}"
  exit 1
fi
echo "[verify_profile_fail_fast][OK] Invalid profile failed as expected (exit=${status})"

echo "[verify_profile_fail_fast] Writing valid minimal profile..."
cat >"${tmpdir}/proj/.substrate-profile" <<'YAML'
id: ok
name: ok
world_fs:
  mode: writable
  cage: project
  require_world: false
  read_allowlist: ["*"]
  write_allowlist: []
net_allowed: []
cmd_allowed: []
cmd_denied: []
cmd_isolated: []
require_approval: false
allow_shell_operators: true
YAML

echo "[verify_profile_fail_fast] Running with valid profile (expect success)..."
valid_out="${tmpdir}/valid.out"
set +e
(
  cd "${tmpdir}/proj" || exit 1
  HOME="${tmpdir}/home" USERPROFILE="${tmpdir}/home" "${substrate_bin}" --no-world -c 'true'
) >"${valid_out}" 2>&1
status=$?
set -e

if [[ ${status} -ne 0 ]]; then
  echo "[verify_profile_fail_fast][FAIL] Expected exit=0 with valid profile, got exit=${status}"
  cat "${valid_out}"
  exit 1
fi
if grep -q "^Error:" "${valid_out}"; then
  echo "[verify_profile_fail_fast][FAIL] Valid run output contains an Error:"
  cat "${valid_out}"
  exit 1
fi

echo "[verify_profile_fail_fast][OK] Valid profile succeeded"
