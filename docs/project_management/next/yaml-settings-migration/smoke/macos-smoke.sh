#!/usr/bin/env bash
set -euo pipefail

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "SKIP: yaml-settings-migration macOS smoke (not macOS)"
  exit 0
fi

if ! command -v substrate >/dev/null 2>&1; then
  echo "FAIL: substrate not found on PATH" >&2
  exit 1
fi

if ! command -v jq >/dev/null 2>&1; then
  echo "FAIL: jq is required for YAML settings migration smoke" >&2
  exit 1
fi

Y0_TEST_HOME="$(mktemp -d)"
Y0_TEST_WS="$(mktemp -d)"
cleanup() { rm -rf "$Y0_TEST_HOME" "$Y0_TEST_WS"; }
trap cleanup EXIT

HOME="$Y0_TEST_HOME" substrate config init --force >/dev/null
test -f "$Y0_TEST_HOME/.substrate/config.yaml"
test ! -e "$Y0_TEST_HOME/.substrate/config.toml"

HOME="$Y0_TEST_HOME" substrate config show --json | jq -e '.world.anchor_mode' >/dev/null

HOME="$Y0_TEST_HOME" substrate config set world.anchor_mode=follow-cwd >/dev/null
grep -q 'anchor_mode: follow-cwd' "$Y0_TEST_HOME/.substrate/config.yaml"

mkdir -p "$Y0_TEST_WS/.substrate"
cat > "$Y0_TEST_WS/.substrate/settings.yaml" <<'YAML'
world:
  anchor_mode: project
  caged: true
YAML
cd "$Y0_TEST_WS"
HOME="$Y0_TEST_HOME" substrate config show --json | jq -e '.world.anchor_mode=="project"' >/dev/null

mkdir -p "$Y0_TEST_HOME/.substrate"
cat > "$Y0_TEST_HOME/.substrate/config.toml" <<'TOML'
[world]
anchor_mode = "project"
TOML
set +e
HOME="$Y0_TEST_HOME" substrate config show >/dev/null 2>&1
code=$?
set -e
test "$code" -eq 2

echo "OK: yaml-settings-migration macOS smoke"

