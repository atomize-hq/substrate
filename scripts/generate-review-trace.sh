#!/usr/bin/env bash
set -Eeuo pipefail

restore_tty() {
    stty sane >/dev/null 2>&1 || true
}

on_exit() {
    restore_tty
    if [[ -n "${TRACE_DIR:-}" && -d "${TRACE_DIR:-}" ]]; then
        echo "Trace dir: ${TRACE_DIR}"
        if [[ -f "${TRACE_DIR}/trace.jsonl.gz" ]]; then
            echo "Trace: ${TRACE_DIR}/trace.jsonl.gz"
        elif [[ -f "${TRACE_DIR}/trace.jsonl" ]]; then
            echo "Trace: ${TRACE_DIR}/trace.jsonl"
        fi
        if [[ -f "${TRACE_DIR}.tar.gz" ]]; then
            echo "Bundle: ${TRACE_DIR}.tar.gz"
        fi
    fi
}

trap on_exit EXIT
trap restore_tty INT TERM

log() {
    printf '[trace-demo] %s\n' "$*" >&2
}

need_cmd() {
    command -v "$1" >/dev/null 2>&1
}

run_timed() {
    local seconds="${RUN_TIMEOUT_SECS:-300}"
    if need_cmd timeout; then
        timeout "${seconds}s" "$@"
    else
        "$@"
    fi
}

BUILD="${BUILD:-0}"
INCLUDE_PTY="${INCLUDE_PTY:-0}"
INCLUDE_WORLD_PTY="${INCLUDE_WORLD_PTY:-0}"
SUBSTRATE_BIN="${SUBSTRATE_BIN:-}"
TRACE_DIR="${TRACE_DIR:-}"

if [[ -z "${TRACE_DIR}" ]]; then
    TRACE_DIR="$(mktemp -d /tmp/substrate-trace-review.XXXXXX)"
fi

WORKDIR="${TRACE_DIR}/workspace"
TRACE="${TRACE_DIR}/trace.jsonl"
mkdir -p "${WORKDIR}"

if [[ -n "${SUBSTRATE_BIN}" ]]; then
    SUB="${SUBSTRATE_BIN}"
elif [[ -x "./target/release/substrate" ]]; then
    SUB="./target/release/substrate"
elif need_cmd substrate; then
    SUB="$(command -v substrate)"
else
    SUB=""
fi

if [[ "${BUILD}" == "1" || -z "${SUB}" || "${SUB}" == "substrate" || "${SUB}" == */substrate ]]; then
    if [[ -f "./Cargo.toml" ]]; then
        if [[ ! -x "./target/release/substrate" || "${BUILD}" == "1" ]]; then
            log "Building release binaries..."
            cargo build --release
        fi
        SUB="./target/release/substrate"
    fi
fi

if [[ -z "${SUB}" || ! -x "${SUB}" ]]; then
    log "ERROR: substrate binary not found. Run from the repo root or set SUBSTRATE_BIN=/path/to/substrate."
    exit 2
fi

PY="python3"
if ! need_cmd python3; then
    PY="python"
fi
if ! need_cmd "${PY}"; then
    log "ERROR: need python3 or python in PATH"
    exit 2
fi

mkdir -p "$(dirname "${TRACE}")"
: >"${TRACE}"

SHIM_SESSION_ID="ses_review_$(date +%Y%m%d_%H%M%S)"
export SHIM_TRACE_LOG="${TRACE}"
export SHIM_SESSION_ID
export SUBSTRATE_AGENT_ID="review-demo"
export SHIM_FSYNC=1

log "Substrate: ${SUB}"
log "Session: ${SHIM_SESSION_ID}"
log "Trace: ${TRACE}"
log "Workspace: ${WORKDIR}"

log "Collecting context JSON..."
"${SUB}" --version-json >"${TRACE_DIR}/version.json" 2>"${TRACE_DIR}/version.err" || true
"${SUB}" host doctor --json >"${TRACE_DIR}/host-doctor.json" 2>"${TRACE_DIR}/host-doctor.err" || true
"${SUB}" world doctor --json >"${TRACE_DIR}/world-doctor.json" 2>"${TRACE_DIR}/world-doctor.err" || true
"${SUB}" shim doctor --json >"${TRACE_DIR}/shim-doctor.json" 2>"${TRACE_DIR}/shim-doctor.err" || true
"${SUB}" health --json >"${TRACE_DIR}/health.json" 2>"${TRACE_DIR}/health.err" || true

log "Deploying shims..."
run_timed "${SUB}" --shim-deploy
"${SUB}" --shim-status-json >"${TRACE_DIR}/shim-status.json" 2>"${TRACE_DIR}/shim-status.err" || true

log "1/8 Host-only baseline (no world, no shims)..."
run_timed "${SUB}" --no-world --anchor-mode custom --anchor-path "${WORKDIR}" --caged -c \
    "${PY} -c 'print(\"host-only ok\")'"

log "2/8 World-enabled run with writes+stderr+nonzero exit..."
run_timed "${SUB}" --world --anchor-mode custom --anchor-path "${WORKDIR}" --caged -c \
    "${PY} -c 'from pathlib import Path; import sys; p=Path(\"a.txt\"); p.write_text(\"alpha\\n\"); p.write_text(p.read_text()+\"beta\\n\"); print(p.read_text().strip()); print(\"stderr line\", file=sys.stderr); p.unlink(); sys.exit(7)'" \
    || true

