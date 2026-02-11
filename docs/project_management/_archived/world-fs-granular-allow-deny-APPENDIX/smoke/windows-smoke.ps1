$ErrorActionPreference = "Stop"

$SubstrateBin = if ($env:SUBSTRATE_BIN) { $env:SUBSTRATE_BIN } else { "substrate" }

if (-not (Get-Command $SubstrateBin -ErrorAction SilentlyContinue)) {
  Write-Error "world-fs-granular-allow-deny-appendix: substrate binary not found (SUBSTRATE_BIN=$SubstrateBin)"
  exit 3
}

Write-Error "world-fs-granular-allow-deny-appendix: windows smoke is not implemented in this Planning Pack"
exit 4

