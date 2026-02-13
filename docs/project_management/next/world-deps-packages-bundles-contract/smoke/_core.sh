#!/usr/bin/env bash
set -euo pipefail

# Shared behavior smoke for world-deps-packages-bundles-contract.
#
# Exit codes (aligned to `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`):
# - 0: smoke passed
# - 1: smoke assertion failed / unexpected script error
# - 2: invalid inputs (e.g., unknown SUBSTRATE_SMOKE_SLICE_ID)
# - 3: required dependency unavailable (e.g., substrate not found)
# - 4: not supported / missing prerequisites (e.g., world backend not healthy)

SUBSTRATE_BIN="${SUBSTRATE_BIN:-substrate}"

if ! command -v "$SUBSTRATE_BIN" >/dev/null 2>&1; then
  echo "world-deps-packages-bundles-contract: substrate binary not found (SUBSTRATE_BIN=$SUBSTRATE_BIN)" >&2
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

workspace="$tmp_root/workspace"
mkdir -p "$workspace"
cd "$workspace"

export SUBSTRATE_HOME="$tmp_root/substrate-home"
mkdir -p "$SUBSTRATE_HOME"

slice_id="${SUBSTRATE_SMOKE_SLICE_ID:-WDP5}"
case "$slice_id" in
  WDP2|WDP5) ;;
  *)
    echo "world-deps-packages-bundles-contract: unknown SUBSTRATE_SMOKE_SLICE_ID=$slice_id" >&2
    exit 2
    ;;
esac

PYTHON_BIN=""
if command -v python3 >/dev/null 2>&1; then
  PYTHON_BIN="python3"
elif command -v python >/dev/null 2>&1; then
  PYTHON_BIN="python"
else
  echo "world-deps-packages-bundles-contract: python is required for --json smoke assertions (install python3 and retry)" >&2
  exit 3
fi

echo "== Setup: workspace =="
"$SUBSTRATE_BIN" workspace init --force >/dev/null

echo "== Setup: smoke inventory =="
mkdir -p "$SUBSTRATE_HOME/deps/packages" "$SUBSTRATE_HOME/deps/bundles"

echo "== Setup: legacy files (must be ignored) =="
# The new `substrate world deps` contract MUST NOT read or be influenced by legacy world-deps paths.
# We create intentionally-invalid YAML at those legacy paths to ensure the new implementation is
# fail-safe (i.e., these files are not parsed at all).
mkdir -p "$workspace/.substrate" "$workspace/config" "$workspace/scripts/substrate"
printf "invalid: [\n" >"$workspace/.substrate/world-deps.selection.yaml"
printf "invalid: [\n" >"$workspace/config/manager_hooks.yaml"
printf "invalid: [\n" >"$workspace/scripts/substrate/world-deps.yaml"
printf "invalid: [\n" >"$SUBSTRATE_HOME/manager_hooks.local.yaml"
printf "invalid: [\n" >"$SUBSTRATE_HOME/world-deps.local.yaml"
printf "invalid: [\n" >"$SUBSTRATE_HOME/world-deps.selection.yaml"

cat >"$SUBSTRATE_HOME/deps/packages/smoke-hello.yaml" <<'YAML'
version: 1
name: smoke-hello
description: Deterministic smoke package for world-deps (no network).
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

cat >"$SUBSTRATE_HOME/deps/packages/smoke-manual.yaml" <<'YAML'
version: 1
name: smoke-manual
description: Deterministic manual package for world-deps (expected to be blocked unless preinstalled).
runnable: false
install:
  method: manual
  manual_instructions: |
    This is a smoke fixture package.
    It must never be auto-installed. Treat as manual/blocked unless you install it explicitly.
probe:
  command: "sh -c 'exit 1'"
YAML

cat >"$SUBSTRATE_HOME/deps/packages/smoke-apt.yaml" <<'YAML'
version: 1
name: smoke-apt
description: Deterministic apt package for world-deps (dry-run only in smoke).
runnable: false
install:
  method: apt
  apt:
    - name: bash
probe:
  command: "bash --version"
YAML

cat >"$SUBSTRATE_HOME/deps/bundles/smoke-bundle.yaml" <<'YAML'
version: 1
name: smoke-bundle
description: Deterministic smoke bundle for world-deps.
packages: ["smoke-hello"]
YAML

echo "== Case A: inventory visible =="
out="$("$SUBSTRATE_BIN" world deps current list available 2>&1)"
printf "%s\n" "$out" | grep -q "smoke-hello"
printf "%s\n" "$out" | grep -q "smoke-bundle"

echo "== Case B: enabled patch editing + effective enabled view =="
"$SUBSTRATE_BIN" world deps global reset >/dev/null 2>&1 || true
"$SUBSTRATE_BIN" world deps workspace reset >/dev/null 2>&1 || true

"$SUBSTRATE_BIN" world deps global add smoke-bundle smoke-manual >/dev/null
"$SUBSTRATE_BIN" world deps workspace add smoke-hello >/dev/null

enabled="$("$SUBSTRATE_BIN" world deps current list enabled 2>&1)"
printf "%s\n" "$enabled" | grep -q "smoke-bundle"
printf "%s\n" "$enabled" | grep -q "smoke-hello"
printf "%s\n" "$enabled" | grep -q "smoke-manual"

echo "== Case B2: non-world views do not require backend =="
# Force an unavailable backend; non-world views must still succeed.
unavail_socket="$tmp_root/does-not-exist.sock"
SUBSTRATE_WORLD_SOCKET="$unavail_socket" "$SUBSTRATE_BIN" world deps current list available >/dev/null
SUBSTRATE_WORLD_SOCKET="$unavail_socket" "$SUBSTRATE_BIN" world deps current list enabled >/dev/null
SUBSTRATE_WORLD_SOCKET="$unavail_socket" "$SUBSTRATE_BIN" world deps current show smoke-hello >/dev/null

