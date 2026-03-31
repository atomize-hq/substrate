$ErrorActionPreference = "Stop"

# Exit codes:
# - 0: OK / SKIP
# - 1: assertion failed / unexpected error
# - 2: invalid inputs (e.g., unknown SUBSTRATE_SMOKE_SLICE_ID)

if (-not $IsWindows) {
  Write-Host "SKIP: world-disabled-diagnostics Windows smoke (not Windows)"
  exit 0
}

$sliceId = if ($env:SUBSTRATE_SMOKE_SLICE_ID -and $env:SUBSTRATE_SMOKE_SLICE_ID.Trim() -ne "") { $env:SUBSTRATE_SMOKE_SLICE_ID } else { "WDD2" }
if (@("WDD0", "WDD1", "WDD2") -notcontains $sliceId) {
  Write-Error "FAIL: unsupported SUBSTRATE_SMOKE_SLICE_ID=$sliceId (expected WDD0, WDD1, or WDD2)"
  exit 2
}

function Resolve-SubstrateExe {
  if ($env:SUBSTRATE_EXE -and $env:SUBSTRATE_EXE.Trim() -ne "") { return $env:SUBSTRATE_EXE }
  if ($env:SUBSTRATE_BIN -and $env:SUBSTRATE_BIN.Trim() -ne "") { return $env:SUBSTRATE_BIN }
  $cmd = Get-Command substrate -ErrorAction SilentlyContinue
  if ($cmd) { return $cmd.Path }
  return $null
}

$SubstrateExe = Resolve-SubstrateExe
if (-not $SubstrateExe) {
  Write-Error "FAIL: substrate not found (set SUBSTRATE_EXE or SUBSTRATE_BIN)"
  exit 1
}

function New-TempDir([string]$Prefix) {
  $dir = Join-Path ([System.IO.Path]::GetTempPath()) ($Prefix + "-" + [guid]::NewGuid().ToString("N"))
  New-Item -ItemType Directory -Path $dir | Out-Null
  return $dir
}

function Invoke-Substrate {
  param(
    [string]$Label,
    [int]$ExpectedExit,
    [string[]]$Args,
    [hashtable]$Env = @{}
  )

  $stdout = New-TemporaryFile
  $stderr = New-TemporaryFile

  $old = @{}
  foreach ($k in $Env.Keys) {
    $old[$k] = (Get-Item -Path "Env:$k" -ErrorAction SilentlyContinue).Value
    $env:$k = $Env[$k]
  }

  try {
    & $SubstrateExe @Args 1> $stdout.FullName 2> $stderr.FullName
    $rc = $LASTEXITCODE
  } finally {
    foreach ($k in $Env.Keys) {
      if ($null -eq $old[$k]) {
        Remove-Item -Path "Env:$k" -ErrorAction SilentlyContinue
      } else {
        $env:$k = $old[$k]
      }
    }
  }

  $out = (Get-Content -Raw -Path $stdout.FullName)
  $err = (Get-Content -Raw -Path $stderr.FullName)
  Remove-Item -Force $stdout.FullName, $stderr.FullName -ErrorAction SilentlyContinue

  if ($rc -ne $ExpectedExit) {
    Write-Error "FAIL: $Label expected exit=$ExpectedExit, got=$rc`nSTDOUT:`n$out`nSTDERR:`n$err"
    exit 1
  }

  return [pscustomobject]@{
    ExitCode = $rc
    Stdout   = $out
    Stderr   = $err
  }
}

function Require-Contains([string]$Haystack, [string]$Needle) {
  if ($Haystack -notlike "*$Needle*") {
    Write-Error "FAIL: missing expected substring: $Needle"
    exit 1
  }
}

function Require-NotContains([string]$Haystack, [string]$Needle) {
  if ($Haystack -like "*$Needle*") {
    Write-Error "FAIL: found forbidden substring: $Needle"
    exit 1
  }
}

function Test-Wdd0ConfigResolutionError {
  $home = New-TempDir "wdd-home"
  Set-Content -Encoding utf8 -Path (Join-Path $home "config.yaml") -Value "world: [`n"

  $r = Invoke-Substrate -Label "shim doctor invalid config" -ExpectedExit 2 -Args @("shim", "doctor") -Env @{ SUBSTRATE_HOME = $home }
  Require-Contains ($r.Stdout + $r.Stderr) "config.yaml"

  $r = Invoke-Substrate -Label "shim doctor --json invalid config" -ExpectedExit 2 -Args @("shim", "doctor", "--json") -Env @{ SUBSTRATE_HOME = $home }
  Require-Contains ($r.Stdout + $r.Stderr) "config.yaml"

  $r = Invoke-Substrate -Label "health invalid config" -ExpectedExit 2 -Args @("health") -Env @{ SUBSTRATE_HOME = $home }
  Require-Contains ($r.Stdout + $r.Stderr) "config.yaml"

  $r = Invoke-Substrate -Label "health --json invalid config" -ExpectedExit 2 -Args @("health", "--json") -Env @{ SUBSTRATE_HOME = $home }
  Require-Contains ($r.Stdout + $r.Stderr) "config.yaml"

  Remove-Item -Recurse -Force $home -ErrorAction SilentlyContinue
}

