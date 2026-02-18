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

echo "INFO: slice=$slice_id"

# OR0 invariants (required on all slices).
cargo test -p substrate-common --test agent_hub_event_envelope_schema -- --nocapture
cargo test -p shell --test agent_hub_trace_persistence -- --nocapture

if [[ "$slice_id" == "OR1" ]]; then
  cargo test -p shell --test repl_output_routing -- --nocapture
  cargo test -p shell --test repl_config_max_pty_buffered_lines -- --nocapture
fi

echo "OK: linux smoke ($slice_id)"

