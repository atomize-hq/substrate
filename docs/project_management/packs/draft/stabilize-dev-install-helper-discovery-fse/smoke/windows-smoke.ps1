$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

$SmokeName = "stabilize-dev-install-helper-discovery-windows-smoke"

function Write-Log {
    param([string]$Message)
    Write-Host "[$SmokeName] $Message"
}

function Invoke-Step {
    param(
        [string]$Label,
        [Parameter(ValueFromRemainingArguments = $true)]
        [string[]]$Command
    )

    Write-Log "RUN $Label"
    & $Command[0] $Command[1..($Command.Length - 1)]
    if ($LASTEXITCODE -ne 0) {
        throw "Step '$Label' failed with exit code $LASTEXITCODE"
    }
}

if (-not $env:REPO_ROOT) {
    $env:REPO_ROOT = (& git rev-parse --show-toplevel).Trim()
}

if (-not (Test-Path (Join-Path $env:REPO_ROOT "scripts/substrate"))) {
    throw "Expected repo root at $($env:REPO_ROOT), but scripts/substrate is missing"
}

Set-Location $env:REPO_ROOT

Write-Log "Windows smoke is compile parity only."
Write-Log "This surface does not claim supported 'substrate world enable' behavior on Windows."

Invoke-Step "cargo-test-shell-world-enable-no-run" cargo test -p shell world_enable --no-run
Invoke-Step "cargo-test-shell-locate-helper-script-no-run" cargo test -p shell locate_helper_script --no-run

Write-Log "Windows smoke complete"
