#!/usr/bin/env pwsh
param(
    [string]$DistroName = 'substrate-wsl',
    [switch]$Json
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function New-Result {
    param(
        [string]$Name,
        [string]$Status,
        [string]$Detail,
        [string]$Remediation
    )
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

function Test-NamedPipe {
    param(
        [Parameter(Mandatory = $true)][string]$Path,
        [int]$TimeoutMs = 2000
    )
    $pattern = '^\\\\.\\pipe\\(.+)$'
    $match = [regex]::Match($Path, $pattern)
    if (-not $match.Success) {
        throw "Invalid pipe path: $Path"
    }
    $pipeName = $match.Groups[1].Value
    $client = New-Object System.IO.Pipes.NamedPipeClientStream('.', $pipeName, [System.IO.Pipes.PipeDirection]::InOut, [System.IO.Pipes.PipeOptions]::None)
    try {
        $client.Connect($TimeoutMs)
        "Connected ($pipeName)"
    } catch {
        throw $_.Exception.Message
    } finally {
        $client.Dispose()
    }
}

function Get-ForwarderTargetInfo {
    param(
        [string]$ConfigPath
    )

    $defaultUds = '/run/substrate.sock'
    $defaultTcpPort = 61337

    $override = $env:SUBSTRATE_FORWARDER_TARGET
    if ($override) {
        $clean = $override.Trim()
        if (-not $clean) {
            throw 'SUBSTRATE_FORWARDER_TARGET is empty'
        }
        $parts = $clean.Split(':', 2)
        $mode = $parts[0].ToLowerInvariant()
        $value = if ($parts.Length -gt 1) { $parts[1].Trim() } else { '' }
        switch ($mode) {
            'tcp' {
                try {
                    $port = if ($value) { [int]$value } else { $defaultTcpPort }
                } catch {
                    throw "Invalid TCP port in SUBSTRATE_FORWARDER_TARGET: '$value'"
                }
                if ($port -lt 1 -or $port -gt 65535) {
                    throw "TCP port out of range in SUBSTRATE_FORWARDER_TARGET: $port"
                }
                return [PSCustomObject]@{
                    Mode     = 'tcp'
                    Endpoint = "127.0.0.1:$port"
                    Source   = 'environment'
                }
            }
            'uds' {
                $path = if ($value) { $value } else { $defaultUds }
                return [PSCustomObject]@{
                    Mode     = 'uds'
                    Endpoint = $path
                    Source   = 'environment'
                }
            }
            'unix' {
                $path = if ($value) { $value } else { $defaultUds }
                return [PSCustomObject]@{
                    Mode     = 'uds'
                    Endpoint = $path
                    Source   = 'environment'
                }
            }
            'unix_socket' {
                $path = if ($value) { $value } else { $defaultUds }
                return [PSCustomObject]@{
                    Mode     = 'uds'
                    Endpoint = $path
                    Source   = 'environment'
                }
            }
            default {
                throw "Unsupported target mode in SUBSTRATE_FORWARDER_TARGET: $mode"
            }
        }
    }

    if ($ConfigPath -and (Test-Path $ConfigPath)) {
        $content = Get-Content -Path $ConfigPath -Raw
        $modeMatch = [regex]::Match($content, '(?im)^\s*mode\s*=\s*"(?<mode>[^"]+)"')
        $mode = if ($modeMatch.Success) { $modeMatch.Groups['mode'].Value.ToLowerInvariant() } else { 'uds' }
        $tcpMatch = [regex]::Match($content, '(?im)^\s*tcp_port\s*=\s*(?<port>\d+)')
        $udsMatch = [regex]::Match($content, '(?im)^\s*uds_path\s*=\s*"(?<path>[^"]+)"')

        switch ($mode) {
            'tcp' {
                try {
                    $port = if ($tcpMatch.Success) { [int]$tcpMatch.Groups['port'].Value } else { $defaultTcpPort }
                } catch {
                    throw "Invalid tcp_port value in $ConfigPath"
                }
                if ($port -lt 1 -or $port -gt 65535) {
                    throw "tcp_port out of range in $ConfigPath: $port"
                }
                return [PSCustomObject]@{
                    Mode     = 'tcp'
                    Endpoint = "127.0.0.1:$port"
                    Source   = "config ($ConfigPath)"
                }
            }
            'uds' {
                $path = if ($udsMatch.Success) { $udsMatch.Groups['path'].Value } else { $defaultUds }
                return [PSCustomObject]@{
                    Mode     = 'uds'
                    Endpoint = $path
                    Source   = "config ($ConfigPath)"
                }
            }
            'unix' {
                $path = if ($udsMatch.Success) { $udsMatch.Groups['path'].Value } else { $defaultUds }
                return [PSCustomObject]@{
                    Mode     = 'uds'
                    Endpoint = $path
                    Source   = "config ($ConfigPath)"
                }
            }
            'unix_socket' {
                $path = if ($udsMatch.Success) { $udsMatch.Groups['path'].Value } else { $defaultUds }
                return [PSCustomObject]@{
                    Mode     = 'uds'
                    Endpoint = $path
                    Source   = "config ($ConfigPath)"
                }
            }
            default {
                throw "Unsupported target mode '$mode' in $ConfigPath"
            }
        }
    }

    return [PSCustomObject]@{
        Mode     = 'uds'
        Endpoint = $defaultUds
        Source   = 'default'
    }
}

$results = @()
$forwarderConfigPath = if ($env:LOCALAPPDATA) { Join-Path $env:LOCALAPPDATA 'Substrate\forwarder.toml' } else { $null }

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

$results += Invoke-Check 'WSL CLI' {
    $cmd = Get-Command wsl -ErrorAction Stop
    "wsl.exe located at $($cmd.Source)"
} 'Install Windows Subsystem for Linux binaries (T-011)'

$results += Invoke-Check 'WSL Status' {
    $status = & wsl --status 2>&1
    if ($LASTEXITCODE -ne 0) { throw $status }
    $status
} 'Run "wsl --install" or repair WSL'

$results += Invoke-Check 'WSL Mount (/mnt/c)' {
    $output = & wsl -d $DistroName -- bash -lc 'mount | grep "/mnt/c"'
    if ($LASTEXITCODE -ne 0) { throw 'Host C: drive not mounted under /mnt/c' }
    ($output -split "`n" | Select-Object -First 1).Trim()
} 'Ensure /mnt/c is mounted inside the distro (T-012)'


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
    Test-NamedPipe -Path $pipePath -TimeoutMs 2000
} 'Restart forwarder via wsl-warm.ps1'

$results += Invoke-Check 'Forwarder Target' {
    $info = Get-ForwarderTargetInfo -ConfigPath $forwarderConfigPath
    "Mode=$($info.Mode); Endpoint=$($info.Endpoint); Source=$($info.Source)"
} 'Set SUBSTRATE_FORWARDER_TARGET or update forwarder.toml [target]'


$results += Invoke-Check 'Forwarder Log' {
    $logDir = Join-Path $env:LOCALAPPDATA 'Substrate\logs'
    if (-not (Test-Path $logDir)) { throw "Log directory missing: $logDir" }
    $latest = Get-ChildItem $logDir -Filter 'forwarder*.log*' -ErrorAction Stop | Sort-Object LastWriteTime -Descending | Select-Object -First 1
    if (-not $latest) { throw 'No forwarder logs found' }
    $age = (Get-Date) - $latest.LastWriteTime
    if ($age.TotalMinutes -gt 10) { throw "Latest log stale ($([int]$age.TotalMinutes) min old): $($latest.Name)" }
    "Log $($latest.Name) updated $([int]$age.TotalSeconds) seconds ago"
} 'Inspect %LOCALAPPDATA%\Substrate\logs for forwarder errors'

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
    $output = & wsl -d $DistroName -- bash -lc 'journalctl -u substrate-world-agent -n 20 --no-pager'
    $text = if ($output -is [System.Array]) { ($output | Out-String).Trim() } else { [string]$output }
    $clean = $text -replace "`0", ''
    if ($LASTEXITCODE -ne 0 -or $clean -match 'There is no distribution') { throw $clean }
    $clean
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

