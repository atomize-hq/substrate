#!/usr/bin/env pwsh
param(
    [string]$DistroName = 'substrate-wsl',
    [string]$ProjectPath = (Resolve-Path '..\\..' | Select-Object -ExpandProperty Path),
    [switch]$WhatIf
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Write-Info($Message) { Write-Host "[INFO] $Message" -ForegroundColor Cyan }
function Write-Warn($Message) { Write-Host "[WARN] $Message" -ForegroundColor Yellow }
function Write-ErrorAndExit($Message) { Write-Host "[FAIL] $Message" -ForegroundColor Red; exit 1 }

Write-Info "Starting wsl-warm for distro '$DistroName'"

$projectPath = Resolve-Path $ProjectPath | Select-Object -ExpandProperty Path
Write-Info "Project path: $projectPath"

if (-not (Test-Path (Join-Path $projectPath 'Cargo.toml'))) {
    Write-ErrorAndExit "Project path does not contain Cargo.toml"
}

# Ensure WSL installed
$wslStatus = & wsl --status 2>$null
if ($LASTEXITCODE -ne 0) {
    Write-ErrorAndExit "WSL not available. Run 'wsl --install' first."
}

# Import distro if missing
$distroList = & wsl -l -v | Out-String
if ($distroList -notmatch [regex]::Escape($DistroName)) {
    Write-Info "Importing distro '$DistroName'"

    if ($WhatIf) {
        Write-Warn "WhatIf mode enabled - skipping provisioning"
        return
    }

    $baseUrl = 'https://cdimage.ubuntu.com/ubuntu-wsl/noble/daily-live/current'
    $arch = [System.Runtime.InteropServices.RuntimeInformation]::ProcessArchitecture
    if ($arch -eq [System.Runtime.InteropServices.Architecture]::Arm64) {
        $imageName = 'noble-wsl-arm64.wsl'
    } else {
        $imageName = 'noble-wsl-amd64.wsl'
    }

    $imagePath = Join-Path $env:TEMP $imageName
    $shaPath = Join-Path $env:TEMP 'noble-wsl-SHA256SUMS'

    Write-Info "Downloading Ubuntu WSL image manifest"
    Invoke-WebRequest -Uri "$baseUrl/SHA256SUMS" -OutFile $shaPath
    $expectedLine = Get-Content $shaPath | Where-Object { $_ -match "\*$imageName$" }
    if (-not $expectedLine) {
        Write-ErrorAndExit "SHA256SUMS does not list $imageName"
    }
    $expectedHash = (($expectedLine -split ' \*')[0]).Trim()

    Write-Info "Downloading Ubuntu WSL image ($imageName)"
    Invoke-WebRequest -Uri "$baseUrl/$imageName" -OutFile $imagePath

    $fileHash = (Get-FileHash -Path $imagePath -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($fileHash -ne ($expectedHash.ToLowerInvariant())) {
        Remove-Item $imagePath -ErrorAction SilentlyContinue
        Write-ErrorAndExit "Hash mismatch for $imageName (expected $expectedHash, got $fileHash)"
    }

    $installDir = Join-Path $env:LOCALAPPDATA 'substrate\\wsl'
    New-Item -ItemType Directory -Force $installDir | Out-Null
    & wsl --import $DistroName $installDir $imagePath --version 2
    Remove-Item $imagePath -ErrorAction SilentlyContinue
    Remove-Item $shaPath -ErrorAction SilentlyContinue
}

if ($WhatIf) {
    Write-Warn "WhatIf mode enabled - skipping provisioning"
    return
}

# Copy provisioning script and run
$hostProvisionPath = Join-Path $projectPath 'docs\\dev\\wsl\\provision.sh'
if (-not (Test-Path $hostProvisionPath)) {
    Write-ErrorAndExit "Provisioning script not found at $hostProvisionPath"
}

Write-Info "Updating package cache and running provision script"
& wsl -d $DistroName -- bash -lc "set -euo pipefail; cp /mnt/c/$(($projectPath -replace ':', '') -replace '\\','/')/docs/dev/wsl/provision.sh /tmp/provision.sh && chmod +x /tmp/provision.sh && sudo /tmp/provision.sh"
if ($LASTEXITCODE -ne 0) {
    Write-ErrorAndExit "Provision script failed"
}

# Build world-agent if absent
$agentHostPath = Join-Path $projectPath 'target\\release\\world-agent.exe'
if (-not (Test-Path $agentHostPath)) {
    Write-Info "Building world-agent (release)"
    cargo build -p world-agent --release
}

# Copy agent binary into WSL
Write-Info "Copying world-agent into WSL"
$agentUnixPath = ($projectPath -replace ':', '') -replace '\\','/'
& wsl -d $DistroName -- bash -lc "set -euo pipefail; sudo cp /mnt/c/$agentUnixPath/target/release/world-agent.exe /usr/local/bin/substrate-world-agent && sudo chmod 755 /usr/local/bin/substrate-world-agent"

# Restart service
Write-Info "Restarting substrate-world-agent service"
& wsl -d $DistroName -- bash -lc "sudo systemctl restart substrate-world-agent.service"
if ($LASTEXITCODE -ne 0) {
    Write-ErrorAndExit "Failed to restart agent service"
}

# Build forwarder if needed
$forwarderHostPath = Join-Path $projectPath 'target\\release\\substrate-forwarder.exe'
if (-not (Test-Path $forwarderHostPath)) {
    Write-Info "Building substrate-forwarder (release)"
    cargo build -p substrate-forwarder --release
}

# Launch forwarder
Write-Info "Launching forwarder"
$logDir = Join-Path $env:LOCALAPPDATA 'Substrate\\logs'
New-Item -ItemType Directory -Force $logDir | Out-Null
$pipePath = "\\\\.\\pipe\\substrate-agent"
$pidFile = Join-Path $env:LOCALAPPDATA 'Substrate\\forwarder.pid'
if (Test-Path $pidFile) {
    Write-Warn "Forwarder PID file exists; attempting cleanup"
    $existingPid = Get-Content $pidFile
    Stop-Process -Id $existingPid -ErrorAction SilentlyContinue
    Remove-Item $pidFile -ErrorAction SilentlyContinue
}
$forwarderProcess = Start-Process -FilePath $forwarderHostPath -ArgumentList "--distro", $DistroName, "--pipe", $pipePath, "--log-dir", $logDir -WindowStyle Hidden -PassThru
Set-Content $pidFile -Value $forwarderProcess.Id

# Wait for pipe
Write-Info "Waiting for forwarder pipe $pipePath"
$timeout = [DateTime]::UtcNow.AddSeconds(30)
while (-not (Test-Path $pipePath)) {
    if ([DateTime]::UtcNow -gt $timeout) {
        Write-ErrorAndExit "Forwarder pipe not available after 30s"
    }
    Start-Sleep -Seconds 1
}
Write-Info "Forwarder pipe ready"

Write-Info "Warm complete"
