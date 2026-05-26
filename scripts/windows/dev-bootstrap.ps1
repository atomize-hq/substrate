#!/usr/bin/env pwsh
[CmdletBinding()]
param(
    [string]$RustToolchain = '1.89.0'
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Write-Log {
    param([string]$Message)
    Write-Host "[dev-bootstrap] $Message"
}

function Write-Warn {
    param([string]$Message)
    Write-Host "[dev-bootstrap][WARN] $Message" -ForegroundColor Yellow
}

function Write-ErrorAndExit {
    param(
        [string]$Message,
        [int]$Code = 1
    )
    Write-Host "[dev-bootstrap][ERROR] $Message" -ForegroundColor Red
    exit $Code
}

function Test-Truthy {
    param([string]$Value)
    if (-not $Value) { return $false }
    return $Value.Trim().ToLowerInvariant() -in @('1', 'true', 'yes', 'y', 'on')
}

function Refresh-CargoPath {
    $cargoBin = Join-Path $env:USERPROFILE '.cargo\bin'
    if (Test-Path $cargoBin) {
        $env:Path = "$cargoBin;$env:Path"
    }
}

function Ensure-Rustup {
    if (Get-Command rustup -ErrorAction SilentlyContinue) {
        return
    }

    if (Get-Command winget -ErrorAction SilentlyContinue) {
        Write-Log 'Installing rustup via winget'
        & winget install --id Rustlang.Rustup --silent --accept-package-agreements --accept-source-agreements
    }
    elseif (Get-Command choco -ErrorAction SilentlyContinue) {
        Write-Log 'Installing rustup via Chocolatey'
        & choco install rustup.install -y
    }
    else {
        Write-ErrorAndExit 'rustup is not installed and neither winget nor choco is available. Install rustup, then rerun make dev-bootstrap.'
    }

    Refresh-CargoPath
    if (-not (Get-Command rustup -ErrorAction SilentlyContinue)) {
        Write-ErrorAndExit 'rustup installation completed, but rustup is still not on PATH. Open a new shell and rerun make dev-bootstrap.'
    }
}

function Ensure-RustToolchain {
    Write-Log "Ensuring Rust toolchain $RustToolchain"
    & rustup toolchain install $RustToolchain --profile minimal
    & rustup component add rustfmt clippy llvm-tools --toolchain $RustToolchain
}

$enableWinPreflight = Test-Truthy $env:ENABLE_WIN_PREFLIGHT
if ($enableWinPreflight) {
    Write-ErrorAndExit 'ENABLE_WIN_PREFLIGHT=1 is only supported on Linux hosts because make preflight-win is Linux-only.' 2
}

Refresh-CargoPath
Ensure-Rustup
Refresh-CargoPath

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-ErrorAndExit 'cargo is still not available after rustup installation. Open a new shell and rerun make dev-bootstrap.'
}

Ensure-RustToolchain

if (-not (Get-Command wsl.exe -ErrorAction SilentlyContinue)) {
    Write-Warn 'WSL is not installed. Substrate Windows world flows rely on WSL; install it with `wsl --install` when you are ready.'
}
else {
    Write-Log 'WSL detected; run `wsl --status` if you want to confirm distro readiness.'
}

Write-Host ''
Write-Host 'Next steps:'
Write-Host '  1. Run `pwsh -File scripts\windows\dev-install-substrate.ps1 -Profile debug` to build and wire the local dev install.'
Write-Host '  2. Run `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace --all-targets`.'
