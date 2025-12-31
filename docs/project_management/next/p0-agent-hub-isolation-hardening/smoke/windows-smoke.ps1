$ErrorActionPreference = "Stop"

if (-not $IsWindows) {
  Write-Host "SKIP: agent hub hardening Windows smoke (not Windows)"
  exit 0
}

if (-not (Get-Command substrate -ErrorAction SilentlyContinue)) {
  Write-Error "FAIL: substrate not found on PATH"
  exit 1
}

$out = & substrate --version 2>$null
if ([string]::IsNullOrWhiteSpace($out)) {
  Write-Error "FAIL: substrate --version produced no output"
  exit 1
}

Write-Host "OK: agent hub hardening Windows smoke"
exit 0