function Test-Wdd1ShimDoctorDisabledAndBroken {
  $home = New-TempDir "wdd-home"

  $r = Invoke-Substrate -Label "shim doctor disabled" -ExpectedExit 0 -Args @("shim", "doctor") -Env @{ SUBSTRATE_HOME = $home; SUBSTRATE_OVERRIDE_WORLD = "disabled" }
  Require-Contains $r.Stdout 'World backend:'
  Require-Contains $r.Stdout '  Status: disabled'
  Require-Contains $r.Stdout '  Next: run `substrate world enable` to provision'
  Require-Contains $r.Stdout 'World deps:'
  Require-Contains $r.Stdout '  Status: skipped (world disabled)'
  Require-NotContains $r.Stdout '  Error:'

  $r = Invoke-Substrate -Label "shim doctor --json disabled" -ExpectedExit 0 -Args @("shim", "doctor", "--json") -Env @{ SUBSTRATE_HOME = $home; SUBSTRATE_OVERRIDE_WORLD = "disabled" }
  $d = $r.Stdout | ConvertFrom-Json
  if ($d.world.status -ne "disabled") { Write-Error "FAIL: expected .world.status=disabled"; exit 1 }
  if ($d.world_deps.status -ne "skipped_disabled") { Write-Error "FAIL: expected .world_deps.status=skipped_disabled"; exit 1 }
  foreach ($k in @("error", "stderr", "exit_code", "details")) {
    if ($d.world.PSObject.Properties.Name -contains $k) { Write-Error "FAIL: world.$k must be omitted"; exit 1 }
  }
  foreach ($k in @("error", "report")) {
    if ($d.world_deps.PSObject.Properties.Name -contains $k) { Write-Error "FAIL: world_deps.$k must be omitted"; exit 1 }
  }

  $brokenPipe = "\\\\.\\pipe\\substrate-agent-wdd-broken-$([guid]::NewGuid())"
  $r = Invoke-Substrate -Label "shim doctor enabled-but-broken" -ExpectedExit 0 -Args @("--world", "shim", "doctor") -Env @{
    SUBSTRATE_HOME          = $home
    SUBSTRATE_FORWARDER_PIPE = $brokenPipe
    SUBSTRATE_FORWARDER_TCP = "0"
  }
  Require-Contains $r.Stdout 'World backend:'
  Require-Contains $r.Stdout '  Status: needs attention'
  Require-Contains $r.Stdout '  Error:'
  Require-NotContains $r.Stdout '  Status: disabled'

  $r = Invoke-Substrate -Label "shim doctor --json enabled-but-broken" -ExpectedExit 0 -Args @("--world", "shim", "doctor", "--json") -Env @{
    SUBSTRATE_HOME          = $home
    SUBSTRATE_FORWARDER_PIPE = $brokenPipe
    SUBSTRATE_FORWARDER_TCP = "0"
  }
  $d = $r.Stdout | ConvertFrom-Json
  if ($d.world.status -ne "needs_attention") { Write-Error "FAIL: expected .world.status=needs_attention"; exit 1 }
  if (-not $d.world.error -or $d.world.error.Trim() -eq "") { Write-Error "FAIL: expected non-empty .world.error"; exit 1 }
  if ($d.world_deps.status -ne "error") { Write-Error "FAIL: expected .world_deps.status=error"; exit 1 }
  if (-not $d.world_deps.error -or $d.world_deps.error.Trim() -eq "") { Write-Error "FAIL: expected non-empty .world_deps.error"; exit 1 }

  Remove-Item -Recurse -Force $home -ErrorAction SilentlyContinue
}

