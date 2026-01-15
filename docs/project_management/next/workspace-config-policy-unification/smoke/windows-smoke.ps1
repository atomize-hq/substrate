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

  # Phase B (ADR-0012): edit `world.deps.enabled` via config editor at both scopes (include a deliberate duplicate across scopes).
  substrate config global set world.deps.enabled+=bun world.deps.enabled+=node-runtime | Out-Null
  substrate config workspace set world.deps.enabled+=node-runtime world.deps.enabled+=deno | Out-Null

  $effectiveJson = substrate config current show --json
  $effective = $effectiveJson | ConvertFrom-Json

  if ($null -eq $effective.world.deps.enabled) { throw "missing world.deps.enabled in effective config" }
  if ($effective.world.deps.enabled -notcontains "bun") { throw "expected bun enabled" }
  if ($effective.world.deps.enabled -notcontains "node-runtime") { throw "expected node-runtime enabled" }
  if ($effective.world.deps.enabled -notcontains "deno") { throw "expected deno enabled" }

  $bunIndex = [array]::IndexOf($effective.world.deps.enabled, "bun")
  $nodeIndex = [array]::IndexOf($effective.world.deps.enabled, "node-runtime")
  $denoIndex = [array]::IndexOf($effective.world.deps.enabled, "deno")
  if ($bunIndex -lt 0 -or $nodeIndex -lt 0 -or $denoIndex -lt 0) { throw "missing expected item index" }
  if (-not ($bunIndex -lt $nodeIndex -and $nodeIndex -lt $denoIndex)) { throw "expected ordered-set merge order bun < node-runtime < deno" }

  # Determinism/idempotence: re-running show + explain without changes yields identical outputs.
  $effectiveJson2 = substrate config current show --json
  if ($effectiveJson2 -ne $effectiveJson) { throw "effective JSON must be identical for identical inputs" }

  # Phase A (ADR-0012): `--explain` supports merge_strategy + multi-source provenance.
  $stderrFile = Join-Path $ScratchRoot "explain-stderr.txt"
  if (Test-Path $stderrFile) { Remove-Item -Force $stderrFile }
  $null = substrate config current show --json --explain 2> $stderrFile
  $stderr = Get-Content -Raw $stderrFile
  $stderrFile2 = Join-Path $ScratchRoot "explain-stderr-2.txt"
  if (Test-Path $stderrFile2) { Remove-Item -Force $stderrFile2 }
  $null = substrate config current show --json --explain 2> $stderrFile2
  $stderr2 = Get-Content -Raw $stderrFile2
  if ($stderr2 -ne $stderr) { throw "--explain stderr must be identical for identical inputs" }

  if ($stderr -notmatch "concat_dedupe_ordered_set") { throw "missing concat_dedupe_ordered_set in --explain stderr" }
  if ($stderr -notmatch "global_patch") { throw "missing global_patch in --explain stderr" }
  if ($stderr -notmatch "workspace_patch") { throw "missing workspace_patch in --explain stderr" }

  $globalPos = $stderr.IndexOf("global_patch")
  $workspacePos = $stderr.IndexOf("workspace_patch")
  if ($globalPos -lt 0 -or $workspacePos -lt 0 -or $globalPos -ge $workspacePos) { throw "expected global_patch to appear before workspace_patch in --explain stderr" }

  # Workspace disabled marker must ignore workspace contribution for this merge key.
  substrate workspace disable . | Out-Null
  $disabledJson = substrate config current show --json
  $disabled = $disabledJson | ConvertFrom-Json
  if ($disabled.world.deps.enabled -contains "deno") { throw "expected deno absent when workspace is disabled" }
  $stderrDisabledFile = Join-Path $ScratchRoot "explain-disabled-stderr.txt"
  if (Test-Path $stderrDisabledFile) { Remove-Item -Force $stderrDisabledFile }
  $null = substrate config current show --json --explain 2> $stderrDisabledFile
  $stderrDisabled = Get-Content -Raw $stderrDisabledFile
  if ($stderrDisabled -notmatch "global_patch") { throw "missing global_patch when workspace is disabled" }
  if ($stderrDisabled -match "workspace_patch") { throw "expected workspace_patch absent when workspace is disabled" }
  substrate workspace enable . | Out-Null

  # Workspace reset must remove the key from the workspace patch (inherit-only) while global still contributes.
  substrate config workspace reset world.deps.enabled | Out-Null
  $postResetJson = substrate config current show --json
  $postReset = $postResetJson | ConvertFrom-Json
  if ($postReset.world.deps.enabled -notcontains "bun") { throw "expected bun still enabled after workspace reset" }
  if ($postReset.world.deps.enabled -contains "deno") { throw "expected deno absent after workspace reset" }

  Write-Output "OK: $FeatureDir windows smoke passed"
} finally {
  Pop-Location
}
