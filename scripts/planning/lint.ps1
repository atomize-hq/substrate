param(
    [Parameter(Mandatory = $true)]
    [string]$FeatureDir
)

$ErrorActionPreference = "Stop"

$repoRoot = (& git rev-parse --show-toplevel).Trim()
if (-not $repoRoot) { throw "ERROR: failed to locate repo root via git" }

$pmSystemRoot = $env:PM_SYSTEM_ROOT
if (-not $pmSystemRoot) { $pmSystemRoot = "docs/project_management/system" }
if (-not [System.IO.Path]::IsPathRooted($pmSystemRoot)) {
    $pmSystemRoot = Join-Path $repoRoot $pmSystemRoot
}

$target = Join-Path $pmSystemRoot "scripts/planning/lint.ps1"
& $target -FeatureDir $FeatureDir
exit $LASTEXITCODE
