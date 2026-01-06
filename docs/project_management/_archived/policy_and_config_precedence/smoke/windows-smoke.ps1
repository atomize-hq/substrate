$ErrorActionPreference = "Stop"

if (-not $IsWindows) {
  Write-Host "SKIP: policy/config precedence Windows smoke (not Windows)"
  exit 0
}

if (-not (Get-Command substrate -ErrorAction SilentlyContinue)) {
  Write-Error "FAIL: substrate not found on PATH"
  exit 3
}

$tmpRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("substrate-pcp0-" + [System.Guid]::NewGuid().ToString("N"))
$tmpHome = Join-Path $tmpRoot "home"
$tmpWs = Join-Path $tmpRoot "ws"
$tmpNoWs = Join-Path $tmpRoot "no-ws"
New-Item -ItemType Directory -Force -Path $tmpHome | Out-Null
New-Item -ItemType Directory -Force -Path $tmpWs | Out-Null
New-Item -ItemType Directory -Force -Path $tmpNoWs | Out-Null

try {
  $env:SUBSTRATE_HOME = $tmpHome
  $env:HOME = $tmpHome
  $env:USERPROFILE = $tmpHome

  Push-Location $tmpNoWs
  try {
    $null = & substrate config show --json 2>$null
    if ($LASTEXITCODE -ne 2) {
      Write-Error ("FAIL: expected exit code 2 for workspace-scoped config show without a workspace, got: " + $LASTEXITCODE)
      exit 1
    }
  } finally {
    Pop-Location
  }

  & substrate workspace init $tmpWs | Out-Null

  Push-Location $tmpWs
  try {
    & substrate config set world.caged=false | Out-Null
    $env:SUBSTRATE_CAGED = "1"
    $out = & substrate config show --json
    $json = $out | ConvertFrom-Json
    if ($json.world.caged -ne $false) {
      Write-Error ("FAIL: expected world.caged=false, got: " + $json.world.caged)
      exit 1
    }
  } finally {
    Pop-Location
    Remove-Item Env:SUBSTRATE_CAGED -ErrorAction SilentlyContinue
  }
} finally {
  Remove-Item -Recurse -Force $tmpRoot -ErrorAction SilentlyContinue
}

Write-Host "OK: policy/config precedence Windows smoke"
exit 0
