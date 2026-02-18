param(
    [Parameter(Mandatory = $true)]
    [string]$Feature,
    [string]$SlicePrefix = "",
    [switch]$DecisionHeavy,
    [switch]$CrossPlatform,
    [string]$BehaviorPlatforms = "",
    [string]$CiParityPlatforms = "",
    [switch]$WslRequired,
    [switch]$WslSeparate,
    [switch]$Automation
)

$ErrorActionPreference = "Stop"

$repoRoot = (& git -C $PSScriptRoot rev-parse --show-toplevel).Trim()
if (-not $repoRoot) { throw "ERROR: failed to locate repo root via git" }
Set-Location $repoRoot
$pmRootsJson = (& python3 scripts/planning/pm_paths.py print-roots) | Out-String
$pmRoots = $pmRootsJson | ConvertFrom-Json
if (-not $pmRoots.pm_packs_root) { throw "ERROR: pm_paths.py print-roots returned empty pm_packs_root" }

$featureDir = "$($pmRoots.pm_packs_root)/active/$Feature"
$templatesDir = "docs/project_management/standards/templates"

if (Test-Path -LiteralPath $featureDir) {
    throw "Refusing to overwrite existing directory: $featureDir"
}

if ($WslSeparate.IsPresent -and -not $WslRequired.IsPresent) {
    throw "-WslSeparate requires -WslRequired"
}

if (($WslRequired.IsPresent -or $WslSeparate.IsPresent) -and -not $CrossPlatform.IsPresent) {
    throw "-WslRequired/-WslSeparate require -CrossPlatform"
}

$hasBehaviorPlatforms = -not [string]::IsNullOrWhiteSpace($BehaviorPlatforms)
$hasCiParityPlatforms = -not [string]::IsNullOrWhiteSpace($CiParityPlatforms)
if (($hasBehaviorPlatforms -or $hasCiParityPlatforms) -and -not $CrossPlatform.IsPresent) {
    throw "-BehaviorPlatforms/-CiParityPlatforms require -CrossPlatform"
}

$nowUtc = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")

function Derive-SlicePrefix([string]$Raw) {
    $stop = @("and", "or", "the", "a", "an", "of", "to", "for", "in", "on", "with", "via", "vs", "by")
    $genericTail = @("simplification", "refactor", "cleanup", "hardening", "migration", "parity", "stability", "integration", "reliability", "improvement", "improvements", "fix", "fixes", "update", "updates")

    $parts = $Raw -split "[-_.]"
    $words = @()
    foreach ($p in $parts) {
        $w = ($p.ToLowerInvariant() -replace "[^a-z0-9]", "")
        if ([string]::IsNullOrWhiteSpace($w)) { continue }
        if ($stop -contains $w) { continue }
        $words += $w
    }

    if ($words.Count -eq 0) { return "X" }

    $last = $words[$words.Count - 1]
    if (($genericTail -contains $last) -and $words.Count -ge 3) {
        return (($words[0].Substring(0, 1) + $words[1].Substring(0, 1) + $words[2].Substring(0, 1)).ToUpperInvariant())
    }

    if ($words.Count -ge 3) {
        return (($words[0].Substring(0, 1) + $words[1].Substring(0, 1) + $last.Substring(0, 1)).ToUpperInvariant())
    }
    if ($words.Count -eq 2) {
        return (($words[0].Substring(0, 1) + $words[1].Substring(0, 1)).ToUpperInvariant())
    }
    return ($words[0].Substring(0, [Math]::Min(3, $words[0].Length)).ToUpperInvariant())
}

$slicePrefixResolved = $SlicePrefix
if ([string]::IsNullOrWhiteSpace($slicePrefixResolved)) {
    $slicePrefixResolved = Derive-SlicePrefix $Feature
}
if ($slicePrefixResolved -notmatch '^[A-Za-z][A-Za-z0-9]*$') {
    throw "Invalid -SlicePrefix: $slicePrefixResolved (expected alnum, starting with a letter)"
}

$script:SlicePrefix = $slicePrefixResolved
$script:SliceId = "$slicePrefixResolved" + "0"
$script:SliceIdLower = $script:SliceId.ToLowerInvariant()
$script:SliceSpecFile = "$($script:SliceId)-spec.md"
$script:SliceCloseoutFile = "$($script:SliceId)-closeout_report.md"

function Render-Template([string]$TemplatePath, [string]$OutPath, [hashtable]$Vars) {
    $text = Get-Content -LiteralPath $TemplatePath -Raw
    foreach ($k in $Vars.Keys) {
        $text = $text.Replace("{{${k}}}", $Vars[$k])
    }
    New-Item -ItemType Directory -Force -Path (Split-Path -Parent $OutPath) | Out-Null
    Set-Content -LiteralPath $OutPath -Value $text -NoNewline
}

$vars = @{
    FEATURE     = $Feature
    FEATURE_DIR = $featureDir
    NOW_UTC     = $nowUtc
    ORCH_BRANCH = "feat/$Feature"
    SLICE_PREFIX = $script:SlicePrefix
    SLICE_ID     = $script:SliceId
}

New-Item -ItemType Directory -Force -Path (Join-Path $featureDir "kickoff_prompts") | Out-Null

