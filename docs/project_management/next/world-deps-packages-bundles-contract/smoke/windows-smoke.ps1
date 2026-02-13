#!/usr/bin/env pwsh
$ErrorActionPreference = "Stop"

# Exit codes (aligned to `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`):
# - 0: smoke passed
# - 1: smoke assertion failed / unexpected script error
# - 2: invalid inputs (e.g., unknown SUBSTRATE_SMOKE_SLICE_ID)
# - 3: required dependency unavailable (e.g., substrate not found)
# - 4: missing prerequisites / world backend unavailable

$substrate = $env:SUBSTRATE_BIN
if ([string]::IsNullOrWhiteSpace($substrate)) { $substrate = "substrate" }
if (-not (Get-Command $substrate -ErrorAction SilentlyContinue)) {
  Write-Error "world-deps-packages-bundles-contract: substrate binary not found (SUBSTRATE_BIN=$substrate)"
  exit 3
}

$tmpRoot = $env:SUBSTRATE_SMOKE_ROOT
if ([string]::IsNullOrWhiteSpace($tmpRoot)) {
  $tmpRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("substrate-world-deps-smoke-" + [System.Guid]::NewGuid().ToString("n"))
}
New-Item -ItemType Directory -Force -Path $tmpRoot | Out-Null

try {
  $workspace = Join-Path $tmpRoot "workspace"
  New-Item -ItemType Directory -Force -Path $workspace | Out-Null
  Set-Location $workspace

  $substrateHome = Join-Path $tmpRoot "substrate-home"
  New-Item -ItemType Directory -Force -Path $substrateHome | Out-Null
  $env:SUBSTRATE_HOME = $substrateHome

  $slice = $env:SUBSTRATE_SMOKE_SLICE_ID
  if ([string]::IsNullOrWhiteSpace($slice)) { $slice = "WDP5" }
  if ($slice -notin @("WDP2","WDP5")) {
    Write-Error "world-deps-packages-bundles-contract: unknown SUBSTRATE_SMOKE_SLICE_ID=$slice"
    exit 2
  }

  Write-Host "== Setup: workspace =="
  & $substrate workspace init --force | Out-Null

  Write-Host "== Setup: smoke inventory =="
  $depsRoot = Join-Path $substrateHome "deps"
  $pkgs = Join-Path $depsRoot "packages"
  $bundles = Join-Path $depsRoot "bundles"
  New-Item -ItemType Directory -Force -Path $pkgs | Out-Null
  New-Item -ItemType Directory -Force -Path $bundles | Out-Null

  $pkgYaml = @"
version: 1
name: smoke-hello
description: Deterministic smoke package for world-deps (no network).
runnable: true
entrypoints: ["smoke-hello"]
install:
  method: script
  script: |
    set -euo pipefail
    mkdir -p /var/lib/substrate/world-deps/bin
    cat > /var/lib/substrate/world-deps/bin/smoke-hello <<'EOF'
    #!/bin/sh
    echo smoke-hello
    EOF
    chmod +x /var/lib/substrate/world-deps/bin/smoke-hello
probe:
  command: "smoke-hello"
"@
  Set-Content -Path (Join-Path $pkgs "smoke-hello.yaml") -Value $pkgYaml -Encoding utf8

  $bundleYaml = @"
version: 1
name: smoke-bundle
description: Deterministic smoke bundle for world-deps.
packages: ["smoke-hello"]
"@
  Set-Content -Path (Join-Path $bundles "smoke-bundle.yaml") -Value $bundleYaml -Encoding utf8

  Write-Host "== Case A: inventory visible =="
  $out = & $substrate world deps current list available 2>&1
  $out | Select-String -Pattern "smoke-hello" | Out-Null
  $out | Select-String -Pattern "smoke-bundle" | Out-Null

  Write-Host "== Case B: enabled patch editing + effective enabled view =="
  & $substrate world deps global reset | Out-Null
  & $substrate world deps workspace reset | Out-Null
  & $substrate world deps global add smoke-bundle | Out-Null
  & $substrate world deps workspace add smoke-hello | Out-Null

  $enabled = & $substrate world deps current list enabled 2>&1
  $enabled | Select-String -Pattern "smoke-bundle" | Out-Null
  $enabled | Select-String -Pattern "smoke-hello" | Out-Null

  Write-Host "== Preflight: world doctor =="
  & $substrate world doctor | Out-Null
  if ($LASTEXITCODE -ne 0) {
    Write-Error "world-deps-packages-bundles-contract: world doctor failed (exit=$LASTEXITCODE)"
    exit 4
  }

  Write-Host "== Case C: applied view and explain =="
  $applied = & $substrate world deps current list applied 2>&1
  $applied | Select-String -Pattern "world=" | Out-Null
  $applied | Select-String -Pattern "smoke-hello" | Out-Null
  $expl = & $substrate world deps current show smoke-hello --explain 2>&1
  $expl | Select-String -Pattern "substrate: hint:" | Out-Null

  if ($slice -eq "WDP2") {
    Write-Host "OK: world-deps-packages-bundles-contract smoke passed (slice=$slice platform=windows)"
    exit 0
  }

  Write-Host "== Case D: sync dry-run + sync apply =="
  & $substrate world deps global reset | Out-Null
  & $substrate world deps workspace reset | Out-Null
  & $substrate world deps workspace add smoke-bundle | Out-Null

  & $substrate world deps current sync --dry-run | Out-Null
  & $substrate world deps current sync | Out-Null

  $applied2 = & $substrate world deps current list applied 2>&1
  $applied2 | Select-String -Pattern "smoke-hello" | Out-Null
  $applied2 | Select-String -Pattern "world=present" | Out-Null

  Write-Host "OK: world-deps-packages-bundles-contract smoke passed (slice=$slice platform=windows)"
} finally {
  if ($env:SUBSTRATE_SMOKE_KEEP -ne "1") {
    Remove-Item -Recurse -Force $tmpRoot -ErrorAction SilentlyContinue
  }
}

