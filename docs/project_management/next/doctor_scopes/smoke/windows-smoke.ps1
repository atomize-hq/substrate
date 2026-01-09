$ErrorActionPreference = "Stop"

if ($env:OS -ne "Windows_NT") {
  Write-Host "SKIP: doctor_scopes smoke (not Windows)"
  exit 0
}

# DS0 declares Windows as CI-parity-only (no behavioral smoke required).
Write-Host "SKIP: doctor_scopes smoke (Windows is CI-parity-only for this feature)"
exit 0

