Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

param()

function Usage {
    @"
Usage:
  docs/project_management/system/scripts/planning/sync_sequencing_json.ps1

Copies the canonical sequencing spine:
  docs/project_management/packs/sequencing.json
to the legacy compatibility mirror:
  docs/project_management/next/sequencing.json

Notes:
  - No symlinks (Windows compatibility); the legacy file is a real copy.
  - Fails if the canonical file is missing.
"@ | Write-Host
}

if ($args -contains "-h" -or $args -contains "--help") {
    Usage
    exit 0
}

if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
    throw "FAIL: git is required"
}

$repoRoot = (& git -C $PSScriptRoot rev-parse --show-toplevel).Trim()
if (-not $repoRoot) { throw "FAIL: failed to locate repo root via git" }

$pmSystemRoot = $env:PM_SYSTEM_ROOT
if (-not $pmSystemRoot) { $pmSystemRoot = "docs/project_management/system" }
if (-not [System.IO.Path]::IsPathRooted($pmSystemRoot)) {
    $pmSystemRoot = Join-Path $repoRoot $pmSystemRoot
}
$planningScriptsDir = Join-Path $pmSystemRoot "scripts/planning"

Set-Location $repoRoot

$rootsJson = & python (Join-Path $planningScriptsDir "pm_paths.py") print-roots
if ($LASTEXITCODE -ne 0) { throw "FAIL: could not resolve PM roots via pm_paths.py" }
$roots = $rootsJson | ConvertFrom-Json

$pmRoot = [string]$roots.pm_root
$pmPacksRoot = [string]$roots.pm_packs_root

$canonical = Join-Path $pmPacksRoot "sequencing.json"
$legacy = Join-Path (Join-Path $pmRoot "next") "sequencing.json"

if (-not (Test-Path -LiteralPath $canonical)) {
    throw "FAIL: canonical sequencing.json missing: $canonical`nHint: expected packs root at $pmPacksRoot (override via PM_PACKS_ROOT)"
}

$legacyDir = Split-Path -Parent $legacy
if (-not (Test-Path -LiteralPath $legacyDir)) {
    New-Item -ItemType Directory -Path $legacyDir -Force | Out-Null
}

Copy-Item -LiteralPath $canonical -Destination $legacy -Force

function Validate-Json([string]$Path) {
    if (Get-Command jq -ErrorAction SilentlyContinue) {
        & jq -e . $Path *> $null
        if ($LASTEXITCODE -ne 0) { throw "FAIL: JSON is not valid: $Path" }
        return
    }

    $raw = Get-Content -LiteralPath $Path -Raw
    $null = $raw | ConvertFrom-Json
}

Validate-Json $canonical
Validate-Json $legacy

Write-Host "OK: synced $canonical -> $legacy"

