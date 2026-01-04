$ErrorActionPreference = "Stop"

if (-not $IsWindows) {
  Write-Host "SKIP: yaml-settings-migration Windows smoke (not Windows)"
  exit 0
}

if (-not (Get-Command substrate -ErrorAction SilentlyContinue)) {
  Write-Error "FAIL: substrate not found on PATH"
  exit 1
}

$y0TestHome = Join-Path $env:TEMP ("substrate-y0-home-" + [guid]::NewGuid().ToString("N"))
$y0TestWs = Join-Path $env:TEMP ("substrate-y0-ws-" + [guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Force -Path $y0TestHome | Out-Null
New-Item -ItemType Directory -Force -Path $y0TestWs | Out-Null

try {
  $env:HOME = $y0TestHome
  & substrate config init --force | Out-Null

  $configYaml = Join-Path $y0TestHome ".substrate\\config.yaml"
  $configToml = Join-Path $y0TestHome ".substrate\\config.toml"
  if (-not (Test-Path $configYaml)) { throw "config.yaml missing at $configYaml" }
  if (Test-Path $configToml) { throw "config.toml must not exist at $configToml" }

  & substrate config set world.anchor_mode=follow-cwd | Out-Null
  if (-not (Select-String -Path $configYaml -Pattern "anchor_mode: follow-cwd" -Quiet)) {
    throw "anchor_mode did not persist to config.yaml"
  }

  Write-Host "OK: yaml-settings-migration Windows smoke"
  exit 0
} finally {
  Remove-Item -Recurse -Force $y0TestHome, $y0TestWs -ErrorAction SilentlyContinue
}
