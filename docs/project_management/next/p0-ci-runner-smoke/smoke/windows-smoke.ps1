Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

Write-Host "[windows-smoke] substrate --version"
if (-not (Get-Command substrate -ErrorAction SilentlyContinue)) {
    throw "substrate not found on PATH"
}

$ver = substrate --version
if ([string]::IsNullOrWhiteSpace($ver)) {
    throw "substrate --version produced empty output"
}

Write-Host "OK: $ver"
