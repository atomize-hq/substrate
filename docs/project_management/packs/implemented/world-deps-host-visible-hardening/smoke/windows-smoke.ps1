$ErrorActionPreference = "Stop"

# Exit codes (aligned to docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md):
# 0: smoke passed
# 3: required dependency unavailable
# 4: not supported / missing prerequisites

if (-not $IsWindows) {
  Write-Error ("world-deps-host-visible-hardening: windows smoke is supported only on Windows")
  exit 4
}

$SubstrateBin = $env:SUBSTRATE_BIN
if ([string]::IsNullOrEmpty($SubstrateBin)) { $SubstrateBin = "substrate" }

# Prefer running the Linux smoke inside WSL when available.
$wsl = (Get-Command wsl.exe -ErrorAction SilentlyContinue)
if ($null -eq $wsl) {
  Write-Error ("world-deps-host-visible-hardening: wsl.exe not found; run smoke inside WSL or install WSL")
  exit 4
}

# Assume the repo is available in WSL at the same path; if not, the command will fail with a clear error.
$core = "bash docs/project_management/packs/active/world-deps-host-visible-hardening/smoke/linux-smoke.sh"
$cmd = "export SUBSTRATE_BIN='$SubstrateBin'; $core"

& wsl.exe bash -lc $cmd
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
exit 0

