Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$FeatureDir = Split-Path -Parent $MyInvocation.MyCommand.Path | Split-Path -Parent
$SubstrateBin = if ($env:SUBSTRATE_BIN) { $env:SUBSTRATE_BIN } else { "substrate" }

function Test-CommandExists($CommandName) {
  $null -ne (Get-Command $CommandName -ErrorAction SilentlyContinue)
}

function Assert-Contains($Path, $Pattern) {
  $content = Get-Content -Raw -Path $Path
  if ($content -notmatch $Pattern) {
    throw "expected pattern '$Pattern' in $Path"
  }
}

function Invoke-ExpectExit([int]$ExpectedExit, [string]$CommandName, [string[]]$Arguments) {
  $output = & $CommandName @Arguments 2>&1
  $exitCode = $LASTEXITCODE
  if ($exitCode -ne $ExpectedExit) {
    throw "expected exit $ExpectedExit, got $exitCode for: $CommandName $($Arguments -join ' ')" + [Environment]::NewLine + ($output -join [Environment]::NewLine)
  }
}

function Invoke-Capture([string]$CommandName, [string[]]$Arguments) {
  $stdoutPath = Join-Path $tmpRoot ([System.Guid]::NewGuid().ToString() + ".stdout")
  $stderrPath = Join-Path $tmpRoot ([System.Guid]::NewGuid().ToString() + ".stderr")
  $proc = Start-Process -FilePath $CommandName -ArgumentList $Arguments -NoNewWindow -Wait -PassThru -RedirectStandardOutput $stdoutPath -RedirectStandardError $stderrPath
  return [pscustomobject]@{
    ExitCode = $proc.ExitCode
    StdoutPath = $stdoutPath
    StderrPath = $stderrPath
    Stdout = Get-Content -Raw -Path $stdoutPath
    Stderr = Get-Content -Raw -Path $stderrPath
  }
}

if (-not $IsWindows) {
  Write-Error "adr-0027-identity-tuple-policy-surface: windows smoke is supported only on Windows"
  exit 4
}

if (-not (Test-CommandExists $SubstrateBin)) {
  Write-Error "adr-0027-identity-tuple-policy-surface: substrate binary not found (SUBSTRATE_BIN=$SubstrateBin)"
  exit 3
}

Write-Output "== Doc contract checks =="
$tasksPath = Join-Path $FeatureDir "tasks.json"
$tasksDoc = Get-Content -Raw -Path $tasksPath | ConvertFrom-Json
$behaviorPlatforms = @($tasksDoc.meta.behavior_platforms_required)
if (($behaviorPlatforms -join ",") -ne "linux,macos") {
  throw "expected behavior_platforms_required to be linux,macos"
}
$parityPlatforms = @($tasksDoc.meta.ci_parity_platforms_required)
if (($parityPlatforms -join ",") -ne "linux,macos,windows") {
  throw "expected ci_parity_platforms_required to be linux,macos,windows"
}

$requiredDecisionTasks = @(
  "ITPS0-code",
  "ITPS0-test",
  "ITPS0-integ",
  "ITPS1-code",
  "ITPS1-test",
  "ITPS1-integ"
)

foreach ($taskId in $requiredDecisionTasks) {
  $task = $tasksDoc.tasks | Where-Object { $_.id -eq $taskId }
  if (-not $task) {
    throw "missing task $taskId"
  }
  if (($task.references -notcontains "docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/decision_register.md (DR-ITPS-01)") -or
      ($task.references -notcontains "docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/decision_register.md (DR-ITPS-02)")) {
    throw "task $taskId is missing required decision register references"
  }
}

Assert-Contains (Join-Path $FeatureDir "contract.md") "substrate policy current show --explain"
Assert-Contains (Join-Path $FeatureDir "manual_testing_playbook.md") "smoke/windows-smoke.ps1"
Assert-Contains (Join-Path $FeatureDir "manual_testing_playbook.md") "smoke/linux-smoke.sh"
Assert-Contains (Join-Path $FeatureDir "manual_testing_playbook.md") "smoke/macos-smoke.sh"

$tmpRoot = if ($env:SUBSTRATE_SMOKE_ROOT) { $env:SUBSTRATE_SMOKE_ROOT } else { Join-Path ([System.IO.Path]::GetTempPath()) ([System.Guid]::NewGuid().ToString()) }
New-Item -ItemType Directory -Force -Path $tmpRoot | Out-Null

try {
  $env:SUBSTRATE_HOME = if ($env:SUBSTRATE_HOME) { $env:SUBSTRATE_HOME } else { Join-Path $tmpRoot "substrate-home" }
  $workspace = Join-Path $tmpRoot "workspace"
  New-Item -ItemType Directory -Force -Path $workspace | Out-Null
  Push-Location $workspace

  Write-Output "== CLI contract smoke =="
  Invoke-ExpectExit 0 $SubstrateBin @("workspace", "init", "--force")
  Invoke-ExpectExit 0 $SubstrateBin @("config", "global", "init", "--force")
  Invoke-ExpectExit 0 $SubstrateBin @("policy", "global", "init", "--force")
  Invoke-ExpectExit 0 $SubstrateBin @("policy", "global", "set", "--json", 'llm.constraints.providers=["openai"]')
  Invoke-ExpectExit 0 $SubstrateBin @("policy", "global", "set", "--json", 'llm.constraints.protocols=["openai.responses"]')
  $policyView = Invoke-Capture $SubstrateBin @("policy", "current", "show", "--json", "--explain")
  if ($policyView.ExitCode -ne 0) {
    throw "policy current show --json --explain failed with exit $($policyView.ExitCode)`nSTDOUT:`n$($policyView.Stdout)`nSTDERR:`n$($policyView.Stderr)"
  }
  $policyJson = $policyView.Stdout | ConvertFrom-Json
  if (($policyJson.llm.constraints.providers -join ",") -ne "openai") {
    throw "expected llm.constraints.providers to contain openai"
  }
  if (($policyJson.llm.constraints.protocols -join ",") -ne "openai.responses") {
    throw "expected llm.constraints.protocols to contain openai.responses"
  }
  $explainJson = $policyView.Stderr | ConvertFrom-Json
  if ($explainJson.kind -ne "substrate.policy.explain.v1") {
    throw "expected explain kind substrate.policy.explain.v1"
  }
  if ($explainJson.keys.'llm.constraints.providers'.sources[0].layer -ne "global_patch") {
    throw "expected providers provenance to come from global_patch"
  }
  if ($explainJson.keys.'llm.constraints.protocols'.sources[0].layer -ne "global_patch") {
    throw "expected protocols provenance to come from global_patch"
  }
  Invoke-ExpectExit 2 $SubstrateBin @("policy", "global", "set", "--json", 'llm.constraints.providers=["OpenAI"]')
  Invoke-ExpectExit 2 $SubstrateBin @("policy", "global", "set", "--json", 'llm.constraints.protocols=["openai"]')

  Write-Output "OK: adr-0027-identity-tuple-policy-surface windows smoke passed"
}
finally {
  Pop-Location | Out-Null
  if ($env:SUBSTRATE_SMOKE_KEEP -ne "1") {
    Remove-Item -Recurse -Force $tmpRoot -ErrorAction SilentlyContinue
  }
}
