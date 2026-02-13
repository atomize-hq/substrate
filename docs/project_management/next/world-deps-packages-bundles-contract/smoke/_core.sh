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

echo "== Setup: workspace =="
"$SUBSTRATE_BIN" workspace init --force >/dev/null

echo "== Setup: smoke inventory =="
mkdir -p "$SUBSTRATE_HOME/deps/packages" "$SUBSTRATE_HOME/deps/bundles"

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

"$SUBSTRATE_BIN" world deps global add smoke-bundle >/dev/null
"$SUBSTRATE_BIN" world deps workspace add smoke-hello >/dev/null

enabled="$("$SUBSTRATE_BIN" world deps current list enabled 2>&1)"
printf "%s\n" "$enabled" | grep -q "smoke-bundle"
printf "%s\n" "$enabled" | grep -q "smoke-hello"

echo "== Preflight: world doctor =="
if ! "$SUBSTRATE_BIN" world doctor >/dev/null 2>&1; then
  echo "world-deps-packages-bundles-contract: world backend not healthy; run 'substrate world doctor' remediation and retry" >&2
  exit 4
fi

echo "== Case C: applied view and explain =="
applied="$("$SUBSTRATE_BIN" world deps current list applied 2>&1)"
printf "%s\n" "$applied" | grep -q "world="
printf "%s\n" "$applied" | grep -q "smoke-hello"

expl="$("$SUBSTRATE_BIN" world deps current show smoke-hello --explain 2>&1)"
printf "%s\n" "$expl" | grep -q "substrate: hint:"

if [[ "$slice_id" == "WDP2" ]]; then
  echo "OK: world-deps-packages-bundles-contract smoke passed (slice=$slice_id platform=${OSTYPE:-unknown})"
  exit 0
fi

echo "== Case D: sync dry-run + sync apply =="
"$SUBSTRATE_BIN" world deps global reset >/dev/null
"$SUBSTRATE_BIN" world deps workspace reset >/dev/null
"$SUBSTRATE_BIN" world deps workspace add smoke-bundle >/dev/null

"$SUBSTRATE_BIN" world deps current sync --dry-run >/dev/null
"$SUBSTRATE_BIN" world deps current sync >/dev/null

applied2="$("$SUBSTRATE_BIN" world deps current list applied 2>&1)"
printf "%s\n" "$applied2" | grep -q "smoke-hello"
printf "%s\n" "$applied2" | grep -q "world=present"

echo "OK: world-deps-packages-bundles-contract smoke passed (slice=$slice_id platform=${OSTYPE:-unknown})"