log "3/8 Redaction example (dummy token; should be redacted in shim argv)..."
run_timed "${SUB}" --world --anchor-mode custom --anchor-path "${WORKDIR}" --caged -c \
    "${PY} -c 'print(\"redaction\")' --token SUPER_SECRET_VALUE"

log "4/8 Nested process tree + pipeline (bash --noprofile --norc)..."
# Avoid `bash -l` here: some user login scripts can block on non-interactive shells.
run_timed "${SUB}" --world --anchor-mode custom --anchor-path "${WORKDIR}" --caged -c \
    "bash --noprofile --norc -c 'echo nested | wc -c'" \
    || true

if [[ "${INCLUDE_PTY}" == "1" ]]; then
    if [[ -t 0 && -t 1 ]]; then
        if need_cmd script; then
            log "5/8 PTY path (host PTY via 'script' wrapper)..."
            script -q -c "env SUBSTRATE_FORCE_PTY=1 \"${SUB}\" --no-world --anchor-mode custom --anchor-path \"${WORKDIR}\" --caged -c \"${PY} -c 'print(\\\"pty host path\\\")'\"" /dev/null \
                || true
        else
            log "5/8 PTY path (host PTY; no 'script' available)..."
            SUBSTRATE_FORCE_PTY=1 run_timed "${SUB}" --no-world --anchor-mode custom --anchor-path "${WORKDIR}" --caged -c \
                "${PY} -c 'print(\"pty host path\")'" \
                || true
        fi
    else
        log "5/8 Skipping PTY path (no TTY detected). Run in a real terminal with INCLUDE_PTY=1."
    fi
else
    log "5/8 Skipping PTY path (set INCLUDE_PTY=1 to enable)."
fi

if [[ "${INCLUDE_WORLD_PTY}" == "1" ]]; then
    if need_cmd script && need_cmd timeout; then
        log "5b/8 Optional world PTY WS path (isolated under 'script', 10s timeout)..."
        script -q -c "timeout 10s env SUBSTRATE_FORCE_PTY=1 \"${SUB}\" --world --anchor-mode custom --anchor-path \"${WORKDIR}\" --caged -c \"${PY} -c 'print(\\\"pty world path\\\")'\"" /dev/null \
            || true
    else
        log "5b/8 Skipping world PTY WS path (need 'script' + 'timeout'; set INCLUDE_WORLD_PTY=0 to silence)"
    fi
fi

log "6/8 Pipe mode (stdin not a tty)..."
printf '%s\n' \
    "echo pipe-mode" \
    "${PY} -c 'print(\"pipe python\")'" \
    | run_timed "${SUB}" --world --anchor-mode custom --anchor-path "${WORKDIR}" --caged

log "7/8 Script mode (-f)..."
cat >"${TRACE_DIR}/demo.sh" <<EOF
echo script-mode
${PY} -c 'import sys; print("script stdout"); print("script stderr", file=sys.stderr)'
EOF
run_timed "${SUB}" --world --anchor-mode custom --anchor-path "${WORKDIR}" --caged -f "${TRACE_DIR}/demo.sh"

log "8/8 Policy deny in enforce mode (default denylist blocks 'rm -rf *')..."
mkdir -p "${WORKDIR}/deny_me"
touch "${WORKDIR}/deny_me/file.txt"
SUBSTRATE_OVERRIDE_POLICY_MODE=enforce run_timed "${SUB}" --world --anchor-mode custom --anchor-path "${WORKDIR}" --caged -c \
    "rm -rf deny_me" \
    || true

log "Replay a recent span to append replay_strategy telemetry..."
SPAN_ID=""
if need_cmd jq; then
    SPAN_ID="$(jq -r 'select(.event_type=="command_complete" and (.span_id? != null)) | .span_id' "${TRACE}" | tail -n 1 || true)"
else
    SPAN_ID="$("${PY}" - <<PY
import json
span = ""
with open(r"""${TRACE}""","r",encoding="utf-8") as f:
    for line in f:
        try:
            obj = json.loads(line)
        except Exception:
            continue
        if obj.get("event_type") == "command_complete" and obj.get("span_id"):
            span = obj["span_id"]
print(span)
PY
)"
fi

if [[ -n "${SPAN_ID}" ]]; then
    run_timed "${SUB}" --trace "${SPAN_ID}" >"${TRACE_DIR}/trace-inspect.json" 2>"${TRACE_DIR}/trace-inspect.err" || true
    run_timed "${SUB}" --replay "${SPAN_ID}" --replay-verbose >"${TRACE_DIR}/replay.txt" 2>&1 || true
else
    log "WARNING: could not find a span_id for replay; leaving replay step out"
fi

log "Finalizing bundle..."
wc -l "${TRACE}" >"${TRACE_DIR}/trace-lines.txt" || true
if need_cmd gzip; then
    gzip -9 -f "${TRACE}"
fi
tar -C "${TRACE_DIR}" -czf "${TRACE_DIR}.tar.gz" .
log "Done."
