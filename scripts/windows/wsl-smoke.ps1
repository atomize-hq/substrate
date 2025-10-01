#!/usr/bin/env pwsh
param(
    [string]$DistroName = 'substrate-wsl',
    [string]$ProjectPath = (Resolve-Path '..\\..' | Select-Object -ExpandProperty Path),
    [switch]$SkipWarm
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Invoke-Step {
    param(
        [string]$Name,
        [scriptblock]$Block
    )
    Write-Host "[STEP] $Name" -ForegroundColor Cyan
    try {
        & $Block
        Write-Host "[PASS] $Name" -ForegroundColor Green
    } catch {
        Write-Host "[FAIL] $Name - $_" -ForegroundColor Red
        throw
    }
}

if (-not $SkipWarm) {
    Invoke-Step "Warm environment" {
        pwsh -File scripts/windows/wsl-warm.ps1 -DistroName $DistroName -ProjectPath $ProjectPath | Out-Host
    }
}

Invoke-Step "Forwarder pipe capabilities (HTTP 200)" {
    $resp = pwsh -File scripts/windows/pipe-http.ps1 -PipePath "\\.\pipe\substrate-agent" -TimeoutSeconds 10
    if ($LASTEXITCODE -ne 0) { throw $resp }
    if ($resp -notmatch '^Status: .*200\s+OK') { throw "Unexpected status: $resp" }
    $resp | Out-Host
}

Invoke-Step "Doctor checks" {
    pwsh -File scripts/windows/wsl-doctor.ps1 -DistroName $DistroName | Out-Host
}

$tracePath = Join-Path $env:USERPROFILE '.substrate\trace.jsonl'
if (-not (Test-Path $tracePath)) {
    New-Item -ItemType File -Force $tracePath | Out-Null
}

Invoke-Step "Non-PTY command produces world span" {
    $marker = [guid]::NewGuid().ToString()
    substrate -c "python - <<'PY'
import pathlib
pathlib.Path('win_smoke.txt').write_text('$marker')
PY" | Out-Host
    $entry = Get-Content $tracePath | Select-Object -Last 1 | ConvertFrom-Json
    if (-not $entry.world_id) { throw 'world_id missing from span' }
    if (-not ($entry.fs_diff.writes -match 'win_smoke.txt')) { throw 'fs_diff does not mention win_smoke.txt' }
}

Invoke-Step "PTY command" {
    $output = substrate --pty -c "bash -lc 'echo pty-smoke'"
    if ($output -notmatch 'pty-smoke') { throw 'PTY output missing expected text' }
}

Invoke-Step "Replay" {
    $last = Get-Content $tracePath | Select-Object -Last 1 | ConvertFrom-Json
    if (-not $last.span_id) { throw 'span_id missing' }
    $replay = substrate replay $last.span_id 2>&1
    if ($LASTEXITCODE -ne 0) { throw "Replay failed: $replay" }
}

Invoke-Step "Forwarder restart resilience" {
    pwsh -File scripts/windows/wsl-stop.ps1 -DistroName $DistroName | Out-Host
    pwsh -File scripts/windows/wsl-warm.ps1 -DistroName $DistroName -ProjectPath $ProjectPath | Out-Host
    substrate -c "echo restart-smoke" | Out-Host
}

Write-Host "Smoke suite completed successfully" -ForegroundColor Green
