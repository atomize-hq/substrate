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
    Require-Path (Join-Path $FeatureDir "smoke/linux-smoke.sh")
    Require-Path (Join-Path $FeatureDir "smoke/macos-smoke.sh")
    Require-Path (Join-Path $FeatureDir "smoke/windows-smoke.ps1")
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
        & rg -n 'smoke/(linux-smoke\.sh|macos-smoke\.sh|windows-smoke\.ps1)' $playbook *> $null
        if ($LASTEXITCODE -ne 0) { throw "FAIL: manual_testing_playbook.md must reference smoke scripts" }
    }
}

Write-Host "-- Sequencing alignment"
& jq -e --arg dir $FeatureDir '.sprints[] | select(.directory==$dir) | .id' "docs/project_management/next/sequencing.json" *> $null
if ($LASTEXITCODE -ne 0) { throw "FAIL: sequencing.json missing sprint entry for $FeatureDir" }

Write-Host "OK: planning lint passed"
