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

if (-not (Get-Command jq -ErrorAction SilentlyContinue)) {
  Write-Error "FAIL: jq not found on PATH"
  exit 3
}

$substrateBin = if ([string]::IsNullOrWhiteSpace($env:SUBSTRATE_BIN)) { "substrate" } else { $env:SUBSTRATE_BIN }
if (-not (Get-Command $substrateBin -ErrorAction SilentlyContinue)) {
  Write-Error "FAIL: substrate binary not found: $substrateBin"
  exit 3
}

$tmpRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("substrate-ahor-" + [System.Guid]::NewGuid().ToString("N"))
$homeDir = Join-Path $tmpRoot "home"
$workspace = Join-Path $tmpRoot "ws"

New-Item -ItemType Directory -Force -Path $homeDir | Out-Null
New-Item -ItemType Directory -Force -Path $workspace | Out-Null

try {
  $env:HOME = $homeDir
  $env:USERPROFILE = $homeDir
  $env:SUBSTRATE_HOME = Join-Path $tmpRoot "substrate-home"

  Push-Location $workspace
  try {
    & $substrateBin workspace init --force | Out-Null
    & $substrateBin --no-world --command ":demo-agent" | Out-Null
  } finally {
    Pop-Location
  }

  $trace = Join-Path $env:SUBSTRATE_HOME "trace.jsonl"
  if (!(Test-Path $trace)) {
    Write-Error "FAIL: trace.jsonl missing: $trace"
    exit 1
  }

  Get-Content $trace | jq -e 'any(select(.event_type=="agent_event" and .component=="agent-hub" and (.orchestration_session_id|type=="string") and (.run_id|type=="string") and (.data|type=="object")))' | Out-Null
  if ($LASTEXITCODE -ne 0) {
    Write-Error "FAIL: agent_event trace record missing required fields"
    exit 1
  }
} finally {
  Remove-Item -Recurse -Force $tmpRoot -ErrorAction SilentlyContinue
  Remove-Item Env:SUBSTRATE_HOME -ErrorAction SilentlyContinue
  Remove-Item Env:HOME -ErrorAction SilentlyContinue
  Remove-Item Env:USERPROFILE -ErrorAction SilentlyContinue
}

Write-Output "OK: Windows smoke ($sliceId)"
exit 0
