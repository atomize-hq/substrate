#!/usr/bin/env bash
set -euo pipefail

if [[ "$(uname -s)" != "Linux" ]]; then
  echo "SKIP: env var override split linux smoke (not Linux)"
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
substrate config global set world.caged=true >/dev/null
substrate config global set world.anchor_mode=follow-cwd >/dev/null

out="$(substrate --no-world --shell /bin/bash -c 'printf "%s|%s|%s" "${SUBSTRATE_POLICY_MODE:-}" "${SUBSTRATE_CAGED:-}" "${SUBSTRATE_ANCHOR_MODE:-}"')"
[[ "$out" == "observe|1|follow-cwd" ]] || fail "expected observe|1|follow-cwd from config; got '$out'"

out="$(SUBSTRATE_POLICY_MODE=disabled SUBSTRATE_CAGED=0 SUBSTRATE_ANCHOR_MODE=workspace substrate --no-world --shell /bin/bash -c 'printf "%s|%s|%s" "${SUBSTRATE_POLICY_MODE:-}" "${SUBSTRATE_CAGED:-}" "${SUBSTRATE_ANCHOR_MODE:-}"')"
[[ "$out" == "observe|1|follow-cwd" ]] || fail "expected legacy SUBSTRATE_* to not override; got '$out'"

out="$(SUBSTRATE_OVERRIDE_POLICY_MODE=enforce SUBSTRATE_OVERRIDE_CAGED=0 SUBSTRATE_OVERRIDE_ANCHOR_MODE=workspace substrate --no-world --shell /bin/bash -c 'printf "%s|%s|%s" "${SUBSTRATE_POLICY_MODE:-}" "${SUBSTRATE_CAGED:-}" "${SUBSTRATE_ANCHOR_MODE:-}"')"
[[ "$out" == "enforce|0|workspace" ]] || fail "expected override enforce|0|workspace; got '$out'"

substrate workspace init "$TMP_WS" >/dev/null
cd "$TMP_WS"
substrate config set policy.mode=observe >/dev/null
substrate config set world.caged=true >/dev/null
substrate config set world.anchor_mode=follow-cwd >/dev/null

out="$(SUBSTRATE_OVERRIDE_POLICY_MODE=enforce SUBSTRATE_OVERRIDE_CAGED=0 SUBSTRATE_OVERRIDE_ANCHOR_MODE=workspace substrate --no-world --shell /bin/bash -c 'printf "%s|%s|%s" "${SUBSTRATE_POLICY_MODE:-}" "${SUBSTRATE_CAGED:-}" "${SUBSTRATE_ANCHOR_MODE:-}"')"
[[ "$out" == "observe|1|follow-cwd" ]] || fail "expected workspace to win over overrides; got '$out'"

set +e
SUBSTRATE_OVERRIDE_POLICY_MODE=bogus substrate config show --json >/dev/null 2>&1
code=$?
set -e
[[ "$code" -eq 2 ]] || fail "expected exit code 2 for invalid override value; got $code"

set +e
SUBSTRATE_OVERRIDE_CAGED=bogus substrate config show --json >/dev/null 2>&1
code=$?
set -e
[[ "$code" -eq 2 ]] || fail "expected exit code 2 for invalid override boolean; got $code"

echo "OK: env var override split linux smoke"
