#!/usr/bin/env bash
set -euo pipefail

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "SKIP: world_deps_selection_layer macOS smoke (not macOS)"
  exit 0
fi

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "FAIL: missing required dependency: $1" >&2
    exit 3
  fi
}

require_cmd substrate
require_cmd jq

help="$(substrate world deps --help 2>/dev/null || true)"
if ! grep -Eq '\bstatus\b' <<<"$help" || ! grep -Eq '\bsync\b' <<<"$help" || ! grep -Eq '\binstall\b' <<<"$help"; then
  echo "FAIL: substrate world deps subcommands missing (expected status/sync/install)" >&2
  exit 1
fi
if ! grep -Eq '\binit\b' <<<"$help" || ! grep -Eq '\bselect\b' <<<"$help"; then
  echo "FAIL: substrate world deps subcommands missing (expected init/select for WDL0)" >&2
  exit 1
fi

has_provision=0
if grep -Eq '\bprovision\b' <<<"$help"; then
  has_provision=1
fi

tmp_root="$(mktemp -d)"
cleanup() { rm -rf "$tmp_root"; }
trap cleanup EXIT

export SUBSTRATE_HOME="$tmp_root/substrate-home"
mkdir -p "$SUBSTRATE_HOME"

workspace="$tmp_root/workspace"
mkdir -p "$workspace"
cd "$workspace"

run_expect() {
  local expected_code="$1"
  local expected_substring="$2"
  shift 2

  set +e
  local out
  out="$("$@" 2>&1)"
  local code="$?"
  set -e

  if [[ "$code" -ne "$expected_code" ]]; then
    echo "FAIL: expected exit $expected_code, got $code: $*" >&2
    echo "$out" >&2
    exit 1
  fi

  if [[ -n "$expected_substring" ]] && ! grep -Fq "$expected_substring" <<<"$out"; then
    echo "FAIL: expected output substring not found: $expected_substring" >&2
    echo "CMD: $*" >&2
    echo "$out" >&2
    exit 1
  fi
}

require_world_backend() {
  local doctor
  set +e
  doctor="$(substrate world doctor --json 2>&1)"
  local code="$?"
  set -e

  if [[ "$code" -ne 0 ]]; then
    echo "FAIL: substrate world doctor --json failed (exit $code)" >&2
    echo "$doctor" >&2
    exit 3
  fi

  if ! echo "$doctor" | jq -e '.world.ok==true' >/dev/null; then
    echo "FAIL: world backend not ready for backend-required smoke steps (expected .world.ok==true)" >&2
    echo "$doctor" >&2
    echo "" >&2
    echo "Remediation:" >&2
    echo "  - Run: scripts/mac/lima-warm.sh" >&2
    echo "  - Re-check: substrate world doctor --json | jq '{ok,world:{ok:.world.ok,status:.world.status}}'" >&2
    exit 3
  fi
}

echo "== WDL0: selection missing no-op (prove no world calls via invalid socket) =="
rm -rf .substrate
rm -f "$SUBSTRATE_HOME/world-deps.selection.yaml"
export SUBSTRATE_WORLD_SOCKET="$tmp_root/does-not-exist.sock"

run_expect 0 "world deps not configured (selection file missing)" substrate world deps status
run_expect 0 "world deps not configured (selection file missing)" substrate world deps sync
run_expect 0 "world deps not configured (selection file missing)" substrate world deps install nvm
if [[ "$has_provision" -eq 1 ]]; then
  run_expect 0 "world deps not configured (selection file missing)" substrate world deps provision
fi

echo "== WDL0: configured-but-empty selection is valid and makes no world calls =="
run_expect 0 "" substrate world deps init --workspace --force

status_json="$(substrate world deps status --json)"
echo "$status_json" | jq -e '
  .selection.configured == true
  and .selection.active_scope == "workspace"
  and (.selection.selected | length) == 0
  and .selection.active_path == ".substrate/world-deps.selection.yaml"
' >/dev/null

run_expect 0 "No tools selected; nothing to do." substrate world deps sync
run_expect 2 "tool not selected" substrate world deps install nvm

echo "== WDL0: select updates scope deterministically =="
run_expect 0 "" substrate world deps select --workspace nvm bun
status_json="$(substrate world deps status --json)"
echo "$status_json" | jq -e '.selection.selected | index("nvm") != null and index("bun") != null' >/dev/null

echo "== WDL1/WDL2: backend-required checks (capability-gated) =="
unset SUBSTRATE_WORLD_SOCKET

status_all_json="$(substrate world deps status --all --json)"
if echo "$status_all_json" | jq -e '.tools[0].install_class? != null' >/dev/null; then
  echo "OK: WDL1 capability detected (install_class present)"

  cat >"$SUBSTRATE_HOME/manager_hooks.local.yaml" <<'YAML'
version: 2
managers:
  - name: wdl-smoke-system-packages
    guest_detect:
      command: "dpkg -s cowsay >/dev/null 2>&1"
    guest_install:
      class: system_packages
      system_packages:
        apt:
          - cowsay
YAML

  run_expect 0 "" substrate world deps init --workspace --force
  run_expect 0 "" substrate world deps select --workspace wdl-smoke-system-packages

  require_world_backend

  status_all_json="$(substrate world deps status --all --json)"
  echo "$status_all_json" | jq -e '
    ( .tools[] | select(.name=="wdl-smoke-system-packages") | .install_class=="system_packages" )
  ' >/dev/null

  pre_guest_status="$(echo "$status_all_json" | jq -r '.tools[] | select(.name=="wdl-smoke-system-packages") | .guest.status')"
  if [[ "$pre_guest_status" == "skipped" ]]; then
    run_expect 4 "substrate world deps provision" substrate world deps sync
  elif [[ "$pre_guest_status" == "present" ]]; then
    run_expect 0 "" substrate world deps sync
  else
    echo "FAIL: unexpected guest.status for wdl-smoke-system-packages: $pre_guest_status" >&2
    echo "$status_all_json" >&2
    exit 1
  fi

  if [[ "$has_provision" -eq 1 ]]; then
    echo "OK: WDL2 capability detected (provision present)"

    run_expect 0 "cowsay" substrate world deps provision --dry-run
    run_expect 0 "system packages installed" substrate world deps provision
    run_expect 0 "system packages installed" substrate world deps provision

    post_json="$(substrate world deps status --all --json)"
    post_status="$(echo "$post_json" | jq -r '.tools[] | select(.name=="wdl-smoke-system-packages") | .guest.status')"
    if [[ "$post_status" != "present" ]]; then
      echo "FAIL: expected wdl-smoke-system-packages guest.status=present after provision, got: $post_status" >&2
      echo "$post_json" >&2
      exit 1
    fi

    run_expect 0 "" substrate world deps sync
  else
    echo "INFO: WDL2 capability not detected (provision absent); skipping provisioning assertions"
  fi
else
  echo "INFO: WDL1 capability not detected (install_class absent); skipping WDL1/WDL2 assertions"
fi

echo "OK: world_deps_selection_layer macOS smoke"
