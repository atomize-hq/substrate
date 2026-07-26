#!/usr/bin/env bash
set -euo pipefail

SCRIPT_NAME="dev-fresh-install-gateway-smoke-modes"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

log() {
  printf '[%s] %s\n' "${SCRIPT_NAME}" "$*" >&2
}

fatal() {
  log "ERROR: $*"
  exit 1
}

assert_mode() {
  local path="$1"
  local expected="$2"
  local actual
  actual="$(stat -c '%a' "${path}")"
  [[ "${actual}" == "${expected}" ]] || fatal "expected ${path} mode ${expected}, got ${actual}"
}

write_stub_substrate() {
  local path="$1"
  cat >"${path}" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail

if [[ "${1:-}" == "host" && "${2:-}" == "doctor" && "${3:-}" == "--json" ]]; then
  printf '%s\n' '{"host":{"world_socket":{"probe_ok":true,"access":{"status":"ok.active_group","authorization_source":"active-group","remediation":""}}}}'
  exit 0
fi

if [[ "${1:-}" == "--world" ]]; then
  exit 0
fi

exit 0
EOF
  chmod +x "${path}"
}

run_helper_case() {
  local helper_path="$1"
  local agent_manifest="$2"
  local copied_name="$3"
  local prefix_root="$4"
  local stub="$5"

  "${helper_path}" \
    --prefix "${prefix_root}" \
    --bin "${stub}" \
    --repo-root "${REPO_ROOT}" \
    --agent-manifest "${agent_manifest}" \
    --skip-sync >/dev/null

  assert_mode "${prefix_root}/agents" 700
  assert_mode "${prefix_root}/agents/${copied_name}" 600
}

WORK_ROOT="$(mktemp -d "/tmp/substrate-fresh-gateway-modes.XXXXXX")"
cleanup() {
  rm -rf "${WORK_ROOT}"
}
trap cleanup EXIT

STUB="${WORK_ROOT}/substrate-stub"
write_stub_substrate "${STUB}"

run_helper_case \
  "${REPO_ROOT}/scripts/substrate/dev-fresh-install-gateway-smoke.sh" \
  "${REPO_ROOT}/config/agents/codex.yaml" \
  "codex.yaml" \
  "${WORK_ROOT}/codex-prefix" \
  "${STUB}"

run_helper_case \
  "${REPO_ROOT}/scripts/substrate/dev-fresh-install-gateway-smoke-claude-code.sh" \
  "${REPO_ROOT}/config/agents/claude_code.yaml" \
  "claude_code.yaml" \
  "${WORK_ROOT}/claude-prefix" \
  "${STUB}"

log "Verified fresh-install gateway smoke helpers stage trusted agent inventory with 0700 directories and 0600 files."