Render-Template (Join-Path $templatesDir "plan.md.tmpl") (Join-Path $featureDir "plan.md") $vars
Render-Template (Join-Path $templatesDir "session_log.md.tmpl") (Join-Path $featureDir "session_log.md") $vars
Render-Template (Join-Path $templatesDir "contract.md.tmpl") (Join-Path $featureDir "contract.md") $vars
Render-Template (Join-Path $templatesDir "spec_manifest.md.tmpl") (Join-Path $featureDir "spec_manifest.md") $vars
Render-Template (Join-Path $templatesDir "impact_map.md.tmpl") (Join-Path $featureDir "impact_map.md") $vars
if ($CrossPlatform.IsPresent -and $Automation.IsPresent) {
    Render-Template (Join-Path $templatesDir "ci_checkpoint_plan.md.tmpl") (Join-Path $featureDir "ci_checkpoint_plan.md") $vars
}

$varsCp1 = $vars.Clone()
$varsCp1["TASK_ID"] = "CP1-ci-checkpoint"
if ($CrossPlatform.IsPresent -and $Automation.IsPresent) {
    Render-Template (Join-Path $templatesDir "kickoff_ci_checkpoint.md.tmpl") (Join-Path $featureDir "kickoff_prompts/CP1-ci-checkpoint.md") $varsCp1
}

Render-Template (Join-Path $templatesDir "slice_spec.v2.md.tmpl") (Join-Path $featureDir $script:SliceSpecFile) $vars

function New-TaskBase([string]$Id, [string]$Name, [string]$Type, [string]$Description) {
    $refs = @("$featureDir/plan.md", "$featureDir/spec_manifest.md", "$featureDir/impact_map.md", "$featureDir/$($script:SliceSpecFile)")
    if ($CrossPlatform.IsPresent -and $Automation.IsPresent) {
        $refs = @("$featureDir/plan.md", "$featureDir/spec_manifest.md", "$featureDir/impact_map.md", "$featureDir/ci_checkpoint_plan.md", "$featureDir/$($script:SliceSpecFile)")
    }
    return @{
        id                 = $Id
        name               = $Name
        type               = $Type
        phase              = $script:SliceId
        status             = "pending"
        description        = $Description
        references         = $refs
        acceptance_criteria = @()
        start_checklist    = @()
        end_checklist      = @()
        worktree           = $null
        integration_task   = $null
        kickoff_prompt     = "$featureDir/kickoff_prompts/$Id.md"
        depends_on         = @()
        concurrent_with    = @()
        git_branch         = $null
        required_make_targets = $null
    }
}

$allowedRequiredPlatforms = @("linux", "macos", "windows")

function Parse-PlatformCsv([string]$Raw, [string]$FieldName) {
    if ([string]::IsNullOrWhiteSpace($Raw)) {
        return @()
    }
    $parts = $Raw.Split(",") | ForEach-Object { $_.Trim() } | Where-Object { $_ -ne "" }

    $unknown = $parts | Where-Object { $allowedRequiredPlatforms -notcontains $_ } | Sort-Object -Unique
    if ($unknown.Count -gt 0) {
        throw "Invalid $FieldName platform(s): $($unknown -join ', ') (allowed: $($allowedRequiredPlatforms -join ', '))"
    }

    $duplicates = $parts | Group-Object | Where-Object { $_.Count -gt 1 } | ForEach-Object { $_.Name } | Sort-Object -Unique
    if ($duplicates.Count -gt 0) {
        throw "Duplicate $FieldName platform(s): $($duplicates -join ', ')"
    }

    return @($parts)
}

$ciParityPlatformsList = @()
$behaviorPlatformsList = @()
if ($CrossPlatform.IsPresent) {
    $ciParityPlatformsList = if ($hasCiParityPlatforms) { Parse-PlatformCsv $CiParityPlatforms "ci_parity_platforms_required" } else { @("linux", "macos", "windows") }
    $behaviorPlatformsList = if ($hasBehaviorPlatforms) { Parse-PlatformCsv $BehaviorPlatforms "behavior_platforms_required" } else { @($ciParityPlatformsList) }
}

if ($WslRequired.IsPresent -and -not ($behaviorPlatformsList -contains "linux")) {
    throw "-WslRequired requires linux in -BehaviorPlatforms (behavior platform set)"
}

$schemaVersion = if ($Automation.IsPresent -and $CrossPlatform.IsPresent) { 4 } elseif ($Automation.IsPresent) { 3 } elseif ($CrossPlatform.IsPresent) { 2 } else { 1 }
$meta = @{
    schema_version = $schemaVersion
    feature        = $Feature
    cross_platform = [bool]$CrossPlatform.IsPresent
    execution_gates = $true
    slice_spec_version = 2
}

if ($Automation.IsPresent) {
    $meta.automation = @{
        enabled = $true
        orchestration_branch = "feat/$Feature"
    }
}

if ($CrossPlatform.IsPresent) {
    $meta.behavior_platforms_required = @($behaviorPlatformsList)
    $meta.ci_parity_platforms_required = @($ciParityPlatformsList)
    if ($WslRequired.IsPresent) {
        $meta.wsl_required = $true
        $meta.wsl_task_mode = if ($WslSeparate.IsPresent) { "separate" } else { "bundled" }
    }
}

