#!/usr/bin/env pwsh
param(
    [string]$DistroName = 'substrate-wsl',
    [switch]$Json
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function New-Result($Name, $Status, $Detail, $Remediation) {
    [PSCustomObject]@{
        Name        = $Name
        Status      = $Status
        Detail      = $Detail
        Remediation = $Remediation
    }
}

function Invoke-Check {
    param(
        [string]$Name,
        [scriptblock]$Probe,
        [string]$Remediation
    )
    try {
        $detail = & $Probe
        if ($detail -is [System.Array]) { $detail = ($detail | Out-String).Trim() }
        elseif ($detail -isnot [string]) { $detail = [string]$detail }
        New-Result $Name 'PASS' ($detail.Trim()) $Remediation
    } catch {
        New-Result $Name 'FAIL' ($_.Exception.Message.Trim()) $Remediation
    }
}

$results = @()

$results += Invoke-Check 'Virtualization' {
    $line = systeminfo | Select-String 'Virtualization'
    if (-not $line) { throw 'Virtualization status not reported' }
    if ($line.ToString() -notmatch 'Yes') { throw "Virtualization disabled: $line" }
    $line.ToString().Trim()
} 'Enable VT-x/AMD-V in BIOS/UEFI'

$results += Invoke-Check 'WSL Feature' {
    $feature = Get-WindowsOptionalFeature -Online -FeatureName Microsoft-Windows-Subsystem-Linux
    if ($feature.State -ne 'Enabled') { throw "State: $($feature.State)" }
    "Microsoft-Windows-Subsystem-Linux: $($feature.State)"
} 'Enable Windows Subsystem for Linux feature and reboot'

$results += Invoke-Check 'VirtualMachinePlatform Feature' {
    $feature = Get-WindowsOptionalFeature -Online -FeatureName VirtualMachinePlatform
    if ($feature.State -ne 'Enabled') { throw "State: $($feature.State)" }
    "VirtualMachinePlatform: $($feature.State)"
} 'Enable VirtualMachinePlatform feature and reboot'

$results += Invoke-Check 'WSL Status' {
    $status = & wsl --status 2>&1
    if ($LASTEXITCODE -ne 0) { throw $status }
    $status
} 'Run "wsl --install" or repair WSL'

$results += Invoke-Check "Distro $DistroName" {
    $listing = & wsl -l -v | Out-String
    if ($LASTEXITCODE -ne 0) { throw $listing }
    if ($listing -notmatch [regex]::Escape($DistroName)) { throw 'Distro not found' }
    ($listing -split "`n") | Where-Object { $_ -match $DistroName } | ForEach-Object { $_.Trim() }
} 'Import distro via scripts/windows/wsl-warm.ps1'

$results += Invoke-Check 'Forwarder PID' {
    $pidFile = Join-Path $env:LOCALAPPDATA 'Substrate\forwarder.pid'
    if (-not (Test-Path $pidFile)) { throw 'PID file not found' }
    $pid = [int](Get-Content $pidFile)
    $proc = Get-Process -Id $pid -ErrorAction Stop
    "PID $pid ($($proc.Path))"
} 'Run wsl-warm.ps1 to launch forwarder'

$results += Invoke-Check 'Forwarder Pipe' {
    $pipePath = "\\.\pipe\substrate-agent"
    if (-not (Test-Path $pipePath)) { throw 'Pipe not available' }
    $pipePath
} 'Restart forwarder via wsl-warm.ps1'

$results += Invoke-Check 'Agent Socket' {
    & wsl -d $DistroName -- bash -lc 'test -S /run/substrate.sock'
    if ($LASTEXITCODE -ne 0) { throw 'Socket /run/substrate.sock missing' }
    '/run/substrate.sock present'
} 'Verify substrate-world-agent systemd service is running'

$results += Invoke-Check 'Agent Capabilities' {
    $output = & wsl -d $DistroName -- bash -lc "curl --unix-socket /run/substrate.sock -s http://localhost/v1/capabilities"
    if ($LASTEXITCODE -ne 0) { throw $output }
    $json = $output | ConvertFrom-Json
    "version=$($json.version) features=$($json.features -join ',')"
} 'Inspect agent logs via journalctl -u substrate-world-agent'

$results += Invoke-Check 'nftables' {
    $output = & wsl -d $DistroName -- bash -lc 'nft list tables'
    if ($LASTEXITCODE -ne 0) { throw $output }
    ($output -split "`n" | Select-Object -First 5) -join '; '
} 'Install nftables package inside WSL distro'

$results += Invoke-Check 'Disk (/)' {
    $output = & wsl -d $DistroName -- bash -lc 'df -h /'
    if ($LASTEXITCODE -ne 0) { throw $output }
    ($output -split "`n" | Select-Object -Last 1).Trim()
} 'Free disk space or expand WSL virtual disk'

$results += Invoke-Check 'Agent Logs' {
    & wsl -d $DistroName -- bash -lc 'journalctl -u substrate-world-agent -n 20 --no-pager'
} 'Investigate errors shown in journal'

if ($Json) {
    $results | ConvertTo-Json -Depth 3
} else {
    $results | Format-Table -AutoSize
}

if ($results.Status -contains 'FAIL') {
    Write-Host "One or more checks FAILED" -ForegroundColor Red
    exit 1
} else {
    Write-Host "All checks PASS" -ForegroundColor Green
}
