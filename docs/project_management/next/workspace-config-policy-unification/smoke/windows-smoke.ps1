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

  # Workspace init flags: `--examples` creates non-active templates and Substrate does not read them for behavior.
  substrate workspace init . --examples | Out-Null
  if (-not (Test-Path ".substrate\\workspace.example.yaml")) { throw "missing .substrate\\workspace.example.yaml" }
  if (-not (Test-Path ".substrate\\policy.example.yaml")) { throw "missing .substrate\\policy.example.yaml" }
  Set-Content -Path ".substrate\\workspace.example.yaml" -Value ":\n"
  Set-Content -Path ".substrate\\policy.example.yaml" -Value ":\n"
  substrate config current show --json | Out-Null
  substrate policy current show --json | Out-Null

  # Workspace init flags: `--force` repairs missing entries and does not overwrite existing non-empty patch files.
  substrate config workspace set world.caged=false | Out-Null
  $workspaceYamlHashBefore = (Get-FileHash ".substrate\\workspace.yaml" -Algorithm SHA256).Hash
  Remove-Item -Recurse -Force ".substrate\\git\\repo.git" -ErrorAction SilentlyContinue
  Remove-Item -Force ".substrate\\policy.yaml" -ErrorAction SilentlyContinue
  substrate workspace init . --force | Out-Null
  if (-not (Test-Path ".substrate\\git\\repo.git")) { throw "missing .substrate\\git\\repo.git after --force repair" }
  if (-not (Test-Path ".substrate\\policy.yaml")) { throw "missing .substrate\\policy.yaml after --force repair" }
  $workspaceYamlHashAfter = (Get-FileHash ".substrate\\workspace.yaml" -Algorithm SHA256).Hash
  if ($workspaceYamlHashAfter -ne $workspaceYamlHashBefore) { throw "expected .substrate\\workspace.yaml unchanged by workspace init --force" }

	  # Phase B (ADR-0012): edit `world.deps.enabled` via config editor at both scopes (include a deliberate duplicate across scopes).
	  substrate config global set world.deps.enabled+=bun world.deps.enabled+=node-runtime | Out-Null
	  substrate config workspace set world.deps.enabled+=node-runtime world.deps.enabled+=deno | Out-Null

	  # World-deps contract parity: enum keys (replace precedence).
	  substrate config global set world.deps.inventory_mode=merged world.deps.builtins=enabled | Out-Null
	  substrate config workspace set world.deps.inventory_mode=workspace_only world.deps.builtins=disabled | Out-Null

  $effectiveJson = substrate config current show --json
  $effective = $effectiveJson | ConvertFrom-Json

	  if ($null -eq $effective.world.deps.enabled) { throw "missing world.deps.enabled in effective config" }
	  if ($effective.world.deps.enabled -notcontains "bun") { throw "expected bun enabled" }
	  if ($effective.world.deps.enabled -notcontains "node-runtime") { throw "expected node-runtime enabled" }
	  if ($effective.world.deps.enabled -notcontains "deno") { throw "expected deno enabled" }
	  if ($effective.world.deps.inventory_mode -ne "workspace_only") { throw "expected world.deps.inventory_mode=workspace_only when workspace is enabled" }
	  if ($effective.world.deps.builtins -ne "disabled") { throw "expected world.deps.builtins=disabled when workspace is enabled" }

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

	  # Replace-key provenance: enum keys report merge_strategy=replace and exactly one contributing layer (workspace_patch).
	  $jsonStart = $stderr.IndexOf("{")
	  if ($jsonStart -lt 0) { throw "failed to locate JSON object in --explain stderr" }
	  $explainJson = $stderr.Substring($jsonStart)
	  $explain = $explainJson | ConvertFrom-Json
	  function Assert-ReplaceKeyProvenance($explainObj, $key, $expectedLayer) {
	    $entry = $explainObj.keys.$key
	    if ($null -eq $entry) { throw "missing key in --explain: $key" }
	    if ($entry.merge_strategy -ne "replace") { throw "$key: expected merge_strategy=replace" }
	    if ($entry.sources.Count -ne 1) { throw "$key: expected exactly 1 source" }
	    if ($entry.sources[0].layer -ne $expectedLayer) { throw "$key: expected source layer $expectedLayer" }
	  }
	  Assert-ReplaceKeyProvenance $explain 'world.deps.inventory_mode' 'workspace_patch'
	  Assert-ReplaceKeyProvenance $explain 'world.deps.builtins' 'workspace_patch'

  # Workspace disabled marker must ignore workspace contribution for this merge key.
  substrate workspace disable . | Out-Null
	  $disabledJson = substrate config current show --json
	  $disabled = $disabledJson | ConvertFrom-Json
	  if ($disabled.world.deps.enabled -contains "deno") { throw "expected deno absent when workspace is disabled" }
	  if ($disabled.world.deps.inventory_mode -ne "merged") { throw "expected world.deps.inventory_mode=merged when workspace is disabled" }
	  if ($disabled.world.deps.builtins -ne "enabled") { throw "expected world.deps.builtins=enabled when workspace is disabled" }
  $stderrDisabledFile = Join-Path $ScratchRoot "explain-disabled-stderr.txt"
  if (Test-Path $stderrDisabledFile) { Remove-Item -Force $stderrDisabledFile }
  $null = substrate config current show --json --explain 2> $stderrDisabledFile
	  $stderrDisabled = Get-Content -Raw $stderrDisabledFile
	  if ($stderrDisabled -notmatch "global_patch") { throw "missing global_patch when workspace is disabled" }
	  if ($stderrDisabled -match "workspace_patch") { throw "expected workspace_patch absent when workspace is disabled" }

	  # When workspace is disabled, enum keys report a single contributing layer (global_patch).
	  $jsonStartDisabled = $stderrDisabled.IndexOf("{")
	  if ($jsonStartDisabled -lt 0) { throw "failed to locate JSON object in --explain stderr (disabled)" }
	  $explainDisabledJson = $stderrDisabled.Substring($jsonStartDisabled)
	  $explainDisabled = $explainDisabledJson | ConvertFrom-Json
	  Assert-ReplaceKeyProvenance $explainDisabled 'world.deps.inventory_mode' 'global_patch'
	  Assert-ReplaceKeyProvenance $explainDisabled 'world.deps.builtins' 'global_patch'
	  substrate workspace enable . | Out-Null

  # List removal operator syntax (`-=`) removes the exact item from the patch list.
  substrate config workspace set world.deps.enabled-=deno | Out-Null
  $afterRemoveJson = substrate config current show --json
  $afterRemove = $afterRemoveJson | ConvertFrom-Json
  if ($afterRemove.world.deps.enabled -contains "deno") { throw "expected deno removed via -=" }

  # Workspace reset must remove the key from the workspace patch (inherit-only) while global still contributes.
  substrate config workspace reset world.deps.enabled | Out-Null
	  $postResetJson = substrate config current show --json
	  $postReset = $postResetJson | ConvertFrom-Json
	  if ($postReset.world.deps.enabled -notcontains "bun") { throw "expected bun still enabled after workspace reset" }
	  if ($postReset.world.deps.enabled -contains "deno") { throw "expected deno absent after workspace reset" }

	  # Invalid enum values must be exit 2 and perform no writes (patch bytes unchanged).
	  $configPath = Join-Path $HomeDir "config.yaml"
	  $cfgHashBefore = (Get-FileHash $configPath -Algorithm SHA256).Hash
	  & substrate config global set world.deps.builtins=bogus 2>$null | Out-Null
	  if ($LASTEXITCODE -ne 2) { throw "expected exit 2 for invalid enum value (world.deps.builtins=bogus), got $LASTEXITCODE" }
	  $cfgHashAfter = (Get-FileHash $configPath -Algorithm SHA256).Hash
	  if ($cfgHashAfter -ne $cfgHashBefore) { throw "expected global patch bytes unchanged after invalid enum value" }

	  $workspacePath = ".substrate\\workspace.yaml"
	  $wsHashBefore = (Get-FileHash $workspacePath -Algorithm SHA256).Hash
	  & substrate config workspace set world.deps.inventory_mode=nope 2>$null | Out-Null
	  if ($LASTEXITCODE -ne 2) { throw "expected exit 2 for invalid enum value (world.deps.inventory_mode=nope), got $LASTEXITCODE" }
	  $wsHashAfter = (Get-FileHash $workspacePath -Algorithm SHA256).Hash
	  if ($wsHashAfter -ne $wsHashBefore) { throw "expected workspace patch bytes unchanged after invalid enum value" }

	  Write-Output "OK: $FeatureDir windows smoke passed"
} finally {
  Pop-Location
}
