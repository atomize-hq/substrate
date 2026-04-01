$ErrorActionPreference = "Stop"

if ($env:SUBSTRATE_SMOKE_SLICE_ID) {
  $SliceId = $env:SUBSTRATE_SMOKE_SLICE_ID
} else {
  $SliceId = "WDRA2"
}

switch ($SliceId) {
  "WDRA0" {
    cargo test -p shell --test replay_world replay_recorded_host_origin_attributes_override_env -- --exact --nocapture
    cargo test -p shell --test replay_world replay_recorded_host_origin_redacts_absolute_config_paths -- --exact --nocapture
  }
  "WDRA1" {
    cargo test -p shell --test replay_world replay_trace_strategy_emits_world_disable_source_for_override_env -- --exact --nocapture
    cargo test -p shell --test replay_world replay_trace_strategy_omits_world_disable_source_for_replay_opt_out -- --exact --nocapture
  }
  "WDRA2" {
    cargo test -p shell --test replay_world replay_recorded_host_origin_attributes_workspace_config -- --exact --nocapture
    cargo test -p shell --test replay_world replay_recorded_host_origin_attributes_global_config -- --exact --nocapture
    cargo test -p shell --test replay_world replay_trace_strategy_emits_world_disable_source_for_workspace_config -- --exact --nocapture
    cargo test -p shell --test replay_world replay_trace_strategy_emits_world_disable_source_for_global_config -- --exact --nocapture
  }
  default {
    Write-Error "unsupported SUBSTRATE_SMOKE_SLICE_ID=$SliceId"
    exit 2
  }
}
