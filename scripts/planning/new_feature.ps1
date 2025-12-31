param(
    [Parameter(Mandatory = $true)]
    [string]$Feature,
    [switch]$DecisionHeavy,
    [switch]$CrossPlatform,
    [switch]$WslRequired,
    [switch]$WslSeparate,
    [switch]$Automation
)

$ErrorActionPreference = "Stop"

$featureDir = Join-Path "docs/project_management/next" $Feature
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

$nowUtc = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")

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
}

New-Item -ItemType Directory -Force -Path (Join-Path $featureDir "kickoff_prompts") | Out-Null

Render-Template (Join-Path $templatesDir "plan.md.tmpl") (Join-Path $featureDir "plan.md") $vars
Render-Template (Join-Path $templatesDir "session_log.md.tmpl") (Join-Path $featureDir "session_log.md") $vars
Render-Template (Join-Path $templatesDir "contract.md.tmpl") (Join-Path $featureDir "contract.md") $vars

@"
# C0-spec

## Scope
- None yet.

## Behavior
- None yet.

## Acceptance criteria
- None yet.

## Out of scope
- None yet.
"@ | Set-Content -LiteralPath (Join-Path $featureDir "C0-spec.md")

function New-TaskBase([string]$Id, [string]$Name, [string]$Type, [string]$Description) {
    return @{
        id                 = $Id
        name               = $Name
        type               = $Type
        phase              = "C0"
        status             = "pending"
        description        = $Description
        references         = @("$featureDir/plan.md", "$featureDir/C0-spec.md")
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

$schemaVersion = if ($Automation.IsPresent) { 3 } elseif ($CrossPlatform.IsPresent) { 2 } else { 1 }
$meta = @{
    schema_version = $schemaVersion
    feature        = $Feature
    cross_platform = [bool]$CrossPlatform.IsPresent
    execution_gates = $true
}

if ($Automation.IsPresent) {
    $meta.automation = @{
        enabled = $true
        orchestration_branch = "feat/$Feature"
    }
}

if ($CrossPlatform.IsPresent) {
    $meta.platforms_required = @("linux", "macos", "windows")
    if ($WslRequired.IsPresent) {
        $meta.wsl_required = $true
        $meta.wsl_task_mode = if ($WslSeparate.IsPresent) { "separate" } else { "bundled" }
    }
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

$code = New-TaskBase "C0-code" "C0 slice (code)" "code" "Implement C0 spec (production code only)."
$code.acceptance_criteria = @("Meets all acceptance criteria in C0-spec.md")
$code.start_checklist = @(
    "git checkout feat/$Feature && git pull --ff-only",
    "Read plan.md, tasks.json, session_log.md, C0-spec.md, kickoff prompt",
    "Set status to in_progress; add START entry; commit docs",
    $(if ($Automation.IsPresent) { "Run: make triad-task-start-pair FEATURE_DIR=`"$featureDir`" SLICE_ID=`"C0`"" } else { "Run: git worktree add -b c0-code wt/$Feature-c0-code feat/$Feature" })
)
$code.end_checklist = @(
    "cargo fmt",
    "cargo clippy --workspace --all-targets -- -D warnings",
    $(if ($Automation.IsPresent) { "From inside the worktree: make triad-task-finish TASK_ID=`"C0-code`"" } else { "From inside the worktree: git add -A && git commit -m `"code: $Feature C0-code`"" }),
    $(if ($Automation.IsPresent) { "Update tasks/session_log on orchestration branch; do not delete worktrees (feature cleanup removes worktrees at feature end)" } else { "Update tasks/session_log on the orchestration branch; optionally remove the worktree when done: git worktree remove wt/$Feature-c0-code (per plan.md)" })
)
$code.worktree = "wt/$Feature-c0-code"
$code.git_branch = if ($Automation.IsPresent) { "$Feature-c0-code" } else { "c0-code" }
$code.required_make_targets = if ($Automation.IsPresent) { @("triad-code-checks") } else { $null }
$code.integration_task = if ($CrossPlatform.IsPresent) { "C0-integ-core" } else { "C0-integ" }
$code.kickoff_prompt = "$featureDir/kickoff_prompts/C0-code.md"
$code.depends_on = @("F0-exec-preflight")
$code.concurrent_with = @("C0-test")
$tasks += $code

$test = New-TaskBase "C0-test" "C0 slice (test)" "test" "Add/modify tests for C0 spec (tests only)."
$test.acceptance_criteria = @("Tests enforce C0 acceptance criteria")
$test.start_checklist = @(
    "git checkout feat/$Feature && git pull --ff-only",
    "Read plan.md, tasks.json, session_log.md, C0-spec.md, kickoff prompt",
    "Set status to in_progress; add START entry; commit docs",
    $(if ($Automation.IsPresent) { "Run: make triad-task-start-pair FEATURE_DIR=`"$featureDir`" SLICE_ID=`"C0`"" } else { "Run: git worktree add -b c0-test wt/$Feature-c0-test feat/$Feature" })
)
$test.end_checklist = @(
    "cargo fmt",
    "Run the targeted tests you add/touch",
    $(if ($Automation.IsPresent) { "From inside the worktree: make triad-task-finish TASK_ID=`"C0-test`"" } else { "From inside the worktree: git add -A && git commit -m `"test: $Feature C0-test`"" }),
    $(if ($Automation.IsPresent) { "Update tasks/session_log on orchestration branch; do not delete worktrees (feature cleanup removes worktrees at feature end)" } else { "Update tasks/session_log on the orchestration branch; optionally remove the worktree when done: git worktree remove wt/$Feature-c0-test (per plan.md)" })
)
$test.worktree = "wt/$Feature-c0-test"
$test.git_branch = if ($Automation.IsPresent) { "$Feature-c0-test" } else { "c0-test" }
$test.required_make_targets = if ($Automation.IsPresent) { @("triad-test-checks") } else { $null }
$test.integration_task = if ($CrossPlatform.IsPresent) { "C0-integ-core" } else { "C0-integ" }
$test.kickoff_prompt = "$featureDir/kickoff_prompts/C0-test.md"
$test.depends_on = @("F0-exec-preflight")
$test.concurrent_with = @("C0-code")
$tasks += $test

if ($CrossPlatform.IsPresent) {
    $core = New-TaskBase "C0-integ-core" "C0 slice (integration core)" "integration" "Merge C0 code+tests and make the slice green on the primary dev platform."
    $core.integration_task = "C0-integ-core"
    $core.acceptance_criteria = @("Core slice is green under make integ-checks and matches the spec")
    $core.references += @("$featureDir/smoke/linux-smoke.sh", "$featureDir/smoke/macos-smoke.sh", "$featureDir/smoke/windows-smoke.ps1")
    $core.start_checklist = @(
        "git checkout feat/$Feature && git pull --ff-only",
        "Read plan.md, tasks.json, session_log.md, C0-spec.md, kickoff prompt",
        "Set status to in_progress; add START entry; commit docs",
        $(if ($Automation.IsPresent) { "Run: make triad-task-start FEATURE_DIR=`"$featureDir`" TASK_ID=`"C0-integ-core`"" } else { "Run: git worktree add -b c0-integ-core wt/$Feature-c0-integ-core feat/$Feature" })
    )
    $dispatchAll = "scripts/ci/dispatch_feature_smoke.sh --feature-dir `"$featureDir`" --runner-kind self-hosted --platform all"
    if ($WslRequired.IsPresent) { $dispatchAll += " --run-wsl" }
    $dispatchAll += " --cleanup"
    $core.end_checklist = @(
        "cargo fmt",
        "cargo clippy --workspace --all-targets -- -D warnings",
        "Run relevant tests",
        "make integ-checks",
        "Dispatch cross-platform smoke via CI: $dispatchAll (record run ids/URLs)",
        "If any platform smoke fails: start only failing platform-fix tasks via: make triad-task-start-platform-fixes FEATURE_DIR=`"$featureDir`" SLICE_ID=`"C0`" PLATFORMS=`"linux,macos,windows`"",
        "After all failing platforms are green: start final aggregator via: make triad-task-start-integ-final FEATURE_DIR=`"$featureDir`" SLICE_ID=`"C0`"",
        $(if ($Automation.IsPresent) { "From inside the worktree: make triad-task-finish TASK_ID=`"C0-integ-core`"" } else { "From inside the worktree: git add -A && git commit -m `"integ: $Feature C0-integ-core`"" }),
        $(if ($Automation.IsPresent) { "Update tasks/session_log on orchestration branch; do not delete worktrees (feature cleanup removes worktrees at feature end)" } else { "Update tasks/session_log on the orchestration branch; optionally remove the worktree when done: git worktree remove wt/$Feature-c0-integ-core (per plan.md)" })
    )
    $core.worktree = "wt/$Feature-c0-integ-core"
    $core.git_branch = if ($Automation.IsPresent) { "$Feature-c0-integ-core" } else { "c0-integ-core" }
    $core.required_make_targets = if ($Automation.IsPresent) { @("integ-checks") } else { $null }
    $core.merge_to_orchestration = if ($Automation.IsPresent) { $false } else { $null }
    $core.kickoff_prompt = "$featureDir/kickoff_prompts/C0-integ-core.md"
    $core.depends_on = @("C0-code", "C0-test")
    $tasks += $core

    $platforms = @("linux", "macos", "windows")
    if ($WslRequired.IsPresent -and $WslSeparate.IsPresent) { $platforms += "wsl" }

    foreach ($platform in $platforms) {
        $id = "C0-integ-$platform"
        switch ($platform) {
            "linux" {
                $name = "C0 slice (integration Linux)"
                $desc = "Linux platform-fix integration task (may be a no-op if already green)."
                $refs = @("$featureDir/smoke/linux-smoke.sh")
                $dispatch = "scripts/ci/dispatch_feature_smoke.sh --feature-dir `"$featureDir`" --runner-kind self-hosted --platform linux"
                if ($WslRequired.IsPresent -and -not $WslSeparate.IsPresent) { $dispatch += " --run-wsl" }
                $dispatch += " --cleanup"
            }
            "macos" {
                $name = "C0 slice (integration macOS)"
                $desc = "macOS platform-fix integration task (may be a no-op if already green)."
                $refs = @("$featureDir/smoke/macos-smoke.sh")
                $dispatch = "scripts/ci/dispatch_feature_smoke.sh --feature-dir `"$featureDir`" --runner-kind self-hosted --platform macos --cleanup"
            }
            "windows" {
                $name = "C0 slice (integration Windows)"
                $desc = "Windows platform-fix integration task (may be a no-op if already green)."
                $refs = @("$featureDir/smoke/windows-smoke.ps1")
                $dispatch = "scripts/ci/dispatch_feature_smoke.sh --feature-dir `"$featureDir`" --runner-kind self-hosted --platform windows --cleanup"
            }
            "wsl" {
                $name = "C0 slice (integration WSL)"
                $desc = "WSL platform-fix integration task (Linux-in-WSL)."
                $refs = @("$featureDir/smoke/linux-smoke.sh")
                $dispatch = "scripts/ci/dispatch_feature_smoke.sh --feature-dir `"$featureDir`" --runner-kind self-hosted --platform wsl --cleanup"
            }
        }

        $t = New-TaskBase $id $name "integration" $desc
        $t.integration_task = $id
        $t.acceptance_criteria = @("$platform smoke is green for this slice")
        $t.references += $refs
        $t.start_checklist = @(
            "Run on $platform host if possible",
            "git checkout feat/$Feature && git pull --ff-only",
            "Read plan.md, tasks.json, session_log.md, C0-spec.md, kickoff prompt",
            "Set status to in_progress; add START entry; commit docs",
            $(if ($Automation.IsPresent) { "Run: make triad-task-start FEATURE_DIR=`"$featureDir`" TASK_ID=`"$id`"" } else { "Run: git worktree add -b c0-integ-$platform wt/$Feature-c0-integ-$platform feat/$Feature" })
        )
        $t.end_checklist = @(
            "Dispatch platform smoke via CI: $dispatch",
            "If needed: fix + fmt/clippy + targeted tests",
            "Ensure smoke is green; record run id/URL",
            $(if ($Automation.IsPresent) { "From inside the worktree: make triad-task-finish TASK_ID=`"$id`" SMOKE=1 TASK_PLATFORM=`"$platform`"" } else { "From inside the worktree: git add -A && git commit -m `"integ: $Feature $id`"" }),
            $(if ($Automation.IsPresent) { "Update tasks/session_log on orchestration branch; do not delete worktrees (feature cleanup removes worktrees at feature end)" } else { "Update tasks/session_log on the orchestration branch; optionally remove the worktree when done: git worktree remove wt/$Feature-c0-integ-$platform (per plan.md)" })
        )
        $t.worktree = "wt/$Feature-c0-integ-$platform"
        $t.git_branch = if ($Automation.IsPresent) { "$Feature-c0-integ-$platform" } else { "c0-integ-$platform" }
        $t.required_make_targets = if ($Automation.IsPresent) { @("triad-code-checks") } else { $null }
        $t.merge_to_orchestration = if ($Automation.IsPresent) { $false } else { $null }
        $t.kickoff_prompt = "$featureDir/kickoff_prompts/$id.md"
        $t.depends_on = @("C0-integ-core")
        $t.platform = $platform
        $t.runner = "github-actions"
        $t.workflow = ".github/workflows/feature-smoke.yml"
        $tasks += $t
    }

    $final = New-TaskBase "C0-integ" "C0 slice (integration final)" "integration" "Final cross-platform integration: merge any platform fixes and confirm all platforms are green."
    $final.integration_task = "C0-integ"
    $final.acceptance_criteria = @("All required platforms are green and the slice matches the spec")
    $final.references += @("$featureDir/smoke/linux-smoke.sh", "$featureDir/smoke/macos-smoke.sh", "$featureDir/smoke/windows-smoke.ps1", "$featureDir/C0-closeout_report.md")
    $final.start_checklist = @(
        "git checkout feat/$Feature && git pull --ff-only",
        "Read plan.md, tasks.json, session_log.md, C0-spec.md, kickoff prompt",
        "Set status to in_progress; add START entry; commit docs",
        $(if ($Automation.IsPresent) { "Run: make triad-task-start FEATURE_DIR=`"$featureDir`" TASK_ID=`"C0-integ`"" } else { "Run: git worktree add -b c0-integ wt/$Feature-c0-integ feat/$Feature" })
    )
    $dispatchFinal = "scripts/ci/dispatch_feature_smoke.sh --feature-dir `"$featureDir`" --runner-kind self-hosted --platform all"
    if ($WslRequired.IsPresent) { $dispatchFinal += " --run-wsl" }
    $dispatchFinal += " --cleanup"
    $final.end_checklist = @(
        "Merge platform-fix branches (if any) + resolve conflicts",
        "cargo fmt",
        "cargo clippy --workspace --all-targets -- -D warnings",
        "Run relevant tests",
        "make integ-checks",
        "Re-run cross-platform smoke via CI: $dispatchFinal",
        "Complete slice closeout gate report: $featureDir/C0-closeout_report.md",
        $(if ($Automation.IsPresent) { "From inside the worktree: make triad-task-finish TASK_ID=`"C0-integ`"" } else { "From inside the worktree: git add -A && git commit -m `"integ: $Feature C0-integ`"" }),
        $(if ($Automation.IsPresent) { "Update tasks/session_log on orchestration branch; do not delete worktrees (feature cleanup removes worktrees at feature end)" } else { "Update tasks/session_log on the orchestration branch; optionally remove the worktree when done: git worktree remove wt/$Feature-c0-integ (per plan.md)" })
    )
    $final.worktree = "wt/$Feature-c0-integ"
    $final.git_branch = if ($Automation.IsPresent) { "$Feature-c0-integ" } else { "c0-integ" }
    $final.required_make_targets = if ($Automation.IsPresent) { @("integ-checks") } else { $null }
    $final.merge_to_orchestration = if ($Automation.IsPresent) { $true } else { $null }
    $final.kickoff_prompt = "$featureDir/kickoff_prompts/C0-integ.md"
    $final.depends_on = @("C0-integ-core") + ($platforms | ForEach-Object { "C0-integ-$_" })
    $tasks += $final
} else {
    $integ = New-TaskBase "C0-integ" "C0 slice (integration)" "integration" "Integrate C0 code+tests, reconcile to spec, and run integration gate."
    $integ.integration_task = "C0-integ"
    $integ.acceptance_criteria = @("Slice is green under make integ-checks and matches the spec")
    $integ.references += @("$featureDir/C0-closeout_report.md")
    $integ.start_checklist = @(
        "git checkout feat/$Feature && git pull --ff-only",
        "Read plan.md, tasks.json, session_log.md, C0-spec.md, kickoff prompt",
        "Set status to in_progress; add START entry; commit docs",
        $(if ($Automation.IsPresent) { "Run: make triad-task-start FEATURE_DIR=`"$featureDir`" TASK_ID=`"C0-integ`"" } else { "Run: git worktree add -b c0-integ wt/$Feature-c0-integ feat/$Feature" })
    )
    $integ.end_checklist = @(
        "cargo fmt",
        "cargo clippy --workspace --all-targets -- -D warnings",
        "Run relevant tests",
        "make integ-checks",
        "Complete slice closeout gate report: $featureDir/C0-closeout_report.md",
        $(if ($Automation.IsPresent) { "From inside the worktree: make triad-task-finish TASK_ID=`"C0-integ`"" } else { "From inside the worktree: git add -A && git commit -m `"integ: $Feature C0-integ`"" }),
        $(if ($Automation.IsPresent) { "Update tasks/session_log on orchestration branch; do not delete worktrees (feature cleanup removes worktrees at feature end)" } else { "Update tasks/session_log on the orchestration branch; optionally remove the worktree when done: git worktree remove wt/$Feature-c0-integ (per plan.md)" })
    )
    $integ.worktree = "wt/$Feature-c0-integ"
    $integ.git_branch = if ($Automation.IsPresent) { "$Feature-c0-integ" } else { "c0-integ" }
    $integ.required_make_targets = if ($Automation.IsPresent) { @("integ-checks") } else { $null }
    $integ.merge_to_orchestration = if ($Automation.IsPresent) { $true } else { $null }
    $integ.kickoff_prompt = "$featureDir/kickoff_prompts/C0-integ.md"
    $integ.depends_on = @("C0-code", "C0-test")
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
        depends_on = @("C0-integ")
        concurrent_with = @()
    }
}

$data = @{ meta = $meta; tasks = $tasks }
$data | ConvertTo-Json -Depth 20 | Set-Content -LiteralPath (Join-Path $featureDir "tasks.json") -NoNewline

$varsExec = $vars.Clone()
Render-Template (Join-Path $templatesDir "execution_preflight_report.md.tmpl") (Join-Path $featureDir "execution_preflight_report.md") $varsExec

$varsCloseout = $vars.Clone()
$varsCloseout["SLICE_ID"] = "C0"
$varsCloseout["SPEC_FILE"] = "C0-spec.md"
Render-Template (Join-Path $templatesDir "slice_closeout_report.md.tmpl") (Join-Path $featureDir "C0-closeout_report.md") $varsCloseout

<# LEGACY tasks.json generator (string heredoc). Keep temporarily for diffability; safe to delete later.
if ($CrossPlatform.IsPresent) {
@"
{
  `"meta`": {
    `"schema_version`": 2,
    `"feature`": `"$Feature`",
    `"cross_platform`": true,
    `"platforms_required`": [`"linux`", `"macos`", `"windows`"]
  },
  `"tasks`": [
    {
      `"id`": `"C0-code`",
      `"name`": `"C0 slice (code)`",
      `"type`": `"code`",
      `"phase`": `"C0`",
      `"status`": `"pending`",
      `"description`": `"Implement C0 spec (production code only).`",
      `"references`": [`"$featureDir/plan.md`", `"$featureDir/C0-spec.md`"],
      `"acceptance_criteria`": [`"Meets all acceptance criteria in C0-spec.md`"],
      `"start_checklist`": [
        `"git checkout feat/$Feature && git pull --ff-only`",
        `"Read plan.md, tasks.json, session_log.md, C0-spec.md, kickoff prompt`",
        `"Set status to in_progress; add START entry; commit docs`",
        `"Create branch c0-code and worktree wt/$Feature-c0-code; do not edit planning docs inside the worktree`"
      ],
      `"end_checklist`": [
        `"cargo fmt`",
        `"cargo clippy --workspace --all-targets -- -D warnings`",
        `"Commit worktree changes; merge back ff-only; update docs; remove worktree`"
      ],
      `"worktree`": `"wt/$Feature-c0-code`",
      `"integration_task`": `"C0-integ`",
      `"kickoff_prompt`": `"$featureDir/kickoff_prompts/C0-code.md`",
      `"depends_on`": [],
      `"concurrent_with`": [`"C0-test`"]
    },
    {
      `"id`": `"C0-test`",
      `"name`": `"C0 slice (test)`",
      `"type`": `"test`",
      `"phase`": `"C0`",
      `"status`": `"pending`",
      `"description`": `"Add/modify tests for C0 spec (tests only).`",
      `"references`": [`"$featureDir/plan.md`", `"$featureDir/C0-spec.md`"],
      `"acceptance_criteria`": [`"Tests enforce C0 acceptance criteria`"],
      `"start_checklist`": [
        `"git checkout feat/$Feature && git pull --ff-only`",
        `"Read plan.md, tasks.json, session_log.md, C0-spec.md, kickoff prompt`",
        `"Set status to in_progress; add START entry; commit docs`",
        `"Create branch c0-test and worktree wt/$Feature-c0-test; do not edit planning docs inside the worktree`"
      ],
      `"end_checklist`": [
        `"cargo fmt`",
        `"Run the targeted tests you add/touch`",
        `"Commit worktree changes; merge back ff-only; update docs; remove worktree`"
      ],
      `"worktree`": `"wt/$Feature-c0-test`",
      `"integration_task`": `"C0-integ`",
      `"kickoff_prompt`": `"$featureDir/kickoff_prompts/C0-test.md`",
      `"depends_on`": [],
      `"concurrent_with`": [`"C0-code`"]
    },
    {
      `"id`": `"C0-integ-core`",
      `"name`": `"C0 slice (integration core)`",
      `"type`": `"integration`",
      `"phase`": `"C0`",
      `"status`": `"pending`",
      `"description`": `"Merge C0 code+tests and make the slice green on the primary dev platform.`",
      `"references`": [`"$featureDir/plan.md`", `"$featureDir/C0-spec.md`"],
      `"acceptance_criteria`": [`"Core slice is green under make integ-checks and matches the spec`"],
      `"start_checklist`": [
        `"git checkout feat/$Feature && git pull --ff-only`",
        `"Read plan.md, tasks.json, session_log.md, C0-spec.md, kickoff prompt`",
        `"Set status to in_progress; add START entry; commit docs`",
        `"Create branch c0-integ-core and worktree wt/$Feature-c0-integ-core; do not edit planning docs inside the worktree`"
      ],
      `"end_checklist`": [
        `"cargo fmt`",
        `"cargo clippy --workspace --all-targets -- -D warnings`",
        `"Run relevant tests`",
        `"make integ-checks`",
        `"Dispatch cross-platform smoke via scripts/ci/dispatch_feature_smoke.sh (record run ids/URLs)`",
        `"Commit worktree changes; merge back ff-only; update docs; remove worktree`"
      ],
      `"worktree`": `"wt/$Feature-c0-integ-core`",
      `"integration_task`": `"C0-integ-core`",
      `"kickoff_prompt`": `"$featureDir/kickoff_prompts/C0-integ-core.md`",
      `"depends_on`": [`"C0-code`", `"C0-test`"],
      `"concurrent_with`": []
    },
    {
      `"id`": `"C0-integ-linux`",
      `"name`": `"C0 slice (integration linux)`",
      `"type`": `"integration`",
      `"phase`": `"C0`",
      `"status`": `"pending`",
      `"description`": `"Linux platform-fix integration task (may be a no-op if already green).`",
      `"references`": [`"$featureDir/plan.md`", `"$featureDir/C0-spec.md`"],
      `"acceptance_criteria`": [`"Linux smoke is green for this slice`"],
      `"start_checklist`": [
        `"Run on Linux host if possible`",
        `"git checkout feat/$Feature && git pull --ff-only`",
        `"Read plan.md, tasks.json, session_log.md, C0-spec.md, kickoff prompt`",
        `"Set status to in_progress; add START entry; commit docs`",
        `"Create branch c0-integ-linux and worktree wt/$Feature-c0-integ-linux; do not edit planning docs inside the worktree`"
      ],
      `"end_checklist`": [
        `"Dispatch platform smoke: scripts/ci/dispatch_feature_smoke.sh --platform linux`",
        `"If needed: fix + fmt/clippy + targeted tests`",
        `"Ensure Linux smoke is green; record run id/URL`",
        `"Commit worktree changes (if any); merge back ff-only; update docs; remove worktree`"
      ],
      `"worktree`": `"wt/$Feature-c0-integ-linux`",
      `"integration_task`": `"C0-integ-linux`",
      `"kickoff_prompt`": `"$featureDir/kickoff_prompts/C0-integ-linux.md`",
      `"depends_on`": [`"C0-integ-core`"],
      `"concurrent_with`": [],
      `"platform`": `"linux`",
      `"runner`": `"github-actions`",
      `"workflow`": `".github/workflows/feature-smoke.yml`"
    },
    {
      `"id`": `"C0-integ-macos`",
      `"name`": `"C0 slice (integration macOS)`",
      `"type`": `"integration`",
      `"phase`": `"C0`",
      `"status`": `"pending`",
      `"description`": `"macOS platform-fix integration task (may be a no-op if already green).`",
      `"references`": [`"$featureDir/plan.md`", `"$featureDir/C0-spec.md`"],
      `"acceptance_criteria`": [`"macOS smoke is green for this slice`"],
      `"start_checklist`": [
        `"Run on macOS host if possible`",
        `"git checkout feat/$Feature && git pull --ff-only`",
        `"Read plan.md, tasks.json, session_log.md, C0-spec.md, kickoff prompt`",
        `"Set status to in_progress; add START entry; commit docs`",
        `"Create branch c0-integ-macos and worktree wt/$Feature-c0-integ-macos; do not edit planning docs inside the worktree`"
      ],
      `"end_checklist`": [
        `"Dispatch platform smoke: scripts/ci/dispatch_feature_smoke.sh --platform macos`",
        `"If needed: fix + fmt/clippy + targeted tests`",
        `"Ensure macOS smoke is green; record run id/URL`",
        `"Commit worktree changes (if any); merge back ff-only; update docs; remove worktree`"
      ],
      `"worktree`": `"wt/$Feature-c0-integ-macos`",
      `"integration_task`": `"C0-integ-macos`",
      `"kickoff_prompt`": `"$featureDir/kickoff_prompts/C0-integ-macos.md`",
      `"depends_on`": [`"C0-integ-core`"],
      `"concurrent_with`": [],
      `"platform`": `"macos`",
      `"runner`": `"github-actions`",
      `"workflow`": `".github/workflows/feature-smoke.yml`"
    },
    {
      `"id`": `"C0-integ-windows`",
      `"name`": `"C0 slice (integration Windows)`",
      `"type`": `"integration`",
      `"phase`": `"C0`",
      `"status`": `"pending`",
      `"description`": `"Windows platform-fix integration task (may be a no-op if already green).`",
      `"references`": [`"$featureDir/plan.md`", `"$featureDir/C0-spec.md`"],
      `"acceptance_criteria`": [`"Windows smoke is green for this slice`"],
      `"start_checklist`": [
        `"Run on Windows host if possible`",
        `"git checkout feat/$Feature && git pull --ff-only`",
        `"Read plan.md, tasks.json, session_log.md, C0-spec.md, kickoff prompt`",
        `"Set status to in_progress; add START entry; commit docs`",
        `"Create branch c0-integ-windows and worktree wt/$Feature-c0-integ-windows; do not edit planning docs inside the worktree`"
      ],
      `"end_checklist`": [
        `"Dispatch platform smoke: scripts/ci/dispatch_feature_smoke.sh --platform windows`",
        `"If needed: fix + fmt/clippy + targeted tests`",
        `"Ensure Windows smoke is green; record run id/URL`",
        `"Commit worktree changes (if any); merge back ff-only; update docs; remove worktree`"
      ],
      `"worktree`": `"wt/$Feature-c0-integ-windows`",
      `"integration_task`": `"C0-integ-windows`",
      `"kickoff_prompt`": `"$featureDir/kickoff_prompts/C0-integ-windows.md`",
      `"depends_on`": [`"C0-integ-core`"],
      `"concurrent_with`": [],
      `"platform`": `"windows`",
      `"runner`": `"github-actions`",
      `"workflow`": `".github/workflows/feature-smoke.yml`"
    },
    {
      `"id`": `"C0-integ`",
      `"name`": `"C0 slice (integration final)`",
      `"type`": `"integration`",
      `"phase`": `"C0`",
      `"status`": `"pending`",
      `"description`": `"Final cross-platform integration: merge any platform fixes and confirm all platforms are green.`",
      `"references`": [`"$featureDir/plan.md`", `"$featureDir/C0-spec.md`"],
      `"acceptance_criteria`": [`"All required platforms are green and the slice matches the spec`"],
      `"start_checklist`": [
        `"git checkout feat/$Feature && git pull --ff-only`",
        `"Read plan.md, tasks.json, session_log.md, C0-spec.md, kickoff prompt`",
        `"Set status to in_progress; add START entry; commit docs`",
        `"Create branch c0-integ and worktree wt/$Feature-c0-integ; do not edit planning docs inside the worktree`"
      ],
      `"end_checklist`": [
        `"Merge platform-fix branches (if any) + resolve conflicts`",
        `"cargo fmt`",
        `"cargo clippy --workspace --all-targets -- -D warnings`",
        `"Run relevant tests`",
        `"make integ-checks`",
        `"Dispatch cross-platform smoke via scripts/ci/dispatch_feature_smoke.sh (record run ids/URLs)`",
        `"Commit worktree changes; merge back ff-only; update docs; remove worktree`"
      ],
      `"worktree`": `"wt/$Feature-c0-integ`",
      `"integration_task`": `"C0-integ`",
      `"kickoff_prompt`": `"$featureDir/kickoff_prompts/C0-integ.md`",
      `"depends_on`": [`"C0-integ-core`", `"C0-integ-linux`", `"C0-integ-macos`", `"C0-integ-windows`"],
      `"concurrent_with`": []
    }
  ]
}
"@ | Set-Content -LiteralPath (Join-Path $featureDir "tasks.json")
} else {
@"
{
  `"meta`": {
    `"schema_version`": 2,
    `"feature`": `"$Feature`",
    `"cross_platform`": false
  },
  `"tasks`": [
    {
      `"id`": `"C0-code`",
      `"name`": `"C0 slice (code)`",
      `"type`": `"code`",
      `"phase`": `"C0`",
      `"status`": `"pending`",
      `"description`": `"Implement C0 spec (production code only).`",
      `"references`": [`"$featureDir/plan.md`", `"$featureDir/C0-spec.md`"],
      `"acceptance_criteria`": [`"Meets all acceptance criteria in C0-spec.md`"],
      `"start_checklist`": [
        `"git checkout feat/$Feature && git pull --ff-only`",
        `"Read plan.md, tasks.json, session_log.md, C0-spec.md, kickoff prompt`",
        `"Set status to in_progress; add START entry; commit docs`",
        `"Create branch c0-code and worktree wt/$Feature-c0-code; do not edit planning docs inside the worktree`"
      ],
      `"end_checklist`": [
        `"cargo fmt`",
        `"cargo clippy --workspace --all-targets -- -D warnings`",
        `"Commit worktree changes; merge back ff-only; update docs; remove worktree`"
      ],
      `"worktree`": `"wt/$Feature-c0-code`",
      `"integration_task`": `"C0-integ`",
      `"kickoff_prompt`": `"$featureDir/kickoff_prompts/C0-code.md`",
      `"depends_on`": [],
      `"concurrent_with`": [`"C0-test`"]
    },
    {
      `"id`": `"C0-test`",
      `"name`": `"C0 slice (test)`",
      `"type`": `"test`",
      `"phase`": `"C0`",
      `"status`": `"pending`",
      `"description`": `"Add/modify tests for C0 spec (tests only).`",
      `"references`": [`"$featureDir/plan.md`", `"$featureDir/C0-spec.md`"],
      `"acceptance_criteria`": [`"Tests enforce C0 acceptance criteria`"],
      `"start_checklist`": [
        `"git checkout feat/$Feature && git pull --ff-only`",
        `"Read plan.md, tasks.json, session_log.md, C0-spec.md, kickoff prompt`",
        `"Set status to in_progress; add START entry; commit docs`",
        `"Create branch c0-test and worktree wt/$Feature-c0-test; do not edit planning docs inside the worktree`"
      ],
      `"end_checklist`": [
        `"cargo fmt`",
        `"Run the targeted tests you add/touch`",
        `"Commit worktree changes; merge back ff-only; update docs; remove worktree`"
      ],
      `"worktree`": `"wt/$Feature-c0-test`",
      `"integration_task`": `"C0-integ`",
      `"kickoff_prompt`": `"$featureDir/kickoff_prompts/C0-test.md`",
      `"depends_on`": [],
      `"concurrent_with`": [`"C0-code`"]
    },
    {
      `"id`": `"C0-integ`",
      `"name`": `"C0 slice (integration)`",
      `"type`": `"integration`",
      `"phase`": `"C0`",
      `"status`": `"pending`",
      `"description`": `"Integrate C0 code+tests, reconcile to spec, and run integration gate.`",
      `"references`": [`"$featureDir/plan.md`", `"$featureDir/C0-spec.md`"],
      `"acceptance_criteria`": [`"Slice is green under make integ-checks and matches the spec`"],
      `"start_checklist`": [
        `"git checkout feat/$Feature && git pull --ff-only`",
        `"Read plan.md, tasks.json, session_log.md, C0-spec.md, kickoff prompt`",
        `"Set status to in_progress; add START entry; commit docs`",
        `"Create branch c0-integ and worktree wt/$Feature-c0-integ; do not edit planning docs inside the worktree`"
      ],
      `"end_checklist`": [
        `"cargo fmt`",
        `"cargo clippy --workspace --all-targets -- -D warnings`",
        `"Run relevant tests`",
        `"make integ-checks`",
        `"Commit worktree changes; merge back ff-only; update docs; remove worktree`"
      ],
      `"worktree`": `"wt/$Feature-c0-integ`",
      `"integration_task`": `"C0-integ`",
      `"kickoff_prompt`": `"$featureDir/kickoff_prompts/C0-integ.md`",
      `"depends_on`": [`"C0-code`", `"C0-test`"],
      `"concurrent_with`": []
    }
  ]
}
"@ | Set-Content -LiteralPath (Join-Path $featureDir "tasks.json")
}
#>

function Render-Kickoff([string]$Template, [string]$OutFile, [string]$TaskId, [string]$Branch, [string]$Worktree, [string]$Platform = "") {
    $vars2 = $vars.Clone()
    $vars2["TASK_ID"] = $TaskId
    $vars2["SPEC_FILE"] = "C0-spec.md"
    $vars2["BRANCH"] = $Branch
    $vars2["WORKTREE"] = $Worktree
    $vars2["PLATFORM"] = $Platform
    Render-Template (Join-Path $templatesDir $Template) (Join-Path $featureDir "kickoff_prompts/$OutFile") $vars2
}

$c0CodeBranch = if ($Automation.IsPresent) { "$Feature-c0-code" } else { "c0-code" }
$c0TestBranch = if ($Automation.IsPresent) { "$Feature-c0-test" } else { "c0-test" }
$c0IntegBranch = if ($Automation.IsPresent) { "$Feature-c0-integ" } else { "c0-integ" }
$c0IntegCoreBranch = if ($Automation.IsPresent) { "$Feature-c0-integ-core" } else { "c0-integ-core" }

Render-Kickoff "kickoff_code.md.tmpl" "C0-code.md" "C0-code" $c0CodeBranch "wt/$Feature-c0-code"
Render-Kickoff "kickoff_test.md.tmpl" "C0-test.md" "C0-test" $c0TestBranch "wt/$Feature-c0-test"
Render-Kickoff "kickoff_exec_preflight.md.tmpl" "F0-exec-preflight.md" "F0-exec-preflight" "" "" ""
if ($Automation.IsPresent) {
    Render-Kickoff "kickoff_feature_cleanup.md.tmpl" "FZ-feature-cleanup.md" "FZ-feature-cleanup" "" "" ""
}
if ($CrossPlatform.IsPresent) {
    Render-Kickoff "kickoff_integ_core.md.tmpl" "C0-integ-core.md" "C0-integ-core" $c0IntegCoreBranch "wt/$Feature-c0-integ-core"
    Render-Kickoff "kickoff_integ_platform.md.tmpl" "C0-integ-linux.md" "C0-integ-linux" $(if ($Automation.IsPresent) { "$Feature-c0-integ-linux" } else { "c0-integ-linux" }) "wt/$Feature-c0-integ-linux" "linux"
    Render-Kickoff "kickoff_integ_platform.md.tmpl" "C0-integ-macos.md" "C0-integ-macos" $(if ($Automation.IsPresent) { "$Feature-c0-integ-macos" } else { "c0-integ-macos" }) "wt/$Feature-c0-integ-macos" "macos"
    Render-Kickoff "kickoff_integ_platform.md.tmpl" "C0-integ-windows.md" "C0-integ-windows" $(if ($Automation.IsPresent) { "$Feature-c0-integ-windows" } else { "c0-integ-windows" }) "wt/$Feature-c0-integ-windows" "windows"
    if ($WslRequired.IsPresent -and $WslSeparate.IsPresent) {
        Render-Kickoff "kickoff_integ_platform.md.tmpl" "C0-integ-wsl.md" "C0-integ-wsl" $(if ($Automation.IsPresent) { "$Feature-c0-integ-wsl" } else { "c0-integ-wsl" }) "wt/$Feature-c0-integ-wsl" "wsl"
    }
    Render-Kickoff "kickoff_integ_final.md.tmpl" "C0-integ.md" "C0-integ" $c0IntegBranch "wt/$Feature-c0-integ"
} else {
    Render-Kickoff "kickoff_integ.md.tmpl" "C0-integ.md" "C0-integ" $c0IntegBranch "wt/$Feature-c0-integ"
}

	if ($DecisionHeavy.IsPresent -or $CrossPlatform.IsPresent) {
	    "# Decision Register`n`nUse the template in:`n- `docs/project_management/standards/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md`" |
	        Set-Content -LiteralPath (Join-Path $featureDir "decision_register.md")
	    "# Integration Map`n`nUse the standard in:`n- `docs/project_management/standards/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md`" |
	        Set-Content -LiteralPath (Join-Path $featureDir "integration_map.md")
	    "# Manual Testing Playbook`n`nThis playbook must contain runnable commands and expected exit codes/output.`n`n## CI Smoke Scripts`n`nThese are invoked by the Feature Smoke workflow. Keep them deterministic and fast.`n`n- Linux: `bash smoke/linux-smoke.sh` (expected exit: 0)`n- macOS: `bash smoke/macos-smoke.sh` (expected exit: 0)`n- Windows: `pwsh -File smoke/windows-smoke.ps1` (expected exit: 0)" |
	        Set-Content -LiteralPath (Join-Path $featureDir "manual_testing_playbook.md")
	}

if ($CrossPlatform.IsPresent) {
    New-Item -ItemType Directory -Force -Path (Join-Path $featureDir "smoke") | Out-Null
    @"
#!/usr/bin/env bash
set -euo pipefail
echo `"Smoke script scaffold (linux) - replace with feature checks`"
exit 1
"@ | Set-Content -LiteralPath (Join-Path $featureDir "smoke/linux-smoke.sh") -NoNewline

    @"
#!/usr/bin/env bash
set -euo pipefail
echo `"Smoke script scaffold (macos) - replace with feature checks`"
exit 1
"@ | Set-Content -LiteralPath (Join-Path $featureDir "smoke/macos-smoke.sh") -NoNewline

    @"
param()
\$ErrorActionPreference = `"Stop`"
Write-Host `"Smoke script scaffold (windows) - replace with feature checks`"
exit 1
"@ | Set-Content -LiteralPath (Join-Path $featureDir "smoke/windows-smoke.ps1") -NoNewline
}

Write-Host "OK: created $featureDir"