function Test-Wdd2HealthDisabledAndBroken {
  $home = New-TempDir "wdd-home"

  $r = Invoke-Substrate -Label "health disabled" -ExpectedExit 0 -Args @("health") -Env @{ SUBSTRATE_HOME = $home; SUBSTRATE_OVERRIDE_WORLD = "disabled" }
  Require-Contains $r.Stdout 'World backend: disabled'
  Require-Contains $r.Stdout '  Next: run `substrate world enable` to provision'
  Require-Contains $r.Stdout 'World deps: skipped (world disabled)'
  Require-NotContains $r.Stdout 'substrate world deps current'

  $r = Invoke-Substrate -Label "health --json disabled" -ExpectedExit 0 -Args @("health", "--json") -Env @{ SUBSTRATE_HOME = $home; SUBSTRATE_OVERRIDE_WORLD = "disabled" }
  $h = $r.Stdout | ConvertFrom-Json
  if ($h.shim.world.status -ne "disabled") { Write-Error "FAIL: expected .shim.world.status=disabled"; exit 1 }
  if ($h.shim.world_deps.status -ne "skipped_disabled") { Write-Error "FAIL: expected .shim.world_deps.status=skipped_disabled"; exit 1 }
  if ($null -ne $h.summary.world_ok) { Write-Error "FAIL: expected summary.world_ok=null"; exit 1 }
  if ($h.summary.PSObject.Properties.Name -contains "world_error") { Write-Error "FAIL: summary.world_error must be omitted"; exit 1 }
  if ($h.summary.PSObject.Properties.Name -contains "world_deps_error") { Write-Error "FAIL: summary.world_deps_error must be omitted"; exit 1 }
  if (($h.summary.world_deps_missing | ConvertTo-Json -Compress) -ne "[]") { Write-Error "FAIL: expected summary.world_deps_missing=[]"; exit 1 }
  if (($h.summary.world_deps_blocked | ConvertTo-Json -Compress) -ne "[]") { Write-Error "FAIL: expected summary.world_deps_blocked=[]"; exit 1 }

  $brokenPipe = "\\\\.\\pipe\\substrate-agent-wdd-broken-$([guid]::NewGuid())"
  $r = Invoke-Substrate -Label "health enabled-but-broken" -ExpectedExit 0 -Args @("--world", "health") -Env @{
    SUBSTRATE_HOME          = $home
    SUBSTRATE_FORWARDER_PIPE = $brokenPipe
    SUBSTRATE_FORWARDER_TCP = "0"
  }
  Require-Contains $r.Stdout 'World backend: needs attention'
  Require-Contains $r.Stdout '  Error:'
  Require-Contains $r.Stdout 'World deps: unavailable'
  Require-Contains $r.Stdout 'Overall status: attention required'
  Require-NotContains $r.Stdout 'World backend: disabled'

  $r = Invoke-Substrate -Label "health --json enabled-but-broken" -ExpectedExit 0 -Args @("--world", "health", "--json") -Env @{
    SUBSTRATE_HOME          = $home
    SUBSTRATE_FORWARDER_PIPE = $brokenPipe
    SUBSTRATE_FORWARDER_TCP = "0"
  }
  $h = $r.Stdout | ConvertFrom-Json
  if ($h.shim.world.status -ne "needs_attention") { Write-Error "FAIL: expected .shim.world.status=needs_attention"; exit 1 }
  if ($h.summary.world_ok -ne $false) { Write-Error "FAIL: expected summary.world_ok=false"; exit 1 }
  if (-not $h.summary.world_error -or $h.summary.world_error.Trim() -eq "") { Write-Error "FAIL: expected non-empty summary.world_error"; exit 1 }
  if ($h.shim.world_deps.status -ne "error") { Write-Error "FAIL: expected .shim.world_deps.status=error"; exit 1 }
  if (-not $h.summary.world_deps_error -or $h.summary.world_deps_error.Trim() -eq "") { Write-Error "FAIL: expected non-empty summary.world_deps_error"; exit 1 }

  Remove-Item -Recurse -Force $home -ErrorAction SilentlyContinue
}

Write-Host "INFO: world-disabled-diagnostics Windows smoke slice=$sliceId"

$workDir = New-TempDir "wdd-work"
Push-Location $workDir
try {
  Test-Wdd0ConfigResolutionError
  if ($sliceId -eq "WDD0") {
    Write-Host "OK: world-disabled-diagnostics Windows smoke ($sliceId)"
    exit 0
  }

  Test-Wdd1ShimDoctorDisabledAndBroken
  if ($sliceId -eq "WDD1") {
    Write-Host "OK: world-disabled-diagnostics Windows smoke ($sliceId)"
    exit 0
  }

  Test-Wdd2HealthDisabledAndBroken
  Write-Host "OK: world-disabled-diagnostics Windows smoke ($sliceId)"
} finally {
  Pop-Location
  Remove-Item -Recurse -Force $workDir -ErrorAction SilentlyContinue
}

