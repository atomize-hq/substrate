$ErrorActionPreference = "Stop"

if (-not $IsWindows) {
  Write-Host "SKIP: world-deps selection Windows smoke (not Windows)"
  exit 0
}

if (-not (Get-Command substrate -ErrorAction SilentlyContinue)) {
  Write-Error "FAIL: substrate not found on PATH"
  exit 1
}

$help = & substrate world deps --help 2>$null
if ($help -notmatch "\binit\b") {
  Write-Host "SKIP: world-deps selection smoke (world deps init not available in this build)"
  exit 0
}

Write-Host "OK: world-deps selection Windows smoke (gating)"
exit 0

