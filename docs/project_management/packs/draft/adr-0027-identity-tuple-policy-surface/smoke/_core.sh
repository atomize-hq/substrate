#!/usr/bin/env bash
set -euo pipefail

PLATFORM_NAME="${ITPS_SMOKE_PLATFORM:?ITPS_SMOKE_PLATFORM is required}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FEATURE_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
FEATURE_PATH="docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface"
SUBSTRATE_BIN="${SUBSTRATE_BIN:-substrate}"

expect_exit() {
  local want="$1"
  shift
  local out got
  set +e
  out="$("$@" 2>&1)"
  got="$?"
  set -e
  if [[ "$got" -ne "$want" ]]; then
    echo "FAIL: expected exit $want, got $got: $*" >&2
    echo "$out" >&2
    exit 1
  fi
}

echo "== Doc contract checks =="
jq -e '
  .meta.behavior_platforms_required == ["linux","macos"]
  and .meta.ci_parity_platforms_required == ["linux","macos","windows"]
' "$FEATURE_DIR/tasks.json" >/dev/null
jq -e '
  [
    .tasks[]
    | select(
        .id == "ITPS0-code"
        or .id == "ITPS0-test"
        or .id == "ITPS0-integ"
        or .id == "ITPS1-code"
        or .id == "ITPS1-test"
        or .id == "ITPS1-integ"
      )
    | (
        (.references | index("'"$FEATURE_PATH"'/decision_register.md (DR-ITPS-01)") != null)
        and (.references | index("'"$FEATURE_PATH"'/decision_register.md (DR-ITPS-02)") != null)
      )
  ]
  | length == 6
  and all(.[])
' "$FEATURE_DIR/tasks.json" >/dev/null

rg -q 'substrate policy current show --explain.*authoritative|authoritative.*substrate policy current show --explain' \
  "$FEATURE_DIR/contract.md" \
  "$FEATURE_DIR/manual_testing_playbook.md" \
  "$FEATURE_DIR/pre-planning/spec_manifest.md"
rg -q 'smoke/linux-smoke.sh' "$FEATURE_DIR/manual_testing_playbook.md" "$FEATURE_DIR/tasks.json"
rg -q 'smoke/macos-smoke.sh' "$FEATURE_DIR/manual_testing_playbook.md" "$FEATURE_DIR/tasks.json"
rg -q 'smoke/windows-smoke.ps1' "$FEATURE_DIR/manual_testing_playbook.md" "$FEATURE_DIR/tasks.json"

if ! command -v "$SUBSTRATE_BIN" >/dev/null 2>&1; then
  echo "adr-0027-identity-tuple-policy-surface: substrate binary not found (SUBSTRATE_BIN=$SUBSTRATE_BIN)" >&2
  exit 3
fi

tmp_root="${SUBSTRATE_SMOKE_ROOT:-}"
if [[ -z "$tmp_root" ]]; then
  tmp_root="$(mktemp -d)"
fi

cleanup() {
  if [[ "${SUBSTRATE_SMOKE_KEEP:-0}" == "1" ]]; then
    return 0
  fi
  rm -rf "$tmp_root"
}
trap cleanup EXIT

export SUBSTRATE_HOME="${SUBSTRATE_HOME:-$tmp_root/substrate-home}"
workspace="$tmp_root/workspace"
mkdir -p "$workspace"
cd "$workspace"

echo "== CLI contract smoke =="
expect_exit 0 "$SUBSTRATE_BIN" workspace init --force
expect_exit 0 "$SUBSTRATE_BIN" config global init --force
expect_exit 0 "$SUBSTRATE_BIN" policy global init --force
expect_exit 0 "$SUBSTRATE_BIN" policy current show --json --explain
expect_exit 2 "$SUBSTRATE_BIN" policy global set --json 'llm.constraints.providers=["OpenAI"]'
expect_exit 2 "$SUBSTRATE_BIN" policy global set --json 'llm.constraints.protocols=["openai"]'

echo "OK: adr-0027-identity-tuple-policy-surface ${PLATFORM_NAME} smoke passed"
