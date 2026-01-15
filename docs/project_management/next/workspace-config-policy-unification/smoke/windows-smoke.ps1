Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$FeatureDir = "docs/project_management/next/workspace-config-policy-unification"

$ScratchRoot = Join-Path $env:TEMP "substrate-wcu-smoke"
$Workspace = Join-Path $ScratchRoot "ws"
$HomeDir = Join-Path $ScratchRoot "home"

Remove-Item -Recurse -Force $ScratchRoot -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force $Workspace | Out-Null
New-Item -ItemType Directory -Force $HomeDir | Out-Null

$env:SUBSTRATE_HOME = $HomeDir
Push-Location $Workspace

try {
  substrate config global init --force | Out-Null
  substrate policy global init --force | Out-Null
  substrate workspace init . | Out-Null

  # Phase B (ADR-0012): edit `world.deps.enabled` via config editor at both scopes.
  substrate config global set world.deps.enabled+=bun | Out-Null
  substrate config workspace set world.deps.enabled+=node-runtime | Out-Null

  $effectiveJson = substrate config current show --json
  $effective = $effectiveJson | ConvertFrom-Json

  if ($null -eq $effective.world.deps.enabled) { throw "missing world.deps.enabled in effective config" }
  if ($effective.world.deps.enabled -notcontains "bun") { throw "expected bun enabled" }
  if ($effective.world.deps.enabled -notcontains "node-runtime") { throw "expected node-runtime enabled" }

  # Phase A (ADR-0012): `--explain` supports merge_strategy + multi-source provenance.
  $stderrFile = Join-Path $ScratchRoot "explain-stderr.txt"
  if (Test-Path $stderrFile) { Remove-Item -Force $stderrFile }
  $null = substrate config current show --json --explain 2> $stderrFile
  $stderr = Get-Content -Raw $stderrFile

  if ($stderr -notmatch "concat_dedupe_ordered_set") { throw "missing concat_dedupe_ordered_set in --explain stderr" }
  if ($stderr -notmatch "global_patch") { throw "missing global_patch in --explain stderr" }
  if ($stderr -notmatch "workspace_patch") { throw "missing workspace_patch in --explain stderr" }

  Write-Output "OK: $FeatureDir windows smoke passed"
} finally {
  Pop-Location
}

