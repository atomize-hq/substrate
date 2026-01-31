#!/usr/bin/env bash
set -euo pipefail

if [[ "$(uname -s)" != "Linux" ]]; then
  echo "SKIP: linux-smoke.sh intended for Linux (uname=$(uname -s))"
  exit 0
fi

SUBSTRATE_BIN="${SUBSTRATE_BIN:-substrate}"

# Resolve substrate binary.
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

# Initialize workspace structure.
"$SUBSTRATE_BIN" workspace init "$ws" >/dev/null

# Write a global config patch.
cat >"$SUBSTRATE_HOME/config.yaml" <<'YAML'
world:
  caged: false
YAML

pushd "$ws" >/dev/null

# Case: empty workspace patch => no note.
cat >"$ws/.substrate/workspace.yaml" <<'YAML'
{}
YAML

"$SUBSTRATE_BIN" config global show >"$tmp/stdout-empty.txt" 2>"$tmp/stderr-empty.txt"

if grep -q "substrate: note: workspace config" "$tmp/stderr-empty.txt"; then
  echo "FAIL: unexpected workspace-override note for empty workspace patch"
  cat "$tmp/stderr-empty.txt"
  exit 1
fi

# Case: non-empty workspace patch => note present.
cat >"$ws/.substrate/workspace.yaml" <<'YAML'
world:
  caged: true
YAML

"$SUBSTRATE_BIN" config global show >"$tmp/stdout-nonempty.txt" 2>"$tmp/stderr-nonempty.txt"

grep -q "substrate: note: workspace config" "$tmp/stderr-nonempty.txt" || {
  echo "FAIL: missing workspace-override note (non-empty workspace patch)"
  cat "$tmp/stderr-nonempty.txt"
  exit 1
}
grep -q "workspace.yaml" "$tmp/stderr-nonempty.txt" || { echo "FAIL: note missing workspace.yaml hint"; cat "$tmp/stderr-nonempty.txt"; exit 1; }
grep -q "overrides global config here" "$tmp/stderr-nonempty.txt" || { echo "FAIL: note missing override phrase"; cat "$tmp/stderr-nonempty.txt"; exit 1; }
grep -q "substrate config show --explain" "$tmp/stderr-nonempty.txt" || { echo "FAIL: note missing guidance to config show --explain"; cat "$tmp/stderr-nonempty.txt"; exit 1; }

if grep -q "substrate: note:" "$tmp/stdout-nonempty.txt"; then
  echo "FAIL: stdout is contaminated with note text"
  cat "$tmp/stdout-nonempty.txt"
  exit 1
fi

# Case: invalid YAML workspace patch => command still succeeds and emits note.
cat >"$ws/.substrate/workspace.yaml" <<'YAML'
world: [this is not valid
YAML

"$SUBSTRATE_BIN" config global show >"$tmp/stdout-invalid.txt" 2>"$tmp/stderr-invalid.txt"

grep -q "substrate: note: workspace config" "$tmp/stderr-invalid.txt" || {
  echo "FAIL: missing workspace-override note (invalid workspace patch YAML)"
  cat "$tmp/stderr-invalid.txt"
  exit 1
}

# Case: --json stdout remains valid JSON when note is present.
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

grep -q "substrate: note: workspace config" "$tmp/stderr-json.txt" || {
  echo "FAIL: missing workspace-override note (--json mode)"
  cat "$tmp/stderr-json.txt"
  exit 1
}

popd >/dev/null

echo "PASS: linux smoke"
