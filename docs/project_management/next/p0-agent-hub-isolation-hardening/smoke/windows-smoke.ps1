$ErrorActionPreference = "Stop"

if (-not $IsWindows) {
  Write-Host "SKIP: agent hub hardening Windows smoke (not Windows)"
  exit 0
}

if (-not (Get-Command substrate -ErrorAction SilentlyContinue)) {
  Write-Error "FAIL: substrate not found on PATH"
  exit 1
}

Write-Host "OK: agent hub hardening Windows smoke (preflight)"
exit 0

