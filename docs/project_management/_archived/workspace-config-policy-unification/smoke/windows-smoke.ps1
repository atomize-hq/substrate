Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$FeatureDir = "docs/project_management/_archived/workspace-config-policy-unification"
$SliceId = $env:SUBSTRATE_SMOKE_SLICE_ID
if ($null -eq $SliceId) { $SliceId = "" }

Write-Host "INFO: $FeatureDir windows smoke (SUBSTRATE_SMOKE_SLICE_ID='$SliceId')"

$ScratchRoot = Join-Path $env:TEMP ("substrate-wcu-smoke-" + [Guid]::NewGuid().ToString("N"))
$Workspace = Join-Path $ScratchRoot "ws"
$HomeDir = Join-Path $ScratchRoot "home"

New-Item -ItemType Directory -Force $Workspace | Out-Null
New-Item -ItemType Directory -Force $HomeDir | Out-Null

$env:SUBSTRATE_HOME = $HomeDir
Push-Location $Workspace

function Canonical-Json([string]$JsonText) {
  return (($JsonText | ConvertFrom-Json) | ConvertTo-Json -Compress -Depth 100)
}

function Parse-ExplainJsonFromStderr([string]$StderrText) {
  $jsonStart = $StderrText.IndexOf("{")
  if ($jsonStart -lt 0) { throw "failed to locate JSON object in --explain stderr" }
  $explainJson = $StderrText.Substring($jsonStart)
  return ($explainJson | ConvertFrom-Json)
}

function Assert-ReplaceKeyProvenance($explainObj, [string]$key, [string]$expectedLayer) {
  $entry = $explainObj.keys.$key
  if ($null -eq $entry) { throw "missing key in --explain: $key" }
  if ($entry.merge_strategy -ne "replace") { throw "${key}: expected merge_strategy=replace" }
  if ($entry.sources.Count -ne 1) { throw "${key}: expected exactly 1 source" }
  if ($entry.sources[0].layer -ne $expectedLayer) { throw "${key}: expected source layer ${expectedLayer}" }
}

function Init-Minimal() {
  substrate config global init --force | Out-Null
  substrate policy global init --force | Out-Null
  substrate workspace init . | Out-Null
}

