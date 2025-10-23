#!/usr/bin/env pwsh
<#!
.SYNOPSIS
    Uninstall Substrate on Windows hosts via PowerShell.
.DESCRIPTION
    Stops forwarder processes, removes PATH/profile integrations, deletes the
    local installation prefix, and optionally tears down the WSL distro used by
    Substrate.
.PARAMETER Prefix
    Installation prefix to remove (defaults to %LOCALAPPDATA%\Substrate).
.PARAMETER DistroName
    WSL distribution name to clean up (defaults to substrate-wsl).
.PARAMETER RemoveWSLDistro
    Unregister the WSL distribution after stopping services.
.PARAMETER DryRun
    Print actions without executing them.
.EXAMPLE
    pwsh -File uninstall-substrate.ps1
.EXAMPLE
    pwsh -File uninstall-substrate.ps1 -RemoveWSLDistro
#>

[CmdletBinding()]
param(
    [string]$Prefix = (Join-Path $env:LOCALAPPDATA 'Substrate'),
    [string]$DistroName = 'substrate-wsl',
    [switch]$RemoveWSLDistro,
    [switch]$DryRun
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Write-Log {
    param([string]$Message)
    Write-Host "[substrate-uninstall] $Message"
}
function Write-Warn {
    param([string]$Message)
    Write-Host "[substrate-uninstall][WARN] $Message" -ForegroundColor Yellow
}

$dry = $DryRun.IsPresent
$forwarderPidPath = Join-Path $env:LOCALAPPDATA 'Substrate\forwarder.pid'
$profileScript = Join-Path $Prefix 'substrate-profile.ps1'
$profilePattern = '# Added by Substrate installer \(Windows\)\r?\nif \(Test-Path ''[^'']+''\) \{\r?\n    \. ''[^'']+''\r?\n\}\r?\n?'
$profileTargets = @($PROFILE.CurrentUserAllHosts, $PROFILE.CurrentUserCurrentHost) | Where-Object { $_ }

Write-Log "Stopping forwarder (if running)"
if (Test-Path $forwarderPidPath) {
    $forwarderPid = Get-Content -Path $forwarderPidPath -ErrorAction SilentlyContinue
    if ($forwarderPid -and -not $dry) {
        try { Stop-Process -Id [int]$forwarderPid -ErrorAction SilentlyContinue } catch {}
    }
    if ($dry) {
        Write-Log "[dry-run] Remove-Item $forwarderPidPath"
    } else {
        Remove-Item -Force -Path $forwarderPidPath -ErrorAction SilentlyContinue
    }
}

# Kill any lingering host processes so files can be deleted
if (-not $dry) {
    foreach ($name in 'substrate-forwarder', 'substrate', 'host-proxy') {
        Get-Process -Name $name -ErrorAction SilentlyContinue |
            ForEach-Object {
                try {
                    Stop-Process -Id $_.Id -Force -ErrorAction SilentlyContinue
                } catch {}
            }
    }
}

Write-Log "Removing profile snippet"
foreach ($target in $profileTargets | Select-Object -Unique) {
    if (-not (Test-Path $target)) { continue }
    $content = if ($dry) { Get-Content -Raw -Path $target } else { Get-Content -Raw -Path $target -ErrorAction SilentlyContinue }
    if (-not $content) { continue }
    $updated = [regex]::Replace($content, $profilePattern, '', [System.Text.RegularExpressions.RegexOptions]::IgnoreCase)
    if ($updated -ne $content) {
        if ($dry) {
            Write-Log "[dry-run] Update $target to remove Substrate snippet"
        } else {
            $normalized = $updated.TrimEnd() + [Environment]::NewLine
            Set-Content -Path $target -Value $normalized -Encoding UTF8
        }
    }
}

if ($dry) {
    Write-Log "[dry-run] Remove profile helper at $profileScript"
} else {
    Remove-Item -Force -Path $profileScript -ErrorAction SilentlyContinue
}

Write-Log "Clearing installation directory: $Prefix"
if ($dry) {
    Write-Log "[dry-run] Remove-Item -Recurse -Force -Path $Prefix"
} else {
    if (Test-Path $Prefix) {
        $children = Get-ChildItem -Path $Prefix -Force -ErrorAction SilentlyContinue
        foreach ($child in $children) {
            if ($child.Name -ieq 'wsl') {
                if (-not $RemoveWSLDistro.IsPresent) {
                    Write-Warn "Leaving $($child.FullName) (substrate-wsl still registered)."
                    continue
                }

                Write-Log "Terminating WSL distro $DistroName (if running)"
                try { & wsl --terminate $DistroName | Out-Null } catch {}

                try {
                    Remove-Item -Recurse -Force -Path $child.FullName -ErrorAction Stop
                } catch {
                    Write-Warn ("Unable to remove {0}: {1}" -f $child.FullName, $_.Exception.Message)
                }
                continue
            }

            try {
                Remove-Item -Recurse -Force -Path $child.FullName -ErrorAction SilentlyContinue
            } catch {}
        }

        # Remove the prefix directory if it is now empty
        if (-not (Get-ChildItem -Path $Prefix -Force -ErrorAction SilentlyContinue)) {
            Remove-Item -Force -Path $Prefix -ErrorAction SilentlyContinue
        }
    }
}

Write-Log "Cleaning shim cache"
if ($dry) {
    Write-Log "[dry-run] Remove-Item -Recurse -Force -Path $env:USERPROFILE\.substrate*"
} else {
    Get-ChildItem -Path $env:USERPROFILE -Filter '.substrate*' -ErrorAction SilentlyContinue |
        ForEach-Object { Remove-Item -Recurse -Force -Path $_.FullName -ErrorAction SilentlyContinue }
}

Write-Log "Stopping substrate-world-agent inside WSL (if present)"
if ($dry) {
    Write-Log "[dry-run] wsl -d $DistroName -- bash -lc 'sudo systemctl disable --now substrate-world-agent.service'"
} else {
    try {
        & wsl -d $DistroName -- bash -lc "sudo systemctl disable --now substrate-world-agent.service" | Out-Null
    } catch {
        Write-Warn ("Unable to disable substrate-world-agent inside {0}: {1}" -f $DistroName, $_.Exception.Message)
    }
}

if ($RemoveWSLDistro.IsPresent) {
    Write-Log "Unregistering WSL distro $DistroName"
    if ($dry) {
        Write-Log "[dry-run] wsl --unregister $DistroName"
    } else {
        try {
            & wsl --unregister $DistroName
        } catch {
            Write-Warn ("Failed to unregister {0}: {1}" -f $DistroName, $_.Exception.Message)
        }
    }
}

Write-Log "Uninstall complete. Open a new PowerShell session to refresh PATH."
