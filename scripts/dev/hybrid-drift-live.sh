#!/usr/bin/env bash
set -euo pipefail

SCRIPT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_ROOT}/../.." && pwd)"

SESSION_ID=""
CODEX_HOME_OVERRIDE=""
FRESH_STATE=0
SKIP_STATIC=0
NO_GROWTH_CHECK=0

usage() {
  cat <<'EOF'
Usage:
  scripts/dev/hybrid-drift-live.sh [options] <session-id>

Options:
  --codex-home PATH   Use a specific CODEX_HOME instead of $HOME/.codex.
  --fresh-state       Remove target/hybrid-drift-live/<session-id> before starting live mode.
  --skip-static       Skip the one-shot compactor/analyzer/replay preflight.
  --no-growth-check   Skip the short rollout size probe before starting.
  -h, --help          Show this help.

Behavior:
  1. Resolve the session rollout artifact under CODEX_HOME.
  2. Derive static and live output directories from the session id.
  3. Optionally run compactor, analyzer, and sentinel replay once.
  4. Exec agent-drift-sentinel --mode live in the foreground.
EOF
}

log() {
  printf '[hybrid-drift-live] %s\n' "$*"
}

die() {
  printf '[hybrid-drift-live] ERROR: %s\n' "$*" >&2
  exit 1
}

require_cmd() {
  local cmd="$1"
  if ! command -v "$cmd" >/dev/null 2>&1; then
    die "$cmd not found on PATH"
  fi
}

stat_size() {
  local path="$1"
  if stat -f '%z' "$path" >/dev/null 2>&1; then
    stat -f '%z' "$path"
    return
  fi
  if stat -c '%s' "$path" >/dev/null 2>&1; then
    stat -c '%s' "$path"
    return
  fi
  die "unable to determine file size for $path"
}

resolve_rollout_path() {
  local codex_home="$1"
  local session_id="$2"
  local matches=()

  while IFS= read -r line; do
    matches+=("$line")
  done < <(find "$codex_home/sessions" -name "rollout-*${session_id}*.jsonl" -type f | sort)

  if [[ "${#matches[@]}" -eq 0 ]]; then
    die "no rollout-*.jsonl artifact found for session ${session_id} under ${codex_home}"
  fi
  if [[ "${#matches[@]}" -gt 1 ]]; then
    printf '[hybrid-drift-live] ERROR: multiple rollout artifacts matched session %s:\n' "$session_id" >&2
    printf '  %s\n' "${matches[@]}" >&2
    exit 1
  fi

  printf '%s\n' "${matches[0]}"
}

run_repo_cmd() {
  log "$*"
  (
    cd "$REPO_ROOT"
    "$@"
  )
}

while [[ "$#" -gt 0 ]]; do
  case "$1" in
    --codex-home)
      [[ "$#" -ge 2 ]] || die "--codex-home requires a path"
      CODEX_HOME_OVERRIDE="$2"
      shift 2
      ;;
    --fresh-state)
      FRESH_STATE=1
      shift
      ;;
    --skip-static)
      SKIP_STATIC=1
      shift
      ;;
    --no-growth-check)
      NO_GROWTH_CHECK=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    -*)
      die "unknown option: $1"
      ;;
    *)
      if [[ -n "$SESSION_ID" ]]; then
        die "only one session id may be provided"
      fi
      SESSION_ID="$1"
      shift
      ;;
  esac
done

[[ -n "$SESSION_ID" ]] || {
  usage
  exit 1
}

require_cmd cargo
require_cmd find
require_cmd sleep

CODEX_HOME="${CODEX_HOME_OVERRIDE:-${CODEX_HOME:-$HOME/.codex}}"
ROLLOUT_PATH="$(resolve_rollout_path "$CODEX_HOME" "$SESSION_ID")"
SMOKE_ROOT="target/hybrid-drift-smoke/$SESSION_ID"
COMPACTOR_OUT="$SMOKE_ROOT/compactor"
ANALYZER_OUT="$SMOKE_ROOT/analyzer"
LIVE_STATE_DIR="target/hybrid-drift-live/$SESSION_ID"

if [[ "$FRESH_STATE" -eq 1 ]]; then
  log "Removing prior live state at $LIVE_STATE_DIR"
  rm -rf "$REPO_ROOT/$LIVE_STATE_DIR"
fi

log "Session id: $SESSION_ID"
log "CODEX_HOME: $CODEX_HOME"
log "Rollout path: $ROLLOUT_PATH"
log "Smoke root: $SMOKE_ROOT"
log "Live state dir: $LIVE_STATE_DIR"

if [[ "$NO_GROWTH_CHECK" -eq 0 ]]; then
  before_size="$(stat_size "$ROLLOUT_PATH")"
  sleep 2
  after_size="$(stat_size "$ROLLOUT_PATH")"
  if [[ "$after_size" -gt "$before_size" ]]; then
    log "Rollout growth probe: ${before_size} -> ${after_size} bytes"
  else
    log "Rollout growth probe: unchanged at ${after_size} bytes; continuing, but live output may stay quiet until the session writes more rows"
  fi
fi

if [[ "$SKIP_STATIC" -eq 0 ]]; then
  log "Removing prior static smoke output at $SMOKE_ROOT"
  rm -rf "$REPO_ROOT/$SMOKE_ROOT"

  run_repo_cmd cargo run -p agent-session-compactor -- \
    --codex-home "$CODEX_HOME" \
    --session-id "$SESSION_ID" \
    --output-dir "$COMPACTOR_OUT"

  run_repo_cmd cargo run -p agent-drift-analyzer -- \
    --input-dir "$COMPACTOR_OUT" \
    --output-dir "$ANALYZER_OUT"

  run_repo_cmd cargo run -p agent-drift-sentinel -- \
    --checkpoint-dir "$ANALYZER_OUT"
fi

log "Starting live sentinel; stop with Ctrl-C"
cd "$REPO_ROOT"
exec cargo run -p agent-drift-sentinel -- \
  --mode live \
  --codex-home "$CODEX_HOME" \
  --session-id "$SESSION_ID" \
  --checkpoint-dir "$LIVE_STATE_DIR"
