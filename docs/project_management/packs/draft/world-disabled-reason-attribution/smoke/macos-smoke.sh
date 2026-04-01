#!/usr/bin/env bash
set -euo pipefail

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "SKIP: world-disabled-reason-attribution macos smoke (not macOS)"
  exit 0
fi

SLICE_ID="${SUBSTRATE_SMOKE_SLICE_ID:-WDRA2}"
case "$SLICE_ID" in
  WDRA0)
    cargo test -p shell --test replay_world replay_recorded_host_origin_attributes_override_env -- --exact --nocapture
    cargo test -p shell --test replay_world replay_recorded_host_origin_redacts_absolute_config_paths -- --exact --nocapture
    ;;
  WDRA1)
    cargo test -p shell --test replay_world replay_trace_strategy_emits_world_disable_source_for_override_env -- --exact --nocapture
    cargo test -p shell --test replay_world replay_trace_strategy_omits_world_disable_source_for_replay_opt_out -- --exact --nocapture
    ;;
  WDRA2)
    cargo test -p shell --test replay_world replay_recorded_host_origin_attributes_workspace_config -- --exact --nocapture
    cargo test -p shell --test replay_world replay_recorded_host_origin_attributes_global_config -- --exact --nocapture
    cargo test -p shell --test replay_world replay_trace_strategy_emits_world_disable_source_for_workspace_config -- --exact --nocapture
    cargo test -p shell --test replay_world replay_trace_strategy_emits_world_disable_source_for_global_config -- --exact --nocapture
    ;;
  *)
    echo "FAIL: unsupported SUBSTRATE_SMOKE_SLICE_ID=$SLICE_ID" >&2
    exit 2
    ;;
esac
