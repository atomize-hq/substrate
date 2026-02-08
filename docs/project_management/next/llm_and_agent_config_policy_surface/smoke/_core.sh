#!/usr/bin/env bash
set -euo pipefail

# Schema-only smoke for ADR-0027 config/policy keys.
#
# Caller responsibilities:
# - Ensure `substrate` is available via `$SUBSTRATE_BIN` (default: `substrate`).

SUBSTRATE_BIN="${SUBSTRATE_BIN:-substrate}"

if ! command -v "$SUBSTRATE_BIN" >/dev/null 2>&1; then
  echo "llm_and_agent_config_policy_surface: substrate binary not found (SUBSTRATE_BIN=$SUBSTRATE_BIN)" >&2
  exit 3
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

export SUBSTRATE_HOME="${SUBSTRATE_HOME:-$tmp_root/substrate-home}"
workspace="$tmp_root/workspace"
mkdir -p "$workspace"
cd "$workspace"

echo "== Setup: workspace + patches =="
"$SUBSTRATE_BIN" workspace init --force >/dev/null
"$SUBSTRATE_BIN" config global init --force >/dev/null
"$SUBSTRATE_BIN" policy global init --force >/dev/null

echo "== Setup: minimal agent inventory =="
mkdir -p "$SUBSTRATE_HOME/agents"
cat >"$SUBSTRATE_HOME/agents/codex.yaml" <<'YAML'
version: 1
id: codex
config:
  kind: cli
  enabled: true
  execution:
    scope: world
  cli:
    binary: codex
    mode: persistent
  capabilities:
    llm: true
policy_overlay:
  agents:
    fail_closed:
      routing: true
YAML

expect_exit() {
  local want="$1"
  shift
  local out got
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

echo "== Case 1: config keys accept =="
expect_exit 0 "$SUBSTRATE_BIN" config global set \
  'llm.enabled=true' \
  'llm.gateway.enabled=true' \
  'llm.gateway.mode=in_world' \
  'llm.routing.default_backend=cli:codex' \
  'agents.enabled=true' \
  'agents.defaults.execution.scope=world' \
  'agents.defaults.cli.mode=persistent'

echo "== Case 2: policy keys accept =="
expect_exit 0 "$SUBSTRATE_BIN" policy global set \
  'llm.fail_closed.routing=true' \
  'llm.require_approval=false' \
  'llm.allowed_backends+=cli:codex' \
  'agents.allowed_backends+=cli:codex' \
  'agents.fail_closed.routing=true'

echo "== Case 3: unknown keys reject (exit 2) =="
expect_exit 2 "$SUBSTRATE_BIN" config global set 'llm.unknown_key=true'
expect_exit 2 "$SUBSTRATE_BIN" policy global set 'agents.unknown_key=true'

echo "== Case 4: explain includes new keys =="
expect_exit 0 "$SUBSTRATE_BIN" config current show --explain
expect_exit 0 "$SUBSTRATE_BIN" policy current show --explain

echo "OK: llm_and_agent_config_policy_surface smoke passed"
