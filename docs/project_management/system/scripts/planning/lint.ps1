param(
    [Parameter(Mandatory = $true)]
    [string]$FeatureDir
)

$ErrorActionPreference = "Stop"

if (-not (Get-Command rg -ErrorAction SilentlyContinue)) {
    throw "FAIL: ripgrep (rg) is required for planning lint (install ripgrep and retry)"
}

if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
    throw "FAIL: git is required for planning lint"
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

function Require-Path([string]$Path) {
    if (-not (Test-Path -LiteralPath $Path)) {
        throw "Missing required path: $Path"
    }
}

function Require-AnyPath([string]$Label, [string[]]$Paths) {
    foreach ($p in $Paths) {
        if (Test-Path -LiteralPath $p) { return }
    }
    $bullets = ($Paths | ForEach-Object { "  - $_" }) -join "`n"
    throw "Missing required path ($Label); expected one of:`n$bullets"
}

Write-Host "== Planning lint: $FeatureDir =="

Require-Path $FeatureDir
Require-Path (Join-Path $FeatureDir "plan.md")
Require-Path (Join-Path $FeatureDir "tasks.json")
Require-Path (Join-Path $FeatureDir "session_log.md")
Require-Path (Join-Path $FeatureDir "kickoff_prompts")
Require-AnyPath "spec_manifest.md" @(
    (Join-Path $FeatureDir "pre-planning/spec_manifest.md"),
    (Join-Path $FeatureDir "spec_manifest.md")
)
Require-AnyPath "impact_map.md" @(
    (Join-Path $FeatureDir "pre-planning/impact_map.md"),
    (Join-Path $FeatureDir "impact_map.md")
)

$featureDirRel = & python (Join-Path $planningScriptsDir "pm_paths.py") resolve-feature-dir --feature-dir $FeatureDir
if ($LASTEXITCODE -ne 0) { throw "FAIL: could not normalize feature dir via pm_paths.py" }

$rootsJson = & python (Join-Path $planningScriptsDir "pm_paths.py") print-roots
if ($LASTEXITCODE -ne 0) { throw "FAIL: could not resolve PM roots via pm_paths.py" }
$roots = $rootsJson | ConvertFrom-Json
$pmRoot = [string]$roots.pm_root
$pmPacksRoot = [string]$roots.pm_packs_root

$pmPacksPrefix = ($pmPacksRoot.TrimEnd("/", "\\") + "/")

$schemaVersion = & jq -r '.meta.schema_version // 1' (Join-Path $FeatureDir "tasks.json")
$automationEnabled = & jq -r '.meta.automation.enabled // false' (Join-Path $FeatureDir "tasks.json")
$crossPlatformEnabled = & jq -r '.meta.cross_platform // false' (Join-Path $FeatureDir "tasks.json")

if ([int]$schemaVersion -ge 3 -and $automationEnabled -eq "true" -and $crossPlatformEnabled -eq "true") {
    Require-AnyPath "ci_checkpoint_plan.md" @(
        (Join-Path $FeatureDir "pre-planning/ci_checkpoint_plan.md"),
        (Join-Path $FeatureDir "ci_checkpoint_plan.md")
    )
}

if (Test-Path -LiteralPath (Join-Path $FeatureDir "smoke")) {
    $behaviorPlatforms = @()
    try {
        $behaviorPlatforms = @(& jq -r '[.meta.behavior_platforms_required // .meta.ci_parity_platforms_required // .meta.platforms_required // []] | flatten | .[]' (Join-Path $FeatureDir "tasks.json"))
    } catch {
        throw "FAIL: could not parse behavior platforms from tasks.json (requires jq): $_"
    }
    if (-not $behaviorPlatforms -or $behaviorPlatforms.Count -eq 0) {
        throw "FAIL: smoke/ exists but tasks.json meta is missing behavior platform declaration (expected meta.behavior_platforms_required, or legacy meta.platforms_required)"
    }

    foreach ($p in $behaviorPlatforms) {
        $p = $p.Trim()
        if (-not $p) { continue }
        switch ($p) {
            "linux"   { Require-Path (Join-Path $FeatureDir "smoke/linux-smoke.sh") }
            "macos"   { Require-Path (Join-Path $FeatureDir "smoke/macos-smoke.sh") }
            "windows" { Require-Path (Join-Path $FeatureDir "smoke/windows-smoke.ps1") }
            default   { throw "FAIL: invalid platform in behavior platforms: $p" }
        }
    }

    Write-Host "-- Smoke script scaffold scan"
    $scaffold = "Smoke script scaffold .*replace with feature checks"
    & rg -n --hidden --glob '!**/.git/**' $scaffold (Join-Path $FeatureDir "smoke")
    if ($LASTEXITCODE -eq 0) {
        throw "FAIL: smoke scripts still contain scaffolds; replace them with contract assertions (manual_testing_playbook.md should mirror these checks)"
    }
    if ($LASTEXITCODE -ne 1) { throw "rg failed with exit code $LASTEXITCODE" }
}

