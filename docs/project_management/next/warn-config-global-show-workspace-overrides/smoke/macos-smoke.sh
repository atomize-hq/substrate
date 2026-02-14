#!/usr/bin/env bash
set -euo pipefail

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "SKIP: macos-smoke.sh intended for macOS (uname=$(uname -s))"
  exit 0
fi

SUBSTRATE_BIN="${SUBSTRATE_BIN:-substrate}"

if [[ "$SUBSTRATE_BIN" == "substrate" ]]; then
  command -v substrate >/dev/null 2>&1 || { echo "FAIL: substrate not found on PATH"; exit 2; }
else
  [[ -x "$SUBSTRATE_BIN" ]] || { echo "FAIL: SUBSTRATE_BIN is not executable: $SUBSTRATE_BIN"; exit 2; }
fi

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

home_dir="$tmp/home"
substrate_home="$tmp/substrate-home"
ws="$tmp/ws"

mkdir -p "$home_dir" "$substrate_home" "$ws"

export HOME="$home_dir"
export USERPROFILE="$home_dir"
export SUBSTRATE_HOME="$substrate_home"

"$SUBSTRATE_BIN" workspace init "$ws" >/dev/null

cat >"$SUBSTRATE_HOME/config.yaml" <<'YAML'
world:
  caged: false
YAML

pushd "$ws" >/dev/null

# Empty workspace patch => no note.
cat >"$ws/.substrate/workspace.yaml" <<'YAML'
{}
YAML

"$SUBSTRATE_BIN" config global show >"$tmp/stdout-empty.txt" 2>"$tmp/stderr-empty.txt"
if grep -q "substrate: note: workspace config" "$tmp/stderr-empty.txt"; then
  echo "FAIL: unexpected workspace-override note for empty workspace patch"
  cat "$tmp/stderr-empty.txt"
  exit 1
fi

# Non-empty workspace patch => note present.
cat >"$ws/.substrate/workspace.yaml" <<'YAML'
world:
  caged: true
YAML

"$SUBSTRATE_BIN" config global show >"$tmp/stdout-nonempty.txt" 2>"$tmp/stderr-nonempty.txt"

grep -q "substrate: note: workspace config" "$tmp/stderr-nonempty.txt" || { echo "FAIL: missing note"; cat "$tmp/stderr-nonempty.txt"; exit 1; }
grep -q "workspace.yaml" "$tmp/stderr-nonempty.txt" || { echo "FAIL: note missing workspace.yaml"; cat "$tmp/stderr-nonempty.txt"; exit 1; }
grep -q "overrides global config here" "$tmp/stderr-nonempty.txt" || { echo "FAIL: note missing override phrase"; cat "$tmp/stderr-nonempty.txt"; exit 1; }
grep -q "substrate config show --explain" "$tmp/stderr-nonempty.txt" || { echo "FAIL: note missing guidance"; cat "$tmp/stderr-nonempty.txt"; exit 1; }

if grep -q "substrate: note:" "$tmp/stdout-nonempty.txt"; then
  echo "FAIL: stdout contaminated with note text"
  cat "$tmp/stdout-nonempty.txt"
  exit 1
fi

# Invalid YAML => command succeeds and emits note.
cat >"$ws/.substrate/workspace.yaml" <<'YAML'
world: [this is not valid
YAML

"$SUBSTRATE_BIN" config global show >"$tmp/stdout-invalid.txt" 2>"$tmp/stderr-invalid.txt"
grep -q "substrate: note: workspace config" "$tmp/stderr-invalid.txt" || { echo "FAIL: missing note (invalid YAML)"; cat "$tmp/stderr-invalid.txt"; exit 1; }

# --json stdout parseable.
cat >"$ws/.substrate/workspace.yaml" <<'YAML'
world:
  caged: true
YAML

"$SUBSTRATE_BIN" config global show --json >"$tmp/stdout-json.txt" 2>"$tmp/stderr-json.txt"

python3 - <<PY
import json
with open("$tmp/stdout-json.txt","r",encoding="utf-8") as f:
    json.load(f)
PY

grep -q "substrate: note: workspace config" "$tmp/stderr-json.txt" || { echo "FAIL: missing note (--json)"; cat "$tmp/stderr-json.txt"; exit 1; }

# Case: implicit `config set` emits write-target note; stdout remains uncontaminated.
"$SUBSTRATE_BIN" config set sync.auto_sync=true >"$tmp/stdout-config-set.txt" 2>"$tmp/stderr-config-set.txt"

grep -q "substrate: note: write target is workspace config" "$tmp/stderr-config-set.txt" || {
  echo "FAIL: missing write-target note (config set)"
  cat "$tmp/stderr-config-set.txt"
  exit 1
}
grep -q "workspace.yaml" "$tmp/stderr-config-set.txt" || { echo "FAIL: write-target note missing workspace.yaml"; cat "$tmp/stderr-config-set.txt"; exit 1; }
grep -q "(implicit scope)" "$tmp/stderr-config-set.txt" || { echo "FAIL: write-target note missing (implicit scope)"; cat "$tmp/stderr-config-set.txt"; exit 1; }

if grep -q "substrate: note:" "$tmp/stdout-config-set.txt"; then
  echo "FAIL: stdout contaminated with note text (config set)"
  cat "$tmp/stdout-config-set.txt"
  exit 1
fi

# Case: `config set --json` stdout remains valid JSON when note is present.
"$SUBSTRATE_BIN" config set --json sync.auto_sync=true >"$tmp/stdout-config-set-json.txt" 2>"$tmp/stderr-config-set-json.txt"

python3 - <<PY
import json
with open("$tmp/stdout-config-set-json.txt","r",encoding="utf-8") as f:
    json.load(f)
PY

grep -q "substrate: note: write target is workspace config" "$tmp/stderr-config-set-json.txt" || {
  echo "FAIL: missing write-target note (config set --json)"
  cat "$tmp/stderr-config-set-json.txt"
  exit 1
}

popd >/dev/null

echo "PASS: macos smoke"
