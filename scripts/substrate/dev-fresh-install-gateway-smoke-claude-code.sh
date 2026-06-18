#!/usr/bin/env bash
set -euo pipefail

SCRIPT_NAME="dev-fresh-install-gateway-smoke-claude-code"

log()   { printf '[%s] %s\n' "${SCRIPT_NAME}" "$1"; }
warn()  { printf '[%s][WARN] %s\n' "${SCRIPT_NAME}" "$1" >&2; }
fatal() { printf '[%s][ERROR] %s\n' "${SCRIPT_NAME}" "$1" >&2; exit 1; }

usage() {
  cat <<'USAGE'
Substrate Fresh-Install Gateway Smoke Helper

Automates the common post-install configuration used to manually smoke-test the
Linux Claude Code host-orchestrator plus world-agent dispatch path on a fresh install.

Usage:
  dev-fresh-install-gateway-smoke-claude-code.sh [--prefix <path>] [--bin <path>] [--repo-root <path>] [--agent-manifest <path>] [--world-agent-manifest <path>] [--skip-sync]
  dev-fresh-install-gateway-smoke-claude-code.sh --help

Options:
  --prefix <path>           Installed Substrate home (default: ~/.substrate)
  --bin <path>              Explicit substrate binary path (default: <prefix>/bin/substrate)
  --repo-root <path>        Repo root used to locate config/agents/claude_code.yaml
  --agent-manifest <path>   Explicit placement-aware claude_code agent manifest source
  --world-agent-manifest <path>
                            Deprecated split world manifest source; must match --agent-manifest when provided
  --skip-sync               Stop after gateway status instead of running gateway sync
  --help                    Show this message

This helper assumes the fresh install is using the default Claude Code host/world
smoke path:
  - llm.routing.default_backend = cli:claude_code-host
  - claude_code-host is the orchestrator agent
  - config/agents/claude_code.yaml is copied into <prefix>/agents/claude_code.yaml
  - agents.toolbox is enabled with UDS transport
  - agents.world_dispatch is enabled for exact backend cli:claude_code-world
USAGE
}

SCRIPT_SOURCE="${BASH_SOURCE[0]:-}"
SCRIPT_DIR="$(cd "$(dirname "${SCRIPT_SOURCE}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
PREFIX="${HOME}/.substrate"
SUBSTRATE_BIN=""
AGENT_MANIFEST=""
WORLD_AGENT_MANIFEST=""
RUN_SYNC=1

while [[ $# -gt 0 ]]; do
  case "$1" in
    --prefix)
      [[ $# -ge 2 ]] || fatal "--prefix requires a value"
      PREFIX="$2"
      shift 2
      ;;
    --bin)
      [[ $# -ge 2 ]] || fatal "--bin requires a value"
      SUBSTRATE_BIN="$2"
      shift 2
      ;;
    --repo-root)
      [[ $# -ge 2 ]] || fatal "--repo-root requires a value"
      REPO_ROOT="$2"
      shift 2
      ;;
    --agent-manifest)
      [[ $# -ge 2 ]] || fatal "--agent-manifest requires a value"
      AGENT_MANIFEST="$2"
      shift 2
      ;;
    --world-agent-manifest)
      [[ $# -ge 2 ]] || fatal "--world-agent-manifest requires a value"
      WORLD_AGENT_MANIFEST="$2"
      shift 2
      ;;
    --skip-sync)
      RUN_SYNC=0
      shift
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      fatal "Unknown argument: $1"
      ;;
  esac
done

PREFIX="${PREFIX%/}"
if [[ -z "${SUBSTRATE_BIN}" ]]; then
  SUBSTRATE_BIN="${PREFIX}/bin/substrate"
fi
if [[ -z "${AGENT_MANIFEST}" ]]; then
  AGENT_MANIFEST="${REPO_ROOT}/config/agents/claude_code.yaml"
fi
if [[ -z "${WORLD_AGENT_MANIFEST}" ]]; then
  WORLD_AGENT_MANIFEST="${AGENT_MANIFEST}"
fi

[[ -x "${SUBSTRATE_BIN}" ]] || fatal "substrate binary not found or not executable at ${SUBSTRATE_BIN}"
[[ -f "${AGENT_MANIFEST}" ]] || fatal "agent manifest not found at ${AGENT_MANIFEST}"
[[ -f "${WORLD_AGENT_MANIFEST}" ]] || fatal "world agent manifest not found at ${WORLD_AGENT_MANIFEST}"
if [[ "${WORLD_AGENT_MANIFEST}" != "${AGENT_MANIFEST}" ]]; then
  fatal "split world agent manifests are no longer supported; point --world-agent-manifest at the same placement-aware claude_code manifest as --agent-manifest"
fi

run_substrate() {
  log "Running: ${SUBSTRATE_BIN} $*"
  "${SUBSTRATE_BIN}" "$@"
}

agents_dir="${PREFIX}/agents"
mkdir -p "${agents_dir}"
cp "${AGENT_MANIFEST}" "${agents_dir}/claude_code.yaml"
log "Copied claude_code agent manifest into ${agents_dir}/claude_code.yaml"

run_substrate config global set llm.routing.default_backend=cli:claude_code-host
run_substrate config global set agents.enabled=true
run_substrate config global set agents.hub.orchestrator_agent_id=claude_code-host
run_substrate config global set agents.toolbox.enabled=true
run_substrate config global set agents.toolbox.bind.transport=uds
run_substrate config global set llm.enabled=true
run_substrate config global set llm.gateway.enabled=true
run_substrate policy global set 'llm.allowed_backends=["cli:claude_code-host"]'
run_substrate policy global set 'llm.secrets.env_allowed=["ANTHROPIC_API_KEY"]'
run_substrate policy global set 'agents.allowed_backends=["cli:claude_code-host","cli:claude_code-world"]'
run_substrate policy global set 'agents.host_credentials.read.allowed_backends=["cli:claude_code-host"]'
run_substrate policy global set 'agents.world_dispatch.enabled=true'
run_substrate policy global set 'agents.world_dispatch.allowed_backends=["cli:claude_code-world"]'
run_substrate policy global set 'agents.world_dispatch.allowed_actions=["run_world_task","spawn_world_worker","fork_world_worker","continue_world_worker","inspect_world_worker","cancel_world_work","stop_world_worker"]'
run_substrate policy global set 'agents.world_dispatch.allowed_modes=["ephemeral","retained"]'
run_substrate policy global set 'agents.world_dispatch.same_session_only=true'
run_substrate policy global set 'agents.world_dispatch.same_world_binding_only=true'
run_substrate policy global set 'agents.world_dispatch.allow_capability_narrowing=false'
run_substrate policy global set 'agents.world_dispatch.max_live_retained_workers=8'
run_substrate policy global set 'agents.world_dispatch.max_concurrent_ephemeral=8'

log "Configured fresh install for Claude Code host-orchestrator plus world-dispatch smoke."
run_substrate world gateway status

if [[ "${RUN_SYNC}" -eq 1 ]]; then
  run_substrate world gateway sync
else
  warn "Skipping 'substrate world gateway sync' because --skip-sync was requested."
fi

run_substrate agent doctor --json
run_substrate agent toolbox status --json

log "Fresh-install gateway smoke setup complete."
