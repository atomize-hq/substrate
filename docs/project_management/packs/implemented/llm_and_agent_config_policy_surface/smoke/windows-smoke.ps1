Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$SubstrateBin = if ($env:SUBSTRATE_BIN) { $env:SUBSTRATE_BIN } else { "substrate" }
$SmokeDir = Split-Path -Parent $MyInvocation.MyCommand.Path

function Test-CommandExists($cmd) {
  $null -ne (Get-Command $cmd -ErrorAction SilentlyContinue)
}

if (-not (Test-CommandExists $SubstrateBin)) {
  Write-Error "llm_and_agent_config_policy_surface: substrate binary not found (SUBSTRATE_BIN=$SubstrateBin)"
  exit 3
}

$tmpRoot = if ($env:SUBSTRATE_SMOKE_ROOT) { $env:SUBSTRATE_SMOKE_ROOT } else { Join-Path ([System.IO.Path]::GetTempPath()) ([System.Guid]::NewGuid().ToString()) }
New-Item -ItemType Directory -Force -Path $tmpRoot | Out-Null

try {
  $env:SUBSTRATE_HOME = if ($env:SUBSTRATE_HOME) { $env:SUBSTRATE_HOME } else { Join-Path $tmpRoot "substrate-home" }
  $workspace = Join-Path $tmpRoot "workspace"
  New-Item -ItemType Directory -Force -Path $workspace | Out-Null
  Push-Location $workspace

  & $SubstrateBin workspace init --force | Out-Null
  & $SubstrateBin config global init --force | Out-Null
  & $SubstrateBin policy global init --force | Out-Null

  $agentsDir = Join-Path $env:SUBSTRATE_HOME "agents"
  New-Item -ItemType Directory -Force -Path $agentsDir | Out-Null
  $agentFile = Join-Path $agentsDir "codex.yaml"
  @"
version: 1
id: codex
config:
  kind: cli
  enabled: true
  execution:
    scope: world
  cli:
    binary: codex
    mode: persistent
  capabilities:
    llm: true
policy_overlay:
  agents:
    fail_closed:
      routing: true
"@ | Set-Content -Path $agentFile -Encoding UTF8

  & $SubstrateBin config global set `
    "llm.enabled=true" `
    "llm.gateway.enabled=true" `
    "llm.gateway.mode=in_world" `
    "llm.routing.default_backend=cli:codex" `
    "agents.enabled=true" `
    "agents.defaults.execution.scope=world" `
    "agents.defaults.cli.mode=persistent" | Out-Null

  & $SubstrateBin policy global set `
    "llm.fail_closed.routing=true" `
    "llm.require_approval=false" `
    "llm.allowed_backends+=cli:codex" `
    "agents.allowed_backends+=cli:codex" `
    "agents.fail_closed.routing=true" | Out-Null

  $failed = $false
  try { & $SubstrateBin config global set "llm.unknown_key=true" | Out-Null; $failed = $true } catch {}
  if ($failed) { throw "expected config unknown key to fail" }
  $failed = $false
  try { & $SubstrateBin policy global set "agents.unknown_key=true" | Out-Null; $failed = $true } catch {}
  if ($failed) { throw "expected policy unknown key to fail" }

  & $SubstrateBin config current show --explain | Out-Null
  & $SubstrateBin policy current show --explain | Out-Null

  Write-Output "OK: llm_and_agent_config_policy_surface smoke passed"
} finally {
  Pop-Location | Out-Null
  if ($env:SUBSTRATE_SMOKE_KEEP -ne "1") {
    Remove-Item -Recurse -Force $tmpRoot -ErrorAction SilentlyContinue
  }
}
