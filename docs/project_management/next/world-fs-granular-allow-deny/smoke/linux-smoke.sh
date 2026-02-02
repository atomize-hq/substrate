#!/usr/bin/env bash
set -euo pipefail

SUBSTRATE_BIN="${SUBSTRATE_BIN:-substrate}"

if [[ "${OSTYPE:-}" != linux* ]]; then
  echo "world-fs-granular-allow-deny: linux smoke is supported only on Linux (OSTYPE=${OSTYPE:-unknown})" >&2
  exit 4
fi

if ! command -v "$SUBSTRATE_BIN" >/dev/null 2>&1; then
  echo "world-fs-granular-allow-deny: substrate binary not found (SUBSTRATE_BIN=$SUBSTRATE_BIN)" >&2
  exit 3
fi

tmp_root="$(mktemp -d)"
cleanup() { rm -rf "$tmp_root"; }
trap cleanup EXIT

export SUBSTRATE_HOME="${SUBSTRATE_HOME:-$tmp_root/substrate-home}"
workspace="$tmp_root/workspace"
mkdir -p "$workspace"

cd "$workspace"

mkdir -p secrets docs certs outputs/private
printf '%s\n' 'secret' > secrets/secret.txt
printf '%s\n' 'public' > docs/public.txt
printf '%s\n' 'pem' > certs/a.pem

echo "== Setup: workspace + policy patch =="
"$SUBSTRATE_BIN" workspace init --force >/dev/null
"$SUBSTRATE_BIN" policy init --force >/dev/null

policy_reset() {
  "$SUBSTRATE_BIN" policy init --force >/dev/null
}

policy_set() {
  "$SUBSTRATE_BIN" policy set "$@" >/dev/null
}

slice_id="${SUBSTRATE_SMOKE_SLICE_ID:-}"
mode=""
enforcement=""
case "$slice_id" in
  "")
    mode="full"
    enforcement="strict"
    ;;
  WFGAD0|WFGAD1|WFGAD2)
    mode="schema"
    ;;
  WFGAD3)
    mode="wfgad3"
    enforcement="best_effort"
    ;;
  WFGAD4)
    mode="wfgad4"
    enforcement="best_effort"
    ;;
  WFGAD5)
    mode="full"
    enforcement="strict"
    ;;
  *)
    echo "world-fs-granular-allow-deny: unknown SUBSTRATE_SMOKE_SLICE_ID=$slice_id" >&2
    exit 2
    ;;
esac

expect_exit() {
  local want="$1"
  shift
  local out
  set +e
  out="$("$@" 2>&1)"
  got="$?"
  set -e
  if [[ "$got" -ne "$want" ]]; then
    echo "FAIL: expected exit $want, got $got: $*" >&2
    echo "$out" >&2
    exit 1
  fi
}

schema_smoke() {
  echo "== Schema smoke (slice=$slice_id) =="

  # Legacy keys must be hard errors (exit 2).
  expect_exit 2 "$SUBSTRATE_BIN" policy set 'world_fs.read_allowlist+=.'
  expect_exit 2 "$SUBSTRATE_BIN" policy set 'world_fs.write_allowlist+=.'

  # Invalid patterns must be hard errors (exit 2).
  expect_exit 2 "$SUBSTRATE_BIN" policy set 'world_fs.read.allow_list+=../x'
  expect_exit 2 "$SUBSTRATE_BIN" policy set 'world_fs.read.allow_list+=/abs'
  expect_exit 2 "$SUBSTRATE_BIN" policy set 'world_fs.read.allow_list+=src/**'
  expect_exit 2 "$SUBSTRATE_BIN" policy set 'world_fs.read.deny_list+=file?.txt'

  # Minimal valid V2 configuration must be accepted (exit 0).
  policy_reset
  policy_set \
    'world_fs.mode=read_only' \
    'world_fs.isolation=full' \
    'world_fs.require_world=true' \
    'world_fs.read.allow_list+=.'
}

world_preflight() {
  echo "== Preflight: world doctor =="
  if ! "$SUBSTRATE_BIN" world doctor >/dev/null 2>&1; then
    echo "world-fs-granular-allow-deny: world backend not healthy; run 'substrate world enable' and retry" >&2
    exit 4
  fi
}

run_world() {
  local cmd="$1"
  "$SUBSTRATE_BIN" --world --command "$cmd" 2>&1
}

expect_ok() {
  local cmd="$1"
  local out
  if ! out="$(run_world "$cmd")"; then
    echo "FAIL: expected success: $cmd" >&2
    echo "$out" >&2
    exit 1
  fi
}