Write-Host "-- Hard-ban scan"
$hardBan = "\\b(TBD|TODO|WIP|TBA)\\b|open question|\\betc\\.|and so on"
& rg -n --hidden --glob '!**/.git/**' $hardBan $FeatureDir
if ($LASTEXITCODE -eq 0) { throw "FAIL: hard-ban matches found (remove these from planning outputs)" }
if ($LASTEXITCODE -ne 1) { throw "rg failed with exit code $LASTEXITCODE" }

Write-Host "-- Ambiguity scan"
$ambig = "\\b(should|could|might|maybe)\\b"
& rg -n --hidden --glob '!**/.git/**' --glob '!**/decision_register.md' --glob '!**/session_log.md' --glob '!**/quality_gate_report.md' --glob '!**/final_alignment_report.md' $ambig $FeatureDir
if ($LASTEXITCODE -eq 0) { throw "FAIL: ambiguity-word matches found (rewrite behavioral contracts)" }
if ($LASTEXITCODE -ne 1) { throw "rg failed with exit code $LASTEXITCODE" }

Write-Host "-- JSON validity"
& jq -e . (Join-Path $FeatureDir "tasks.json") *> $null
if ($LASTEXITCODE -ne 0) { throw "FAIL: tasks.json is not valid JSON" }

$sequencingCanonical = Join-Path $pmPacksRoot "sequencing.json"
$sequencingJson = $sequencingCanonical
if (-not (Test-Path -LiteralPath $sequencingJson)) {
    throw "FAIL: sequencing.json missing: $sequencingJson"
}

& jq -e . $sequencingJson *> $null
if ($LASTEXITCODE -ne 0) { throw "FAIL: sequencing.json is not valid JSON: $sequencingJson" }

Write-Host "-- tasks.json invariants"
& python (Join-Path $planningScriptsDir "validate_tasks_json.py") --feature-dir $FeatureDir
if ($LASTEXITCODE -ne 0) { throw "FAIL: tasks.json invariants failed" }

Write-Host "-- spec_manifest.md required-doc existence"
& python (Join-Path $planningScriptsDir "validate_spec_manifest.py") --feature-dir $FeatureDir
if ($LASTEXITCODE -ne 0) { throw "FAIL: spec_manifest.md required-doc existence failed" }

Write-Host "-- slice spec invariants (gated by meta.slice_spec_version)"
& python (Join-Path $planningScriptsDir "validate_slice_specs.py") --feature-dir $FeatureDir
if ($LASTEXITCODE -ne 0) { throw "FAIL: slice spec invariants failed" }

Write-Host "-- impact_map.md Touch Set validation (gated by meta.slice_spec_version)"
& python (Join-Path $planningScriptsDir "validate_impact_map.py") --feature-dir $FeatureDir
if ($LASTEXITCODE -ne 0) { throw "FAIL: impact_map Touch Set validation failed" }

if ($env:PM_LIFT_ADVISORY -eq "1") {
    Write-Host "-- Work Lift advisory report (PM_LIFT_ADVISORY=1)"
    & python (Join-Path $planningScriptsDir "pm_lift_report.py") --feature-dir $FeatureDir
    if ($LASTEXITCODE -ne 0) { throw "FAIL: Work Lift advisory report failed" }
}

if ([int]$schemaVersion -ge 3 -and $automationEnabled -eq "true" -and $crossPlatformEnabled -eq "true") {
    Write-Host "-- ci_checkpoint_plan.md invariants"
    & python (Join-Path $planningScriptsDir "validate_ci_checkpoint_plan.py") --feature-dir $FeatureDir
    if ($LASTEXITCODE -ne 0) { throw "FAIL: ci_checkpoint_plan.md invariants failed" }
}

