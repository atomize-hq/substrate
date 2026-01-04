$ErrorActionPreference = "Stop"

if (-not $IsWindows) {
  Write-Host "SKIP: p0 platform stability Windows smoke (not Windows)"
  exit 0
}

if (-not (Get-Command substrate -ErrorAction SilentlyContinue)) {
  Write-Error "FAIL: substrate not found on PATH"
  exit 1
}

if (-not (Get-Command jq -ErrorAction SilentlyContinue)) {
  Write-Error "FAIL: jq not found on PATH"
  exit 2
}

Write-Host "SMOKE: world doctor JSON parses"
$doctor = & substrate world doctor --json 2>$null
if ([string]::IsNullOrWhiteSpace($doctor)) {
  Write-Error "FAIL: substrate world doctor --json returned empty output"
  exit 3
}
$null = $doctor | jq -e "." 1>$null

Write-Host "SMOKE: shim status + health JSON parses"
$null = (& substrate --shim-status-json) | jq -e "." 1>$null
$null = (& substrate health --json) | jq -e ".summary != null" 1>$null

if (Test-Path "scripts\\windows\\wsl-warm.ps1") {
  Write-Host "SMOKE: WSL warm -WhatIf"
  & pwsh -File scripts\\windows\\wsl-warm.ps1 -WhatIf 1>$null
}

Write-Host "OK: p0 platform stability Windows smoke"
exit 0

