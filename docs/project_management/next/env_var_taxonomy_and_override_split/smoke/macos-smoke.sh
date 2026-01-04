#!/usr/bin/env bash
set -euo pipefail

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "SKIP: env var override split macOS smoke (not macOS)"
  exit 0
fi

fail() {
  echo "FAIL: $*" >&2
  exit 1
}

need_cmd() {
  local name="$1"
  if ! command -v "$name" >/dev/null 2>&1; then
    echo "MISSING: $name not found on PATH" >&2
    exit 3
  fi
}

need_cmd substrate
need_cmd mktemp

TMP_HOME="$(mktemp -d)"
TMP_WS="$(mktemp -d)"
cleanup() { rm -rf "$TMP_HOME" "$TMP_WS"; }
trap cleanup EXIT

export HOME="$TMP_HOME"
export SUBSTRATE_HOME="$TMP_HOME"

substrate config global init --force >/dev/null
substrate config global set policy.mode=observe >/dev/null

out="$(substrate --no-world --shell /bin/bash -c 'printf "%s" "${SUBSTRATE_POLICY_MODE:-}"')"
[[ "$out" == "observe" ]] || fail "expected SUBSTRATE_POLICY_MODE=observe from config; got '$out'"

out="$(SUBSTRATE_POLICY_MODE=disabled substrate --no-world --shell /bin/bash -c 'printf "%s" "${SUBSTRATE_POLICY_MODE:-}"')"
[[ "$out" == "observe" ]] || fail "expected legacy SUBSTRATE_POLICY_MODE to not override; got '$out'"

out="$(SUBSTRATE_OVERRIDE_POLICY_MODE=enforce substrate --no-world --shell /bin/bash -c 'printf "%s" "${SUBSTRATE_POLICY_MODE:-}"')"
[[ "$out" == "enforce" ]] || fail "expected override SUBSTRATE_OVERRIDE_POLICY_MODE=enforce; got '$out'"

substrate workspace init "$TMP_WS" >/dev/null
cd "$TMP_WS"
substrate config set policy.mode=observe >/dev/null

out="$(SUBSTRATE_OVERRIDE_POLICY_MODE=enforce substrate --no-world --shell /bin/bash -c 'printf "%s" "${SUBSTRATE_POLICY_MODE:-}"')"
[[ "$out" == "observe" ]] || fail "expected workspace policy.mode=observe to win over overrides; got '$out'"

set +e
SUBSTRATE_OVERRIDE_POLICY_MODE=bogus substrate config show --json >/dev/null 2>&1
code=$?
set -e
[[ "$code" -eq 2 ]] || fail "expected exit code 2 for invalid override value; got $code"

echo "OK: env var override split macOS smoke"

