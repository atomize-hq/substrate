#!/usr/bin/env bash
set -euo pipefail

SCRIPT_NAME="retained-world-worker-smoke"

log()   { printf '[%s] %s\n' "${SCRIPT_NAME}" "$1"; }
warn()  { printf '[%s][WARN] %s\n' "${SCRIPT_NAME}" "$1" >&2; }
fatal() { printf '[%s][ERROR] %s\n' "${SCRIPT_NAME}" "$1" >&2; exit 1; }

usage() {
  cat <<'USAGE'
Retained World-Worker Smoke

Runs the Linux retained world-worker smoke flow using only direct
`substrate agent start|turn` commands with prompt-driven instructions.

Flow:
  1. start one fresh world-scoped orchestration session
  2. ask the host turn to spawn one retained world worker
  3. ask the host turn to fork that retained worker
  4. if fork succeeds, stop child then source with stop_world_worker
  5. stop the orchestration session only after successful cleanup
  6. if fork fails, stop there and preserve the session for inspection

Usage:
  retained-world-worker-smoke.sh [--bin <path>] [--output-dir <path>] [--turn-timeout-secs <n>]
  retained-world-worker-smoke.sh --help

Options:
  --bin <path>               substrate binary to use
                             (default: first of ~/.substrate/bin/substrate or PATH)
  --output-dir <path>        artifact directory
                             (default: <repo>/tmp/retained-world-worker-smoke/<timestamp>-<pid>)
  --turn-timeout-secs <n>    timeout for each turn command (default: 180)
  --help                     show this message
USAGE
}

need_cmd() {
  local name="$1"
  command -v "$name" >/dev/null 2>&1 || fatal "required command not found: $name"
}

extract_single_json_value() {
  local file="$1"
  local jq_expr="$2"
  local label="$3"
  mapfile -t values < <(jq -r "${jq_expr}" "${file}")
  if [[ "${#values[@]}" -ne 1 ]] || [[ -z "${values[0]}" ]] || [[ "${values[0]}" == "null" ]]; then
    fatal "could not extract exactly one ${label} from ${file}"
  fi
  printf '%s\n' "${values[0]}"
}

