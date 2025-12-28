$ErrorActionPreference = "Stop"

if (-not $IsWindows) {
  Write-Host "SKIP: policy/config mental model Windows smoke (not Windows)"
  exit 0
}

if (-not (Get-Command substrate -ErrorAction SilentlyContinue)) {
  Write-Error "FAIL: substrate not found on PATH"
  exit 1
}

$help = & substrate world enable --help 2>$null
if ($help -notmatch "\-\-home") {
  Write-Error "FAIL: --home missing from substrate world enable --help"
  exit 1
}
if ($help -match "\-\-prefix") {
  Write-Error "FAIL: --prefix present in substrate world enable --help"
  exit 1
}

Write-Host "OK: policy/config mental model Windows smoke (gating)"
exit 0

