#!/usr/bin/env pwsh
<#!
.SYNOPSIS
    Install Substrate on Windows hosts via PowerShell.
.DESCRIPTION
    Downloads (or consumes a local archive of) the Substrate release bundle,
    installs binaries under the chosen prefix, configures PATH/profile updates,
    deploys shims, and optionally provisions the WSL world backend.
.PARAMETER Version
    Release version to install (defaults to 0.2.0-beta).
.PARAMETER Prefix
    Installation prefix (defaults to %LOCALAPPDATA%\Substrate).
.PARAMETER Archive
    Optional path to a pre-downloaded release archive (.zip) for offline installs.
.PARAMETER BaseUrl
    Base URL for hosted releases (defaults to GitHub releases).
.PARAMETER NoWorld
    Skip WSL world provisioning.
.PARAMETER NoShims
    Skip shim deployment.
.PARAMETER DryRun
    Print steps without executing them.
.PARAMETER DistroName
    Target WSL distribution name (defaults to substrate-wsl).
.EXAMPLE
    pwsh -File install-substrate.ps1
.EXAMPLE
    pwsh -File install-substrate.ps1 -Version 0.2.0-beta -Archive C:\Downloads\substrate.zip
#>

[CmdletBinding()]
param(
    [string]$Version = '0.2.0-beta',
    [string]$Prefix = (Join-Path $env:LOCALAPPDATA 'Substrate'),
    [string]$Archive,
    [string]$BaseUrl = 'https://github.com/atomize-hq/substrate/releases/download',
    [switch]$NoWorld,
    [switch]$NoShims,
    [switch]$DryRun,
    [string]$DistroName = 'substrate-wsl'
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Write-Log {
    param([string]$Message)
    Write-Host "[substrate-install] $Message"
}
function Write-Warn {
    param([string]$Message)
    Write-Host "[substrate-install][WARN] $Message" -ForegroundColor Yellow
}
function Write-ErrorAndExit {
    param([string]$Message)
    Write-Host "[substrate-install][ERROR] $Message" -ForegroundColor Red
    exit 1
}

$versionNormalized = $Version.TrimStart('v')
if ([string]::IsNullOrWhiteSpace($versionNormalized)) {
    Write-ErrorAndExit "Version parameter cannot be empty"
}
$versionTag = if ($Version.StartsWith('v')) { $Version } else { "v$versionNormalized" }
$artifactName = "substrate-v$versionNormalized-windows_x86_64.zip"
$checksumName = 'SHA256SUMS'
$dry = $DryRun.IsPresent

$tempRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("substrate-install-" + [System.Guid]::NewGuid())
if (-not $dry) {
    New-Item -ItemType Directory -Path $tempRoot | Out-Null
}
$archivePath = if ($Archive) { (Resolve-Path $Archive).Path } else { Join-Path $tempRoot $artifactName }
$checksumPath = Join-Path $tempRoot $checksumName
$extractRoot = Join-Path $tempRoot 'extract'

try {
    if (-not $Archive) {
        $downloadUrl = "$BaseUrl/$versionTag/$artifactName"
        Write-Log "Downloading $artifactName from $downloadUrl"
        if ($dry) {
            Write-Log "[dry-run] Invoke-WebRequest -Uri $downloadUrl -OutFile $archivePath"
        } else {
            Invoke-WebRequest -Uri $downloadUrl -OutFile $archivePath
        }

        $checksumUrl = "$BaseUrl/$versionTag/$checksumName"
        try {
            if ($dry) {
                Write-Log "[dry-run] Invoke-WebRequest -Uri $checksumUrl -OutFile $checksumPath"
            } else {
                Invoke-WebRequest -Uri $checksumUrl -OutFile $checksumPath
            }
        } catch {
            Write-Warn "Failed to download checksum file ($checksumUrl); skipping verification"
        }
    } else {
        Write-Log "Using local archive: $archivePath"
    }

    if (-not $dry -and (Test-Path $checksumPath)) {
        $expectedLine = Get-Content $checksumPath | Where-Object { $_ -match "  $artifactName$" }
        if (-not $expectedLine) {
            Write-Warn "Checksum entry for $artifactName not found; skipping verification"
        } else {
            $expectedHash = ($expectedLine -split ' ')[0].Trim()
            $actualHash = (Get-FileHash -Algorithm SHA256 -Path $archivePath).Hash.ToLowerInvariant()
            if ($expectedHash.ToLowerInvariant() -ne $actualHash) {
                Write-ErrorAndExit "Checksum mismatch for $artifactName (expected $expectedHash, got $actualHash)"
            }
            Write-Log "Checksum verified for $artifactName"
        }
    }

    Write-Log "Extracting archive"
    if ($dry) {
        Write-Log "[dry-run] Expand-Archive -Path $archivePath -DestinationPath $extractRoot -Force"
    } else {
        Expand-Archive -Path $archivePath -DestinationPath $extractRoot -Force
    }

    $releaseRoot = if ($dry) {
        Join-Path $extractRoot 'SIMULATED_ROOT'
    } else {
        $entries = @(Get-ChildItem -Path $extractRoot)
        if ($entries.Count -eq 1 -and $entries[0].PSIsContainer) {
            $entries[0].FullName
        } else {
            $extractRoot
        }
    }

    $versionsDir = Join-Path $Prefix 'versions'
    $versionDir = Join-Path $versionsDir $versionNormalized
    $binDir = Join-Path $Prefix 'bin'
    $shimDir = Join-Path $Prefix 'shims'

    Write-Log "Installing to $versionDir"
    if ($dry) {
        Write-Log "[dry-run] New-Item -ItemType Directory -Force -Path $versionsDir"
        Write-Log "[dry-run] Remove-Item -Recurse -Force -Path $versionDir"
        Write-Log "[dry-run] Copy-Item -Path $releaseRoot/* -Destination $versionDir -Recurse"
    } else {
        New-Item -ItemType Directory -Force -Path $versionsDir | Out-Null
        if (Test-Path $versionDir) {
            Remove-Item -Recurse -Force -Path $versionDir
        }
        New-Item -ItemType Directory -Force -Path $versionDir | Out-Null
        Copy-Item -Path (Join-Path $releaseRoot '*') -Destination $versionDir -Recurse -Force
    }

    if ($dry) {
        Write-Log "[dry-run] Ensure $binDir contains latest binaries"
    } else {
        New-Item -ItemType Directory -Force -Path $binDir | Out-Null
        Get-ChildItem -Path $binDir | Remove-Item -Recurse -Force -ErrorAction SilentlyContinue
        Copy-Item -Path (Join-Path $versionDir 'bin\*') -Destination $binDir -Recurse -Force
    }

    if ($dry) {
        Write-Log "[dry-run] New-Item -ItemType Directory -Force -Path $shimDir"
    } else {
        New-Item -ItemType Directory -Force -Path $shimDir | Out-Null
    }

    $profileScript = Join-Path $Prefix 'substrate-profile.ps1'
    $timestamp = Get-Date -Format 'yyyy-MM-dd HH:mm:ss'
    $substrateRootValue = $Prefix
$profileContent = @"
# Generated by substrate installer on $timestamp
# Do not edit manually; re-run install-substrate.ps1 if changes are needed.
$substrateRoot = '$substrateRootValue'
$shimDir = "$Prefix\shims"
$binDir = "$Prefix\bin"
$env:SUBSTRATE_ROOT = $substrateRoot
if (-not $env:SHIM_ORIGINAL_PATH) {
    $env:SHIM_ORIGINAL_PATH = $env:PATH
}
$desired = @($shimDir, $binDir)
$pathParts = ($env:PATH -split ';') | Where-Object { $_ -and ($desired -notcontains $_) }
$env:PATH = ($desired + $pathParts) -join ';'
"@

    if ($dry) {
        Write-Log "[dry-run] Write substrate profile script to $profileScript"
    } else {
        New-Item -ItemType Directory -Force -Path $Prefix | Out-Null
        Set-Content -Path $profileScript -Value $profileContent -Encoding UTF8
    }

    $profileSnippet = @"
# Added by Substrate installer (Windows)
if (Test-Path '$profileScript') {
    . '$profileScript'
}
"@
    $profileTargets = @($PROFILE.CurrentUserAllHosts, $PROFILE.CurrentUserCurrentHost) | Where-Object { $_ }
    foreach ($target in $profileTargets | Select-Object -Unique) {
        $targetDir = Split-Path -Path $target -Parent
        if ($dry) {
            Write-Log "[dry-run] Ensure profile directory $targetDir exists"
        } else {
            if ($targetDir) { New-Item -ItemType Directory -Force -Path $targetDir | Out-Null }
        }

        if ($dry) {
            Write-Log "[dry-run] Append substrate snippet to $target"
            continue
        }

        if (-not (Test-Path $target)) {
            Set-Content -Path $target -Value $profileSnippet -Encoding UTF8
        } else {
            $existing = Get-Content -Path $target -ErrorAction SilentlyContinue
            if ($existing -notmatch 'Substrate installer \(Windows\)') {
                Add-Content -Path $target -Value $profileSnippet
            }
        }
    }

    if (-not $dry -and (Test-Path $profileScript)) {
        . $profileScript
    }

    $substrateExe = Join-Path $binDir 'substrate.exe'
    if (-not $NoShims.IsPresent) {
        Write-Log "Deploying shims"
        if ($dry) {
            Write-Log "[dry-run] & '$substrateExe' --shim-deploy"
        } else {
            try {
                & $substrateExe --shim-deploy | Out-Null
            } catch {
                Write-Warn "Shim deployment encountered an error: $($_.Exception.Message)"
            }
        }
    } else {
        Write-Log "Skipping shim deployment (--NoShims)"
    }

    if (-not $NoWorld.IsPresent) {
        $warmScript = Join-Path $versionDir 'scripts\windows\wsl-warm.ps1'
        if (-not (Test-Path $warmScript)) {
            Write-Warn "wsl-warm.ps1 not found at $warmScript; skipping world provisioning"
        } else {
            Write-Log "Provisioning WSL world backend (distro: $DistroName)"
            if ($dry) {
                Write-Log "[dry-run] & '$warmScript' -DistroName $DistroName -ProjectPath '$versionDir'"
            } else {
                try {
                    $warmParams = @{ DistroName = $DistroName; ProjectPath = $versionDir }
                    & $warmScript @warmParams
                } catch {
                    Write-Warn "World provisioning reported an error: $($_.Exception.Message)"
                }
            }
        }
    } else {
        Write-Log "Skipping world provisioning (--NoWorld)"
    }

    if (-not $dry) {
        try {
            Write-Log "Running substrate world doctor"
            & $substrateExe world doctor --json | Out-Null
        } catch {
            Write-Warn "world doctor reported issues: $($_.Exception.Message)"
        }
    }

    Write-Log "Installation complete. Open a new PowerShell session or run '. $profileScript' to refresh PATH."
}
finally {
    if (-not $dry -and (Test-Path $tempRoot)) {
        Remove-Item -Recurse -Force -Path $tempRoot
    }
}
