#!/usr/bin/env bash
set -euo pipefail

if [[ "$(uname -s)" != "Linux" ]]; then
  echo "SKIP: linux-smoke.sh intended for Linux (uname=$(uname -s))"
  exit 0
fi

slice_id="${SUBSTRATE_SMOKE_SLICE_ID:-OR1}"

case "$slice_id" in
  OR0|OR1) ;;
  *)
    echo "FAIL: unsupported SUBSTRATE_SMOKE_SLICE_ID: $slice_id (expected OR0 or OR1)"
    exit 2
    ;;
esac

fail() {
  echo "FAIL: $*" >&2
  exit 1
}

need_cmd() {
  local name="$1"
  if ! command -v "$name" >/dev/null 2>&1; then
    echo "MISSING: $name not found on PATH" >&2
    exit 3
  fi
}

substrate_bin="${SUBSTRATE_BIN:-substrate}"
need_cmd "$substrate_bin"
need_cmd jq
need_cmd mktemp
need_cmd script

tmp_root="$(mktemp -d)"
home_dir="$tmp_root/home"
workspace="$tmp_root/ws"
trace="$tmp_root/substrate-home/trace.jsonl"
transcript="$tmp_root/transcript.txt"
input_file="$tmp_root/repl-input.txt"

cleanup() {
  rm -rf "$tmp_root"
}
trap cleanup EXIT

mkdir -p "$home_dir" "$workspace"
export HOME="$home_dir"
export SUBSTRATE_HOME="$tmp_root/substrate-home"
export SUBSTRATE_BIN="$substrate_bin"

echo "INFO: slice=$slice_id"

(
  cd "$workspace"
  "$substrate_bin" workspace init --force >/dev/null
)

run_or0() {
  (
    cd "$workspace"
    "$substrate_bin" --no-world --command ":demo-agent" >/dev/null
  )
}

run_or1() {
  cat >"$workspace/.substrate/workspace.yaml" <<'YAML'
repl:
  max_pty_buffered_lines: 0
YAML

  cat >"$input_file" <<'EOF'
:demo-agent
:pty bash -lc 'echo PTY_START; sleep 2; echo PTY_END'
exit
EOF

  local repl_cmd
  printf -v repl_cmd '%q --no-world' "$substrate_bin"
  (
    cd "$workspace"
    script -q -e -c "$repl_cmd" "$transcript" <"$input_file" >/dev/null
  )

  grep -q "PTY_START" "$transcript" || fail "PTY_START missing from transcript"
  grep -q "PTY_END" "$transcript" || fail "PTY_END missing from transcript"
  grep -q "Demo agent event #1" "$transcript" || fail "structured agent output missing from transcript"

  local between_markers
  between_markers="$(awk '/PTY_START/{flag=1;next}/PTY_END/{flag=0;exit}flag{print}' "$transcript")"
  if printf '%s' "$between_markers" | grep -q "Demo agent event"; then
    fail "structured agent output was injected during PTY passthrough"
  fi

  local after_pty
  after_pty="$(awk '/PTY_END/{flag=1;next}flag{print}' "$transcript")"
  if ! printf '%s' "$after_pty" | grep -q "substrate: warning:"; then
    fail "post-passthrough warning line missing from transcript"
  fi
}

case "$slice_id" in
  OR0) run_or0 ;;
  OR1) run_or1 ;;
esac

test -f "$trace" || fail "trace.jsonl missing: $trace"
jq -e 'any(select(.event_type=="agent_event" and .component=="agent-hub" and (.orchestration_session_id|type=="string") and (.run_id|type=="string") and (.data|type=="object")))' \
  "$trace" >/dev/null || fail "agent_event trace record missing required fields"

if [[ "$slice_id" == "OR1" ]]; then
  jq -e 'any(select(.event_type=="warning" and .component=="shell" and .code=="pty_structured_event_drops" and (.dropped_structured_event_lines|type=="number") and (.max_pty_buffered_lines|type=="number")))' \
    "$trace" >/dev/null || fail "pty_structured_event_drops warning record missing required fields"
fi

echo "OK: linux smoke ($slice_id)"