function Run-Wcu1Smoke() {
  Init-Minimal

  if (-not (Test-Path ".substrate\\workspace.yaml")) { throw "missing .substrate\\workspace.yaml" }
  if (-not (Test-Path ".substrate\\policy.yaml")) { throw "missing .substrate\\policy.yaml" }
  if (-not (Test-Path ".substrate\\git\\repo.git")) { throw "missing .substrate\\git\\repo.git" }

  if (-not (Select-String -Path ".gitignore" -SimpleMatch ".substrate/" -Quiet)) { throw "missing .substrate/ ignore in .gitignore" }
  if (-not (Select-String -Path ".gitignore" -SimpleMatch "!.substrate/workspace.yaml" -Quiet)) { throw "missing !.substrate/workspace.yaml allowlist in .gitignore" }
  if (-not (Select-String -Path ".gitignore" -SimpleMatch "!.substrate/policy.yaml" -Quiet)) { throw "missing !.substrate/policy.yaml allowlist in .gitignore" }

  # Workspace init flags: `--examples` creates non-active templates and Substrate does not read them for behavior.
  substrate workspace init . --examples | Out-Null
  if (-not (Test-Path ".substrate\\workspace.example.yaml")) { throw "missing .substrate\\workspace.example.yaml" }
  if (-not (Test-Path ".substrate\\policy.example.yaml")) { throw "missing .substrate\\policy.example.yaml" }
  Set-Content -Path ".substrate\\workspace.example.yaml" -Value "this: is: invalid: yaml: [`n"
  Set-Content -Path ".substrate\\policy.example.yaml" -Value "this: is: invalid: yaml: [`n"

  # Workspace init flags: `--force` repairs missing entries and does not overwrite existing non-empty patch files.
  Set-Content -Path ".substrate\\workspace.yaml" -Value "wcu1_sentinel: true`n"
  Set-Content -Path ".substrate\\policy.yaml" -Value "wcu1_sentinel: true`n"
  $workspaceYamlHashBefore = (Get-FileHash ".substrate\\workspace.yaml" -Algorithm SHA256).Hash
  $policyYamlHashBefore = (Get-FileHash ".substrate\\policy.yaml" -Algorithm SHA256).Hash
  Remove-Item -Recurse -Force ".substrate\\git\\repo.git" -ErrorAction SilentlyContinue
  substrate workspace init . --force | Out-Null
  if (-not (Test-Path ".substrate\\git\\repo.git")) { throw "missing .substrate\\git\\repo.git after --force repair" }
  $workspaceYamlHashAfter = (Get-FileHash ".substrate\\workspace.yaml" -Algorithm SHA256).Hash
  $policyYamlHashAfter = (Get-FileHash ".substrate\\policy.yaml" -Algorithm SHA256).Hash
  if ($workspaceYamlHashAfter -ne $workspaceYamlHashBefore) { throw "expected .substrate\\workspace.yaml unchanged by workspace init --force" }
  if ($policyYamlHashAfter -ne $policyYamlHashBefore) { throw "expected .substrate\\policy.yaml unchanged by workspace init --force" }

  # Nested workspace refusal (when workspace is enabled).
  New-Item -ItemType Directory -Force "nested_refused" | Out-Null
  & substrate workspace init "nested_refused" 2>$null | Out-Null
  if ($LASTEXITCODE -ne 2) { throw "expected exit 2 for nested workspace init refusal, got $LASTEXITCODE" }

  # Disabled marker makes the parent workspace ignored for discovery.
  substrate workspace disable . | Out-Null
  if (-not (Test-Path ".substrate\\workspace.disabled")) { throw "missing .substrate\\workspace.disabled after disable" }
  New-Item -ItemType Directory -Force "nested_ok" | Out-Null
  substrate workspace init "nested_ok" | Out-Null
  if (-not (Test-Path "nested_ok\\.substrate\\workspace.yaml")) { throw "missing nested_ok\\.substrate\\workspace.yaml" }
  if (-not (Test-Path "nested_ok\\.substrate\\policy.yaml")) { throw "missing nested_ok\\.substrate\\policy.yaml" }
  if (-not (Test-Path "nested_ok\\.substrate\\git\\repo.git")) { throw "missing nested_ok\\.substrate\\git\\repo.git" }
  substrate workspace enable . | Out-Null
  if (Test-Path ".substrate\\workspace.disabled") { throw "expected workspace.disabled removed by enable" }

  Write-Output "OK: $FeatureDir windows smoke passed (slice=WCU1)"
}

