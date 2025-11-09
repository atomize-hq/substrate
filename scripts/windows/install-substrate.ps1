#!/usr/bin/env pwsh
<#!
.SYNOPSIS
    Install Substrate on Windows hosts via PowerShell.
.DESCRIPTION
    Downloads (or consumes a local archive of) the Substrate release bundle,
    installs binaries under the chosen prefix, configures PATH/profile updates,
    deploys shims, and optionally provisions the WSL world backend.
.PARAMETER Version
    Release version to install (defaults to the latest GitHub release).
.PARAMETER Prefix
    Installation prefix (defaults to %LOCALAPPDATA%\Substrate).
.PARAMETER ArtifactDir
    Optional directory containing pre-downloaded host bundle(s) and SHA256SUMS.
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
.PARAMETER NoAutoSource
    Do not dot-source the generated profile for the current PowerShell session.
.EXAMPLE
    pwsh -File install-substrate.ps1
.EXAMPLE
    pwsh -File install-substrate.ps1 -Version 0.2.0-beta -ArtifactDir C:\Downloads\substrate-artifacts
#>

[CmdletBinding()]
param(
    [string]$Version,
    [string]$Prefix = (Join-Path $env:LOCALAPPDATA 'Substrate'),
    [Alias('Archive')] [string]$ArtifactDir,
    [string]$BaseUrl = 'https://github.com/atomize-hq/substrate/releases/download',
    [switch]$NoWorld,
    [switch]$NoShims,
    [switch]$DryRun,
    [string]$DistroName = 'substrate-wsl',
    [switch]$NoAutoSource
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$FallbackVersion = '0.2.2'
$LatestReleaseApi = if ($env:SUBSTRATE_INSTALL_LATEST_API) { $env:SUBSTRATE_INSTALL_LATEST_API } else { 'https://api.github.com/repos/atomize-hq/substrate/releases/latest' }
$GitHubToken = $env:SUBSTRATE_INSTALL_GITHUB_TOKEN

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

function Get-LatestReleaseTag {
    param(
        [string]$ApiUrl,
        [string]$Token
    )

    $headers = @{ 'Accept' = 'application/vnd.github+json' }
    if ($Token) {
        $headers['Authorization'] = "Bearer $Token"
    }

    try {
        $response = Invoke-RestMethod -Uri $ApiUrl -Headers $headers -UseBasicParsing
        return $response.tag_name
    } catch {
        return $null
    }
}

function Resolve-VersionTag {
    param(
        [string]$RequestedVersion,
        [string]$FallbackVersion,
        [string]$ApiUrl,
        [string]$Token
    )

    if (-not [string]::IsNullOrWhiteSpace($RequestedVersion)) {
        Write-Log "Using requested version $RequestedVersion"
        return $RequestedVersion
    }

    $latest = Get-LatestReleaseTag -ApiUrl $ApiUrl -Token $Token
    if ($latest) {
        Write-Log "No version specified; defaulting to latest release $latest"
        return $latest
    }

    $fallbackTag = "v$FallbackVersion"
    Write-Warn "Unable to resolve latest release tag; falling back to $fallbackTag"
    return $fallbackTag
}

$resolvedVersion = Resolve-VersionTag -RequestedVersion $Version -FallbackVersion $FallbackVersion -ApiUrl $LatestReleaseApi -Token $GitHubToken
$versionNormalized = $resolvedVersion.TrimStart('v')
if ([string]::IsNullOrWhiteSpace($versionNormalized)) {
    Write-ErrorAndExit "Version parameter cannot be empty"
}
$versionTag = if ($resolvedVersion.StartsWith('v')) { $resolvedVersion } else { "v$versionNormalized" }
$bundleName = "substrate-v$versionNormalized-windows_x86_64.zip"
$checksumName = 'SHA256SUMS'
$dry = $DryRun.IsPresent

$tempRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("substrate-install-" + [System.Guid]::NewGuid())
if (-not $dry) {
    New-Item -ItemType Directory -Path $tempRoot | Out-Null
}
$payloadRoot = Join-Path $tempRoot 'payload'
if (-not $dry) {
    New-Item -ItemType Directory -Force -Path $payloadRoot | Out-Null
}

$artifactDirectory = if ($PSBoundParameters.ContainsKey('ArtifactDir')) {
    (Resolve-Path $ArtifactDir).Path
} else {
    $null
}

$checksumPath = Join-Path $tempRoot $checksumName
if ($artifactDirectory) {
    Write-Log "Using local artifact directory: $artifactDirectory"
    if (-not $dry) {
        $localChecksum = Join-Path $artifactDirectory $checksumName
        if (Test-Path $localChecksum) {
            Copy-Item -Path $localChecksum -Destination $checksumPath -Force
        }
    }
} else {
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
}

$checksumsAvailable = (-not $dry) -and (Test-Path $checksumPath)

function Resolve-LocalArtifact {
    param([string]$ArtifactName)
    if (-not $artifactDirectory) { return $null }
    $candidate = Join-Path $artifactDirectory $ArtifactName
    if (-not (Test-Path $candidate)) {
        Write-ErrorAndExit "Artifact '$ArtifactName' not found in $artifactDirectory"
    }
    return $candidate
}

function Verify-Checksum {
    param(
        [string]$ChecksumsFile,
        [string]$ArtifactName,
        [string]$LocalPath
    )
    if (-not (Test-Path $ChecksumsFile)) { return }
    $expectedLine = Get-Content $ChecksumsFile | Where-Object { $_ -match "  $ArtifactName$" }
    if (-not $expectedLine) {
        Write-Warn "Checksum entry for $ArtifactName not found; skipping verification"
        return
    }
    $expectedHash = ($expectedLine -split ' ')[0].Trim().ToLowerInvariant()
    $actualHash = (Get-FileHash -Algorithm SHA256 -Path $LocalPath).Hash.ToLowerInvariant()
    if ($expectedHash -ne $actualHash) {
        Write-ErrorAndExit "Checksum mismatch for $ArtifactName (expected $expectedHash, got $actualHash)"
    }
    Write-Log "Checksum verified for $ArtifactName"
}

try {
    $bundleDest = Join-Path $tempRoot $bundleName
    $localBundle = Resolve-LocalArtifact $bundleName

    if ($localBundle) {
        if ($dry) {
            Write-Log "[dry-run] Copy-Item -Path $localBundle -Destination $bundleDest"
        } else {
            Copy-Item -Path $localBundle -Destination $bundleDest -Force
        }
    } else {
        $bundleUrl = "$BaseUrl/$versionTag/$bundleName"
        Write-Log "Downloading $bundleName from $bundleUrl"
        if ($dry) {
            Write-Log "[dry-run] Invoke-WebRequest -Uri $bundleUrl -OutFile $bundleDest"
        } else {
            Invoke-WebRequest -Uri $bundleUrl -OutFile $bundleDest
        }
    }

    if ($checksumsAvailable -and -not $dry) {
        Verify-Checksum $checksumPath $bundleName $bundleDest
    }

    if ($dry) {
        Write-Log "[dry-run] Expand-Archive -Path $bundleDest -DestinationPath $payloadRoot -Force"
    } else {
        Expand-Archive -Path $bundleDest -DestinationPath $payloadRoot -Force
    }

    if ($dry) {
        $releaseRoot = Join-Path $payloadRoot 'SIMULATED_ROOT'
    } else {
        $entries = @(Get-ChildItem -Path $payloadRoot)
        if ($entries.Count -eq 1 -and $entries[0].PSIsContainer) {
            $releaseRoot = $entries[0].FullName
        } else {
            $releaseRoot = $payloadRoot
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
`$substrateRoot = '$substrateRootValue'
`$shimDir = "$Prefix\shims"
`$binDir = "$Prefix\bin"
`$env:SUBSTRATE_ROOT = `$substrateRoot
if (-not `$env:SHIM_ORIGINAL_PATH) {
    `$env:SHIM_ORIGINAL_PATH = `$env:PATH
}
`$desired = @(`$shimDir, `$binDir)
`$pathParts = @()
foreach (`$part in (`$env:PATH -split ';')) {
    if (`$part -and (`$desired -notcontains `$part)) {
        `$pathParts += `$part
    }
}
`$env:PATH = (`$desired + `$pathParts) -join ';'
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

    if ($dry) {
        Write-Log "[dry-run] Would dot-source $profileScript for this session"
    } elseif ($NoAutoSource.IsPresent) {
        Write-Log "Skipping dot-sourcing of $profileScript (--NoAutoSource). Run '. $profileScript' manually to refresh PATH."
    } elseif (Test-Path $profileScript) {
        . $profileScript
        Write-Log "Sourced $profileScript; environment ready in this session."
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

    if ($dry) {
        Write-Log "Installation complete (dry run). Open a new PowerShell session or run '. $profileScript' when performing a real install."
    } elseif ($NoAutoSource.IsPresent) {
        Write-Log "Installation complete. Run '. $profileScript' or open a new PowerShell session to use Substrate."
    } else {
        Write-Log "Installation complete. Substrate is ready to use in this PowerShell session and future sessions."
    }
}
finally {
    if (-not $dry -and (Test-Path $tempRoot)) {
        Remove-Item -Recurse -Force -Path $tempRoot
    }
}
