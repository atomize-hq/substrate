$ErrorActionPreference = "Stop"

if (-not $IsWindows) {
  Write-Host "SKIP: world-sync Windows smoke (not Windows)"
  exit 0
}

if (-not (Get-Command substrate -ErrorAction SilentlyContinue)) {
  Write-Error "FAIL: substrate not found on PATH"
  exit 1
}

$help = & substrate --help 2>$null
if ($help -notmatch "\bsync\b") {
  Write-Host "SKIP: world-sync smoke (substrate sync not available in this build)"
  exit 0
}

$ws = Join-Path $env:TEMP ("substrate-world-sync-" + [guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Force -Path $ws | Out-Null
try {
  Push-Location $ws
  & git init -q | Out-Null
  & substrate init | Out-Null
  if (-not (Test-Path ".substrate")) { throw ".substrate missing" }
  if (-not (Test-Path ".substrate-git\\repo.git")) { throw ".substrate-git\\repo.git missing" }
  & substrate sync --dry-run | Out-Null
  Write-Host "OK: world-sync Windows smoke (core gating)"
  exit 0
} finally {
  Pop-Location
  Remove-Item -Recurse -Force $ws -ErrorAction SilentlyContinue
}

