#!/usr/bin/env bash
set -euo pipefail

# Test harness for how `world.anchor_mode` / `world.anchor_path` affect world root selection.
#
# This script creates a temporary workspace with a restrictive write allowlist and runs a few
# commands under different anchor modes to demonstrate behavioral differences.
#
# Usage:
#   scripts/dev/test_anchor_modes.sh
#
# Optional:
#   SUBSTRATE_BIN=~/.substrate/bin/substrate scripts/dev/test_anchor_modes.sh
#   ISOLATION=full scripts/dev/test_anchor_modes.sh   # (default: full; also try: workspace)

SUBSTRATE_BIN="${SUBSTRATE_BIN:-substrate}"
ISOLATION="${ISOLATION:-full}"

if [[ ! -x "$SUBSTRATE_BIN" ]]; then
  if ! command -v "$SUBSTRATE_BIN" >/dev/null 2>&1; then
    echo "error: SUBSTRATE_BIN not found/executable: $SUBSTRATE_BIN" >&2
    exit 1
  fi
fi

tmp_root_parent="${SUBSTRATE_TEST_ROOT_PARENT:-${HOME:-}}"
if [[ -z "$tmp_root_parent" ]]; then
  echo "error: HOME is empty; set SUBSTRATE_TEST_ROOT_PARENT to a non-/tmp directory" >&2
  exit 1
fi

# NOTE: Do not place the workspace under /tmp for this test. In full isolation, the Landlock
# baseline write allowlist includes /tmp, so a /tmp-rooted project can make "denied" writes appear
# to succeed when operating via the host-absolute project path.
tmp_root="$(mktemp -d -p "$tmp_root_parent" substrate-anchor-modes.XXXXXX)"
tmp_home="$(mktemp -d -t substrate-home.XXXXXX)"
cleanup() {
  rm -rf "$tmp_root" "$tmp_home"
}
trap cleanup EXIT

export SUBSTRATE_HOME="$tmp_home"

mkdir -p "$tmp_root/.substrate" "$tmp_root/testdir"

cat >"$tmp_root/.substrate/workspace.yaml" <<'YAML'
# Substrate config patch (sparse overrides; scope=workspace).
world:
  caged: false
YAML

cat >"$tmp_root/.substrate/policy.yaml" <<YAML
# Substrate policy patch (sparse overrides; scope=workspace).
world_fs:
  mode: writable
  isolation: ${ISOLATION}
  require_world: true
  read:
    allow_list:
    - "."
  write:
    allow_list:
    - "testdir"
YAML

echo "Workspace root: $tmp_root"
echo "SUBSTRATE_HOME: $SUBSTRATE_HOME"
echo
echo "== Effective policy/config at workspace root =="
(cd "$tmp_root" && "$SUBSTRATE_BIN" config show)
(cd "$tmp_root" && "$SUBSTRATE_BIN" policy show)
echo

run_case() {
  local title="$1"
  shift
  echo "== $title =="
  echo "+ (cd $PWD) $SUBSTRATE_BIN $*"
  "$SUBSTRATE_BIN" "$@"
  echo
}

echo "== Expected outcomes (ISOLATION=$ISOLATION) =="
echo "- With --anchor-mode=workspace: project root stays at workspace root."
echo "- With --anchor-mode=follow-cwd: project root becomes the current directory (per invocation cwd)."
echo "- With --anchor-mode=custom --anchor-path=<path>: project root is pinned to <path>."
echo
echo "- When ISOLATION=full: write.allow_list=[testdir] allows writing under testdir only."
echo "  - From workspace root: touching ./testdir/* should be allowed; touching ./root_file should be denied."
echo "  - From ./testdir with anchor-mode=workspace: touching ./file should be allowed."
echo "  - From ./testdir with anchor-mode=follow-cwd: touching ./file should be denied (allowlist becomes ./testdir/*)."
echo

cd "$tmp_root"

