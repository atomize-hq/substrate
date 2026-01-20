param()

$ErrorActionPreference = "Stop"

if ($env:OS -ne "Windows_NT") {
  Write-Host "SKIP: full-isolation-landlock-overlayfs-compat smoke (not Windows)"
  exit 0
}

Write-Host "OK: Windows smoke is a no-op for this feature"
exit 0
