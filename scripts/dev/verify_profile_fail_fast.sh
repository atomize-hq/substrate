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
set +e
(
  cd "${tmpdir}/proj" || exit 1
  HOME="${tmpdir}/home" USERPROFILE="${tmpdir}/home" SUBSTRATE_WORLD=disabled SUBSTRATE_WORLD_ENABLED=0 \
    "${substrate_bin}" -c 'true'
)
status=$?
set -e

if [[ ${status} -eq 0 ]]; then
  echo "[verify_profile_fail_fast][FAIL] Expected non-zero exit with invalid profile, got exit=0"
  exit 1
fi
echo "[verify_profile_fail_fast][OK] Invalid profile failed as expected (exit=${status})"

echo "[verify_profile_fail_fast] Writing valid minimal profile..."
cat >"${tmpdir}/proj/.substrate-profile" <<'YAML'
id: ok
name: ok
world_fs_mode: writable
net_allowed: []
cmd_allowed: []
cmd_denied: []
cmd_isolated: []
require_approval: false
allow_shell_operators: true
YAML

echo "[verify_profile_fail_fast] Running with valid profile (expect success)..."
(
  cd "${tmpdir}/proj" || exit 1
  HOME="${tmpdir}/home" USERPROFILE="${tmpdir}/home" SUBSTRATE_WORLD=disabled SUBSTRATE_WORLD_ENABLED=0 \
    "${substrate_bin}" -c 'true'
)

echo "[verify_profile_fail_fast][OK] Valid profile succeeded"
