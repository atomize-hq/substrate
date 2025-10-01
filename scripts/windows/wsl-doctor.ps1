#!/usr/bin/env pwsh
param(
    [string]$DistroName = 'substrate-wsl',
    [switch]$Json
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
[Console]::OutputEncoding = [Text.Encoding]::UTF8

# ---------- result helpers ----------

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

# Extended: if a probe returns @{ Status = 'SKIP'|'PASS'|'FAIL'; Detail = '...' }
# we honor that; otherwise strings/arrays imply PASS; throw => FAIL.
function Invoke-Check {
    param(
        [string]$Name,
        [scriptblock]$Probe,
        [string]$Remediation
    )
    try {
        $ret = & $Probe

        # passthrough for custom result objects/hashtables
        if ($ret -is [hashtable] -and $ret.ContainsKey('Status')) {
            $status = [string]$ret.Status
            $detail = [string]$ret.Detail
            return (New-Result $Name $status ($detail.Trim()) $Remediation)
        }
        elseif ($ret -is [pscustomobject] -and ($ret.PSObject.Properties.Name -contains 'Status')) {
            $status = [string]$ret.Status
            $detail = [string]$ret.Detail
            return (New-Result $Name $status ($detail.Trim()) $Remediation)
        }

        # default: strings/arrays => PASS
        if ($ret -is [System.Array]) { $ret = ($ret | Out-String) }
        $detail = [string]$ret
        New-Result $Name 'PASS' ($detail.Trim()) $Remediation
    } catch {
        New-Result $Name 'FAIL' ($_.Exception.Message.Trim()) $Remediation
    }
}

# ---------- utilities ----------

function Test-IsAdmin {
    $p = New-Object Security.Principal.WindowsPrincipal([Security.Principal.WindowsIdentity]::GetCurrent())
    return $p.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}
$IsAdmin = Test-IsAdmin

function Normalize-WSLName([string]$s) {
    if (-not $s) { return $s }
    $n = ($s -replace '\s+', ' ').Trim() -replace '\s*\(Default\)\s*$', ''
    try { return $n.Normalize([Text.NormalizationForm]::FormKC) } catch { return $n }
}

function Get-InstalledWslDistros {
    $names = @()
    try {
        $q = & wsl.exe -l -q 2>$null
        if ($q) { $names += $q }
    } catch {}
    if (-not $names -or $names.Count -eq 0) {
        try {
            & wsl.exe -l -v 2>$null | Select-Object -Skip 1 | ForEach-Object {
                ($_ -split '\s{2,}')[0]
            } | ForEach-Object { $names += $_ }
        } catch {}
    }
    $names | ForEach-Object { Normalize-WSLName $_ } |
            Where-Object { $_ } |
            Sort-Object -Unique
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
        if (-not $clean) { throw 'SUBSTRATE_FORWARDER_TARGET is empty' }
        $parts = $clean.Split(':', 2)
        $mode  = $parts[0].ToLowerInvariant()
        $value = if ($parts.Length -gt 1) { $parts[1].Trim() } else { '' }
        switch ($mode) {
            'tcp' {
                try { $port = if ($value) { [int]$value } else { $defaultTcpPort } }
                catch { throw "Invalid TCP port in SUBSTRATE_FORWARDER_TARGET: '$value'" }
                if ($port -lt 1 -or $port -gt 65535) { throw "TCP port out of range in SUBSTRATE_FORWARDER_TARGET: $port" }
                return [PSCustomObject]@{ Mode='tcp'; Endpoint="127.0.0.1:$port"; Source='environment' }
            }
            { $_ -in 'uds','unix','unix_socket' } {
                $path = if ($value) { $value } else { $defaultUds }
                return [PSCustomObject]@{ Mode='uds'; Endpoint=$path; Source='environment' }
            }
            default { throw "Unsupported target mode in SUBSTRATE_FORWARDER_TARGET: $mode" }
        }
    }

    if ($ConfigPath -and (Test-Path $ConfigPath)) {
        $content  = Get-Content -Path $ConfigPath -Raw
        $mode     = ([regex]::Match($content, '(?im)^\s*mode\s*=\s*"(?<mode>[^"]+)"')).Groups['mode'].Value.ToLowerInvariant()
        if (-not $mode) { $mode = 'uds' }
        $tcpMatch = [regex]::Match($content, '(?im)^\s*tcp_port\s*=\s*(?<port>\d+)')
        $udsMatch = [regex]::Match($content, '(?im)^\s*uds_path\s*=\s*"(?<path>[^"]+)"')

        switch ($mode) {
            'tcp' {
                try { $port = if ($tcpMatch.Success) { [int]$tcpMatch.Groups['port'].Value } else { $defaultTcpPort } }
                catch { throw "Invalid tcp_port value in $ConfigPath" }
                if ($port -lt 1 -or $port -gt 65535) { throw "tcp_port out of range in ${ConfigPath}: $port" }
                return [PSCustomObject]@{ Mode='tcp'; Endpoint="127.0.0.1:$port"; Source="config ($ConfigPath)" }
            }
            { $_ -in 'uds','unix','unix_socket' } {
                $path = if ($udsMatch.Success) { $udsMatch.Groups['path'].Value } else { $defaultUds }
                return [PSCustomObject]@{ Mode='uds'; Endpoint=$path; Source="config ($ConfigPath)" }
            }
            default { throw "Unsupported target mode '$mode' in $ConfigPath" }
        }
    }

    [PSCustomObject]@{ Mode='uds'; Endpoint=$defaultUds; Source='default' }
}

