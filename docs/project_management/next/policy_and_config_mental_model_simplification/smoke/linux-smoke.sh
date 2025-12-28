#!/usr/bin/env bash
set -euo pipefail

if [[ "$(uname -s)" != "Linux" ]]; then
  echo "SKIP: policy/config mental model linux smoke (not Linux)"
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

grep -qE '^\\.substrate-git/' "$TMP_WS/.gitignore"
grep -qE '^\\.substrate/\\*$' "$TMP_WS/.gitignore"
grep -qE '^!\\.substrate/workspace\\.yaml$' "$TMP_WS/.gitignore"
grep -qE '^!\\.substrate/policy\\.yaml$' "$TMP_WS/.gitignore"

cd "$TMP_WS"
substrate config show --json | jq -e '.world.anchor_mode=="workspace"' >/dev/null
substrate policy show --json | jq -e '.world_fs.isolation=="project"' >/dev/null

mkdir -p .substrate
printf '%s\n' 'world:' '  enabled: true' > .substrate/settings.yaml
set +e
substrate config show >/dev/null 2>&1
code=$?
set -e
test "$code" -eq 2

substrate world enable --help | grep -q -- '--home'
if substrate world enable --help | grep -q -- '--prefix'; then
  echo "FAIL: --prefix present in help output" >&2
  exit 1
fi

echo "OK: policy/config mental model linux smoke (gating)"

