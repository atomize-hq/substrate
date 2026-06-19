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
  --agent-manifest <path>   Explicit placement-aware codex agent manifest source
  --world-agent-manifest <path>
                            Deprecated split world manifest source; must match --agent-manifest when provided
  --skip-sync               Stop after gateway status instead of running gateway sync
  --help                    Show this message

This helper assumes the fresh install is using the default Codex host/world
smoke path:
  - llm.routing.default_backend = cli:codex-host
  - codex-host is the orchestrator agent
  - config/agents/codex.yaml is copied into <prefix>/agents/codex.yaml
  - agents.toolbox is enabled with UDS transport
  - agents.world_dispatch is enabled for exact backend cli:codex-world
  - the Codex guest runtime is already provisioned at /var/lib/substrate/world-deps/bin/codex
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
  WORLD_AGENT_MANIFEST="${AGENT_MANIFEST}"
fi

[[ -x "${SUBSTRATE_BIN}" ]] || fatal "substrate binary not found or not executable at ${SUBSTRATE_BIN}"
[[ -f "${AGENT_MANIFEST}" ]] || fatal "agent manifest not found at ${AGENT_MANIFEST}"
[[ -f "${WORLD_AGENT_MANIFEST}" ]] || fatal "world agent manifest not found at ${WORLD_AGENT_MANIFEST}"
if [[ "${WORLD_AGENT_MANIFEST}" != "${AGENT_MANIFEST}" ]]; then
  fatal "split world agent manifests are no longer supported; point --world-agent-manifest at the same placement-aware codex manifest as --agent-manifest"
fi

run_substrate() {
  log "Running: ${SUBSTRATE_BIN} $*"
  "${SUBSTRATE_BIN}" "$@"
}

capture_substrate() {
  printf '[%s] Running: %s %s\n' "${SCRIPT_NAME}" "${SUBSTRATE_BIN}" "$*" >&2
  "${SUBSTRATE_BIN}" "$@"
}

preflight_world_socket_access() {
  local host_doctor_json=""
  local host_doctor_status=0
  local probe_ok=""
  local access_status=""
  local access_source=""
  local remediation=""
  local parsed_access=""

  set +e
  host_doctor_json="$(capture_substrate host doctor --json)"
  host_doctor_status=$?
  set -e

  [[ -n "${host_doctor_json}" ]] || fatal "World socket access preflight could not read 'substrate host doctor --json'."

  set +e
  parsed_access="$(python3 - "${host_doctor_json}" <<'PY'
import json
import sys

payload = json.loads(sys.argv[1])
socket = payload.get("host", {}).get("world_socket", {})
access = socket.get("access", {})

print(
    "{}\t{}\t{}\t{}".format(
        "true" if socket.get("probe_ok") else "false",
        access.get("status", "unknown"),
        access.get("authorization_source", "unknown"),
        access.get("remediation", ""),
    )
)
PY
)"
  local parsed_status=$?
  set -e

  [[ ${parsed_status} -eq 0 ]] || fatal "World socket access preflight could not parse 'substrate host doctor --json'."
  read -r probe_ok access_status access_source remediation <<< "${parsed_access}"

  if [[ ${host_doctor_status} -ne 0 && "${probe_ok}" != "true" ]]; then
    fatal "World socket access preflight failed (status=${access_status}, authorization_source=${access_source}). ${remediation}"
  fi

  if [[ "${probe_ok}" != "true" ]]; then
    fatal "World socket access preflight failed (status=${access_status}, authorization_source=${access_source}). ${remediation}"
  fi

  log "World socket access preflight passed via ${access_source} (status=${access_status})."
}

preflight_codex_world_runtime() {
  local helper_path="${PREFIX}/scripts/substrate/world-enable.sh"
  local remediation=""

  if [[ -x "${helper_path}" ]]; then
    remediation="Run '${helper_path} --home ${PREFIX} --provision-agent-runtime codex' and rerun this helper."
  else
    remediation="Provision the world runtime with the installed world-enable helper under '${PREFIX}/scripts/substrate/world-enable.sh --home ${PREFIX} --provision-agent-runtime codex', or rerun the dev install with '--provision-agent-runtime codex'."
  fi

  set +e
  capture_substrate --world -c 'test -x /var/lib/substrate/world-deps/bin/codex' >/dev/null
  local runtime_status=$?
  set -e

  if [[ ${runtime_status} -ne 0 ]]; then
    fatal "Codex world runtime preflight failed: guest entrypoint '/var/lib/substrate/world-deps/bin/codex' is unavailable. ${remediation}"
  fi

  log "Codex world runtime preflight passed at /var/lib/substrate/world-deps/bin/codex."
}

agents_dir="${PREFIX}/agents"
mkdir -p "${agents_dir}"
cp "${AGENT_MANIFEST}" "${agents_dir}/codex.yaml"
log "Copied codex agent manifest into ${agents_dir}/codex.yaml"

run_substrate config global set llm.routing.default_backend=cli:codex-host
run_substrate config global set agents.enabled=true
run_substrate config global set agents.hub.orchestrator_agent_id=codex-host
run_substrate config global set agents.toolbox.enabled=true
run_substrate config global set agents.toolbox.bind.transport=uds
run_substrate config global set llm.enabled=true
run_substrate config global set llm.gateway.enabled=true
run_substrate policy global set 'llm.allowed_backends=["cli:codex-host"]'
run_substrate policy global set 'agents.allowed_backends=["cli:codex-host","cli:codex-world"]'
run_substrate policy global set 'agents.host_credentials.read.allowed_backends=["cli:codex-host"]'
run_substrate policy global set 'agents.world_dispatch.enabled=true'
run_substrate policy global set 'agents.world_dispatch.allowed_backends=["cli:codex-world"]'
run_substrate policy global set 'agents.world_dispatch.allowed_actions=["run_world_task","spawn_world_worker","fork_world_worker","continue_world_worker","inspect_world_worker","cancel_world_work","stop_world_worker"]'
run_substrate policy global set 'agents.world_dispatch.allowed_modes=["ephemeral","retained"]'
run_substrate policy global set 'agents.world_dispatch.same_session_only=true'
run_substrate policy global set 'agents.world_dispatch.same_world_binding_only=true'
run_substrate policy global set 'agents.world_dispatch.allow_capability_narrowing=false'
run_substrate policy global set 'agents.world_dispatch.max_live_retained_workers=8'
run_substrate policy global set 'agents.world_dispatch.max_concurrent_ephemeral=8'

log "Configured fresh install for Codex host-orchestrator plus world-dispatch smoke."
preflight_world_socket_access
preflight_codex_world_runtime
run_substrate world gateway status

if [[ "${RUN_SYNC}" -eq 1 ]]; then
  run_substrate world gateway sync
else
  warn "Skipping 'substrate world gateway sync' because --skip-sync was requested."
fi

run_substrate agent doctor --json
run_substrate agent toolbox status --json

log "Fresh-install gateway smoke setup complete."
