$ErrorActionPreference = "Stop"

if ($IsWindows -ne $true) {
  Write-Output "SKIP: windows-smoke.ps1 intended for Windows"
  exit 0
}

$sliceId = $env:SUBSTRATE_SMOKE_SLICE_ID
if ([string]::IsNullOrWhiteSpace($sliceId)) { $sliceId = "OR1" }

if ($sliceId -ne "OR0" -and $sliceId -ne "OR1") {
  Write-Output "FAIL: unsupported SUBSTRATE_SMOKE_SLICE_ID: $sliceId (expected OR0 or OR1)"
  exit 2
}

Write-Output "INFO: slice=$sliceId"

# OR0 invariants (required on all slices).
cargo test -p substrate-common --test agent_hub_event_envelope_schema -- --nocapture
cargo test -p shell --test agent_hub_trace_persistence -- --nocapture

if ($sliceId -eq "OR1") {
  cargo test -p shell --test repl_output_routing -- --nocapture
  cargo test -p shell --test repl_config_max_pty_buffered_lines -- --nocapture
}

Write-Output "OK: Windows smoke ($sliceId)"
exit 0