Write-Host "-- ADR Executive Summary drift (if ADRs found/referenced)"
$adrPaths = @()

Get-ChildItem -LiteralPath $FeatureDir -Filter "ADR-*.md" -File -ErrorAction SilentlyContinue | ForEach-Object {
    $adrPaths += $_.FullName
}

$refs = & rg -o --no-filename --no-line-number --hidden --glob '!**/.git/**' 'docs/project_management/(next|adrs/[^ )"\r\n]+)/ADR-[^ )"\r\n]+\.md' $FeatureDir 2>$null
if ($LASTEXITCODE -ne 0 -and $LASTEXITCODE -ne 1) { throw "rg failed with exit code $LASTEXITCODE" }
if ($refs) {
    $adrPaths += ($refs | Sort-Object -Unique)
}

$adrPaths = $adrPaths | Sort-Object -Unique
if ($adrPaths.Count -gt 0) {
    foreach ($adr in $adrPaths) {
        if (-not (Test-Path -LiteralPath $adr)) {
            throw "Referenced ADR not found: $adr"
        }
        & python (Join-Path $planningScriptsDir "check_adr_exec_summary.py") --adr $adr
        if ($LASTEXITCODE -ne 0) { throw "FAIL: ADR executive summary drift: $adr" }
    }
} else {
    Write-Host "SKIP: no ADRs found/referenced"
}

Write-Host "-- Kickoff prompt sentinel"
$sentinel = "Do not edit planning docs inside the worktree\."
$missing = @()
Get-ChildItem -LiteralPath $FeatureDir -Recurse -Directory -ErrorAction SilentlyContinue | Where-Object { $_.Name -eq "kickoff_prompts" } | ForEach-Object {
    $kickoffDir = $_.FullName
    Get-ChildItem -LiteralPath $kickoffDir -Filter *.md -File -ErrorAction SilentlyContinue | Where-Object { $_.Name -ne "README.md" } | ForEach-Object {
        $content = Get-Content -LiteralPath $_.FullName -Raw
        if ($content -notmatch $sentinel) {
            $missing += ("Missing sentinel in kickoff prompt: " + $_.FullName)
        }
    }
}
if ($missing.Count -gt 0) {
    throw ($missing -join "`n")
}

Write-Host "-- Manual playbook smoke linkage (if present)"
$playbook = Join-Path $FeatureDir "manual_testing_playbook.md"
if (Test-Path -LiteralPath $playbook) {
    if (Test-Path -LiteralPath (Join-Path $FeatureDir "smoke")) {
        $behaviorPlatforms = @(& jq -r '[.meta.behavior_platforms_required // .meta.ci_parity_platforms_required // .meta.platforms_required // []] | flatten | .[]' (Join-Path $FeatureDir "tasks.json"))
        if (-not $behaviorPlatforms -or $behaviorPlatforms.Count -eq 0) {
            throw "FAIL: smoke/ exists but tasks.json meta is missing behavior platform declaration (expected meta.behavior_platforms_required, or legacy meta.platforms_required)"
        }

        foreach ($p in $behaviorPlatforms) {
            $p = $p.Trim()
            if (-not $p) { continue }
            switch ($p) {
                "linux"   { $smokeRef = "smoke/linux-smoke.sh" }
                "macos"   { $smokeRef = "smoke/macos-smoke.sh" }
                "windows" { $smokeRef = "smoke/windows-smoke.ps1" }
                default   { throw "FAIL: invalid platform in behavior platforms: $p" }
            }
            & rg -nF $smokeRef $playbook *> $null
            if ($LASTEXITCODE -ne 0) { throw "FAIL: manual_testing_playbook.md must reference required smoke script: $smokeRef" }
        }
    }
}

Write-Host "-- Sequencing alignment"
if ($featureDirRel.StartsWith($pmPacksPrefix)) {
    & jq -e --arg dir $featureDirRel '.sprints[] | select(.directory==$dir) | .id' $sequencingJson *> $null
    if ($LASTEXITCODE -ne 0) { throw "FAIL: sequencing.json missing sprint entry for $featureDirRel" }
} else {
    Write-Host "SKIP: sequencing alignment (feature dir not under PM roots)"
}

Write-Host "OK: planning lint passed"
