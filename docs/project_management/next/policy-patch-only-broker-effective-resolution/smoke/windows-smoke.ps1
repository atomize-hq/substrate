Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$slice = $env:SUBSTRATE_SMOKE_SLICE_ID
if ($slice -and $slice -ne "C0" -and $slice -ne "C1") {
  Write-Host "SKIP: SUBSTRATE_SMOKE_SLICE_ID=$slice (supported: C0,C1)"
  exit 0
}

function Require-Cmd($Name) {
  if (-not (Get-Command $Name -ErrorAction SilentlyContinue)) {
    Write-Error "missing dependency: $Name"
    exit 3
  }
}

Require-Cmd cargo
Require-Cmd git

$RepoRoot = (git rev-parse --show-toplevel).Trim()
$Driver = Join-Path $RepoRoot "scripts\\dev\\substrate_shell_driver"

$Tmp = Join-Path $env:TEMP ("substrate-policy-smoke-" + [Guid]::NewGuid().ToString("n"))
New-Item -ItemType Directory -Path $Tmp | Out-Null
try {
  $env:SUBSTRATE_HOME = Join-Path $Tmp "home"
  New-Item -ItemType Directory -Path $env:SUBSTRATE_HOME | Out-Null

  $Workspace = Join-Path $Tmp "ws"
  New-Item -ItemType Directory -Path $Workspace | Out-Null

  Push-Location $RepoRoot
  try {
    cargo build --bin substrate
  } finally {
    Pop-Location
  }

  Push-Location $Workspace
  try {
    & $Driver workspace init --force | Out-Null

    function Invoke-C0Checks {
      New-Item -ItemType Directory -Path ".substrate" -Force | Out-Null

      @"
world_fs:
  require_world: true
"@ | Set-Content -Path (Join-Path $env:SUBSTRATE_HOME "policy.yaml") -Encoding UTF8

      @"
world_fs:
  require_world: false
"@ | Set-Content -Path ".substrate\\policy.yaml" -Encoding UTF8

      $effective = (& $Driver policy current show --json 2>$null) | ConvertFrom-Json
      if ($effective.world_fs_require_world -ne $false) {
        throw "expected world_fs_require_world=false, got: $($effective.world_fs_require_world)"
      }

      New-Item -ItemType File -Path ".substrate\\workspace.disabled" | Out-Null
      $effective = (& $Driver policy current show --json 2>$null) | ConvertFrom-Json
      if ($effective.world_fs_require_world -ne $true) {
        throw "expected world_fs_require_world=true (workspace ignored), got: $($effective.world_fs_require_world)"
      }
      Remove-Item -Force ".substrate\\workspace.disabled"
    }

    function Invoke-C1Checks {
      if (Test-Path ".substrate\\workspace.disabled") { Remove-Item -Force ".substrate\\workspace.disabled" }
      @"
world_fs: [
"@ | Set-Content -Path ".substrate\\policy.yaml" -Encoding UTF8

      $out = & $Driver --command "echo SMOKE_POLICY_INVALID_YAML_RAN" 2>&1
      $rc = $LASTEXITCODE

      if ($rc -ne 2) {
        throw "expected exit 2 for invalid policy patch, got $rc; output: $out"
      }

      if ($out -match "SMOKE_POLICY_INVALID_YAML_RAN") {
        throw "command executed despite invalid policy patch; output: $out"
      }
    }

    if (-not $slice -or $slice -eq "C0") { Invoke-C0Checks }
    if (-not $slice -or $slice -eq "C1") { Invoke-C1Checks }
  } finally {
    Pop-Location
  }
} finally {
  Remove-Item -Recurse -Force $Tmp
}

Write-Host "OK: windows smoke passed"
