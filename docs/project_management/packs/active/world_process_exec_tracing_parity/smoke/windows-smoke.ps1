#!/usr/bin/env pwsh
$ErrorActionPreference = "Stop"

# Exit codes (aligned to `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`):
# - 0: smoke passed
# - 1: smoke assertion failed / unexpected script error
# - 2: invalid inputs (e.g., unknown SUBSTRATE_SMOKE_SLICE_ID)
# - 3: required dependency unavailable (e.g., substrate not found)
# - 4: missing prerequisites / world backend unavailable

$substrate = $env:SUBSTRATE_BIN
if ([string]::IsNullOrWhiteSpace($substrate)) { $substrate = "substrate" }
if (-not (Get-Command $substrate -ErrorAction SilentlyContinue)) {
  Write-Error "world_process_exec_tracing_parity: substrate binary not found (SUBSTRATE_BIN=$substrate)"
  exit 3
}

$tmpRoot = $env:SUBSTRATE_SMOKE_ROOT
if ([string]::IsNullOrWhiteSpace($tmpRoot)) {
  $tmpRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("substrate-wpep-smoke-" + [System.Guid]::NewGuid().ToString("n"))
}
New-Item -ItemType Directory -Force -Path $tmpRoot | Out-Null

try {
  $workspace = Join-Path $tmpRoot "workspace"
  New-Item -ItemType Directory -Force -Path $workspace | Out-Null
  Set-Location $workspace

  $trace = Join-Path $tmpRoot "trace.jsonl"
  if (Test-Path $trace) { Remove-Item -Force $trace }
  $env:SHIM_TRACE_LOG = $trace

  Write-Host "== Setup: workspace =="
  & $substrate workspace init --force | Out-Null

  Write-Host "== Preflight: world doctor =="
  & $substrate world doctor | Out-Null
  if ($LASTEXITCODE -ne 0) {
    Write-Error "world_process_exec_tracing_parity: world doctor failed (exit=$LASTEXITCODE)"
    exit 4
  }

  $slice = $env:SUBSTRATE_SMOKE_SLICE_ID
  if ([string]::IsNullOrWhiteSpace($slice)) { $slice = "WPEP3" }
  if ($slice -notin @("WPEP0","WPEP1","WPEP2","WPEP3")) {
    Write-Error "world_process_exec_tracing_parity: unknown SUBSTRATE_SMOKE_SLICE_ID=$slice"
    exit 2
  }

  Write-Host "== Case A: world completion record carries process_events_status (windows) =="
  $worldMarker = "WPEP_SMOKE_MARKER_WORLD_${slice}_" + (Get-Random)
  & $substrate --world --command "bash -lc ""echo $worldMarker; sh -lc true; echo done""" | Out-Null

  $records = @()
  foreach ($line in (Get-Content -Path $trace -ErrorAction Stop)) {
    if ([string]::IsNullOrWhiteSpace($line)) { continue }
    $records += ($line | ConvertFrom-Json)
  }

  $matches = $records | Where-Object {
    $_.component -eq "shell" -and $_.event_type -eq "command_complete" -and (($_.command -as [string]) -like "*$worldMarker*")
  }
  if (-not $matches) {
    Write-Error "world_process_exec_tracing_parity: missing shell command_complete record for marker=$worldMarker"
    exit 1
  }

  if ($slice -eq "WPEP0") {
    Write-Host "OK: world_process_exec_tracing_parity smoke passed (slice=$slice platform=windows)"
    exit 0
  }

  # Windows capture is unavailable in this feature scope.
  $ok = $false
  foreach ($m in $matches) {
    if ($m.process_events_status -eq "unavailable" -and $m.process_events_reason -eq "not_supported_platform") {
      $ok = $true
    }
  }
  if (-not $ok) {
    Write-Error "world_process_exec_tracing_parity: expected process_events_status=unavailable reason=not_supported_platform on windows"
    exit 1
  }

  Write-Host "OK: world_process_exec_tracing_parity smoke passed (slice=$slice platform=windows)"
} finally {
  if ($env:SUBSTRATE_SMOKE_KEEP -ne "1") {
    Remove-Item -Recurse -Force $tmpRoot -ErrorAction SilentlyContinue
  }
}