function Run-Wcu2Smoke() {
  Init-Minimal

  # Configure Phase A inputs using patch files directly (do not require config editor / set/reset surfaces).
  $globalCfgPath = Join-Path $HomeDir "config.yaml"
  Set-Content -Path $globalCfgPath -Value @"
world:
  deps:
    enabled:
      - bun
      - node-runtime
    inventory_mode: merged
    builtins: enabled
"@

  Set-Content -Path ".substrate\\workspace.yaml" -Value @"
world:
  deps:
    enabled:
      - node-runtime
      - deno
    inventory_mode: workspace_only
    builtins: disabled
"@

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
  if ((Canonical-Json $effectiveJson2) -ne (Canonical-Json $effectiveJson)) { throw "effective JSON must be identical for identical inputs" }

  $stderrFile = Join-Path $ScratchRoot "explain-stderr.txt"
  $stderrFile2 = Join-Path $ScratchRoot "explain-stderr-2.txt"
  if (Test-Path $stderrFile) { Remove-Item -Force $stderrFile }
  if (Test-Path $stderrFile2) { Remove-Item -Force $stderrFile2 }
  $null = substrate config current show --json --explain 2> $stderrFile
  $null = substrate config current show --json --explain 2> $stderrFile2
  $stderr = Get-Content -Raw $stderrFile
  $stderr2 = Get-Content -Raw $stderrFile2
  if ($stderr2 -ne $stderr) { throw "--explain stderr must be byte-identical for identical inputs" }
  if ($stderr -notmatch "concat_dedupe_ordered_set") { throw "missing concat_dedupe_ordered_set in --explain stderr" }
  $globalPos = $stderr.IndexOf("global_patch")
  $workspacePos = $stderr.IndexOf("workspace_patch")
  if ($globalPos -lt 0 -or $workspacePos -lt 0 -or $globalPos -ge $workspacePos) { throw "expected global_patch to appear before workspace_patch in --explain stderr" }

  $explain = Parse-ExplainJsonFromStderr $stderr
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
  $explainDisabled = Parse-ExplainJsonFromStderr $stderrDisabled
  Assert-ReplaceKeyProvenance $explainDisabled 'world.deps.inventory_mode' 'global_patch'
  Assert-ReplaceKeyProvenance $explainDisabled 'world.deps.builtins' 'global_patch'

  substrate workspace enable . | Out-Null

  Write-Output "OK: $FeatureDir windows smoke passed (slice=WCU2)"
}

function Run-FullSmoke() {
  Init-Minimal

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
  if ((Canonical-Json $effectiveJson2) -ne (Canonical-Json $effectiveJson)) { throw "effective JSON must be identical for identical inputs" }

  # Phase A (ADR-0012): `--explain` supports merge_strategy + multi-source provenance.
  $stderrFile = Join-Path $ScratchRoot "explain-stderr.txt"
  $stderrFile2 = Join-Path $ScratchRoot "explain-stderr-2.txt"
  if (Test-Path $stderrFile) { Remove-Item -Force $stderrFile }
  if (Test-Path $stderrFile2) { Remove-Item -Force $stderrFile2 }
  $null = substrate config current show --json --explain 2> $stderrFile
  $null = substrate config current show --json --explain 2> $stderrFile2
  $stderr = Get-Content -Raw $stderrFile
  $stderr2 = Get-Content -Raw $stderrFile2
  if ($stderr2 -ne $stderr) { throw "--explain stderr must be byte-identical for identical inputs" }
  if ($stderr -notmatch "concat_dedupe_ordered_set") { throw "missing concat_dedupe_ordered_set in --explain stderr" }
  $globalPos = $stderr.IndexOf("global_patch")
  $workspacePos = $stderr.IndexOf("workspace_patch")
  if ($globalPos -lt 0 -or $workspacePos -lt 0 -or $globalPos -ge $workspacePos) { throw "expected global_patch to appear before workspace_patch in --explain stderr" }

  # Replace-key provenance: enum keys report merge_strategy=replace and exactly one contributing layer (workspace_patch).
  $explain = Parse-ExplainJsonFromStderr $stderr
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
  $explainDisabled = Parse-ExplainJsonFromStderr $stderrDisabled
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

  Write-Output "OK: $FeatureDir windows smoke passed (slice=${SliceId:-full})"
}

try {
  switch ($SliceId) {
    "" { Run-FullSmoke }
    "WCU1" { Run-Wcu1Smoke }
    "WCU2" { Run-Wcu2Smoke }
    "WCU3" { Run-FullSmoke }
    "WCU4" { Run-FullSmoke }
    "WCU5" { Run-FullSmoke }
    default { throw "unknown SUBSTRATE_SMOKE_SLICE_ID='$SliceId' (expected WCU1..WCU5 or empty)" }
  }
} finally {
  Pop-Location
  Remove-Item -Recurse -Force $ScratchRoot -ErrorAction SilentlyContinue
}

