#!/usr/bin/env bash
set -euo pipefail

# WDAP smoke (Linux).
#
# Exit codes (aligned to `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`):
# - 0: smoke passed / skip (wrong OS)
# - 1: assertion failed / unexpected error
# - 2: invalid inputs
# - 3: required dependency unavailable
# - 4: not supported / missing prerequisites

if [[ "$(uname -s)" != "Linux" ]]; then
  echo "SKIP: WDAP linux smoke (uname=$(uname -s))"
  exit 0
fi

SUBSTRATE_BIN="${SUBSTRATE_BIN:-substrate}"

if [[ "$SUBSTRATE_BIN" == "substrate" ]]; then
  command -v substrate >/dev/null 2>&1 || { echo "FAIL: substrate not found on PATH"; exit 3; }
else
  [[ -x "$SUBSTRATE_BIN" ]] || { echo "FAIL: SUBSTRATE_BIN is not executable: $SUBSTRATE_BIN"; exit 3; }
fi

tmp_root="${SUBSTRATE_SMOKE_ROOT:-}"
if [[ -z "${tmp_root}" ]]; then
  tmp_root="$(mktemp -d)"
fi

cleanup() {
  if [[ "${SUBSTRATE_SMOKE_KEEP:-0}" == "1" ]]; then
    return 0
  fi
  rm -rf "$tmp_root"
}
trap cleanup EXIT

require_contains() {
  local haystack="$1"
  local needle="$2"
  printf '%s\n' "$haystack" | grep -Fq "$needle" || {
    echo "FAIL: missing expected substring: $needle" >&2
    return 1
  }
}

require_line_order() {
  local text="$1"
  local first="$2"
  local second="$3"

  local first_line second_line
  first_line="$(printf '%s\n' "$text" | grep -n -F -x "$first" | head -n 1 | cut -d: -f1 || true)"
  second_line="$(printf '%s\n' "$text" | grep -n -F -x "$second" | head -n 1 | cut -d: -f1 || true)"

  [[ -n "$first_line" ]] || { echo "FAIL: missing required line in stdout: $first" >&2; return 1; }
  [[ -n "$second_line" ]] || { echo "FAIL: missing required line in stdout: $second" >&2; return 1; }
  [[ "$first_line" -lt "$second_line" ]] || {
    echo "FAIL: expected line order '$first' then '$second' (got $first_line then $second_line)" >&2
    return 1
  }
}

run_expect() {
  local label="$1"
  local expected_rc="$2"
  shift 2

  local stdout_file="$tmp_root/${label}.stdout.txt"
  local stderr_file="$tmp_root/${label}.stderr.txt"

  set +e
  "$@" 1>"$stdout_file" 2>"$stderr_file"
  local rc=$?
  set -e

  local out err
  out="$(cat "$stdout_file" 2>/dev/null || true)"
  err="$(cat "$stderr_file" 2>/dev/null || true)"

  if [[ "$rc" -ne "$expected_rc" ]]; then
    echo "FAIL: $label expected exit=$expected_rc, got=$rc" >&2
    printf 'STDOUT:\n%s\nSTDERR:\n%s\n' "$out" "$err" >&2
    exit 1
  fi

  RUN_STDOUT="$out"
  RUN_STDERR="$err"
}

home_dir="$tmp_root/home"
substrate_home="$tmp_root/substrate-home"
world_deps_bin="$tmp_root/world-deps-bin"
ws="$tmp_root/ws"

mkdir -p "$home_dir" "$substrate_home/deps/packages" "$world_deps_bin" "$ws"

export HOME="$home_dir"
export USERPROFILE="$home_dir"
export SUBSTRATE_HOME="$substrate_home"
export SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR="$world_deps_bin"

"$SUBSTRATE_BIN" workspace init "$ws" >/dev/null

cat >"$SUBSTRATE_HOME/deps/packages/smoke-hello.yaml" <<'YAML'
version: 1
name: smoke-hello
description: WDAP smoke fixture (script install).
runnable: true
entrypoints: ["smoke-hello"]
install:
  method: script
  script: |
    set -euo pipefail
    mkdir -p "${SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR:?missing world deps bin}"
    cat > "${SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR:?missing world deps bin}/smoke-hello" <<'EOF'
    #!/bin/sh
    echo smoke-hello
    EOF
    chmod +x "${SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR:?missing world deps bin}/smoke-hello"
probe:
  command: "smoke-hello"
YAML

cat >"$SUBSTRATE_HOME/deps/packages/smoke-apt-a.yaml" <<'YAML'
version: 1
name: smoke-apt-a
description: WDAP smoke fixture (APT; missing by design).
runnable: false
install:
  method: apt
  apt:
    - name: smoke-apt-a
probe:
  command: "sh -c 'exit 1'"
YAML

cat >"$SUBSTRATE_HOME/deps/packages/smoke-apt-b.yaml" <<'YAML'
version: 1
name: smoke-apt-b
description: WDAP smoke fixture (APT; pinned; missing by design).
runnable: false
install:
  method: apt
  apt:
    - name: smoke-apt-b
      version: "1"
probe:
  command: "sh -c 'exit 1'"
YAML

pushd "$ws" >/dev/null

"$SUBSTRATE_BIN" world deps global reset >/dev/null 2>&1 || true
"$SUBSTRATE_BIN" world deps workspace reset >/dev/null 2>&1 || true
"$SUBSTRATE_BIN" world deps workspace add smoke-hello smoke-apt-a smoke-apt-b >/dev/null

echo "== Case A: provisioning fails closed on Linux host-native =="
run_expect "world-enable-provision-dry-run" 4 "$SUBSTRATE_BIN" world enable --provision-deps --dry-run
require_contains "$RUN_STDERR" "Substrate will not mutate the host OS"
require_contains "$RUN_STDERR" "substrate world enable --provision-deps"

if [[ "${SUBSTRATE_SMOKE_SLICE_ID:-}" == "WDAP0" ]]; then
  echo "== Runtime cases are skipped for WDAP0 (owned by WDAP1) =="
  popd >/dev/null
  echo "OK: WDAP linux smoke"
  exit 0
fi

echo "== Preflight: world doctor =="
if ! "$SUBSTRATE_BIN" world doctor --world >/dev/null 2>&1; then
  echo "WDAP linux smoke: world backend not healthy; run 'substrate world doctor' remediation and retry" >&2
  exit 4
fi

echo "== Case B: runtime current sync fails early for APT requirements =="
run_expect "deps-sync-dry-run" 4 "$SUBSTRATE_BIN" world deps current sync --dry-run
require_line_order "$RUN_STDOUT" "smoke-apt-a" "smoke-apt-b=1"
require_contains "$RUN_STDERR" "substrate world enable --provision-deps"
require_contains "$RUN_STDERR" "Substrate will not mutate the host OS"

echo "== Case C: current install explicit args do not add enabled items implicitly =="
run_expect "deps-install-smoke-hello" 0 "$SUBSTRATE_BIN" world deps current install smoke-hello

echo "== Case D: current install fails early for explicit APT-backed items =="
run_expect "deps-install-smoke-apt-a-dry-run" 4 "$SUBSTRATE_BIN" world deps current install smoke-apt-a --dry-run
require_contains "$RUN_STDOUT" "smoke-apt-a"
require_contains "$RUN_STDERR" "substrate world enable --provision-deps"

popd >/dev/null

echo "OK: WDAP linux smoke"