# ---------- checks ----------

$results = @()
$forwarderConfigPath = if ($env:LOCALAPPDATA) { Join-Path $env:LOCALAPPDATA 'Substrate\forwarder.toml' } else { $null }
$script:PipeOk = $false

# Virtualization (tolerant of VBS): PASS if hypervisor present OR WSL kernel exists
$results += Invoke-Check 'Virtualization' {
    $virtOk = $false
    try { $virtOk = (Get-CimInstance Win32_ComputerSystem).HypervisorPresent } catch {}
    if (-not $virtOk) {
        try {
            $status = & wsl.exe --status 2>$null
            if ($status -match 'Kernel version:\s*\d') { $virtOk = $true }
        } catch {}
    }
    if (-not $virtOk) { throw 'Hypervisor not detected and WSL kernel not reported' }
    "HypervisorPresent=$virtOk"
} 'Enable virtualization in BIOS/UEFI (VT-x/AMD-V) or ensure WSL2 kernel is installed'

# WSL feature — SKIP if not admin
$results += Invoke-Check 'WSL Feature' {
    if (-not $IsAdmin) { return @{ Status='SKIP'; Detail='Requires elevation' } }
    $feature = Get-WindowsOptionalFeature -Online -FeatureName Microsoft-Windows-Subsystem-Linux
    if ($feature.State -ne 'Enabled') { throw "State=$($feature.State)" }
    "Microsoft-Windows-Subsystem-Linux: $($feature.State)"
} 'Enable Windows Subsystem for Linux feature and reboot'

# VirtualMachinePlatform feature — SKIP if not admin
$results += Invoke-Check 'VirtualMachinePlatform Feature' {
    if (-not $IsAdmin) { return @{ Status='SKIP'; Detail='Requires elevation' } }
    $feature = Get-WindowsOptionalFeature -Online -FeatureName VirtualMachinePlatform
    if ($feature.State -ne 'Enabled') { throw "State=$($feature.State)" }
    "VirtualMachinePlatform: $($feature.State)"
} 'Enable VirtualMachinePlatform feature and reboot'

# WSL CLI present
$results += Invoke-Check 'WSL CLI' {
    $cmd = Get-Command wsl -ErrorAction Stop
    "wsl.exe located at $($cmd.Source)"
} 'Install Windows Subsystem for Linux binaries (T-011)'

# WSL Status (verbatim)
$results += Invoke-Check 'WSL Status' {
    $status = & wsl --status 2>&1
    if ($LASTEXITCODE -ne 0) { throw $status }
    $status
} 'Run "wsl --install" or repair WSL'

# Distro exists (robust normalization)
$results += Invoke-Check ("Distro $DistroName") {
    $distros = Get-InstalledWslDistros
    $want    = Normalize-WSLName $DistroName
    $found   = ($distros | Where-Object { $_ -ieq $want }) -ne $null
    if (-not $found) { throw ("Distro not found (saw: {0})" -f ($distros -join ', ')) }
    "Installed"
} 'Import or create the distro (see scripts/windows/wsl-warm.ps1)'

# Host C: mounted under /mnt/c
$results += Invoke-Check 'WSL Mount (/mnt/c)' {
    $output = & wsl -d $DistroName -- bash -lc 'mount | grep "/mnt/c"'
    if ($LASTEXITCODE -ne 0) { throw 'Host C: drive not mounted under /mnt/c' }
    ($output -split "`n" | Select-Object -First 1).Trim()
} 'Ensure /mnt/c is mounted inside the distro (T-012)'

# Forwarder Pipe
$results += Invoke-Check 'Forwarder Pipe' {
    $pipePath = "\\.\pipe\substrate-agent"
    $res = Test-NamedPipe -Path $pipePath -TimeoutMs 2000
    $script:PipeOk = $true
    $res
} 'Restart forwarder via wsl-warm.ps1'