expect_fail_contains() {
  local cmd="$1"
  local needle="$2"
  local out
  if out="$(run_world "$cmd")"; then
    echo "FAIL: expected failure: $cmd" >&2
    echo "$out" >&2
    exit 1
  fi
  if ! grep -Fq "$needle" <<<"$out"; then
    echo "FAIL: expected output to contain: $needle" >&2
    echo "CMD: $cmd" >&2
    echo "OUT:" >&2
    echo "$out" >&2
    exit 1
  fi
}

case "$mode" in
  schema)
    schema_smoke
    echo "OK: world-fs-granular-allow-deny linux smoke passed (schema-only; slice=$slice_id)"
    exit 0
    ;;
  wfgad3|wfgad4|full)
    world_preflight
    ;;
  *)
    echo "world-fs-granular-allow-deny: internal error: unknown mode=$mode" >&2
    exit 1
    ;;
esac

echo "== Case 1: deny overrides allow (directory deny) =="
policy_reset
policy_set \
  'world_fs.mode=read_only' \
  'world_fs.isolation=full' \
  'world_fs.require_world=true' \
  "world_fs.enforcement=$enforcement" \
  'world_fs.read.allow_list+=.' \
  'world_fs.read.deny_list+=./secrets/**'

expect_fail_contains 'ls ./secrets' 'Permission denied'
expect_fail_contains 'cat ./secrets/secret.txt' 'Permission denied'
expect_ok 'test -n "$SUBSTRATE_MOUNT_PROJECT_DIR"'
expect_fail_contains 'cat "$SUBSTRATE_MOUNT_PROJECT_DIR/secrets/secret.txt"' 'Permission denied'
expect_ok 'cat ./docs/public.txt >/dev/null'

if [[ "$mode" == "full" ]]; then
  echo "== Case 2: attempted bypass (strict) =="
  expect_fail_contains 'umount /project/secrets' 'Operation not permitted'
  expect_fail_contains 'cat ./secrets/secret.txt' 'Permission denied'
  expect_fail_contains 'cat "$SUBSTRATE_MOUNT_PROJECT_DIR/secrets/secret.txt"' 'Permission denied'
fi

if [[ "$mode" == "wfgad4" || "$mode" == "full" ]]; then
  echo "== Case 3: discover vs read (visible but not readable) =="
  policy_reset
  policy_set \
    'world_fs.mode=read_only' \
    'world_fs.isolation=full' \
    'world_fs.require_world=true' \
    "world_fs.enforcement=$enforcement" \
    'world_fs.discover.allow_list+=.' \
    'world_fs.read.allow_list+=.' \
    'world_fs.read.deny_list+=./secrets/secret.txt'

  expect_ok 'ls ./secrets | grep -qx secret.txt'
  expect_fail_contains 'cat ./secrets/secret.txt' 'Permission denied'
fi

echo "== Case 4: wildcard deny (snapshot at exec start) =="
policy_reset
policy_set \
  'world_fs.mode=read_only' \
  'world_fs.isolation=full' \
  'world_fs.require_world=true' \
  "world_fs.enforcement=$enforcement" \
  'world_fs.read.allow_list+=.' \
  'world_fs.read.deny_list+=**/*.pem'

expect_fail_contains 'cat ./certs/a.pem' 'Permission denied'

echo "== Case 5: write deny (EROFS) =="
policy_reset
policy_set \
  'world_fs.mode=writable' \
  'world_fs.isolation=full' \
  'world_fs.require_world=true' \
  "world_fs.enforcement=$enforcement" \
  'world_fs.write.allow_list+=.' \
  'world_fs.write.deny_list+=./outputs/private/**'

expect_fail_contains 'mkdir -p ./outputs/private/x' 'Read-only file system'

if [[ "$mode" == "wfgad4" || "$mode" == "full" ]]; then
  echo "== Case 6: discover deny (invisible subtree) =="
  policy_reset
  policy_set \
    'world_fs.mode=read_only' \
    'world_fs.isolation=full' \
    'world_fs.require_world=true' \
    "world_fs.enforcement=$enforcement" \
    'world_fs.discover.allow_list+=.' \
    'world_fs.discover.deny_list+=./secrets/**' \
    'world_fs.read.allow_list+=.'

  expect_fail_contains 'ls ./secrets' 'Permission denied'
  expect_fail_contains 'cat ./secrets/secret.txt' 'Permission denied'
fi

echo "OK: world-fs-granular-allow-deny linux smoke passed (slice=${slice_id:-full})"
