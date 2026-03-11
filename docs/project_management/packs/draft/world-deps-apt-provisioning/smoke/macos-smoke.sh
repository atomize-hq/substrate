#!/usr/bin/env bash
set -euo pipefail

# WDAP smoke (macOS).
#
# Exit codes (aligned to `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`):
# - 0: smoke passed / skip (wrong OS)
# - 1: assertion failed / unexpected error
# - 2: invalid inputs
# - 3: required dependency unavailable
# - 4: not supported / missing prerequisites

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "SKIP: WDAP macos smoke (uname=$(uname -s))"
  exit 0
fi

SUBSTRATE_BIN="${SUBSTRATE_BIN:-substrate}"
HOST_HOME="${HOME:-}"
HOST_LIMA_HOME="${LIMA_HOME:-}"

if [[ -z "${HOST_LIMA_HOME}" && -n "${HOST_HOME}" ]]; then
  HOST_LIMA_HOME="${HOST_HOME}/.lima"
fi

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

require_not_contains() {
  local haystack="$1"
  local needle="$2"
  if printf '%s\n' "$haystack" | grep -Fq "$needle"; then
    echo "FAIL: found forbidden substring: $needle" >&2
    return 1
  fi
}

require_exact_stdout() {
  local got_file="$1"
  local expected="$2"
  local expected_file="$tmp_root/expected.stdout.txt"
  printf '%s' "$expected" >"$expected_file"
  if ! cmp -s "$got_file" "$expected_file"; then
    echo "FAIL: unexpected stdout content" >&2
    printf 'EXPECTED:\n' >&2
    cat "$expected_file" >&2 || true
    printf '\nGOT:\n' >&2
    cat "$got_file" >&2 || true
    printf '\n' >&2
    return 1
  fi
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
  RUN_STDOUT_FILE="$stdout_file"
  RUN_STDERR_FILE="$stderr_file"
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
ws="$tmp_root/ws"

mkdir -p "$home_dir" "$substrate_home/deps/packages" "$ws"

export HOME="$home_dir"
export USERPROFILE="$home_dir"
export SUBSTRATE_HOME="$substrate_home"
if [[ -n "${HOST_LIMA_HOME}" ]]; then
  export LIMA_HOME="$HOST_LIMA_HOME"
fi

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
    mkdir -p /var/lib/substrate/world-deps/bin
    cat > /var/lib/substrate/world-deps/bin/smoke-hello <<'EOF'
    #!/bin/sh
    echo smoke-hello
    EOF
    chmod +x /var/lib/substrate/world-deps/bin/smoke-hello
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

cat >"$SUBSTRATE_HOME/deps/packages/smoke-apt-conflict-1.yaml" <<'YAML'
version: 1
name: smoke-apt-conflict-1
description: WDAP smoke fixture (APT conflict 1).
runnable: false
install:
  method: apt
  apt:
    - name: smoke-apt-conflict
      version: "1"
probe:
  command: "sh -c 'exit 1'"
YAML

cat >"$SUBSTRATE_HOME/deps/packages/smoke-apt-conflict-2.yaml" <<'YAML'
version: 1
name: smoke-apt-conflict-2
description: WDAP smoke fixture (APT conflict 2).
runnable: false
install:
  method: apt
  apt:
    - name: smoke-apt-conflict
      version: "2"
probe:
  command: "sh -c 'exit 1'"
YAML

pushd "$ws" >/dev/null

"$SUBSTRATE_BIN" world deps global reset >/dev/null 2>&1 || true
"$SUBSTRATE_BIN" world deps workspace reset >/dev/null 2>&1 || true
"$SUBSTRATE_BIN" world deps workspace add smoke-hello smoke-apt-a smoke-apt-b >/dev/null

echo "== Case A: provisioning dry-run prints normalized APT requirement set =="
run_expect "world-enable-provision-dry-run" 0 "$SUBSTRATE_BIN" world enable --provision-deps --dry-run
expected_stdout=$'smoke-apt-a\nsmoke-apt-b=1\n'
require_exact_stdout "$RUN_STDOUT_FILE" "$expected_stdout"

echo "== Case B: provisioning ignores SUBSTRATE_WORLD_REQUEST_PROFILE =="
run_expect "world-enable-provision-dry-run-verbose" 0 env SUBSTRATE_WORLD_REQUEST_PROFILE="wdap-smoke-profile" "$SUBSTRATE_BIN" world enable --provision-deps --dry-run --verbose
require_contains "$RUN_STDOUT" "world-deps-provision"
require_not_contains "$RUN_STDOUT" "wdap-smoke-profile"

echo "== Case C: provisioning version-pin conflicts exit 2 =="
"$SUBSTRATE_BIN" world deps workspace reset >/dev/null 2>&1 || true
"$SUBSTRATE_BIN" world deps workspace add smoke-apt-conflict-1 smoke-apt-conflict-2 >/dev/null
run_expect "world-enable-provision-conflict" 2 "$SUBSTRATE_BIN" world enable --provision-deps --dry-run
require_contains "$RUN_STDERR" "smoke-apt-conflict"
require_contains "$RUN_STDERR" "1"
require_contains "$RUN_STDERR" "2"

"$SUBSTRATE_BIN" world deps workspace reset >/dev/null 2>&1 || true
"$SUBSTRATE_BIN" world deps workspace add smoke-hello smoke-apt-a smoke-apt-b >/dev/null

echo "== Preflight: world doctor =="
if ! HOME="${HOST_HOME}" USERPROFILE="${HOST_HOME}" "$SUBSTRATE_BIN" world doctor >/dev/null 2>&1; then
  echo "WDAP macos smoke: world backend not healthy; run 'substrate world doctor' remediation and retry" >&2
  exit 4
fi

if [[ "${SUBSTRATE_SMOKE_SLICE_ID:-}" == "WDAP0" ]]; then
  echo "== Runtime cases are skipped for WDAP0 (owned by WDAP1) =="
  popd >/dev/null
  echo "OK: WDAP macos smoke"
  exit 0
fi

echo "== Case D: runtime current sync fails early for APT requirements =="
run_expect "deps-sync-dry-run" 4 "$SUBSTRATE_BIN" world deps current sync --dry-run
require_line_order "$RUN_STDOUT" "smoke-apt-a" "smoke-apt-b=1"
require_contains "$RUN_STDERR" "substrate world enable --provision-deps"

echo "== Case E: current install explicit args do not add enabled items implicitly =="
run_expect "deps-install-smoke-hello" 0 "$SUBSTRATE_BIN" world deps current install smoke-hello

echo "== Case F: current install fails early for explicit APT-backed items =="
run_expect "deps-install-smoke-apt-a-dry-run" 4 "$SUBSTRATE_BIN" world deps current install smoke-apt-a --dry-run
require_contains "$RUN_STDOUT" "smoke-apt-a"
require_contains "$RUN_STDERR" "substrate world enable --provision-deps"

popd >/dev/null

echo "OK: WDAP macos smoke"
