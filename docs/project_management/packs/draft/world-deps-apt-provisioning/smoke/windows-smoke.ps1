$ErrorActionPreference = "Stop"

# WDAP smoke (Windows).
#
# Exit codes (aligned to `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`):
# - 0: smoke passed / skip (wrong OS)
# - 1: assertion failed / unexpected error
# - 2: invalid inputs
# - 3: required dependency unavailable
# - 4: not supported / missing prerequisites

if (-not $IsWindows) {
  Write-Host "SKIP: WDAP windows smoke (not Windows)"
  exit 0
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
  exit 3
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
    Set-Item -Path "Env:$k" -Value $Env[$k]
  }

  try {
    $psi = New-Object System.Diagnostics.ProcessStartInfo
    $psi.FileName = $SubstrateExe
    foreach ($arg in $Args) {
      [void]$psi.ArgumentList.Add($arg)
    }
    $psi.WorkingDirectory = (Get-Location).Path
    $psi.RedirectStandardOutput = $true
    $psi.RedirectStandardError = $true
    $psi.UseShellExecute = $false

    foreach ($name in [System.Environment]::GetEnvironmentVariables().Keys) {
      $stringName = [string]$name
      $psi.Environment[$stringName] = [string][System.Environment]::GetEnvironmentVariable($stringName)
    }

    $proc = New-Object System.Diagnostics.Process
    $proc.StartInfo = $psi
    if (-not $proc.Start()) {
      throw "failed to start $SubstrateExe"
    }
    $out = $proc.StandardOutput.ReadToEnd()
    $err = $proc.StandardError.ReadToEnd()
    $proc.WaitForExit()
    $rc = $proc.ExitCode
    Set-Content -Path $stdout.FullName -Value $out -NoNewline
    Set-Content -Path $stderr.FullName -Value $err -NoNewline
  } finally {
    foreach ($k in $Env.Keys) {
      if ($null -eq $old[$k]) {
        Remove-Item -Path "Env:$k" -ErrorAction SilentlyContinue
      } else {
        Set-Item -Path "Env:$k" -Value $old[$k]
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

function Require-LineOrder([string]$Text, [string]$First, [string]$Second) {
  $lines = $Text -split "`r?`n"
  $firstIdx = -1
  $secondIdx = -1
  for ($i = 0; $i -lt $lines.Length; $i++) {
    if ($firstIdx -lt 0 -and $lines[$i] -eq $First) { $firstIdx = $i }
    if ($secondIdx -lt 0 -and $lines[$i] -eq $Second) { $secondIdx = $i }
  }
  if ($firstIdx -lt 0) { Write-Error "FAIL: missing required line in stdout: $First"; exit 1 }
  if ($secondIdx -lt 0) { Write-Error "FAIL: missing required line in stdout: $Second"; exit 1 }
  if ($firstIdx -ge $secondIdx) {
    Write-Error "FAIL: expected line order '$First' then '$Second' (got $firstIdx then $secondIdx)"
    exit 1
  }
}

$tmpRoot = if ($env:SUBSTRATE_SMOKE_ROOT -and $env:SUBSTRATE_SMOKE_ROOT.Trim() -ne "") { $env:SUBSTRATE_SMOKE_ROOT } else { New-TempDir "wdap-smoke" }
$keep = ($env:SUBSTRATE_SMOKE_KEEP -and $env:SUBSTRATE_SMOKE_KEEP.Trim() -eq "1")

try {
  $homeDir = Join-Path $tmpRoot "home"
  $substrateHome = Join-Path $tmpRoot "substrate-home"
  $ws = Join-Path $tmpRoot "ws"

  New-Item -ItemType Directory -Force -Path $homeDir, (Join-Path $substrateHome "deps\\packages"), $ws | Out-Null

  $env:HOME = $homeDir
  $env:USERPROFILE = $homeDir
  $env:SUBSTRATE_HOME = $substrateHome

  & $SubstrateExe config global init | Out-Null
  & $SubstrateExe workspace init $ws | Out-Null

  @"
version: 1
name: smoke-hello
description: WDAP smoke fixture (script install).
runnable: true
entrypoints: ["smoke-hello"]
install:
  method: script
  script: |
    set -euo pipefail
    mkdir -p /var/lib/substrate/world-deps/bin
    cat > /var/lib/substrate/world-deps/bin/smoke-hello <<'EOF'
    #!/bin/sh
    echo smoke-hello
    EOF
    chmod +x /var/lib/substrate/world-deps/bin/smoke-hello
probe:
  command: "smoke-hello"
"@ | Set-Content -Path (Join-Path $substrateHome "deps\\packages\\smoke-hello.yaml") -NoNewline

  @"
version: 1
name: smoke-apt-a
description: WDAP smoke fixture (APT; missing by design).
runnable: false
install:
  method: apt
  apt:
    - name: smoke-apt-a
probe:
  command: "sh -c 'exit 1'"
"@ | Set-Content -Path (Join-Path $substrateHome "deps\\packages\\smoke-apt-a.yaml") -NoNewline

  @"
version: 1
name: smoke-apt-b
description: WDAP smoke fixture (APT; pinned; missing by design).
runnable: false
install:
  method: apt
  apt:
    - name: smoke-apt-b
      version: "1"
probe:
  command: "sh -c 'exit 1'"
"@ | Set-Content -Path (Join-Path $substrateHome "deps\\packages\\smoke-apt-b.yaml") -NoNewline

  Push-Location $ws
  try {
    & $SubstrateExe world deps global reset | Out-Null
    & $SubstrateExe world deps workspace reset | Out-Null
    & $SubstrateExe world deps workspace add smoke-hello smoke-apt-a smoke-apt-b | Out-Null

    Write-Host "== Case A: provisioning fails closed on Windows =="
    $r = Invoke-Substrate -Label "world enable --provision-deps --dry-run" -ExpectedExit 4 -Args @("world", "enable", "--provision-deps", "--dry-run")
    Require-Contains $r.Stderr "unsupported on Windows"
    Require-Contains $r.Stderr "substrate world enable --provision-deps"

    Write-Host "== Preflight: world doctor =="
    $r = Invoke-Substrate -Label "world doctor" -ExpectedExit 0 -Args @("world", "doctor")

    if ($env:SUBSTRATE_SMOKE_SLICE_ID -eq "WDAP0") {
      Write-Host "== Runtime cases are skipped for WDAP0 (owned by WDAP1) =="
      Write-Host "OK: WDAP windows smoke"
      exit 0
    }

    Write-Host "== Case B: runtime current sync fails early for APT requirements =="
    $r = Invoke-Substrate -Label "deps current sync --dry-run" -ExpectedExit 4 -Args @("world", "deps", "current", "sync", "--dry-run")
    Require-LineOrder $r.Stdout "smoke-apt-a" "smoke-apt-b=1"
    Require-Contains $r.Stderr "substrate world enable --provision-deps"
    Require-Contains $r.Stderr "unsupported on Windows"

    Write-Host "== Case C: current install explicit args do not add enabled items implicitly =="
    $r = Invoke-Substrate -Label "deps current install smoke-hello" -ExpectedExit 0 -Args @("world", "deps", "current", "install", "smoke-hello")

    Write-Host "== Case D: current install fails early for explicit APT-backed items =="
    $r = Invoke-Substrate -Label "deps current install smoke-apt-a --dry-run" -ExpectedExit 4 -Args @("world", "deps", "current", "install", "smoke-apt-a", "--dry-run")
    Require-Contains $r.Stdout "smoke-apt-a"
    Require-Contains $r.Stderr "substrate world enable --provision-deps"
  } finally {
    Pop-Location
  }

  Write-Host "OK: WDAP windows smoke"
  exit 0
} finally {
  if (-not $keep) {
    Remove-Item -Recurse -Force -Path $tmpRoot -ErrorAction SilentlyContinue
  }
}
