#!/usr/bin/env bash
set -euo pipefail

SCRIPT_NAME="dev-fresh-install-gateway-smoke"

log()   { printf '[%s] %s\n' "${SCRIPT_NAME}" "$1"; }
warn()  { printf '[%s][WARN] %s\n' "${SCRIPT_NAME}" "$1" >&2; }
fatal() { printf '[%s][ERROR] %s\n' "${SCRIPT_NAME}" "$1" >&2; exit 1; }

usage() {
  cat <<'USAGE'
Substrate Fresh-Install Gateway Smoke Helper

Automates the common post-install configuration used to manually smoke-test the
Linux Codex host-orchestrator plus world-agent dispatch path on a fresh install.

Usage:
  dev-fresh-install-gateway-smoke.sh [--prefix <path>] [--bin <path>] [--repo-root <path>] [--agent-manifest <path>] [--world-agent-manifest <path>] [--skip-sync]
  dev-fresh-install-gateway-smoke.sh --help

Options:
  --prefix <path>           Installed Substrate home (default: ~/.substrate)
  --bin <path>              Explicit substrate binary path (default: <prefix>/bin/substrate)
  --repo-root <path>        Repo root used to locate config/agents/codex.yaml
  --agent-manifest <path>   Explicit host codex agent manifest source
  --world-agent-manifest <path>
                            Explicit world codex_world agent manifest source
  --skip-sync               Stop after gateway status instead of running gateway sync
  --help                    Show this message

This helper assumes the fresh install is using the default Codex host/world
smoke path:
  - llm.routing.default_backend = cli:codex
  - codex is the orchestrator agent
  - config/agents/codex.yaml is copied into <prefix>/agents/codex.yaml
  - config/agents/codex_world.yaml is copied into <prefix>/agents/codex_world.yaml
  - agents.toolbox is enabled with UDS transport
  - agents.world_dispatch is enabled for exact backend cli:codex_world
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
  AGENT_MANIFEST="${REPO_ROOT}/config/agents/codex.yaml"
fi
if [[ -z "${WORLD_AGENT_MANIFEST}" ]]; then
  WORLD_AGENT_MANIFEST="${REPO_ROOT}/config/agents/codex_world.yaml"
fi

[[ -x "${SUBSTRATE_BIN}" ]] || fatal "substrate binary not found or not executable at ${SUBSTRATE_BIN}"
[[ -f "${AGENT_MANIFEST}" ]] || fatal "agent manifest not found at ${AGENT_MANIFEST}"
[[ -f "${WORLD_AGENT_MANIFEST}" ]] || fatal "world agent manifest not found at ${WORLD_AGENT_MANIFEST}"

run_substrate() {
  log "Running: ${SUBSTRATE_BIN} $*"
  "${SUBSTRATE_BIN}" "$@"
}

agents_dir="${PREFIX}/agents"
mkdir -p "${agents_dir}"
cp "${AGENT_MANIFEST}" "${agents_dir}/codex.yaml"
cp "${WORLD_AGENT_MANIFEST}" "${agents_dir}/codex_world.yaml"
log "Copied codex agent manifest into ${agents_dir}/codex.yaml"
log "Copied codex world agent manifest into ${agents_dir}/codex_world.yaml"

run_substrate config global set llm.routing.default_backend=cli:codex
run_substrate config global set agents.enabled=true
run_substrate config global set agents.hub.orchestrator_agent_id=codex
run_substrate config global set agents.toolbox.enabled=true
run_substrate config global set agents.toolbox.bind.transport=uds
run_substrate config global set llm.enabled=true
run_substrate config global set llm.gateway.enabled=true
run_substrate policy global set 'llm.allowed_backends=["cli:codex"]'
run_substrate policy global set 'agents.allowed_backends=["cli:codex","cli:codex_world"]'
run_substrate policy global set 'agents.host_credentials.read.allowed_backends=["cli:codex"]'
run_substrate policy global set 'agents.world_dispatch.enabled=true'
run_substrate policy global set 'agents.world_dispatch.allowed_backends=["cli:codex_world"]'
run_substrate policy global set 'agents.world_dispatch.allowed_actions=["run_world_task","spawn_world_worker","fork_world_worker","continue_world_worker","inspect_world_worker","cancel_world_work","stop_world_worker"]'
run_substrate policy global set 'agents.world_dispatch.allowed_modes=["ephemeral","retained"]'
run_substrate policy global set 'agents.world_dispatch.same_session_only=true'
run_substrate policy global set 'agents.world_dispatch.same_world_binding_only=true'
run_substrate policy global set 'agents.world_dispatch.allow_capability_narrowing=false'
run_substrate policy global set 'agents.world_dispatch.max_live_retained_workers=8'
run_substrate policy global set 'agents.world_dispatch.max_concurrent_ephemeral=8'

log "Configured fresh install for Codex host-orchestrator plus world-dispatch smoke."
run_substrate world gateway status

if [[ "${RUN_SYNC}" -eq 1 ]]; then
  run_substrate world gateway sync
else
  warn "Skipping 'substrate world gateway sync' because --skip-sync was requested."
fi

run_substrate agent doctor --json
run_substrate agent toolbox status --json

log "Fresh-install gateway smoke setup complete."
