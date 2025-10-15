#!/usr/bin/env bash
set -euo pipefail

if [ "$EUID" -eq 0 ]; then
  echo "Do not run this smoke script as root." >&2
  exit 1
fi

SCRIPTS_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPTS_ROOT}/../.." && pwd)"
SUBSTRATE_BIN="${SUBSTRATE_BIN:-${REPO_ROOT}/target/debug/substrate}"

if ! command -v limactl >/dev/null 2>&1; then
  PATH="/opt/homebrew/opt/lima/bin:/opt/homebrew/bin:$PATH"
fi

if ! command -v limactl >/dev/null 2>&1; then
  echo "ERROR: limactl not found on PATH. Install Lima via Homebrew (brew install lima)." >&2
  exit 1
fi

if [ ! -x "$SUBSTRATE_BIN" ]; then
  echo "Building substrate binary for smoke test..."
  (cd "$REPO_ROOT" && cargo build --bin substrate >/dev/null)
fi

rm -rf "$REPO_ROOT/world-mac-smoke"
"$SCRIPTS_ROOT"/lima-warm.sh
"$SUBSTRATE_BIN" -c 'echo smoke-nonpty'
"$SUBSTRATE_BIN" --pty -c 'printf smoke-pty\n'
trace_log="${SHIM_TRACE_LOG:-$HOME/.substrate/trace.jsonl}"
mkdir -p "$(dirname "$trace_log")"

"$SUBSTRATE_BIN" -c 'rm -rf world-mac-smoke'
PAYLOAD_CMD="(cd /src 2>/dev/null || cd \"$REPO_ROOT\") && (test -d world-mac-smoke || mkdir world-mac-smoke) && printf 'data\n' > world-mac-smoke/file.txt"
"$SUBSTRATE_BIN" -c "$PAYLOAD_CMD"

if [ ! -f "$trace_log" ]; then
  echo "ERROR: Trace log not found at $trace_log" >&2
  exit 1
fi

span=$(jq -r 'select(.event_type=="command_complete" and ((.fs_diff.mods? // []) | index("world-mac-smoke/file.txt") != null)) | .span_id' "$trace_log" | tail -n 1)

if [ -z "${span}" ]; then
  echo "ERROR: failed to locate span id for world-mac-smoke command ($PAYLOAD_CMD)" >&2
  echo "Last few trace lines:" >&2
  tail -n 20 "$trace_log" >&2
  exit 1
fi

"$SUBSTRATE_BIN" --replay "$span" --replay-verbose
"$SUBSTRATE_BIN" --trace "$span" | tee /tmp/world-mac-replay.json
jq '.fs_diff | ((.writes // []) + (.mods // []))' /tmp/world-mac-replay.json | grep 'world-mac-smoke/file.txt'