# Forwarder PID (SKIP if pipe is good but no PID/log yet)
$results += Invoke-Check 'Forwarder PID' {
    $pidFile = Join-Path $env:LOCALAPPDATA 'Substrate\forwarder.pid'

    if (-not (Test-Path $pidFile)) {
        if ($script:PipeOk) { return @{ Status='SKIP'; Detail='PID file not found' } }
        throw 'PID file not found'
    }

    $txt = (Get-Content $pidFile -ErrorAction Stop).Trim()
    $ForwarderPid = 0
    if (-not [int]::TryParse($txt, [ref]$ForwarderPid)) { throw "Invalid PID file contents: '$txt'" }

    $proc = Get-Process -Id $ForwarderPid -ErrorAction SilentlyContinue
    if (-not $proc) {
        if ($script:PipeOk) { return @{ Status='SKIP'; Detail=("Process {0} not running (pipe OK)" -f $ForwarderPid) } }
        throw ("Process {0} not running" -f $ForwarderPid)
    }

    "PID $ForwarderPid ($($proc.Path))"
} 'Run wsl-warm.ps1 to launch forwarder'

# Forwarder Target
$results += Invoke-Check 'Forwarder Target' {
    $info = Get-ForwarderTargetInfo -ConfigPath $forwarderConfigPath
    "Mode=$($info.Mode); Endpoint=$($info.Endpoint); Source=$($info.Source)"
} 'Set SUBSTRATE_FORWARDER_TARGET or update forwarder.toml [target]'

# Forwarder Log (SKIP if pipe ok but no logs)
$results += Invoke-Check 'Forwarder Log' {
    $logDir = Join-Path $env:LOCALAPPDATA 'Substrate\logs'
    if (-not (Test-Path $logDir)) {
        if ($script:PipeOk) { return @{ Status='SKIP'; Detail="Log directory missing: $logDir" } }
        throw "Log directory missing: $logDir"
    }
    $latest = Get-ChildItem $logDir -Filter 'forwarder*.log*' -ErrorAction SilentlyContinue |
              Sort-Object LastWriteTime -Descending | Select-Object -First 1
    if (-not $latest) {
        if ($script:PipeOk) { return @{ Status='SKIP'; Detail='No forwarder logs found' } }
        throw 'No forwarder logs found'
    }
    $age = (Get-Date) - $latest.LastWriteTime
    if ($age.TotalMinutes -gt 10) { throw "Latest log stale ($([int]$age.TotalMinutes) min old): $($latest.Name)" }
    "Log $($latest.Name) updated $([int]$age.TotalSeconds) seconds ago"
} 'Inspect %LOCALAPPDATA%\Substrate\logs for forwarder errors'

# Agent socket
$results += Invoke-Check 'Agent Socket' {
    & wsl -d $DistroName -- bash -lc 'test -S /run/substrate.sock'
    if ($LASTEXITCODE -ne 0) { throw '/run/substrate.sock missing' }
    '/run/substrate.sock present'
} 'Verify substrate-world-agent systemd service is running'

# Agent capabilities
$results += Invoke-Check 'Agent Capabilities' {
    $output = & wsl -d $DistroName -- bash -lc "curl --unix-socket /run/substrate.sock -s http://localhost/v1/capabilities"
    if ($LASTEXITCODE -ne 0) { throw $output }
    $json = $output | ConvertFrom-Json
    "version=$($json.version) features=$($json.features -join ',')"
} 'Inspect agent logs via journalctl -u substrate-world-agent'

# nftables
$results += Invoke-Check 'nftables' {
    $output = & wsl -d $DistroName -- bash -lc 'nft list tables'
    if ($LASTEXITCODE -ne 0) { throw $output }
    ($output -split "`n" | Select-Object -First 5) -join '; '
} 'Install nftables package inside WSL distro'

# disk root
$results += Invoke-Check 'Disk (/)' {
    $output = & wsl -d $DistroName -- bash -lc 'df -h /'
    if ($LASTEXITCODE -ne 0) { throw $output }
    ($output -split "`n" | Select-Object -Last 1).Trim()
} 'Free disk space or expand WSL virtual disk'

# agent logs tail
$results += Invoke-Check 'Agent Logs' {
    $output = & wsl -d $DistroName -- bash -lc 'journalctl -u substrate-world-agent -n 20 --no-pager'
    $text = if ($output -is [System.Array]) { ($output | Out-String).Trim() } else { [string]$output }
    $clean = $text -replace "`0", ''
    if ($LASTEXITCODE -ne 0 -or $clean -match 'There is no distribution') { throw $clean }
    $clean
} 'Investigate errors shown in journal'

# ---------- output ----------

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
