$ErrorActionPreference = "Stop"

if ($env:SUBSTRATE_SMOKE_SLICE_ID) {
  $SliceId = $env:SUBSTRATE_SMOKE_SLICE_ID
} else {
  $SliceId = "WDRA2"
}

switch ($SliceId) {
  "WDRA0" {
    cargo test -p shell --test replay_world replay_no_world_flag_reports_world_toggle -- --exact --nocapture
    cargo test -p shell --test replay_world replay_env_override_reports_world_toggle -- --exact --nocapture
  }
  "WDRA1" {
    cargo test -p shell --test replay_world replay_recorded_host_origin_attributes_override_env -- --exact --nocapture
  }
  "WDRA2" {
    cargo test -p shell --test replay_world replay_recorded_host_origin_attributes_workspace_config -- --exact --nocapture
    cargo test -p shell --test replay_world replay_recorded_host_origin_attributes_global_config -- --exact --nocapture
    cargo test -p shell --test replay_world replay_unknown_source_fallback_uses_published_contract -- --exact --nocapture
  }
  default {
    Write-Error "unsupported SUBSTRATE_SMOKE_SLICE_ID=$SliceId"
    exit 2
  }
}
