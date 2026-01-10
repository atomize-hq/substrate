param(
    [Parameter(Mandatory = $true)]
    [string]$FeatureDir
)

$ErrorActionPreference = "Stop"

function Require-Path([string]$Path) {
    if (-not (Test-Path -LiteralPath $Path)) {
        throw "Missing required path: $Path"
    }
}

Write-Host "== Planning lint: $FeatureDir =="

Require-Path $FeatureDir
Require-Path (Join-Path $FeatureDir "plan.md")
Require-Path (Join-Path $FeatureDir "tasks.json")
Require-Path (Join-Path $FeatureDir "session_log.md")
Require-Path (Join-Path $FeatureDir "kickoff_prompts")

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
& jq -e . "docs/project_management/next/sequencing.json" *> $null
if ($LASTEXITCODE -ne 0) { throw "FAIL: sequencing.json is not valid JSON" }

Write-Host "-- tasks.json invariants"
& python scripts/planning/validate_tasks_json.py --feature-dir $FeatureDir
if ($LASTEXITCODE -ne 0) { throw "FAIL: tasks.json invariants failed" }

Write-Host "-- ADR Executive Summary drift (if ADRs found/referenced)"
$adrPaths = @()

Get-ChildItem -LiteralPath $FeatureDir -Filter "ADR-*.md" -File -ErrorAction SilentlyContinue | ForEach-Object {
    $adrPaths += $_.FullName
}

$refs = & rg -o --no-filename --no-line-number --hidden --glob '!**/.git/**' 'docs/project_management/next/ADR-[^ )"\r\n]+\.md' $FeatureDir 2>$null
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
        & python scripts/planning/check_adr_exec_summary.py --adr $adr
        if ($LASTEXITCODE -ne 0) { throw "FAIL: ADR executive summary drift: $adr" }
    }
} else {
    Write-Host "SKIP: no ADRs found/referenced"
}

Write-Host "-- Kickoff prompt sentinel"
$sentinel = "Do not edit planning docs inside the worktree\."
$missing = @()
Get-ChildItem -LiteralPath (Join-Path $FeatureDir "kickoff_prompts") -Filter *.md | Where-Object { $_.Name -ne "README.md" } | ForEach-Object {
    $content = Get-Content -LiteralPath $_.FullName -Raw
    if ($content -notmatch $sentinel) {
        $missing += $_.FullName
    }
}
if ($missing.Count -gt 0) {
    throw ("Missing sentinel in kickoff prompts:`n" + ($missing -join "`n"))
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
& jq -e --arg dir $FeatureDir '.sprints[] | select(.directory==$dir) | .id' "docs/project_management/next/sequencing.json" *> $null
if ($LASTEXITCODE -ne 0) { throw "FAIL: sequencing.json missing sprint entry for $FeatureDir" }

Write-Host "OK: planning lint passed"
