#!/usr/bin/env bash
set -euo pipefail

# Shared behavior smoke for world-deps-host-visible-hardening.
#
# Exit codes (aligned to `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`):
# - 0: smoke passed
# - 1: smoke assertion failed / unexpected script error
# - 2: invalid inputs
# - 3: required dependency unavailable
# - 4: not supported / missing prerequisites
# - 5: safety / policy violation (surfaced via substrate exit code where applicable)

SUBSTRATE_BIN="${SUBSTRATE_BIN:-substrate}"

if ! command -v "$SUBSTRATE_BIN" >/dev/null 2>&1; then
  echo "world-deps-host-visible-hardening: substrate binary not found (SUBSTRATE_BIN=$SUBSTRATE_BIN)" >&2
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

slice_id="${SUBSTRATE_SMOKE_SLICE_ID:-WDH3}"
case "$slice_id" in
  WDH0|WDH1|WDH2|WDH3) ;;
  *)
    echo "world-deps-host-visible-hardening: unknown SUBSTRATE_SMOKE_SLICE_ID=$slice_id" >&2
    exit 2
    ;;
esac

if [[ "$slice_id" == "WDH3" ]]; then
  echo "== Case Z (WDH3): SUBSTRATE_HOME deps scaffold exists on bootstrap =="
  scaffold_home="$tmp_root/substrate-home-scaffold"
  rm -rf "$scaffold_home"
  SUBSTRATE_HOME="$scaffold_home" "$SUBSTRATE_BIN" --version >/dev/null
  if [[ ! -f "$scaffold_home/deps/README.md" ]]; then
    echo "world-deps-host-visible-hardening: expected deps scaffold README.md at $scaffold_home/deps/README.md" >&2
    exit 1
  fi
fi

echo "== Setup: smoke inventory override (no network) =="
# Override the built-in `npm` package with a deterministic script-installed fixture so this smoke is
# offline and repeatable. Inventory merge semantics should full-replace by name.
mkdir -p "$SUBSTRATE_HOME/deps/packages"
cat >"$SUBSTRATE_HOME/deps/packages/npm.yaml" <<'YAML'
version: 1
name: npm
description: Smoke override for npm (script; no network).
runnable: true
entrypoints: ["npm"]
install:
  method: script
  script: |
    set -euo pipefail
    mkdir -p /var/lib/substrate/world-deps/bin
    cat >/var/lib/substrate/world-deps/bin/npm <<'EOF'
    #!/bin/sh
    echo world-npm
    EOF
    chmod +x /var/lib/substrate/world-deps/bin/npm
probe:
  command: "npm >/dev/null"
YAML

echo "== Preflight: world doctor =="
if ! "$SUBSTRATE_BIN" world doctor >/dev/null 2>&1; then
  echo "world-deps-host-visible-hardening: world backend not healthy; run 'substrate world doctor' remediation and retry" >&2
  exit 4
fi

echo "== Setup: host-visible policy =="
"$SUBSTRATE_BIN" policy global set world_fs.host_visible=true >/dev/null

echo "== Case A: PATH is sanitized and prefixed with world-deps bin =="
path="$("$SUBSTRATE_BIN" --world -c 'printf "%s" "$PATH"')"
printf "%s\n" "$path" | grep -q '^/var/lib/substrate/world-deps/bin:'
printf "%s\n" "$path" | grep -qv '/\\.config/nvm/'
printf "%s\n" "$path" | grep -qv '/\\.pyenv/'
printf "%s\n" "$path" | grep -qv '/\\.cargo/bin'
printf "%s\n" "$path" | grep -qv '/\\.local/bin'

echo "== Case B: no enabled deps => npm not discoverable via PATH =="
"$SUBSTRATE_BIN" world deps global reset >/dev/null 2>&1 || true
"$SUBSTRATE_BIN" world deps workspace reset >/dev/null 2>&1 || true

fake_nvm_bin="$workspace/fake-home/.config/nvm/versions/node/v0.0.0/bin"
mkdir -p "$fake_nvm_bin"
cat >"$fake_nvm_bin/npm" <<'EOF'
#!/bin/sh
echo fake-host-npm
EOF
chmod +x "$fake_nvm_bin/npm"

set +e
PATH="$fake_nvm_bin:/usr/bin:/bin" "$SUBSTRATE_BIN" --world -c 'command -v npm >/dev/null'
status=$?
set -e
if [[ "$status" -ne 1 ]]; then
  echo "world-deps-host-visible-hardening: expected npm to be undiscoverable (exit 1), got exit=$status" >&2
  exit 1
fi

echo "== Case C: enable npm => wrapper resolves under world-deps bin =="
"$SUBSTRATE_BIN" world deps global add npm >/dev/null
"$SUBSTRATE_BIN" world deps current sync >/dev/null
resolved="$("$SUBSTRATE_BIN" --world -c 'command -v npm')"
if [[ "$resolved" != "/var/lib/substrate/world-deps/bin/npm" ]]; then
  echo "world-deps-host-visible-hardening: expected wrapper path, got: $resolved" >&2
  exit 1
fi
out="$("$SUBSTRATE_BIN" --world -c 'npm')"
if [[ "$out" != "world-npm" ]]; then
  echo "world-deps-host-visible-hardening: expected npm to run world wrapper, got: $out" >&2
  exit 1
fi

if [[ "$slice_id" == "WDH0" ]]; then
  echo "OK: smoke passed (slice=$slice_id)"
  exit 0
fi

if [[ "$slice_id" == "WDH2" || "$slice_id" == "WDH3" ]]; then
  echo "== Case D ($slice_id): exec guard denies explicit host toolchain path =="
  set +e
  PATH="$fake_nvm_bin:/usr/bin:/bin" "$SUBSTRATE_BIN" --world -c "$fake_nvm_bin/npm --version" >/dev/null 2>&1
  status=$?
  set -e
  if [[ "$status" -ne 5 ]]; then
    echo "world-deps-host-visible-hardening: expected exec guard deny (exit 5), got exit=$status" >&2
    exit 1
  fi
fi

echo "OK: smoke passed (slice=$slice_id)"