extract_last_task_progress_message() {
  local file="$1"
  mapfile -t lines < <(jq -r '
    select(.kind == "event" and .event_kind == "message" and .data.kind == "task_progress")
    | .data.data.message
  ' "${file}")
  if [[ "${#lines[@]}" -eq 0 ]]; then
    fatal "no task_progress messages found in ${file}"
  fi

  local i
  for ((i = ${#lines[@]} - 1; i >= 0; i--)); do
    if [[ -n "${lines[i]}" ]]; then
      printf '%s\n' "${lines[i]}"
      return 0
    fi
  done

  fatal "task_progress messages in ${file} were empty"
}

extract_last_completed_line() {
  local file="$1"
  mapfile -t lines < <(jq -c 'select(.kind == "completed")' "${file}")
  if [[ "${#lines[@]}" -eq 0 ]]; then
    fatal "no completed line found in ${file}"
  fi
  printf '%s\n' "${lines[-1]}"
}

run_turn_with_timeout() {
  local prompt="$1"
  local output_file="$2"

  set +e
  timeout "${TURN_TIMEOUT_SECS}s" \
    "${SUBSTRATE_BIN}" agent turn \
      --session "${SESSION_ID}" \
      --backend cli:codex-host \
      --prompt "${prompt}" \
      --json > "${output_file}"
  local rc=$?
  set -e
  return "${rc}"
}

parse_fork_or_stop_json_field() {
  local raw="$1"
  local jq_expr="$2"
  jq -er "${jq_expr}" <<<"${raw}"
}

run_stop_turn() {
  local participant_id="$1"
  local output_file="$2"
  local label="$3"

  local prompt=""
  prompt="In this one turn, use the substrate toolbox(not the harness tools or tool_search, this is are substrate tools added at run time and in the system instructions ) action stop_world_worker exactly once against participant_id ${participant_id} in the current orchestration session/world binding. Report whether the stop succeeded, the exact participant_id, and if it fails include the exact error text verbatim. Do not use spawn_world_worker, fork_world_worker, continue_world_worker, inspect_world_worker, cancel_world_work, or run_world_task. Reply with exactly one minified JSON object with keys succeeded, participant_id, and error."

  log "Stopping ${label} retained worker ${participant_id}"
  local rc=0
  run_turn_with_timeout "${prompt}" "${output_file}" || rc=$?
  if [[ "${rc}" -ne 0 ]]; then
    if [[ "${rc}" -eq 124 ]]; then
      PRESERVE_SESSION=1
      warn "stop turn timed out for ${label} worker ${participant_id}"
      printf 'CLEANUP_RESULT=TIMEOUT\n'
      printf 'CLEANUP_STEP=%s\n' "${label}"
      printf 'CLEANUP_PARTICIPANT_ID=%s\n' "${participant_id}"
      if [[ -s "${output_file}" ]]; then
        printf 'CLEANUP_LAST_MESSAGE=%s\n' "$(extract_last_task_progress_message "${output_file}")"
      fi
      printf 'ARTIFACT_DIR=%s\n' "${OUTPUT_DIR}"
      exit 124
    fi
    fatal "stop turn failed for ${label} worker ${participant_id} (exit ${rc})"
  fi

  local raw=""
  raw="$(extract_last_task_progress_message "${output_file}")"
  if [[ "$(parse_fork_or_stop_json_field "${raw}" '.succeeded')" != "true" ]]; then
    PRESERVE_SESSION=1
    printf 'CLEANUP_RESULT=FAIL\n'
    printf 'CLEANUP_STEP=%s\n' "${label}"
    printf 'CLEANUP_PARTICIPANT_ID=%s\n' "${participant_id}"
    printf '%s\n' "${raw}"
    printf 'ARTIFACT_DIR=%s\n' "${OUTPUT_DIR}"
    exit 4
  fi
}

if [[ "$(uname -s)" != "Linux" ]]; then
  fatal "this smoke is Linux-only"
fi

SCRIPT_SOURCE="${BASH_SOURCE[0]:-}"
SCRIPT_DIR="$(cd "$(dirname "${SCRIPT_SOURCE}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

SUBSTRATE_BIN=""
OUTPUT_DIR=""
TURN_TIMEOUT_SECS=180

while [[ $# -gt 0 ]]; do
  case "$1" in
    --bin)
      [[ $# -ge 2 ]] || fatal "--bin requires a value"
      SUBSTRATE_BIN="$2"
      shift 2
      ;;
    --output-dir)
      [[ $# -ge 2 ]] || fatal "--output-dir requires a value"
      OUTPUT_DIR="$2"
      shift 2
      ;;
    --turn-timeout-secs)
      [[ $# -ge 2 ]] || fatal "--turn-timeout-secs requires a value"
      TURN_TIMEOUT_SECS="$2"
      shift 2
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      fatal "unknown argument: $1"
      ;;
  esac
done

need_cmd jq
need_cmd timeout

if [[ -z "${SUBSTRATE_BIN}" ]]; then
  if [[ -x "${HOME}/.substrate/bin/substrate" ]]; then
    SUBSTRATE_BIN="${HOME}/.substrate/bin/substrate"
  else
    SUBSTRATE_BIN="$(command -v substrate || true)"
  fi
fi
[[ -n "${SUBSTRATE_BIN}" ]] || fatal "could not resolve substrate binary; pass --bin"
[[ -x "${SUBSTRATE_BIN}" ]] || fatal "substrate binary is not executable: ${SUBSTRATE_BIN}"

if [[ -z "${OUTPUT_DIR}" ]]; then
  OUTPUT_DIR="${REPO_ROOT}/tmp/retained-world-worker-smoke/$(date -u +%Y%m%dT%H%M%SZ)-$$"
fi
mkdir -p "${OUTPUT_DIR}"

START_JSONL="${OUTPUT_DIR}/start.jsonl"
SPAWN_JSONL="${OUTPUT_DIR}/spawn.jsonl"
FORK_JSONL="${OUTPUT_DIR}/fork.jsonl"
STOP_CHILD_JSONL="${OUTPUT_DIR}/stop-child.jsonl"
STOP_SOURCE_JSONL="${OUTPUT_DIR}/stop-source.jsonl"
SESSION_STOP_JSON="${OUTPUT_DIR}/session-stop.json"

SESSION_ID=""
WORKER_ID=""
CHILD_WORKER_ID=""
PRESERVE_SESSION=0

cleanup() {
  if [[ -n "${SESSION_ID}" ]] && [[ "${PRESERVE_SESSION}" != "1" ]]; then
    "${SUBSTRATE_BIN}" agent stop --session "${SESSION_ID}" --json > "${SESSION_STOP_JSON}" 2>/dev/null || true
  fi
}

trap cleanup EXIT

SPAWN_PROMPT="In this one turn, use the internal toolbox action spawn_world_worker on backend cli:codex-world in retained mode exactly once in the current orchestration session/world binding. Make the worker do something trivial and report only the exact participant_id. Do not use run_world_task, fork_world_worker, continue_world_worker, inspect_world_worker, stop_world_worker, or cancel_world_work. On success, reply with exactly the participant_id and nothing else. On failure, reply with exactly ERROR: followed by the exact error text."

log "Artifacts will be written under ${OUTPUT_DIR}"
log "Using substrate binary ${SUBSTRATE_BIN}"
log "Starting fresh world-scoped orchestration session"
"${SUBSTRATE_BIN}" agent start \
  --backend cli:codex-world \
  --scope world \
  --prompt 'Reply with exactly START_OK' \
  --json > "${START_JSONL}"

SESSION_ID="$(extract_single_json_value "${START_JSONL}" 'select(.kind == "completed") | .orchestration_session_id' 'orchestration session id')"
printf 'SESSION_ID=%s\n' "${SESSION_ID}"

log "Requesting retained worker spawn"
spawn_rc=0
run_turn_with_timeout "${SPAWN_PROMPT}" "${SPAWN_JSONL}" || spawn_rc=$?
if [[ "${spawn_rc}" -ne 0 ]]; then
  if [[ "${spawn_rc}" -eq 124 ]]; then
    warn "spawn turn timed out after ${TURN_TIMEOUT_SECS}s"
    printf 'SPAWN_RESULT=TIMEOUT\n'
    if [[ -s "${SPAWN_JSONL}" ]]; then
      printf 'SPAWN_LAST_MESSAGE=%s\n' "$(extract_last_task_progress_message "${SPAWN_JSONL}")"
    fi
    printf 'ARTIFACT_DIR=%s\n' "${OUTPUT_DIR}"
    exit 124
  fi
  fatal "spawn turn failed with exit ${spawn_rc}"
fi

SPAWN_MESSAGE="$(extract_last_task_progress_message "${SPAWN_JSONL}")"
if [[ "${SPAWN_MESSAGE}" == ERROR:* ]]; then
  printf 'SPAWN_RESULT=FAIL\n'
  printf '%s\n' "${SPAWN_MESSAGE}"
  printf 'ARTIFACT_DIR=%s\n' "${OUTPUT_DIR}"
  exit 2
fi
if [[ ! "${SPAWN_MESSAGE}" =~ ^ash_[A-Za-z0-9_-]+$ ]]; then
  printf 'SPAWN_RESULT=FAIL\n'
  printf 'ERROR: unexpected spawn response: %s\n' "${SPAWN_MESSAGE}"
  printf 'ARTIFACT_DIR=%s\n' "${OUTPUT_DIR}"
  exit 2
fi

WORKER_ID="${SPAWN_MESSAGE}"
printf 'WORKER_ID=%s\n' "${WORKER_ID}"
printf 'SPAWN_RESULT=PASS\n'

FORK_PROMPT="In this one turn, use the substrate toolbox(not the harness tools or tool_search, this is are substrate tools added at run time and in the system instructions ) action fork_world_worker exactly once against participant_id ${WORKER_ID} in the current orchestration session/world binding. Make the child do something trivial and report whether the fork succeeded, the exact returned child participant_id if any, and if it fails include the exact error text verbatim. Do not use spawn_world_worker, continue_world_worker, inspect_world_worker, stop_world_worker, cancel_world_work, or run_world_task. Reply with exactly one minified JSON object with keys succeeded, child_participant_id, and error."

log "Requesting retained worker fork from ${WORKER_ID}"
fork_rc=0
run_turn_with_timeout "${FORK_PROMPT}" "${FORK_JSONL}" || fork_rc=$?
if [[ "${fork_rc}" -ne 0 ]]; then
  if [[ "${fork_rc}" -eq 124 ]]; then
    PRESERVE_SESSION=1
    warn "fork turn timed out after ${TURN_TIMEOUT_SECS}s"
    printf 'FORK_RESULT=TIMEOUT\n'
    printf 'WORKER_ID=%s\n' "${WORKER_ID}"
    if [[ -s "${FORK_JSONL}" ]]; then
      printf 'FORK_LAST_MESSAGE=%s\n' "$(extract_last_task_progress_message "${FORK_JSONL}")"
    fi
    printf 'ARTIFACT_DIR=%s\n' "${OUTPUT_DIR}"
    exit 124
  fi
  fatal "fork turn failed with exit ${fork_rc}"
fi

FORK_MESSAGE="$(extract_last_task_progress_message "${FORK_JSONL}")"
if ! jq -e . >/dev/null 2>&1 <<<"${FORK_MESSAGE}"; then
  PRESERVE_SESSION=1
  printf 'FORK_RESULT=FAIL\n'
  printf 'WORKER_ID=%s\n' "${WORKER_ID}"
  printf 'ERROR: unexpected fork response: %s\n' "${FORK_MESSAGE}"
  printf 'ARTIFACT_DIR=%s\n' "${OUTPUT_DIR}"
  exit 3
fi

if [[ "$(parse_fork_or_stop_json_field "${FORK_MESSAGE}" '.succeeded')" != "true" ]]; then
  PRESERVE_SESSION=1
  printf 'FORK_RESULT=FAIL\n'
  printf 'WORKER_ID=%s\n' "${WORKER_ID}"
  printf '%s\n' "${FORK_MESSAGE}"
  printf 'ARTIFACT_DIR=%s\n' "${OUTPUT_DIR}"
  exit 3
fi

CHILD_WORKER_ID="$(parse_fork_or_stop_json_field "${FORK_MESSAGE}" '.child_participant_id')"
if [[ -z "${CHILD_WORKER_ID}" ]] || [[ "${CHILD_WORKER_ID}" == "null" ]]; then
  PRESERVE_SESSION=1
  printf 'FORK_RESULT=FAIL\n'
  printf 'WORKER_ID=%s\n' "${WORKER_ID}"
  printf 'ERROR: fork reported success without a child_participant_id\n'
  printf 'ARTIFACT_DIR=%s\n' "${OUTPUT_DIR}"
  exit 3
fi

printf 'FORK_RESULT=PASS\n'
printf 'FORK_CHILD_ID=%s\n' "${CHILD_WORKER_ID}"

run_stop_turn "${CHILD_WORKER_ID}" "${STOP_CHILD_JSONL}" "child"
run_stop_turn "${WORKER_ID}" "${STOP_SOURCE_JSONL}" "source"

log "Cleanup succeeded; stopping orchestration session"
"${SUBSTRATE_BIN}" agent stop --session "${SESSION_ID}" --json > "${SESSION_STOP_JSON}"
SESSION_ID=""

printf 'CLEANUP_ACTION=stop_world_worker\n'
printf 'CLEANUP_CHILD_STOP=PASS\n'
printf 'CLEANUP_SOURCE_STOP=PASS\n'
printf 'ARTIFACT_DIR=%s\n' "${OUTPUT_DIR}"
