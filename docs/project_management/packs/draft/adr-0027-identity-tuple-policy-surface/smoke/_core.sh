#!/usr/bin/env bash
set -euo pipefail

PLATFORM_NAME="${ITPS_SMOKE_PLATFORM:?ITPS_SMOKE_PLATFORM is required}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FEATURE_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
FEATURE_PATH="docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface"
SUBSTRATE_BIN="${SUBSTRATE_BIN:-substrate}"
CAPTURE_STDOUT=""
CAPTURE_STDERR=""
CAPTURE_RC=0

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

run_capture() {
  local stem="$1"
  shift
  CAPTURE_STDOUT="$tmp_root/${stem}.stdout"
  CAPTURE_STDERR="$tmp_root/${stem}.stderr"
  set +e
  "$@" >"$CAPTURE_STDOUT" 2>"$CAPTURE_STDERR"
  CAPTURE_RC="$?"
  set -e
}

require_capture_rc() {
  local want="$1"
  local label="$2"
  if [[ "$CAPTURE_RC" -ne "$want" ]]; then
    echo "FAIL: expected exit $want for $label, got $CAPTURE_RC" >&2
    echo "--- stdout ($label) ---" >&2
    cat "$CAPTURE_STDOUT" >&2 || true
    echo "--- stderr ($label) ---" >&2
    cat "$CAPTURE_STDERR" >&2 || true
    exit 1
  fi
}

require_contains() {
  local file="$1"
  local needle="$2"
  if ! rg -F -q -- "$needle" "$file"; then
    echo "FAIL: expected '$needle' in $file" >&2
    cat "$file" >&2 || true
    exit 1
  fi
}

write_gateway_inventory_openai() {
  mkdir -p "$SUBSTRATE_HOME/agents"
  cat >"$SUBSTRATE_HOME/agents/openai.yaml" <<'YAML'
version: 1
id: openai
config:
  enabled: true
  kind: api
  api:
    base_url: https://api.openai.com/v1
    auth:
      env:
        - OPENAI_API_KEY
  capabilities:
    llm: true
    mcp_client: false
YAML
}

write_gateway_config_openai() {
  cat >"$SUBSTRATE_HOME/config.yaml" <<'YAML'
llm:
  enabled: true
  gateway:
    enabled: true
  routing:
    default_backend: api:openai
YAML
}

write_gateway_policy_base() {
  cat >"$SUBSTRATE_HOME/policy.yaml" <<'YAML'
id: "gateway-policy"
name: "gateway-policy"

world_fs:
  host_visible: true
  fail_closed:
    routing: false
  write:
    enabled: true

llm:
  allowed_backends:
    - "api:openai"
  secrets:
    env_allowed:
      - "OPENAI_API_KEY"

net_allowed: []
cmd_allowed: []
cmd_denied: []
cmd_isolated: []

require_approval: false
allow_shell_operators: true

limits:
  max_memory_mb: null
  max_cpu_percent: null
  max_runtime_ms: null
  max_egress_bytes: null

metadata: {}
YAML
}

write_gateway_policy_router_mismatch() {
  cat >"$SUBSTRATE_HOME/policy.yaml" <<'YAML'
id: "gateway-policy"
name: "gateway-policy"

world_fs:
  host_visible: true
  fail_closed:
    routing: false
  write:
    enabled: true

llm:
  allowed_backends:
    - "api:openai"
  constraints:
    routers:
      - "direct_provider_path"
  secrets:
    env_allowed:
      - "OPENAI_API_KEY"

net_allowed: []
cmd_allowed: []
cmd_denied: []
cmd_isolated: []

require_approval: false
allow_shell_operators: true

limits:
  max_memory_mb: null
  max_cpu_percent: null
  max_runtime_ms: null
  max_egress_bytes: null

metadata: {}
YAML
}

