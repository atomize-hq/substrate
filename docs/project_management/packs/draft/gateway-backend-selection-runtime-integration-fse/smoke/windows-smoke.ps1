Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

if (-not $IsWindows) {
  Write-Error "FAIL: windows-smoke.ps1 is intended for Windows"
  exit 4
}

$SubstrateBin = $env:SUBSTRATE_BIN
if ([string]::IsNullOrWhiteSpace($SubstrateBin)) { $SubstrateBin = "substrate" }
if (-not (Get-Command $SubstrateBin -ErrorAction SilentlyContinue)) {
  Write-Error "FAIL: substrate not found on PATH (SUBSTRATE_BIN='$SubstrateBin')"
  exit 3
}
$SubstrateBin = (Get-Command $SubstrateBin -ErrorAction Stop).Source

$tmp = Join-Path ([System.IO.Path]::GetTempPath()) ("substrate-gbsri-smoke-" + [Guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Force -Path $tmp | Out-Null

try {
  $homeDir = Join-Path $tmp "home"
  $substrateHome = Join-Path $tmp "substrate-home"
  $workspace = Join-Path $tmp "workspace"
  New-Item -ItemType Directory -Force -Path $homeDir, $substrateHome, $workspace | Out-Null

  $env:HOME = $homeDir
  $env:USERPROFILE = $homeDir
  $env:SUBSTRATE_HOME = $substrateHome
  $env:SUBSTRATE_WORLD_ENABLED = "1"
  $env:SUBSTRATE_WORLD = "enabled"
  $env:SUBSTRATE_WORLD_SOCKET = Join-Path $tmp "missing.sock"

  @"
llm:
  enabled: true
  gateway:
    enabled: true
  routing:
    default_backend: api:anthropic
"@ | Set-Content -Path (Join-Path $substrateHome "config.yaml") -NoNewline

  @"
id: "gateway-policy"
name: "gateway-policy"

world_fs:
  host_visible: true
  fail_closed:
    routing: false
  write:
    enabled: true

llm:
  allowed_backends:
    - "api:anthropic"

net_allowed: []
cmd_allowed: []
cmd_denied: []
cmd_isolated: []

require_approval: false
allow_shell_operators: true

limits:
  max_memory_mb: null
  max_cpu_percent: null
  max_runtime_ms: null
  max_egress_bytes: null

metadata: {}
"@ | Set-Content -Path (Join-Path $substrateHome "policy.yaml") -NoNewline

Set-Location $workspace

function Invoke-UnsupportedBackendCase {
  param(
    [string] $Action
  )

  $stdout = Join-Path $tmp ("{0}.stdout" -f $Action)
  $stderr = Join-Path $tmp ("{0}.stderr" -f $Action)

  & $SubstrateBin world gateway $Action 1> $stdout 2> $stderr
  if ($LASTEXITCODE -eq 0) {
    Write-Error "FAIL: expected unsupported backend to fail for world gateway $Action"
    exit 1
  }

  if (-not (Select-String -Path $stderr -Pattern "invalid integration" -Quiet)) {
    Write-Error "FAIL: missing invalid integration text for world gateway $Action"
    Get-Content -Path $stderr | Write-Host
    exit 1
  }

  if ((Select-String -Path $stdout -Pattern "cli:codex" -Quiet) -or (Select-String -Path $stderr -Pattern "cli:codex" -Quiet)) {
    Write-Error "FAIL: unsupported backend path mentioned cli:codex fallback for world gateway $Action"
    Get-Content -Path $stderr | Write-Host
    exit 1
  }
}

Invoke-UnsupportedBackendCase -Action "status"
Invoke-UnsupportedBackendCase -Action "sync"
Invoke-UnsupportedBackendCase -Action "restart"

Write-Host "PASS: windows unsupported-backend evidence"
} finally {
  Remove-Item -Recurse -Force -Path $tmp -ErrorAction SilentlyContinue
}