# Schema v4 cross-platform packs require explicit boundary markers.
if ($Automation.IsPresent -and $CrossPlatform.IsPresent) {
    $meta.checkpoint_boundaries = @($script:SliceId)
}

$tasks = @()

$tasks += @{
    id = "F0-exec-preflight"
    name = "Execution preflight gate (feature start)"
    type = "ops"
    phase = "F0"
    status = "pending"
    description = "Run the execution preflight gate to confirm smoke/manual/CI plans are adequate before starting triads."
    references = @(
        "$featureDir/plan.md",
        "$featureDir/spec_manifest.md",
        "$featureDir/impact_map.md",
        "$featureDir/tasks.json",
        "$featureDir/session_log.md",
        "$featureDir/execution_preflight_report.md"
    )
    acceptance_criteria = @("Execution preflight recommendation recorded (ACCEPT or REVISE)")
    start_checklist = @(
        $(if ($Automation.IsPresent) { "Run: make triad-orch-ensure FEATURE_DIR=`"$featureDir`"" } else { "git checkout feat/$Feature && git pull --ff-only" }),
        "Read plan.md, tasks.json, session_log.md, specs, kickoff prompt",
        "Set status to in_progress; add START entry; commit docs"
    )
    end_checklist = @(
        "Complete execution_preflight_report.md with ACCEPT/REVISE and required fixes",
        "Set status to completed; add END entry; commit docs"
    )
    worktree = $null
    integration_task = $null
    kickoff_prompt = "$featureDir/kickoff_prompts/F0-exec-preflight.md"
    depends_on = @()
    concurrent_with = @()
}

$codeId = "$($script:SliceId)-code"
$testId = "$($script:SliceId)-test"
$integId = "$($script:SliceId)-integ"
$integCoreId = "$($script:SliceId)-integ-core"
$seedAcIds = @("AC-$($script:SliceId)-01", "AC-$($script:SliceId)-02", "AC-$($script:SliceId)-03")

$code = New-TaskBase $codeId "$($script:SliceId) slice (code)" "code" "Implement $($script:SliceId) spec (production code only)."
$code.ac_ids = $seedAcIds
$code.acceptance_criteria = @("Implements the behaviors required by ac_ids (see $($script:SliceSpecFile))")
$code.start_checklist = @(
    "git checkout feat/$Feature && git pull --ff-only",
    "Read plan.md, tasks.json, session_log.md, $($script:SliceSpecFile), kickoff prompt",
    "Set status to in_progress; add START entry; commit docs",
    $(if ($Automation.IsPresent) { "Run: make triad-task-start-pair FEATURE_DIR=`"$featureDir`" SLICE_ID=`"$($script:SliceId)`"" } else { "Run: git worktree add -b $($script:SliceIdLower)-code wt/$Feature-$($script:SliceIdLower)-code feat/$Feature" })
)
$code.end_checklist = @(
    "cargo fmt",
    "cargo clippy --workspace --all-targets -- -D warnings",
    $(if ($Automation.IsPresent) { "From inside the worktree: make triad-task-finish TASK_ID=`"$codeId`"" } else { "From inside the worktree: git add -A && git commit -m `"code: $Feature $codeId`"" }),
    $(if ($Automation.IsPresent) { "Update tasks/session_log on orchestration branch; do not delete worktrees (feature cleanup removes worktrees at feature end)" } else { "Update tasks/session_log on the orchestration branch; optionally remove the worktree when done: git worktree remove wt/$Feature-$($script:SliceIdLower)-code (per plan.md)" })
)
$code.worktree = "wt/$Feature-$($script:SliceIdLower)-code"
$code.git_branch = if ($Automation.IsPresent) { "$Feature-$($script:SliceIdLower)-code" } else { "$($script:SliceIdLower)-code" }
$code.required_make_targets = if ($Automation.IsPresent) { @("triad-code-checks") } else { $null }
$code.integration_task = if ($CrossPlatform.IsPresent) { $integCoreId } else { $integId }
$code.kickoff_prompt = "$featureDir/kickoff_prompts/$codeId.md"
$code.depends_on = @("F0-exec-preflight")
$code.concurrent_with = @($testId)
$tasks += $code

$test = New-TaskBase $testId "$($script:SliceId) slice (test)" "test" "Add/modify tests for $($script:SliceId) spec (tests only)."
$test.ac_ids = $seedAcIds
$test.acceptance_criteria = @("Tests enforce the behaviors required by ac_ids (see $($script:SliceSpecFile))")
$test.start_checklist = @(
    "git checkout feat/$Feature && git pull --ff-only",
    "Read plan.md, tasks.json, session_log.md, $($script:SliceSpecFile), kickoff prompt",
    "Set status to in_progress; add START entry; commit docs",
    $(if ($Automation.IsPresent) { "Run: make triad-task-start-pair FEATURE_DIR=`"$featureDir`" SLICE_ID=`"$($script:SliceId)`"" } else { "Run: git worktree add -b $($script:SliceIdLower)-test wt/$Feature-$($script:SliceIdLower)-test feat/$Feature" })
)
$test.end_checklist = @(
    "cargo fmt",
    "Run the targeted tests you add/touch",
    $(if ($Automation.IsPresent) { "From inside the worktree: make triad-task-finish TASK_ID=`"$testId`"" } else { "From inside the worktree: git add -A && git commit -m `"test: $Feature $testId`"" }),
    $(if ($Automation.IsPresent) { "Update tasks/session_log on orchestration branch; do not delete worktrees (feature cleanup removes worktrees at feature end)" } else { "Update tasks/session_log on the orchestration branch; optionally remove the worktree when done: git worktree remove wt/$Feature-$($script:SliceIdLower)-test (per plan.md)" })
)
$test.worktree = "wt/$Feature-$($script:SliceIdLower)-test"
$test.git_branch = if ($Automation.IsPresent) { "$Feature-$($script:SliceIdLower)-test" } else { "$($script:SliceIdLower)-test" }
$test.required_make_targets = if ($Automation.IsPresent) { @("triad-test-checks") } else { $null }
$test.integration_task = if ($CrossPlatform.IsPresent) { $integCoreId } else { $integId }
$test.kickoff_prompt = "$featureDir/kickoff_prompts/$testId.md"
$test.depends_on = @("F0-exec-preflight")
$test.concurrent_with = @($codeId)
$tasks += $test

if ($CrossPlatform.IsPresent) {
    $core = New-TaskBase $integCoreId "$($script:SliceId) slice (integration core)" "integration" "Merge $($script:SliceId) code+tests and make the slice green on the primary dev platform."
    $core.integration_task = $integCoreId
    $core.acceptance_criteria = @("Core slice is green under make integ-checks and matches the spec")
    foreach ($p in $behaviorPlatformsList) {
        switch ($p) {
            "linux" { $core.references += @("$featureDir/smoke/linux-smoke.sh") }
            "macos" { $core.references += @("$featureDir/smoke/macos-smoke.sh") }
            "windows" { $core.references += @("$featureDir/smoke/windows-smoke.ps1") }
        }
    }
    $core.start_checklist = @(
        "git checkout feat/$Feature && git pull --ff-only",
        "Read plan.md, tasks.json, session_log.md, $($script:SliceSpecFile), kickoff prompt",
        "Set status to in_progress; add START entry; commit docs",
        $(if ($Automation.IsPresent) { "Run: make triad-task-start FEATURE_DIR=`"$featureDir`" TASK_ID=`"$integCoreId`"" } else { "Run: git worktree add -b $($script:SliceIdLower)-integ-core wt/$Feature-$($script:SliceIdLower)-integ-core feat/$Feature" })
    )
    $core.end_checklist = @(
        "cargo fmt",
        "cargo clippy --workspace --all-targets -- -D warnings",
        "Run relevant tests",
        "make integ-checks"
    ) + @(
        "If this slice ends a CI checkpoint group: run the checkpoint task (for example, CP1-ci-checkpoint) from the orchestration checkout per $featureDir/ci_checkpoint_plan.md",
        $(if ($Automation.IsPresent) { "From inside the worktree: make triad-task-finish TASK_ID=`"$integCoreId`"" } else { "From inside the worktree: git add -A && git commit -m `"integ: $Feature $integCoreId`"" }),
        $(if ($Automation.IsPresent) { "Update tasks/session_log on orchestration branch; do not delete worktrees (feature cleanup removes worktrees at feature end)" } else { "Update tasks/session_log on the orchestration branch; optionally remove the worktree when done: git worktree remove wt/$Feature-$($script:SliceIdLower)-integ-core (per plan.md)" })
    )
    $core.worktree = "wt/$Feature-$($script:SliceIdLower)-integ-core"
    $core.git_branch = if ($Automation.IsPresent) { "$Feature-$($script:SliceIdLower)-integ-core" } else { "$($script:SliceIdLower)-integ-core" }
    $core.required_make_targets = if ($Automation.IsPresent) { @("integ-checks") } else { $null }
    $core.merge_to_orchestration = if ($Automation.IsPresent) { $false } else { $null }
    $core.kickoff_prompt = "$featureDir/kickoff_prompts/$integCoreId.md"
    $core.depends_on = @($codeId, $testId)
    $tasks += $core

    $platforms = @($ciParityPlatformsList)
    if ($WslRequired.IsPresent -and $WslSeparate.IsPresent) { $platforms += "wsl" }

    foreach ($platform in $platforms) {
        $id = "$($script:SliceId)-integ-$platform"
        $smokeRequired = ($behaviorPlatformsList -contains $platform) -or ($platform -eq "wsl")
        switch ($platform) {
            "linux" {
                $name = "$($script:SliceId) slice (integration Linux)"
                $desc = "Linux platform-fix integration task (may be a no-op if already green)."
                $refs = @("$featureDir/smoke/linux-smoke.sh")
                $dispatch = "scripts/ci/dispatch_feature_smoke.sh --feature-dir `"$featureDir`" --runner-kind self-hosted --platform linux --workflow-ref `\"feat/$Feature`\""
                if ($WslRequired.IsPresent -and -not $WslSeparate.IsPresent) { $dispatch += " --run-wsl" }
                $dispatch += " --cleanup"
            }
            "macos" {
                $name = "$($script:SliceId) slice (integration macOS)"
                $desc = "macOS platform-fix integration task (may be a no-op if already green)."
                $refs = @("$featureDir/smoke/macos-smoke.sh")
                $dispatch = "scripts/ci/dispatch_feature_smoke.sh --feature-dir `"$featureDir`" --runner-kind self-hosted --platform macos --workflow-ref `\"feat/$Feature`\" --cleanup"
            }
            "windows" {
                $name = "$($script:SliceId) slice (integration Windows)"
                $desc = "Windows platform-fix integration task (may be a no-op if already green)."
                $refs = @("$featureDir/smoke/windows-smoke.ps1")
                $dispatch = "scripts/ci/dispatch_feature_smoke.sh --feature-dir `"$featureDir`" --runner-kind self-hosted --platform windows --workflow-ref `\"feat/$Feature`\" --cleanup"
            }
            "wsl" {
                $name = "$($script:SliceId) slice (integration WSL)"
                $desc = "WSL platform-fix integration task (Linux-in-WSL)."
                $refs = @("$featureDir/smoke/linux-smoke.sh")
                $dispatch = "scripts/ci/dispatch_feature_smoke.sh --feature-dir `"$featureDir`" --runner-kind self-hosted --platform wsl --workflow-ref `\"feat/$Feature`\" --cleanup"
            }
        }

        if (-not $smokeRequired) {
            $name = "$($script:SliceId) slice (integration CI parity: $platform)"
            $desc = "$platform CI parity fix task (compile/test/lint only; no behavioral smoke required for this platform)."
            $refs = @()
            $dispatch = "make ci-compile-parity CI_WORKFLOW_REF=`\"feat/$Feature`\" CI_REMOTE=origin CI_CLEANUP=1"
        }
        $workflow = if ($smokeRequired) { ".github/workflows/feature-smoke.yml" } else { ".github/workflows/ci-compile-parity.yml" }

        $t = New-TaskBase $id $name "integration" $desc
        $t.integration_task = $id
        $t.acceptance_criteria = @($(if ($smokeRequired) { "$platform smoke is green for this slice" } else { "$platform CI parity is green for this slice (no behavioral smoke required)" }))
        $t.references += $refs
        $t.start_checklist = @(
            "Run on $platform host if possible",
            "git checkout feat/$Feature && git pull --ff-only",
            "Read plan.md, tasks.json, session_log.md, $($script:SliceSpecFile), kickoff prompt",
            "Set status to in_progress; add START entry; commit docs",
            $(if ($Automation.IsPresent) { "Run: make triad-task-start FEATURE_DIR=`"$featureDir`" TASK_ID=`"$id`"" } else { "Run: git worktree add -b $($script:SliceIdLower)-integ-$platform wt/$Feature-$($script:SliceIdLower)-integ-$platform feat/$Feature" })
        )
        $t.end_checklist = @(
            $(if ($smokeRequired) { "Dispatch platform smoke via CI: $dispatch" } else { "Dispatch CI parity via: $dispatch" }),
            "If needed: fix + fmt/clippy + targeted tests",
            $(if ($smokeRequired) { "Ensure smoke is green; record run id/URL" } else { "Ensure CI parity is green; record run id/URL" }),
            $(if ($Automation.IsPresent) { "From inside the worktree: make triad-task-finish TASK_ID=`"$id`"" } else { "From inside the worktree: git add -A && git commit -m `"integ: $Feature $id`"" }),
            $(if ($Automation.IsPresent) { "Update tasks/session_log on orchestration branch; do not delete worktrees (feature cleanup removes worktrees at feature end)" } else { "Update tasks/session_log on the orchestration branch; optionally remove the worktree when done: git worktree remove wt/$Feature-$($script:SliceIdLower)-integ-$platform (per plan.md)" })
        )
        $t.worktree = "wt/$Feature-$($script:SliceIdLower)-integ-$platform"
        $t.git_branch = if ($Automation.IsPresent) { "$Feature-$($script:SliceIdLower)-integ-$platform" } else { "$($script:SliceIdLower)-integ-$platform" }
        $t.required_make_targets = if ($Automation.IsPresent) { @("triad-code-checks") } else { $null }
        $t.merge_to_orchestration = if ($Automation.IsPresent) { $false } else { $null }
        $t.kickoff_prompt = "$featureDir/kickoff_prompts/$id.md"
        $t.depends_on = @($integCoreId)
        $t.platform = $platform
        $t.runner = "github-actions"
        $t.workflow = $workflow
        $tasks += $t
    }

    $final = New-TaskBase $integId "$($script:SliceId) slice (integration final)" "integration" "Final integration: merge any platform fixes, complete slice closeout, and confirm checkpoint evidence is recorded."
    $final.integration_task = $integId
    $final.ac_ids = $seedAcIds
    $final.acceptance_criteria = @("Slice closeout report completed and local integration gates are green (implements behaviors required by ac_ids; see $($script:SliceSpecFile))")
    foreach ($p in $behaviorPlatformsList) {
        switch ($p) {
            "linux" { $final.references += @("$featureDir/smoke/linux-smoke.sh") }
            "macos" { $final.references += @("$featureDir/smoke/macos-smoke.sh") }
            "windows" { $final.references += @("$featureDir/smoke/windows-smoke.ps1") }
        }
    }
    $final.references += @("$featureDir/$($script:SliceCloseoutFile)")
    $final.start_checklist = @(
        "git checkout feat/$Feature && git pull --ff-only",
        "Read plan.md, tasks.json, session_log.md, $($script:SliceSpecFile), kickoff prompt",
        "Set status to in_progress; add START entry; commit docs",
        $(if ($Automation.IsPresent) { "Run: make triad-task-start FEATURE_DIR=`"$featureDir`" TASK_ID=`"$integId`"" } else { "Run: git worktree add -b $($script:SliceIdLower)-integ wt/$Feature-$($script:SliceIdLower)-integ feat/$Feature" })
    )
    $final.end_checklist = @(
        "Merge platform-fix branches (if any) + resolve conflicts",
        "cargo fmt",
        "cargo clippy --workspace --all-targets -- -D warnings",
        "Run relevant tests",
        "make integ-checks"
    ) + @(
        "Confirm required CI checkpoint tasks that cover this slice are completed and recorded in $featureDir/session_log.md",
        "Complete slice closeout gate report: $featureDir/$($script:SliceCloseoutFile)",
        $(if ($Automation.IsPresent) { "From inside the worktree: make triad-task-finish TASK_ID=`"$integId`"" } else { "From inside the worktree: git add -A && git commit -m `"integ: $Feature $integId`"" }),
        $(if ($Automation.IsPresent) { "Update tasks/session_log on orchestration branch; do not delete worktrees (feature cleanup removes worktrees at feature end)" } else { "Update tasks/session_log on the orchestration branch; optionally remove the worktree when done: git worktree remove wt/$Feature-$($script:SliceIdLower)-integ (per plan.md)" })
    )
    $final.worktree = "wt/$Feature-$($script:SliceIdLower)-integ"
    $final.git_branch = if ($Automation.IsPresent) { "$Feature-$($script:SliceIdLower)-integ" } else { "$($script:SliceIdLower)-integ" }
    $final.required_make_targets = if ($Automation.IsPresent) { @("integ-checks") } else { $null }
    $final.merge_to_orchestration = if ($Automation.IsPresent) { $true } else { $null }
    $final.kickoff_prompt = "$featureDir/kickoff_prompts/$integId.md"
    $final.depends_on = @($integCoreId) + ($platforms | ForEach-Object { "$($script:SliceId)-integ-$_" })
    $tasks += $final

    if ($Automation.IsPresent) {
        $tasks += @{
            id = "CP1-ci-checkpoint"
            name = "CI checkpoint (initial scaffold)"
            type = "ops"
            phase = "CP1"
            status = "pending"
            description = "Run cross-platform CI gates at the checkpoint boundary defined in ci_checkpoint_plan.md."
            references = @(
                "$featureDir/ci_checkpoint_plan.md",
                "$featureDir/impact_map.md",
                "$featureDir/tasks.json",
                "$featureDir/session_log.md"
            )
            acceptance_criteria = @("Checkpoint CI gates executed or skipped per ci-audit, with evidence recorded in session_log.md")
            start_checklist = @(
                "Run: make triad-orch-ensure FEATURE_DIR=`"$featureDir`"",
                "Read ci_checkpoint_plan.md and confirm which slice id this checkpoint validates",
                "Set status to in_progress; add START entry; commit docs"
            )
            end_checklist = @(
                "Run compile parity + behavioral smoke per ci_checkpoint_plan.md (use ci-audit to skip redundant dispatch)",
                "Record run ids/URLs and ci-audit output lines in session_log.md",
                "Set status to completed; add END entry; commit docs"
            )
            worktree = $null
            integration_task = $null
            kickoff_prompt = "$featureDir/kickoff_prompts/CP1-ci-checkpoint.md"
            depends_on = @($integCoreId)
            concurrent_with = @()
        }
    }
} else {
    $integ = New-TaskBase $integId "$($script:SliceId) slice (integration)" "integration" "Integrate $($script:SliceId) code+tests, reconcile to spec, and run integration gate."
    $integ.integration_task = $integId
    $integ.ac_ids = $seedAcIds
    $integ.acceptance_criteria = @("Slice is green under make integ-checks and implements behaviors required by ac_ids (see $($script:SliceSpecFile))")
    $integ.references += @("$featureDir/$($script:SliceCloseoutFile)")
    $integ.start_checklist = @(
        "git checkout feat/$Feature && git pull --ff-only",
        "Read plan.md, tasks.json, session_log.md, $($script:SliceSpecFile), kickoff prompt",
        "Set status to in_progress; add START entry; commit docs",
        $(if ($Automation.IsPresent) { "Run: make triad-task-start FEATURE_DIR=`"$featureDir`" TASK_ID=`"$integId`"" } else { "Run: git worktree add -b $($script:SliceIdLower)-integ wt/$Feature-$($script:SliceIdLower)-integ feat/$Feature" })
    )
    $integ.end_checklist = @(
        "cargo fmt",
        "cargo clippy --workspace --all-targets -- -D warnings",
        "Run relevant tests",
        "make integ-checks",
        "Complete slice closeout gate report: $featureDir/$($script:SliceCloseoutFile)",
        $(if ($Automation.IsPresent) { "From inside the worktree: make triad-task-finish TASK_ID=`"$integId`"" } else { "From inside the worktree: git add -A && git commit -m `"integ: $Feature $integId`"" }),
        $(if ($Automation.IsPresent) { "Update tasks/session_log on orchestration branch; do not delete worktrees (feature cleanup removes worktrees at feature end)" } else { "Update tasks/session_log on the orchestration branch; optionally remove the worktree when done: git worktree remove wt/$Feature-$($script:SliceIdLower)-integ (per plan.md)" })
    )
    $integ.worktree = "wt/$Feature-$($script:SliceIdLower)-integ"
    $integ.git_branch = if ($Automation.IsPresent) { "$Feature-$($script:SliceIdLower)-integ" } else { "$($script:SliceIdLower)-integ" }
    $integ.required_make_targets = if ($Automation.IsPresent) { @("integ-checks") } else { $null }
    $integ.merge_to_orchestration = if ($Automation.IsPresent) { $true } else { $null }
    $integ.kickoff_prompt = "$featureDir/kickoff_prompts/$integId.md"
    $integ.depends_on = @($codeId, $testId)
    $tasks += $integ
}

if ($Automation.IsPresent) {
    $tasks += @{
        id = "FZ-feature-cleanup"
        name = "Feature cleanup (worktrees + branches)"
        type = "ops"
        phase = "FZ"
        status = "pending"
        description = "At feature end, remove retained worktrees and optionally prune task branches via scripts/triad/feature_cleanup.sh."
        references = @(
            "$featureDir/plan.md",
            "$featureDir/tasks.json",
            "$featureDir/session_log.md",
            "scripts/triad/feature_cleanup.sh"
        )
        acceptance_criteria = @("Cleanup completed and summary recorded in session_log.md")
        start_checklist = @(
            "git checkout feat/$Feature && git pull --ff-only",
            "Confirm all tasks are completed and merged as intended",
            "Set status to in_progress; add START entry; commit docs"
        )
        end_checklist = @(
            "Run: make triad-feature-cleanup FEATURE_DIR=`"$featureDir`" DRY_RUN=1 REMOVE_WORKTREES=1 PRUNE_LOCAL=1",
            "Then run: make triad-feature-cleanup FEATURE_DIR=`"$featureDir`" REMOVE_WORKTREES=1 PRUNE_LOCAL=1",
            "Set status to completed; add END entry with script summary; commit docs"
        )
        worktree = $null
        integration_task = $null
        kickoff_prompt = "$featureDir/kickoff_prompts/FZ-feature-cleanup.md"
        depends_on = @($integId)
        concurrent_with = @()
    }
}

$data = @{ meta = $meta; tasks = $tasks }
$data | ConvertTo-Json -Depth 20 | Set-Content -LiteralPath (Join-Path $featureDir "tasks.json") -NoNewline

$varsExec = $vars.Clone()
Render-Template (Join-Path $templatesDir "execution_preflight_report.md.tmpl") (Join-Path $featureDir "execution_preflight_report.md") $varsExec

$varsCloseout = $vars.Clone()
$varsCloseout["SLICE_ID"] = $script:SliceId
$varsCloseout["SPEC_FILE"] = $script:SliceSpecFile
Render-Template (Join-Path $templatesDir "slice_closeout_report.md.tmpl") (Join-Path $featureDir $script:SliceCloseoutFile) $varsCloseout

<# Legacy tasks.json generator removed (kept for history). #>

function Render-Kickoff([string]$Template, [string]$OutFile, [string]$TaskId, [string]$Branch, [string]$Worktree, [string]$Platform = "") {
    $vars2 = $vars.Clone()
    $vars2["TASK_ID"] = $TaskId
    $vars2["SPEC_FILE"] = $script:SliceSpecFile
    $vars2["BRANCH"] = $Branch
    $vars2["WORKTREE"] = $Worktree
    $vars2["PLATFORM"] = $Platform
    Render-Template (Join-Path $templatesDir $Template) (Join-Path $featureDir "kickoff_prompts/$OutFile") $vars2
}

$sliceCodeBranch = if ($Automation.IsPresent) { "$Feature-$($script:SliceIdLower)-code" } else { "$($script:SliceIdLower)-code" }
$sliceTestBranch = if ($Automation.IsPresent) { "$Feature-$($script:SliceIdLower)-test" } else { "$($script:SliceIdLower)-test" }
$sliceIntegBranch = if ($Automation.IsPresent) { "$Feature-$($script:SliceIdLower)-integ" } else { "$($script:SliceIdLower)-integ" }
$sliceIntegCoreBranch = if ($Automation.IsPresent) { "$Feature-$($script:SliceIdLower)-integ-core" } else { "$($script:SliceIdLower)-integ-core" }

Render-Kickoff "kickoff_code.md.tmpl" "$codeId.md" $codeId $sliceCodeBranch "wt/$Feature-$($script:SliceIdLower)-code"
Render-Kickoff "kickoff_test.md.tmpl" "$testId.md" $testId $sliceTestBranch "wt/$Feature-$($script:SliceIdLower)-test"
Render-Kickoff "kickoff_exec_preflight.md.tmpl" "F0-exec-preflight.md" "F0-exec-preflight" "" "" ""
if ($Automation.IsPresent) {
    Render-Kickoff "kickoff_feature_cleanup.md.tmpl" "FZ-feature-cleanup.md" "FZ-feature-cleanup" "" "" ""
}
if ($CrossPlatform.IsPresent) {
    Render-Kickoff "kickoff_integ_core.md.tmpl" "$integCoreId.md" $integCoreId $sliceIntegCoreBranch "wt/$Feature-$($script:SliceIdLower)-integ-core"
    foreach ($p in $ciParityPlatformsList) {
        $p = $p.Trim()
        if (-not $p) { continue }
        $branch = if ($Automation.IsPresent) { "$Feature-$($script:SliceIdLower)-integ-$p" } else { "$($script:SliceIdLower)-integ-$p" }
        $taskId = "$($script:SliceId)-integ-$p"
        Render-Kickoff "kickoff_integ_platform.md.tmpl" "$taskId.md" $taskId $branch "wt/$Feature-$($script:SliceIdLower)-integ-$p" $p
    }
    if ($WslRequired.IsPresent -and $WslSeparate.IsPresent) {
        $wslBranch = if ($Automation.IsPresent) { "$Feature-$($script:SliceIdLower)-integ-wsl" } else { "$($script:SliceIdLower)-integ-wsl" }
        $wslTaskId = "$($script:SliceId)-integ-wsl"
        Render-Kickoff "kickoff_integ_platform.md.tmpl" "$wslTaskId.md" $wslTaskId $wslBranch "wt/$Feature-$($script:SliceIdLower)-integ-wsl" "wsl"
    }
    Render-Kickoff "kickoff_integ_final.md.tmpl" "$integId.md" $integId $sliceIntegBranch "wt/$Feature-$($script:SliceIdLower)-integ"
} else {
    Render-Kickoff "kickoff_integ.md.tmpl" "$integId.md" $integId $sliceIntegBranch "wt/$Feature-$($script:SliceIdLower)-integ"
}

		if ($DecisionHeavy.IsPresent -or $CrossPlatform.IsPresent) {
		    "# Decision Register`n`nUse the template in:`n- `docs/project_management/standards/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md`" |
		        Set-Content -LiteralPath (Join-Path $featureDir "decision_register.md")

		    if ($CrossPlatform.IsPresent) {
		        $lines = @()
		        $lines += "# Manual Testing Playbook"
		        $lines += ""
		        $lines += "This playbook must contain runnable commands and expected exit codes/output."
		        $lines += ""
		        $lines += "## Behavioral Smoke Scripts"
		        $lines += ""
		        $lines += "These scripts define the behavioral platform contract for this feature. Keep them deterministic and fast."
		        foreach ($p in $behaviorPlatformsList) {
		            switch ($p) {
		                "linux" { $lines += "- Linux: ``bash smoke/linux-smoke.sh`` (expected exit: 0)" }
		                "macos" { $lines += "- macOS: ``bash smoke/macos-smoke.sh`` (expected exit: 0)" }
		                "windows" { $lines += "- Windows: ``pwsh -File smoke/windows-smoke.ps1`` (expected exit: 0)" }
		            }
		        }
		        $lines += ""
		        $lines += "## CI Parity (compile/test)"
		        $lines += ""
		        $lines += ("CI parity platforms (may be broader than behavioral scope): ``" + (($ciParityPlatformsList -join ",") ) + "``")
		        $lines += ""
		        $lines += "Recommended gates:"
		        $lines += "- ``make ci-compile-parity CI_WORKFLOW_REF=`"feat/$Feature`" CI_REMOTE=origin CI_CLEANUP=1``"
		        $lines += "- ``scripts/ci/dispatch_ci_testing.sh --workflow-ref `"feat/$Feature`" --remote origin --cleanup``"
		        ($lines -join "`n") | Set-Content -LiteralPath (Join-Path $featureDir "manual_testing_playbook.md")
		    } else {
		        "# Manual Testing Playbook`n`nThis playbook must contain runnable commands and expected exit codes/output." |
		            Set-Content -LiteralPath (Join-Path $featureDir "manual_testing_playbook.md")
		    }
		}

if ($CrossPlatform.IsPresent) {
    New-Item -ItemType Directory -Force -Path (Join-Path $featureDir "smoke") | Out-Null
    foreach ($p in $behaviorPlatformsList) {
        switch ($p) {
            "linux" {
                @"
#!/usr/bin/env bash
set -euo pipefail
echo `"Smoke script scaffold (linux) - replace with feature checks`"
exit 1
"@ | Set-Content -LiteralPath (Join-Path $featureDir "smoke/linux-smoke.sh") -NoNewline
            }
            "macos" {
                @"
#!/usr/bin/env bash
set -euo pipefail
echo `"Smoke script scaffold (macos) - replace with feature checks`"
exit 1
"@ | Set-Content -LiteralPath (Join-Path $featureDir "smoke/macos-smoke.sh") -NoNewline
            }
            "windows" {
                @"
param()
\$ErrorActionPreference = `"Stop`"
Write-Host `"Smoke script scaffold (windows) - replace with feature checks`"
exit 1
"@ | Set-Content -LiteralPath (Join-Path $featureDir "smoke/windows-smoke.ps1") -NoNewline
            }
        }
    }
}

Write-Host "OK: created $featureDir"