echo "== Preflight: world doctor =="
if ! "$SUBSTRATE_BIN" world doctor >/dev/null 2>&1; then
  echo "world-deps-packages-bundles-contract: world backend not healthy; run 'substrate world doctor' remediation and retry" >&2
  exit 4
fi

echo "== Case C0: world-backed views fail-closed when backend unavailable =="
set +e
applied_unavail="$(
  SUBSTRATE_WORLD_SOCKET="$unavail_socket" "$SUBSTRATE_BIN" world deps current list applied 2>&1
)"
status=$?
set -e
if [[ "$status" -ne 3 ]]; then
  echo "world-deps-packages-bundles-contract: expected exit 3 for backend unavailable, got exit=$status" >&2
  printf "%s\n" "$applied_unavail" >&2
  exit 1
fi
printf "%s\n" "$applied_unavail" | grep -Eq "world doctor|doctor --json"

set +e
explain_unavail="$(
  SUBSTRATE_WORLD_SOCKET="$unavail_socket" "$SUBSTRATE_BIN" world deps current show smoke-hello --explain 2>&1
)"
status=$?
set -e
if [[ "$status" -ne 3 ]]; then
  echo "world-deps-packages-bundles-contract: expected exit 3 for backend unavailable (show --explain), got exit=$status" >&2
  printf "%s\n" "$explain_unavail" >&2
  exit 1
fi
printf "%s\n" "$explain_unavail" | grep -Eq "world doctor|doctor --json"

echo "== Case C: applied view and explain =="
applied="$("$SUBSTRATE_BIN" world deps current list applied 2>&1)"
printf "%s\n" "$applied" | grep -q "world="
printf "%s\n" "$applied" | grep -q "smoke-hello"
printf "%s\n" "$applied" | grep -q "smoke-manual"
printf "%s\n" "$applied" | grep -q "blocked"
if printf "%s\n" "$applied" | grep -q "smoke-apt"; then
  echo "world-deps-packages-bundles-contract: expected default applied scope to exclude non-enabled items (unexpected smoke-apt)" >&2
  exit 1
fi

expl="$("$SUBSTRATE_BIN" world deps current show smoke-hello --explain 2>&1)"
printf "%s\n" "$expl" | grep -q "substrate: hint:"

echo "== Case C2: applied --all includes all visible items (and --json is stable) =="
applied_all="$("$SUBSTRATE_BIN" world deps current list applied --all 2>&1)"
printf "%s\n" "$applied_all" | grep -q "smoke-apt"

applied_json="$("$SUBSTRATE_BIN" world deps current list applied --json 2>/dev/null)"
printf "%s" "$applied_json" | "$PYTHON_BIN" - <<'PY'
import json
import sys

data = json.load(sys.stdin)
items = data if isinstance(data, list) else (data.get("items") or data.get("data"))
if not isinstance(items, list) or not items:
    raise SystemExit("expected JSON list (or object with items/data list) for list applied --json")

required = {"name", "kind", "enabled", "world"}
for it in items:
    if not isinstance(it, dict):
        raise SystemExit("expected list items to be objects")
    missing = sorted(required - set(it.keys()))
    if missing:
        raise SystemExit(f"missing required keys in item: {missing}")
PY

applied_all_json="$("$SUBSTRATE_BIN" world deps current list applied --all --json 2>/dev/null)"
printf "%s" "$applied_all_json" | "$PYTHON_BIN" - <<'PY'
import json
import sys

data = json.load(sys.stdin)
items = data if isinstance(data, list) else (data.get("items") or data.get("data"))
if not isinstance(items, list) or not items:
    raise SystemExit("expected JSON list (or object with items/data list) for list applied --all --json")

names = {it.get("name") for it in items if isinstance(it, dict)}
if "smoke-apt" not in names:
    raise SystemExit("expected smoke-apt to be present in applied --all --json output")
PY

show_json="$("$SUBSTRATE_BIN" world deps current show smoke-hello --json 2>/dev/null)"
printf "%s" "$show_json" | "$PYTHON_BIN" - <<'PY'
import json
import sys

data = json.load(sys.stdin)
if not isinstance(data, dict):
    raise SystemExit("expected JSON object for show --json")
for k in ("name", "kind"):
    if k not in data:
        raise SystemExit(f"missing required key in show --json output: {k}")
PY

if [[ "$slice_id" == "WDP2" ]]; then
  echo "OK: world-deps-packages-bundles-contract smoke passed (slice=$slice_id platform=${OSTYPE:-unknown})"
  exit 0
fi

echo "== Case D: sync dry-run + sync apply =="
"$SUBSTRATE_BIN" world deps global reset >/dev/null
"$SUBSTRATE_BIN" world deps workspace reset >/dev/null
"$SUBSTRATE_BIN" world deps workspace add smoke-bundle >/dev/null

plan_install="$("$SUBSTRATE_BIN" world deps current install smoke-apt --dry-run 2>&1)"
printf "%s\n" "$plan_install" | grep -q "bash"

plan_sync="$("$SUBSTRATE_BIN" world deps current sync --dry-run 2>&1)"
printf "%s\n" "$plan_sync" | grep -q "smoke-hello"
"$SUBSTRATE_BIN" world deps current sync >/dev/null

applied2="$("$SUBSTRATE_BIN" world deps current list applied 2>&1)"
printf "%s\n" "$applied2" | grep -q "smoke-hello"
printf "%s\n" "$applied2" | grep -q "world=present"

echo "OK: world-deps-packages-bundles-contract smoke passed (slice=$slice_id platform=${OSTYPE:-unknown})"
