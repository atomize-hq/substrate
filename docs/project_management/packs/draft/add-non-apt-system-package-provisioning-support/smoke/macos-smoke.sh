#!/usr/bin/env bash
set -euo pipefail

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "SKIP: NASP macos smoke (uname=$(uname -s))"
  exit 0
fi

SUBSTRATE_BIN="${SUBSTRATE_BIN:-substrate}"

if [[ "$SUBSTRATE_BIN" == "substrate" ]]; then
  command -v substrate >/dev/null 2>&1 || { echo "FAIL: substrate not found on PATH" >&2; exit 3; }
else
  [[ -x "$SUBSTRATE_BIN" ]] || { echo "FAIL: SUBSTRATE_BIN is not executable: $SUBSTRATE_BIN" >&2; exit 3; }
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
ws="$tmp_root/ws"

mkdir -p "$home_dir" "$substrate_home/deps/packages" "$ws"

export HOME="$home_dir"
export USERPROFILE="$home_dir"
export SUBSTRATE_HOME="$substrate_home"

"$SUBSTRATE_BIN" workspace init "$ws" >/dev/null

cat >"$SUBSTRATE_HOME/deps/packages/smoke-hello.yaml" <<'YAML'
version: 1
name: smoke-hello
description: NASP smoke fixture (script install).
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

cat >"$SUBSTRATE_HOME/deps/packages/smoke-pacman-a.yaml" <<'YAML'
version: 1
name: smoke-pacman-a
description: NASP smoke fixture (pacman list in authored reverse order).
runnable: false
install:
  method: pacman
  pacman:
    - nasp-pacman-b
    - nasp-pacman-a
probe:
  command: "sh -c 'exit 1'"
YAML

cat >"$SUBSTRATE_HOME/deps/packages/smoke-pacman-b.yaml" <<'YAML'
version: 1
name: smoke-pacman-b
description: NASP smoke fixture (pacman duplicate for normalization).
runnable: false
install:
  method: pacman
  pacman:
    - nasp-pacman-a
probe:
  command: "sh -c 'exit 1'"
YAML

cat >"$SUBSTRATE_HOME/deps/packages/smoke-apt-a.yaml" <<'YAML'
version: 1
name: smoke-apt-a
description: NASP smoke fixture (APT item for mixed-manager validation).
runnable: false
install:
  method: apt
  apt:
    - name: nasp-apt-a
probe:
  command: "sh -c 'exit 1'"
YAML

pushd "$ws" >/dev/null

"$SUBSTRATE_BIN" world deps global reset >/dev/null 2>&1 || true
"$SUBSTRATE_BIN" world deps workspace reset >/dev/null 2>&1 || true
"$SUBSTRATE_BIN" world deps workspace add smoke-hello smoke-pacman-a smoke-pacman-b smoke-apt-a >/dev/null

echo "== Preflight: world doctor =="
if ! "$SUBSTRATE_BIN" world doctor >/dev/null 2>&1; then
  echo "NASP macos smoke: world backend not healthy; run 'substrate world doctor' and retry" >&2
  exit 4
fi

echo "== Case A: mixed-manager provisioning rejects on the default macOS guest =="
run_expect "world-enable-provision-dry-run-verbose" 4 "$SUBSTRATE_BIN" world enable --provision-deps --dry-run --verbose
require_contains "$RUN_STDOUT" "world-deps-provision"
require_contains "$RUN_STDERR" "incompatible system-package methods"

echo "== Case B: runtime current sync reports APT then pacman requirements =="
run_expect "deps-sync-dry-run" 4 "$SUBSTRATE_BIN" world deps current sync --dry-run
require_line_order "$RUN_STDOUT" "nasp-apt-a" "nasp-pacman-a"
require_line_order "$RUN_STDOUT" "nasp-pacman-a" "nasp-pacman-b"
require_contains "$RUN_STDERR" "substrate world enable --provision-deps"

echo "== Case C: current install explicit args do not add enabled items implicitly =="
run_expect "deps-install-smoke-hello" 0 "$SUBSTRATE_BIN" world deps current install smoke-hello

echo "== Case D: current install fails early for explicit pacman-backed items =="
run_expect "deps-install-smoke-pacman-a-dry-run" 4 "$SUBSTRATE_BIN" world deps current install smoke-pacman-a --dry-run
require_line_order "$RUN_STDOUT" "nasp-pacman-a" "nasp-pacman-b"
require_contains "$RUN_STDERR" "substrate world enable --provision-deps"

popd >/dev/null

echo "OK: NASP macos smoke"
