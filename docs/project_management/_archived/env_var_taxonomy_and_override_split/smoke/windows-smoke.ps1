$ErrorActionPreference = "Stop"

if (-not $IsWindows) {
  Write-Host "SKIP: env var override split Windows smoke (not Windows)"
  exit 0
}

if (-not (Get-Command substrate -ErrorAction SilentlyContinue)) {
  Write-Error "FAIL: substrate not found on PATH"
  exit 3
}

$tmpRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("substrate-ev0-" + [System.Guid]::NewGuid().ToString("N"))
$tmpHome = Join-Path $tmpRoot "home"
$tmpWs = Join-Path $tmpRoot "ws"
New-Item -ItemType Directory -Force -Path $tmpHome | Out-Null
New-Item -ItemType Directory -Force -Path $tmpWs | Out-Null

try {
  $env:SUBSTRATE_HOME = $tmpHome
  $env:HOME = $tmpHome
  $env:USERPROFILE = $tmpHome

  & substrate config global init --force | Out-Null
  & substrate config global set policy.mode=observe | Out-Null
  & substrate config global set world.caged=true | Out-Null
  & substrate config global set world.anchor_mode=follow-cwd | Out-Null

  $out = & substrate --no-world --shell cmd.exe -c "echo %SUBSTRATE_POLICY_MODE%|%SUBSTRATE_CAGED%|%SUBSTRATE_ANCHOR_MODE%"
  $value = ($out | Select-Object -Last 1).Trim()
  if ($value -ne "observe|1|follow-cwd") {
    Write-Error ("FAIL: expected observe|1|follow-cwd from config, got: " + $value)
    exit 1
  }

  $env:SUBSTRATE_POLICY_MODE = "disabled"
  $env:SUBSTRATE_CAGED = "0"
  $env:SUBSTRATE_ANCHOR_MODE = "workspace"
  $out = & substrate --no-world --shell cmd.exe -c "echo %SUBSTRATE_POLICY_MODE%|%SUBSTRATE_CAGED%|%SUBSTRATE_ANCHOR_MODE%"
  Remove-Item Env:SUBSTRATE_POLICY_MODE -ErrorAction SilentlyContinue
  Remove-Item Env:SUBSTRATE_CAGED -ErrorAction SilentlyContinue
  Remove-Item Env:SUBSTRATE_ANCHOR_MODE -ErrorAction SilentlyContinue
  $value = ($out | Select-Object -Last 1).Trim()
  if ($value -ne "observe|1|follow-cwd") {
    Write-Error ("FAIL: expected legacy SUBSTRATE_* to not override, got: " + $value)
    exit 1
  }

  $env:SUBSTRATE_OVERRIDE_POLICY_MODE = "enforce"
  $env:SUBSTRATE_OVERRIDE_CAGED = "0"
  $env:SUBSTRATE_OVERRIDE_ANCHOR_MODE = "workspace"
  $out = & substrate --no-world --shell cmd.exe -c "echo %SUBSTRATE_POLICY_MODE%|%SUBSTRATE_CAGED%|%SUBSTRATE_ANCHOR_MODE%"
  Remove-Item Env:SUBSTRATE_OVERRIDE_POLICY_MODE -ErrorAction SilentlyContinue
  Remove-Item Env:SUBSTRATE_OVERRIDE_CAGED -ErrorAction SilentlyContinue
  Remove-Item Env:SUBSTRATE_OVERRIDE_ANCHOR_MODE -ErrorAction SilentlyContinue
  $value = ($out | Select-Object -Last 1).Trim()
  if ($value -ne "enforce|0|workspace") {
    Write-Error ("FAIL: expected override enforce|0|workspace, got: " + $value)
    exit 1
  }

  & substrate workspace init $tmpWs | Out-Null

  Push-Location $tmpWs
  try {
    & substrate config set policy.mode=observe | Out-Null
    & substrate config set world.caged=true | Out-Null
    & substrate config set world.anchor_mode=follow-cwd | Out-Null

    $env:SUBSTRATE_OVERRIDE_POLICY_MODE = "enforce"
    $env:SUBSTRATE_OVERRIDE_CAGED = "0"
    $env:SUBSTRATE_OVERRIDE_ANCHOR_MODE = "workspace"
    $out = & substrate --no-world --shell cmd.exe -c "echo %SUBSTRATE_POLICY_MODE%|%SUBSTRATE_CAGED%|%SUBSTRATE_ANCHOR_MODE%"
    Remove-Item Env:SUBSTRATE_OVERRIDE_POLICY_MODE -ErrorAction SilentlyContinue
    Remove-Item Env:SUBSTRATE_OVERRIDE_CAGED -ErrorAction SilentlyContinue
    Remove-Item Env:SUBSTRATE_OVERRIDE_ANCHOR_MODE -ErrorAction SilentlyContinue
    $value = ($out | Select-Object -Last 1).Trim()
    if ($value -ne "observe|1|follow-cwd") {
      Write-Error ("FAIL: expected workspace to win over overrides, got: " + $value)
      exit 1
    }

    $env:SUBSTRATE_OVERRIDE_POLICY_MODE = "bogus"
    & substrate config show --json 2>$null | Out-Null
    $code = $LASTEXITCODE
    Remove-Item Env:SUBSTRATE_OVERRIDE_POLICY_MODE -ErrorAction SilentlyContinue
    if ($code -ne 2) {
      Write-Error ("FAIL: expected exit code 2 for invalid override value, got: " + $code)
      exit 1
    }

    $env:SUBSTRATE_OVERRIDE_CAGED = "bogus"
    & substrate config show --json 2>$null | Out-Null
    $code = $LASTEXITCODE
    Remove-Item Env:SUBSTRATE_OVERRIDE_CAGED -ErrorAction SilentlyContinue
    if ($code -ne 2) {
      Write-Error ("FAIL: expected exit code 2 for invalid override boolean, got: " + $code)
      exit 1
    }
  } finally {
    Pop-Location
  }
} finally {
  Remove-Item -Recurse -Force $tmpRoot -ErrorAction SilentlyContinue
  Remove-Item Env:SUBSTRATE_HOME -ErrorAction SilentlyContinue
  Remove-Item Env:HOME -ErrorAction SilentlyContinue
  Remove-Item Env:USERPROFILE -ErrorAction SilentlyContinue
}

Write-Host "OK: env var override split Windows smoke"
exit 0
