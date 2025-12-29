param(
    [Parameter(Mandatory = $true)]
    [string]$Feature,
    [switch]$DecisionHeavy,
    [switch]$CrossPlatform
)

$ErrorActionPreference = "Stop"

$featureDir = Join-Path "docs/project_management/next" $Feature
$templatesDir = "docs/project_management/standards/templates"

if (Test-Path -LiteralPath $featureDir) {
    throw "Refusing to overwrite existing directory: $featureDir"
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

@"
{
  `"meta`": {
    `"feature`": `"$Feature`",
    `"cross_platform`": $($CrossPlatform.IsPresent.ToString().ToLowerInvariant())
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

function Render-Kickoff([string]$Template, [string]$OutFile, [string]$TaskId, [string]$Branch, [string]$Worktree) {
    $vars2 = $vars.Clone()
    $vars2["TASK_ID"] = $TaskId
    $vars2["SPEC_FILE"] = "C0-spec.md"
    $vars2["BRANCH"] = $Branch
    $vars2["WORKTREE"] = $Worktree
    Render-Template (Join-Path $templatesDir $Template) (Join-Path $featureDir "kickoff_prompts/$OutFile") $vars2
}

Render-Kickoff "kickoff_code.md.tmpl" "C0-code.md" "C0-code" "c0-code" "wt/$Feature-c0-code"
Render-Kickoff "kickoff_test.md.tmpl" "C0-test.md" "C0-test" "c0-test" "wt/$Feature-c0-test"
Render-Kickoff "kickoff_integ.md.tmpl" "C0-integ.md" "C0-integ" "c0-integ" "wt/$Feature-c0-integ"

if ($DecisionHeavy.IsPresent -or $CrossPlatform.IsPresent) {
    "# Decision Register`n`nUse the template in:`n- `docs/project_management/standards/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md`" |
        Set-Content -LiteralPath (Join-Path $featureDir "decision_register.md")
    "# Integration Map`n`nUse the standard in:`n- `docs/project_management/standards/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md`" |
        Set-Content -LiteralPath (Join-Path $featureDir "integration_map.md")
    "# Manual Testing Playbook`n`nThis playbook must contain runnable commands and expected exit codes/output." |
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