# shellcheck disable=SC2016
run_case "AnchorMode=workspace from workspace root" \
  --anchor-mode workspace \
  -c '
    echo "PWD=$(pwd)"
    echo "SUBSTRATE_WORLD_FS_ISOLATION=${SUBSTRATE_WORLD_FS_ISOLATION:-}"
    echo "SUBSTRATE_WORLD_FS_WRITE_ALLOWLIST=${SUBSTRATE_WORLD_FS_WRITE_ALLOWLIST:-}"
    echo "SUBSTRATE_WORLD_FS_LANDLOCK_WRITE_ALLOWLIST=${SUBSTRATE_WORLD_FS_LANDLOCK_WRITE_ALLOWLIST:-}"
    echo "SUBSTRATE_LANDLOCK_HELPER_PATH=${SUBSTRATE_LANDLOCK_HELPER_PATH:-}"
    echo "-- touch testdir/ok_from_root.md (expected: allowed)"
    if touch testdir/ok_from_root.md; then echo "allowed"; else echo "denied"; fi
    echo "-- touch root_denied.md (expected: denied when ISOLATION=full)"
    if touch root_denied.md; then echo "UNEXPECTED: allowed"; else echo "expected: denied"; fi
    echo "-- ls (world view)"
    ls -la
    echo "-- ls testdir (world view)"
    ls -la testdir
    true
  '

cd "$tmp_root/testdir"

# shellcheck disable=SC2016
run_case "AnchorMode=workspace from subdir testdir" \
  --anchor-mode workspace \
  -c '
    echo "PWD=$(pwd)"
    echo "SUBSTRATE_WORLD_FS_ISOLATION=${SUBSTRATE_WORLD_FS_ISOLATION:-}"
    echo "SUBSTRATE_WORLD_FS_WRITE_ALLOWLIST=${SUBSTRATE_WORLD_FS_WRITE_ALLOWLIST:-}"
    echo "SUBSTRATE_WORLD_FS_LANDLOCK_WRITE_ALLOWLIST=${SUBSTRATE_WORLD_FS_LANDLOCK_WRITE_ALLOWLIST:-}"
    echo "SUBSTRATE_LANDLOCK_HELPER_PATH=${SUBSTRATE_LANDLOCK_HELPER_PATH:-}"
    echo "-- touch ok_from_testdir_workspace.md (expected: allowed)"
    if touch ok_from_testdir_workspace.md; then echo "allowed"; else echo "denied"; fi
    echo "-- ls (world view)"
    ls -la
    true
  '

# shellcheck disable=SC2016
run_case "AnchorMode=follow-cwd from subdir testdir" \
  --anchor-mode follow-cwd \
  -c '
    echo "PWD=$(pwd)"
    echo "SUBSTRATE_WORLD_FS_ISOLATION=${SUBSTRATE_WORLD_FS_ISOLATION:-}"
    echo "SUBSTRATE_WORLD_FS_WRITE_ALLOWLIST=${SUBSTRATE_WORLD_FS_WRITE_ALLOWLIST:-}"
    echo "SUBSTRATE_WORLD_FS_LANDLOCK_WRITE_ALLOWLIST=${SUBSTRATE_WORLD_FS_LANDLOCK_WRITE_ALLOWLIST:-}"
    echo "SUBSTRATE_LANDLOCK_HELPER_PATH=${SUBSTRATE_LANDLOCK_HELPER_PATH:-}"
    echo "-- touch denied_from_testdir_follow.md (expected: denied when ISOLATION=full)"
    if touch denied_from_testdir_follow.md; then echo "UNEXPECTED: allowed"; else echo "expected: denied"; fi
    echo "-- mkdir -p testdir && touch testdir/ok_follow_nested.md (expected: allowed)"
    if mkdir -p testdir && touch testdir/ok_follow_nested.md; then echo "allowed"; else echo "denied"; fi
    echo "-- ls (world view)"
    ls -la
    echo "-- ls testdir (world view)"
    ls -la testdir
    true
  '

# shellcheck disable=SC2016
run_case "AnchorMode=custom (pinned to workspace root) from subdir testdir" \
  --anchor-mode custom \
  --anchor-path "$tmp_root" \
  -c '
    echo "PWD=$(pwd)"
    echo "SUBSTRATE_WORLD_FS_ISOLATION=${SUBSTRATE_WORLD_FS_ISOLATION:-}"
    echo "SUBSTRATE_WORLD_FS_WRITE_ALLOWLIST=${SUBSTRATE_WORLD_FS_WRITE_ALLOWLIST:-}"
    echo "SUBSTRATE_WORLD_FS_LANDLOCK_WRITE_ALLOWLIST=${SUBSTRATE_WORLD_FS_LANDLOCK_WRITE_ALLOWLIST:-}"
    echo "SUBSTRATE_LANDLOCK_HELPER_PATH=${SUBSTRATE_LANDLOCK_HELPER_PATH:-}"
    echo "-- touch ok_from_testdir_custom.md (expected: allowed)"
    if touch ok_from_testdir_custom.md; then echo "allowed"; else echo "denied"; fi
    echo "-- ls (world view)"
    ls -la
    true
  '

echo "Done."
