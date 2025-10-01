#!/usr/bin/env pwsh
param(
    [string]$DistroName = 'substrate-wsl'
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Write-Info($Message) { Write-Host "[INFO] $Message" -ForegroundColor Cyan }
function Write-Warn($Message) { Write-Host "[WARN] $Message" -ForegroundColor Yellow }
function Write-ErrorAndExit($Message) { Write-Host "[FAIL] $Message" -ForegroundColor Red; exit 1 }

Write-Info "Stopping forwarder and WSL distro '$DistroName'"

$pidFile = Join-Path $env:LOCALAPPDATA 'Substrate\\forwarder.pid'
if (Test-Path $pidFile) {
    Write-Info "Attempting to stop forwarder recorded in PID file"
    try {
        $forwarderPid = [int](Get-Content $pidFile)
        Stop-Process -Id $forwarderPid -Force -ErrorAction SilentlyContinue
        Write-Info "Forwarder PID $forwarderPid terminated"
    } catch {
        Write-Warn "Unable to terminate PID in ${pidFile}: $_"
    }
    Remove-Item $pidFile -ErrorAction SilentlyContinue
}

# Clean any stray forwarder processes (regardless of install path)
Get-Process -Name substrate-forwarder -ErrorAction SilentlyContinue |
    ForEach-Object {
        Write-Warn "Stopping stray forwarder process Id=$($_.Id) Path=$($_.Path)"
        Stop-Process -Id $_.Id -Force -ErrorAction SilentlyContinue
    }

$pipePath = "\\\\.\\pipe\\substrate-agent"
if (Test-Path $pipePath) {
    Write-Warn "Named pipe $pipePath still exists; it should disappear after forwarder shutdown"
}

Write-Info "Terminating WSL distro $DistroName"
& wsl --terminate $DistroName 2>$null
if ($LASTEXITCODE -ne 0) {
    Write-Warn "wsl --terminate returned non-zero (distro may already be stopped)"
}

# Wait for distro to report Stopped
$timeout = [DateTime]::UtcNow.AddSeconds(60)
while ([DateTime]::UtcNow -lt $timeout) {
    $listing = & wsl -l -v | Out-String
    if ($listing -notmatch [regex]::Escape($DistroName)) {
        Write-Info "Distro $DistroName no longer listed"
        break
    }
    if ($listing -match "$DistroName\s+\d+\s+Stopped") {
        Write-Info "Distro state is Stopped"
        break
    }
    Start-Sleep -Seconds 2
}

if (Test-Path $pipePath) {
    Write-Warn "Pipe $pipePath still present. If subsequent runs fail, delete pipe or reboot."
}

Write-Info "WSL stop complete"
