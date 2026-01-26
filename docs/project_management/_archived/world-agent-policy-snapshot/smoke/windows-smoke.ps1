#!/usr/bin/env pwsh
param(
    [string]$DistroName = 'substrate-wsl'
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$script = Join-Path $PSScriptRoot 'windows-wsl-smoke.ps1'
if (-not (Test-Path -LiteralPath $script)) {
    throw "Missing required smoke script: $script"
}

& $script -DistroName $DistroName
exit $LASTEXITCODE

