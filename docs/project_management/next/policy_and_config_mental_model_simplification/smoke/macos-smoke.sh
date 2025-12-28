#!/usr/bin/env bash
set -euo pipefail

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "SKIP: policy/config mental model macOS smoke (not macOS)"
  exit 0
fi

if ! command -v substrate >/dev/null 2>&1; then
  echo "FAIL: substrate not found on PATH" >&2
  exit 1
fi

if ! command -v jq >/dev/null 2>&1; then
  echo "FAIL: jq not found on PATH" >&2
  exit 3
fi

TMP_HOME="$(mktemp -d)"
TMP_WS="$(mktemp -d)"
cleanup() { rm -rf "$TMP_HOME" "$TMP_WS"; }
trap cleanup EXIT

export SUBSTRATE_HOME="$TMP_HOME"
export HOME="$TMP_HOME"

substrate config global init --force >/dev/null
test -f "$SUBSTRATE_HOME/config.yaml"
test -f "$SUBSTRATE_HOME/env.sh"

substrate policy global init --force >/dev/null
test -f "$SUBSTRATE_HOME/policy.yaml"

substrate workspace init "$TMP_WS" >/dev/null
test -f "$TMP_WS/.substrate/workspace.yaml"
test -f "$TMP_WS/.substrate/policy.yaml"
test -d "$TMP_WS/.substrate-git/repo.git"

cd "$TMP_WS"
substrate config show --json | jq -e '.world.anchor_mode=="workspace"' >/dev/null

substrate world enable --help | grep -q -- '--home'
if substrate world enable --help | grep -q -- '--prefix'; then
  echo "FAIL: --prefix present in help output" >&2
  exit 1
fi

echo "OK: policy/config mental model macOS smoke (gating)"

