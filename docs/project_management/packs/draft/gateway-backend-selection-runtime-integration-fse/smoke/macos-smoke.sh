#!/usr/bin/env bash
set -euo pipefail

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "FAIL: macos-smoke.sh is intended for macOS (uname=$(uname -s))" >&2
  exit 4
fi

SUBSTRATE_BIN="${SUBSTRATE_BIN:-substrate}"
if [[ "$SUBSTRATE_BIN" == "substrate" ]]; then
  command -v substrate >/dev/null 2>&1 || {
    echo "FAIL: substrate not found on PATH" >&2
    exit 3
  }
  SUBSTRATE_BIN="$(command -v substrate)"
else
  [[ -x "$SUBSTRATE_BIN" ]] || {
    echo "FAIL: SUBSTRATE_BIN is not executable: $SUBSTRATE_BIN" >&2
    exit 3
  }
  SUBSTRATE_BIN="$(cd "$(dirname "$SUBSTRATE_BIN")" && pwd)/$(basename "$SUBSTRATE_BIN")"
fi

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

home_dir="$tmp/home"
substrate_home="$tmp/substrate-home"
workspace="$tmp/workspace"
mkdir -p "$home_dir" "$substrate_home" "$workspace"

export HOME="$home_dir"
export USERPROFILE="$home_dir"
export SUBSTRATE_HOME="$substrate_home"
export SUBSTRATE_WORLD_ENABLED=1
export SUBSTRATE_WORLD=enabled
export SUBSTRATE_WORLD_SOCKET="$tmp/missing.sock"

cat >"$SUBSTRATE_HOME/config.yaml" <<'YAML'
llm:
  enabled: true
  gateway:
    enabled: true
  routing:
    default_backend: api:anthropic
YAML

cat >"$SUBSTRATE_HOME/policy.yaml" <<'YAML'
id: "gateway-policy"
name: "gateway-policy"

world_fs:
  host_visible: true
  fail_closed:
    routing: false
  write:
    enabled: true

llm:
  allowed_backends:
    - "api:anthropic"

net_allowed: []
cmd_allowed: []
cmd_denied: []
cmd_isolated: []

require_approval: false
allow_shell_operators: true

limits:
  max_memory_mb: null
  max_cpu_percent: null
  max_runtime_ms: null
  max_egress_bytes: null

metadata: {}
YAML

pushd "$workspace" >/dev/null
status_stdout="$tmp/status.stdout"
status_stderr="$tmp/status.stderr"
sync_stdout="$tmp/sync.stdout"
sync_stderr="$tmp/sync.stderr"
restart_stdout="$tmp/restart.stdout"
restart_stderr="$tmp/restart.stderr"

run_case() {
  local action="$1"
  local stdout_path="$2"
  local stderr_path="$3"
  if "$SUBSTRATE_BIN" world gateway "$action" >"$stdout_path" 2>"$stderr_path"; then
    echo "FAIL: expected unsupported backend to fail for world gateway $action" >&2
    exit 1
  fi

  if ! grep -q "invalid integration" "$stderr_path"; then
    echo "FAIL: missing invalid integration text for world gateway $action" >&2
    cat "$stderr_path" >&2
    exit 1
  fi

  if grep -q "cli:codex" "$stdout_path" || grep -q "cli:codex" "$stderr_path"; then
    echo "FAIL: unsupported backend path mentioned cli:codex fallback for world gateway $action" >&2
    cat "$stderr_path" >&2
    exit 1
  fi
}

run_case status "$status_stdout" "$status_stderr"
run_case sync "$sync_stdout" "$sync_stderr"
run_case restart "$restart_stdout" "$restart_stderr"

popd >/dev/null

echo "PASS: macOS unsupported-backend evidence"
