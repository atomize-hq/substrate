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
Linux world gateway / agent startup path on a fresh install.

Usage:
  dev-fresh-install-gateway-smoke.sh [--prefix <path>] [--bin <path>] [--repo-root <path>] [--agent-manifest <path>] [--skip-sync]
  dev-fresh-install-gateway-smoke.sh --help

Options:
  --prefix <path>           Installed Substrate home (default: ~/.substrate)
  --bin <path>              Explicit substrate binary path (default: <prefix>/bin/substrate)
  --repo-root <path>        Repo root used to locate config/agents/codex.yaml
  --agent-manifest <path>   Explicit codex agent manifest source
  --skip-sync               Stop after gateway status instead of running gateway sync
  --help                    Show this message

This helper assumes the fresh install is using the default Codex gateway smoke
path:
  - llm.routing.default_backend = cli:codex
  - codex is the orchestrator agent
  - config/agents/codex.yaml is copied into <prefix>/agents/
USAGE
}

SCRIPT_SOURCE="${BASH_SOURCE[0]:-}"
SCRIPT_DIR="$(cd "$(dirname "${SCRIPT_SOURCE}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
PREFIX="${HOME}/.substrate"
SUBSTRATE_BIN=""
AGENT_MANIFEST=""
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

[[ -x "${SUBSTRATE_BIN}" ]] || fatal "substrate binary not found or not executable at ${SUBSTRATE_BIN}"
[[ -f "${AGENT_MANIFEST}" ]] || fatal "agent manifest not found at ${AGENT_MANIFEST}"

run_substrate() {
  log "Running: ${SUBSTRATE_BIN} $*"
  "${SUBSTRATE_BIN}" "$@"
}

agents_dir="${PREFIX}/agents"
mkdir -p "${agents_dir}"
cp "${AGENT_MANIFEST}" "${agents_dir}/codex.yaml"
log "Copied codex agent manifest into ${agents_dir}/codex.yaml"

run_substrate config global set llm.routing.default_backend=cli:codex
run_substrate policy global set 'agents.allowed_backends=["cli:codex"]'
run_substrate config global set agents.enabled=true
run_substrate config global set agents.hub.orchestrator_agent_id=codex
run_substrate config global set llm.enabled=true
run_substrate config global set llm.gateway.enabled=true
run_substrate policy global set 'llm.allowed_backends=["cli:codex"]'
run_substrate policy global set 'agents.host_credentials.read.allowed_backends=["cli:codex"]'

log "Configured fresh install for Codex world-gateway smoke."
run_substrate world gateway status

if [[ "${RUN_SYNC}" -eq 1 ]]; then
  run_substrate world gateway sync
else
  warn "Skipping 'substrate world gateway sync' because --skip-sync was requested."
fi

log "Fresh-install gateway smoke setup complete."