write_gateway_policy_provider_mismatch() {
  cat >"$SUBSTRATE_HOME/policy.yaml" <<'YAML'
id: "gateway-policy"
name: "gateway-policy"

world_fs:
  host_visible: true
  fail_closed:
    routing: false
  write:
    enabled: true

llm:
  allowed_backends:
    - "api:openai"
  constraints:
    providers:
      - "anthropic"
  secrets:
    env_allowed:
      - "OPENAI_API_KEY"

net_allowed: []
cmd_allowed: []
cmd_denied: []
cmd_isolated: []

require_approval: false
allow_shell_operators: true

limits:
  max_memory_mb: null
  max_cpu_percent: null
  max_runtime_ms: null
  max_egress_bytes: null

metadata: {}
YAML
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
  and all(.[]; .)
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

expect_exit 0 "$SUBSTRATE_BIN" policy global set --json 'llm.constraints.providers=["openai"]'
expect_exit 0 "$SUBSTRATE_BIN" policy global set --json 'llm.constraints.protocols=["openai.responses"]'

run_capture policy_view "$SUBSTRATE_BIN" policy current show --json --explain
require_capture_rc 0 "policy current show --json --explain"
jq -e '
  .llm.constraints.providers == ["openai"]
  and .llm.constraints.protocols == ["openai.responses"]
' "$CAPTURE_STDOUT" >/dev/null
jq -e '
  .kind == "substrate.policy.explain.v1"
  and .keys["llm.constraints.providers"].sources[0].layer == "global_patch"
  and .keys["llm.constraints.protocols"].sources[0].layer == "global_patch"
' "$CAPTURE_STDERR" >/dev/null

expect_exit 2 "$SUBSTRATE_BIN" policy global set --json 'llm.constraints.providers=["OpenAI"]'
expect_exit 2 "$SUBSTRATE_BIN" policy global set --json 'llm.constraints.protocols=["openai"]'

echo "== Gateway contract smoke =="
write_gateway_inventory_openai
write_gateway_config_openai
write_gateway_policy_base

run_capture gateway_status_json env \
  OPENAI_API_KEY=sk-openai-proof \
  SUBSTRATE_WORLD_ENABLED=1 \
  SUBSTRATE_WORLD=enabled \
  "$SUBSTRATE_BIN" world gateway status --json
require_capture_rc 4 "world gateway status --json (unavailable)"
jq -e '
  .status == "unavailable"
  and .identity_tuple.router == "substrate_gateway"
  and .identity_tuple.protocol == "openai.responses"
  and .identity_tuple.provider == "openai"
  and .placement_posture.execution == "in_world"
  and (.client_wiring.identity_tuple? | not)
  and (.client_wiring.placement_posture? | not)
' "$CAPTURE_STDOUT" >/dev/null
test ! -s "$CAPTURE_STDERR"

write_gateway_policy_router_mismatch
run_capture gateway_router_deny env \
  OPENAI_API_KEY=sk-openai-proof \
  SUBSTRATE_WORLD_ENABLED=1 \
  SUBSTRATE_WORLD=enabled \
  "$SUBSTRATE_BIN" world gateway status
require_capture_rc 5 "world gateway status router mismatch"
test ! -s "$CAPTURE_STDOUT"
require_contains "$CAPTURE_STDERR" "substrate world gateway status: policy or safety failure"
require_contains "$CAPTURE_STDERR" "substrate world gateway: gateway_policy_blocked: effective gateway routing authority 'substrate_gateway' is not allowlisted by llm.constraints.routers"

write_gateway_policy_provider_mismatch
run_capture gateway_provider_deny env \
  OPENAI_API_KEY=sk-openai-proof \
  SUBSTRATE_WORLD_ENABLED=1 \
  SUBSTRATE_WORLD=enabled \
  "$SUBSTRATE_BIN" world gateway status
require_capture_rc 5 "world gateway status provider mismatch"
test ! -s "$CAPTURE_STDOUT"
require_contains "$CAPTURE_STDERR" "substrate world gateway status: policy or safety failure"
require_contains "$CAPTURE_STDERR" "substrate world gateway: gateway_policy_blocked: effective gateway provider 'openai' is not allowlisted by llm.constraints.providers"

echo "OK: adr-0027-identity-tuple-policy-surface ${PLATFORM_NAME} smoke passed"
