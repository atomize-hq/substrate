#!/usr/bin/env bash
set -euo pipefail

scripts/mac/lima-warm.sh
substrate -c 'echo smoke-nonpty'
substrate --pty -c 'printf smoke-pty\n'
trace_log="${SHIM_TRACE_LOG:-$HOME/.substrate/trace.jsonl}"
mkdir -p "$(dirname "$trace_log")"

substrate -c 'rm -rf world-mac-smoke'
substrate -c 'python3 scripts/mac/world_smoke_payload.py'

if [ ! -f "$trace_log" ]; then
  echo "ERROR: Trace log not found at $trace_log" >&2
  exit 1
fi

span=$(jq -r 'select(.event_type=="command_complete" and .cmd=="python3 scripts/mac/world_smoke_payload.py") | .span_id' "$trace_log" | tail -n 1)

if [ -z "${span}" ]; then
  echo "ERROR: failed to locate span id for world-mac-smoke command" >&2
  exit 1
fi

substrate --replay-verbose --replay "$span"
substrate --trace "$span" | tee /tmp/world-mac-replay.json
jq '.fs_diff.writes' /tmp/world-mac-replay.json | grep 'world-mac-smoke/file.txt'
