#!/usr/bin/env pwsh
[CmdletBinding()]
param(
    [string]$Prefix = (Join-Path $env:LOCALAPPDATA 'Substrate'),
    [ValidateSet('debug','release')][string]$Profile,
    [string]$BinPath
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Write-Log {
    param([string]$Message)
    Write-Host "[substrate-dev-uninstall] $Message"
}
function Write-Warn {
    param([string]$Message)
    Write-Host "[substrate-dev-uninstall][WARN] $Message" -ForegroundColor Yellow
}

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$substrateBin = $BinPath
if (-not $substrateBin) {
    switch ($Profile) {
        'release' { $substrateBin = Join-Path $repoRoot 'target\release\substrate.exe' }
        'debug'   { $substrateBin = Join-Path $repoRoot 'target\debug\substrate.exe' }
        default {
            $releasePath = Join-Path $repoRoot 'target\release\substrate.exe'
            $debugPath   = Join-Path $repoRoot 'target\debug\substrate.exe'
            if (Test-Path $releasePath) {
                $substrateBin = $releasePath
            } elseif (Test-Path $debugPath) {
                $substrateBin = $debugPath
            }
        }
    }
}

if ($substrateBin -and -not (Test-Path $substrateBin)) {
    Write-Warn "Specified substrate binary '$substrateBin' does not exist; skipping shim-remove."
    $substrateBin = $null
}

if ($substrateBin) {
    Write-Log "Removing shims via $substrateBin"
    $originalPath = $env:PATH
    $previousShimOriginal = $env:SHIM_ORIGINAL_PATH
    try {
        $env:SUBSTRATE_ROOT = $Prefix
        if (-not $env:SHIM_ORIGINAL_PATH) {
            $env:SHIM_ORIGINAL_PATH = $originalPath
        }
        & $substrateBin --no-world --shim-remove
    } catch {
        Write-Warn "substrate --shim-remove returned an error: $_"
    } finally {
        if ($null -ne $previousShimOriginal) {
            $env:SHIM_ORIGINAL_PATH = $previousShimOriginal
        } else {
            Remove-Item Env:SHIM_ORIGINAL_PATH -ErrorAction SilentlyContinue
        }
        Remove-Item Env:SUBSTRATE_ROOT -ErrorAction SilentlyContinue
        $env:PATH = $originalPath
    }
} else {
    Write-Warn 'No substrate binary located; skipping shim-remove invocation.'
}

$shimDir = Join-Path $Prefix 'shims'
$profileScript = Join-Path $Prefix 'dev-substrate-profile.ps1'

if (Test-Path $shimDir) {
    Write-Log "Deleting $shimDir"
    Remove-Item -Recurse -Force -Path $shimDir
}

if (Test-Path $profileScript) {
    Write-Log "Removing $profileScript"
    Remove-Item -Force -Path $profileScript
}

Write-Host ''
Write-Host 'Dev shims removed. Open a new PowerShell session to clear cached commands.'
Write-Host 'Built artifacts under target\ are left untouched.'
Write-Host ''
