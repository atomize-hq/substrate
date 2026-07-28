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
    [string]$Prefix,
    [Alias('Archive')] [string]$ArtifactDir,
    [string]$BaseUrl = 'https://github.com/atomize-hq/substrate/releases/download',
    [switch]$NoWorld,
    [switch]$NoShims,
    [switch]$DryRun,
    [string]$DistroName = 'substrate-wsl',
    [switch]$NoAutoSource,
    [string]$InstallBootstrapContextV1
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

function Resolve-InstallBootstrapContextV1 {
    param(
        [string]$DeclaredPrefix,
        [string]$EncodedCarrier
    )

    $isWindowsHost = [System.Runtime.InteropServices.RuntimeInformation]::IsOSPlatform(
        [System.Runtime.InteropServices.OSPlatform]::Windows
    )
    $validPrincipalText = {
        param([string]$Value)
        return (-not [string]::IsNullOrEmpty($Value)) -and
            ($Value.IndexOf("`0") -lt 0) -and
            (-not $Value.Contains("`n")) -and
            (-not $Value.Contains("`r"))
    }
    $validWindowsSid = {
        param([string]$Sid)
        if ([string]::IsNullOrEmpty($Sid)) {
            return $false
        }

        $parts = $Sid.Split('-')
        if ($parts.Count -lt 3 -or $parts[0] -cne 'S') {
            return $false
        }

        foreach ($part in $parts[1..($parts.Count - 1)]) {
            if ($part.Length -eq 0 -or $part -notmatch '^[0-9]+$') {
                return $false
            }
            if ($part.Length -gt 1 -and $part.StartsWith('0')) {
                return $false
            }
        }

        return $true
    }
    $hasInvalidUnicode = {
        param([string]$Value)
        $index = 0
        while ($index -lt $Value.Length) {
            $codeUnit = [int][char]$Value[$index]
            if ($codeUnit -ge 0xD800 -and $codeUnit -le 0xDBFF) {
                if ($index + 1 -ge $Value.Length) {
                    return $true
                }
                $next = [int][char]$Value[$index + 1]
                if ($next -lt 0xDC00 -or $next -gt 0xDFFF) {
                    return $true
                }
                $index += 2
                continue
            }
            if ($codeUnit -ge 0xDC00 -and $codeUnit -le 0xDFFF) {
                return $true
            }
            $index += 1
        }
        return $false
    }
    $validWindowsComponent = {
        param([string]$Component)
        if ($Component -eq '.' -or $Component -eq '..' -or $Component.EndsWith('.') -or $Component.EndsWith(' ')) {
            return $false
        }
        if (& $hasInvalidUnicode $Component) {
            return $false
        }

        foreach ($char in $Component.ToCharArray()) {
            $codePoint = [int][char]$char
            if ($codePoint -eq 0 -or ($codePoint -ge 1 -and $codePoint -le 31) -or $char -in @('<', '>', '"', '|', '?', '*', ':')) {
                return $false
            }
        }

        $stem = $Component.Split('.', 2)[0].ToUpperInvariant()
        if ($stem -in @('CON', 'PRN', 'AUX', 'NUL')) {
            return $false
        }

        foreach ($prefix in @('COM', 'LPT')) {
            if ($stem.StartsWith($prefix)) {
                $suffix = $stem.Substring(3)
                if ($suffix -in @('1', '2', '3', '4', '5', '6', '7', '8', '9', '¹', '²', '³')) {
                    return $false
                }
            }
        }

        return $true
    }
    $windowsComponents = {
        param([string]$RawComponents)
        $components = @($RawComponents -split '\\+' | Where-Object { $_ -ne '' })
        if ($components.Count -eq 0) {
            throw 'invalid install bootstrap path'
        }
        foreach ($component in $components) {
            if (-not (& $validWindowsComponent $component)) {
                throw 'invalid install bootstrap path'
            }
        }
        return $components
    }
    $normalizeWindowsPath = {
        param([string]$RawPath)
        if ([string]::IsNullOrEmpty($RawPath) -or $RawPath.IndexOf("`0") -ge 0) {
            throw 'invalid install bootstrap path'
        }

        $normalizedSeparators = $RawPath.Replace('/', '\')
        $lowercase = $normalizedSeparators.ToLowerInvariant()
        if ($lowercase.StartsWith('\\?\') -or $lowercase.StartsWith('\\.\')) {
            throw 'invalid install bootstrap path'
        }

        if ($normalizedSeparators.StartsWith('\\')) {
            $components = & $windowsComponents $normalizedSeparators.Substring(2)
            if ($components.Count -lt 3) {
                throw 'invalid install bootstrap path'
            }
            return ('\\' + ($components -join '\'))
        }

        if (
            $normalizedSeparators.Length -lt 4 -or
            (-not [char]::IsLetter($normalizedSeparators[0])) -or
            $normalizedSeparators[1] -cne ':' -or
            $normalizedSeparators[2] -cne '\'
        ) {
            throw 'invalid install bootstrap path'
        }

        $components = & $windowsComponents $normalizedSeparators.Substring(3)
        $drive = [char]::ToUpperInvariant($normalizedSeparators[0])
        return ('{0}:\{1}' -f $drive, ($components -join '\'))
    }
    $encodeBase64Url = {
        param([string]$Value)
        return [Convert]::ToBase64String([System.Text.Encoding]::UTF8.GetBytes($Value)).TrimEnd('=').Replace('+', '-').Replace('/', '_')
    }
    $decodeBase64Url = {
        param([string]$Value)
        if ([string]::IsNullOrEmpty($Value) -or $Value -notmatch '^[A-Za-z0-9_-]+$') {
            throw 'invalid install bootstrap carrier'
        }

        $padding = (4 - ($Value.Length % 4)) % 4
        $base64 = $Value.Replace('-', '+').Replace('_', '/') + ('=' * $padding)
        try {
            $bytes = [Convert]::FromBase64String($base64)
        } catch {
            throw 'invalid install bootstrap carrier'
        }

        $canonical = [Convert]::ToBase64String($bytes).TrimEnd('=').Replace('+', '-').Replace('/', '_')
        if ($canonical -cne $Value) {
            throw 'invalid install bootstrap carrier'
        }

        return [System.Text.Encoding]::UTF8.GetString($bytes)
    }
    $sha256Hex = {
        param([string]$Value)
        $sha256 = [System.Security.Cryptography.SHA256]::Create()
        try {
            return ([System.BitConverter]::ToString(
                    $sha256.ComputeHash([System.Text.Encoding]::ASCII.GetBytes($Value))
                ).Replace('-', '').ToLowerInvariant())
        } finally {
            $sha256.Dispose()
        }
    }
    $newContext = {
        param(
            [string]$SelectedPrefix,
            [string]$Account,
            [string]$Sid
        )
        $selected = & $normalizeWindowsPath $SelectedPrefix
        if (-not (& $validPrincipalText $Account) -or -not (& $validWindowsSid $Sid)) {
            throw 'invalid install bootstrap principal'
        }

        $selectedEncoded = & $encodeBase64Url $selected
        $accountEncoded = & $encodeBase64Url $Account
        $sidEncoded = & $encodeBase64Url $Sid
        $commitmentInput = @(
            'domain=substrate.install_bootstrap_context'
            'version=1'
            "selected_host_prefix=$selectedEncoded"
            "host_substrate_home=$selectedEncoded"
            "host_substrate_root=$selectedEncoded"
            'principal_kind=windows'
            "principal_account=$accountEncoded"
            "principal_sid=$sidEncoded"
        ) -join "`n"
        $commitmentInput = "$commitmentInput`n"
        $commitment = & $sha256Hex $commitmentInput
        $record = "${commitmentInput}host_context_commitment=$commitment`n"
        $carrier = & $encodeBase64Url $record

        return [pscustomobject]@{
            SelectedHostPrefix    = $selected
            HostSubstrateHome     = $selected
            HostSubstrateRoot     = $selected
            CanonicalAccount      = $Account
            CanonicalSid          = $Sid
            HostContextCommitment = $commitment
            EncodedCarrier        = $carrier
        }
    }
    $observeCurrentWindowsContext = {
        if (-not $isWindowsHost) {
            throw 'Windows install bootstrap context requires a Windows host.'
        }
        if (-not ('SubstrateKnownFolderNative' -as [type])) {
            Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;

public static class SubstrateKnownFolderNative
{
    [DllImport("shell32.dll")]
    public static extern int SHGetKnownFolderPath(ref Guid rfid, uint dwFlags, IntPtr hToken, out IntPtr ppszPath);
}
'@
        }

        $identity = [System.Security.Principal.WindowsIdentity]::GetCurrent()
        if ($null -eq $identity) {
            throw 'Unable to resolve the current Windows principal.'
        }

        try {
            $account = $identity.Name
            $sidObject = $identity.User
            $sid = if ($null -ne $sidObject) { $sidObject.Value } else { $null }
            if (-not (& $validPrincipalText $account) -or -not (& $validWindowsSid $sid)) {
                throw 'Unable to resolve the current Windows principal.'
            }

            $folderGuid = [guid]'F1B32785-6FBA-4FCF-9D55-7B8E7F157091'
            $rawPath = [IntPtr]::Zero
            try {
                $status = [SubstrateKnownFolderNative]::SHGetKnownFolderPath([ref]$folderGuid, 0, $identity.Token, [ref]$rawPath)
                if ($status -ne 0 -or $rawPath -eq [IntPtr]::Zero) {
                    throw 'Unable to resolve the current Windows LocalApplicationData Known Folder.'
                }
                $knownFolder = [System.Runtime.InteropServices.Marshal]::PtrToStringUni($rawPath)
            } finally {
                if ($rawPath -ne [IntPtr]::Zero) {
                    [System.Runtime.InteropServices.Marshal]::FreeCoTaskMem($rawPath)
                }
            }

            if ([string]::IsNullOrWhiteSpace($knownFolder)) {
                throw 'Unable to resolve the current Windows LocalApplicationData Known Folder.'
            }

            return [pscustomobject]@{
                Account     = $account
                Sid         = $sid
                KnownFolder = (& $normalizeWindowsPath $knownFolder)
            }
        } finally {
            if ($identity -is [System.IDisposable]) {
                $identity.Dispose()
            }
        }
    }

    if ([string]::IsNullOrEmpty($EncodedCarrier)) {
        $observation = & $observeCurrentWindowsContext
        $selectedPrefix = if ([string]::IsNullOrEmpty($DeclaredPrefix)) {
            & $normalizeWindowsPath ("{0}\Substrate" -f $observation.KnownFolder)
        } else {
            & $normalizeWindowsPath $DeclaredPrefix
        }

        return (& $newContext $selectedPrefix $observation.Account $observation.Sid)
    }

    $record = & $decodeBase64Url $EncodedCarrier
    if ($record.Contains("`r") -or $record.IndexOf("`0") -ge 0) {
        throw 'invalid install bootstrap carrier'
    }

    $lines = $record -split "`n", 0, 'SimpleMatch'
    if ($lines.Length -ne 10 -or $lines[-1] -ne '') {
        throw 'invalid install bootstrap carrier'
    }

    $expectedKeys = @(
        'domain',
        'version',
        'selected_host_prefix',
        'host_substrate_home',
        'host_substrate_root',
        'principal_kind',
        'principal_account',
        'principal_sid',
        'host_context_commitment'
    )
    $parsed = [ordered]@{}
    for ($index = 0; $index -lt $expectedKeys.Count; $index++) {
        $line = $lines[$index]
        $delimiter = $line.IndexOf('=')
        if ($delimiter -lt 1) {
            throw 'invalid install bootstrap carrier'
        }
        $key = $line.Substring(0, $delimiter)
        $value = $line.Substring($delimiter + 1)
        if ($key -cne $expectedKeys[$index] -or $parsed.Contains($key)) {
            throw 'invalid install bootstrap carrier'
        }
        $parsed[$key] = $value
    }

    if (
        $parsed['domain'] -cne 'substrate.install_bootstrap_context' -or
        $parsed['version'] -cne '1' -or
        $parsed['principal_kind'] -cne 'windows' -or
        $parsed['host_context_commitment'] -notmatch '^[0-9a-f]{64}$'
    ) {
        throw 'invalid install bootstrap carrier'
    }

    $selectedPrefix = & $decodeBase64Url $parsed['selected_host_prefix']
    $home = & $decodeBase64Url $parsed['host_substrate_home']
    $root = & $decodeBase64Url $parsed['host_substrate_root']
    $account = & $decodeBase64Url $parsed['principal_account']
    $sid = & $decodeBase64Url $parsed['principal_sid']
    $context = & $newContext $selectedPrefix $account $sid

    if (
        $context.HostSubstrateHome -cne $home -or
        $context.HostSubstrateRoot -cne $root -or
        $context.HostContextCommitment -cne $parsed['host_context_commitment'] -or
        $context.EncodedCarrier -cne $EncodedCarrier
    ) {
        throw 'invalid install bootstrap carrier'
    }

    Assert-InstallBootstrapContextV1 -InstallContext $context -DeclaredPrefix $DeclaredPrefix -RequireInheritedProjectionMatch
    return $context
}

function Assert-InstallBootstrapContextV1 {
    param(
        [Parameter(Mandatory = $true)]
        [pscustomobject]$InstallContext,
        [string]$DeclaredPrefix,
        [switch]$RequireInheritedProjectionMatch
    )

    $isWindowsHost = [System.Runtime.InteropServices.RuntimeInformation]::IsOSPlatform(
        [System.Runtime.InteropServices.OSPlatform]::Windows
    )
    if (-not $isWindowsHost) {
        throw 'Windows install bootstrap validation requires a Windows host.'
    }

    $normalizeWindowsPath = {
        param([string]$RawPath)
        if ([string]::IsNullOrEmpty($RawPath) -or $RawPath.IndexOf("`0") -ge 0) {
            throw 'invalid install bootstrap path'
        }

        $normalizedSeparators = $RawPath.Replace('/', '\')
        $lowercase = $normalizedSeparators.ToLowerInvariant()
        if ($lowercase.StartsWith('\\?\') -or $lowercase.StartsWith('\\.\')) {
            throw 'invalid install bootstrap path'
        }

        $validWindowsComponent = {
            param([string]$Component)
            if ($Component -eq '.' -or $Component -eq '..' -or $Component.EndsWith('.') -or $Component.EndsWith(' ')) {
                return $false
            }
            $index = 0
            while ($index -lt $Component.Length) {
                $codeUnit = [int][char]$Component[$index]
                if ($codeUnit -ge 0xD800 -and $codeUnit -le 0xDBFF) {
                    if ($index + 1 -ge $Component.Length) {
                        return $false
                    }
                    $next = [int][char]$Component[$index + 1]
                    if ($next -lt 0xDC00 -or $next -gt 0xDFFF) {
                        return $false
                    }
                    $index += 2
                    continue
                }
                if ($codeUnit -ge 0xDC00 -and $codeUnit -le 0xDFFF) {
                    return $false
                }
                $index += 1
            }
            foreach ($char in $Component.ToCharArray()) {
                $codePoint = [int][char]$char
                if ($codePoint -eq 0 -or ($codePoint -ge 1 -and $codePoint -le 31) -or $char -in @('<', '>', '"', '|', '?', '*', ':')) {
                    return $false
                }
            }
            $stem = $Component.Split('.', 2)[0].ToUpperInvariant()
            if ($stem -in @('CON', 'PRN', 'AUX', 'NUL')) {
                return $false
            }
            foreach ($prefix in @('COM', 'LPT')) {
                if ($stem.StartsWith($prefix)) {
                    $suffix = $stem.Substring(3)
                    if ($suffix -in @('1', '2', '3', '4', '5', '6', '7', '8', '9', '¹', '²', '³')) {
                        return $false
                    }
                }
            }
            return $true
        }
        $windowsComponents = @($normalizedSeparators.Substring(
                $(if ($normalizedSeparators.StartsWith('\\')) { 2 } else { 3 })
            ) -split '\\+' | Where-Object { $_ -ne '' })
        if ($windowsComponents.Count -eq 0) {
            throw 'invalid install bootstrap path'
        }
        foreach ($component in $windowsComponents) {
            if (-not (& $validWindowsComponent $component)) {
                throw 'invalid install bootstrap path'
            }
        }
        if ($normalizedSeparators.StartsWith('\\')) {
            if ($windowsComponents.Count -lt 3) {
                throw 'invalid install bootstrap path'
            }
            return ('\\' + ($windowsComponents -join '\'))
        }
        if (
            $normalizedSeparators.Length -lt 4 -or
            (-not [char]::IsLetter($normalizedSeparators[0])) -or
            $normalizedSeparators[1] -cne ':' -or
            $normalizedSeparators[2] -cne '\'
        ) {
            throw 'invalid install bootstrap path'
        }
        $drive = [char]::ToUpperInvariant($normalizedSeparators[0])
        return ('{0}:\{1}' -f $drive, ($windowsComponents -join '\'))
    }

    $identity = [System.Security.Principal.WindowsIdentity]::GetCurrent()
    if ($null -eq $identity) {
        throw 'Unable to resolve the current Windows principal.'
    }

    try {
        $currentAccount = $identity.Name
        $currentSid = if ($null -ne $identity.User) { $identity.User.Value } else { $null }
        if (
            [string]::IsNullOrWhiteSpace($currentAccount) -or
            [string]::IsNullOrWhiteSpace($currentSid) -or
            $currentSid -notmatch '^S-[0-9]+(?:-[0-9]+)+$'
        ) {
            throw 'Unable to resolve the current Windows principal.'
        }

        if (
            $InstallContext.CanonicalAccount -cne $currentAccount -or
            $InstallContext.CanonicalSid -cne $currentSid
        ) {
            throw 'install bootstrap principal does not match the current Windows principal'
        }
    } finally {
        if ($identity -is [System.IDisposable]) {
            $identity.Dispose()
        }
    }

    if (-not [string]::IsNullOrEmpty($DeclaredPrefix)) {
        $normalizedDeclaredPrefix = & $normalizeWindowsPath $DeclaredPrefix
        if ($normalizedDeclaredPrefix -cne $InstallContext.SelectedHostPrefix) {
            throw 'declared install prefix does not match the install bootstrap carrier'
        }
    }

    if ($RequireInheritedProjectionMatch.IsPresent) {
        $expected = @{
            SUBSTRATE_HOME                            = $InstallContext.HostSubstrateHome
            SUBSTRATE_ROOT                            = $InstallContext.HostSubstrateRoot
            SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT = $InstallContext.HostContextCommitment
            SUBSTRATE_INSTALL_PRIMARY_USER            = $InstallContext.CanonicalAccount
            SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1    = $InstallContext.EncodedCarrier
        }
        foreach ($entry in $expected.GetEnumerator()) {
            $current = [Environment]::GetEnvironmentVariable($entry.Key)
            if ($null -ne $current -and $current -cne $entry.Value) {
                throw 'install bootstrap environment projection is conflicting'
            }
        }
    }
}

function Invoke-SubstrateWithInstallContextV1 {
    param(
        [Parameter(Mandatory = $true)]
        [string]$SubstrateExe,
        [Parameter(Mandatory = $true)]
        [pscustomobject]$InstallContext,
        [string[]]$Arguments,
        [hashtable]$AdditionalEnvironment
    )

    $additionalEnvironment = if ($null -ne $AdditionalEnvironment) { $AdditionalEnvironment } else { @{} }
    foreach ($reservedKey in @(
            'SUBSTRATE_HOME',
            'SUBSTRATE_ROOT',
            'SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT',
            'SUBSTRATE_INSTALL_PRIMARY_USER',
            'SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1'
        )) {
        if ($additionalEnvironment.ContainsKey($reservedKey)) {
            throw "AdditionalEnvironment cannot override $reservedKey"
        }
    }

    $requiredEnvironment = @{
        SUBSTRATE_HOME                            = $InstallContext.HostSubstrateHome
        SUBSTRATE_ROOT                            = $InstallContext.HostSubstrateRoot
        SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT = $InstallContext.HostContextCommitment
        SUBSTRATE_INSTALL_PRIMARY_USER            = $InstallContext.CanonicalAccount
        SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1    = $InstallContext.EncodedCarrier
    }
    $trackedKeys = @($requiredEnvironment.Keys + $additionalEnvironment.Keys | Select-Object -Unique)
    $previousEnvironment = @{}
    foreach ($key in $trackedKeys) {
        $currentItem = Get-Item -Path "Env:$key" -ErrorAction SilentlyContinue
        $previousEnvironment[$key] = [pscustomobject]@{
            Present = ($null -ne $currentItem)
            Value   = if ($null -ne $currentItem) { $currentItem.Value } else { $null }
        }
    }

    try {
        foreach ($entry in $requiredEnvironment.GetEnumerator()) {
            Set-Item -Path "Env:$($entry.Key)" -Value $entry.Value
        }
        foreach ($entry in $additionalEnvironment.GetEnumerator()) {
            Set-Item -Path "Env:$($entry.Key)" -Value ([string]$entry.Value)
        }

        $childArguments = @('--install-bootstrap-context-v1', $InstallContext.EncodedCarrier) + @($Arguments)
        & $SubstrateExe @childArguments
    } finally {
        foreach ($key in $trackedKeys) {
            $prior = $previousEnvironment[$key]
            if ($prior.Present) {
                Set-Item -Path "Env:$key" -Value $prior.Value
            } else {
                Remove-Item -Path "Env:$key" -ErrorAction SilentlyContinue
            }
        }
    }
}

$InstallContext = Resolve-InstallBootstrapContextV1 -DeclaredPrefix $Prefix -EncodedCarrier $InstallBootstrapContextV1
$Prefix = $InstallContext.SelectedHostPrefix

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
    $prefixLiteral = $InstallContext.SelectedHostPrefix.Replace("'", "''")
    $shimDirLiteral = $shimDir.Replace("'", "''")
    $binDirLiteral = $binDir.Replace("'", "''")
    $commitmentLiteral = $InstallContext.HostContextCommitment.Replace("'", "''")
    $accountLiteral = $InstallContext.CanonicalAccount.Replace("'", "''")
    $carrierLiteral = $InstallContext.EncodedCarrier.Replace("'", "''")
$profileContent = @"
# Generated by substrate installer on $timestamp
# Do not edit manually; re-run install-substrate.ps1 if changes are needed.
`$substrateRoot = '$prefixLiteral'
`$shimDir = '$shimDirLiteral'
`$binDir = '$binDirLiteral'
`$env:SUBSTRATE_ROOT = '$prefixLiteral'
`$env:SUBSTRATE_HOME = '$prefixLiteral'
`$env:SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT = '$commitmentLiteral'
`$env:SUBSTRATE_INSTALL_PRIMARY_USER = '$accountLiteral'
`$env:SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1 = '$carrierLiteral'
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
            Write-Log "[dry-run] & '$substrateExe' --install-bootstrap-context-v1 <redacted> --shim-deploy"
        } else {
            try {
                Invoke-SubstrateWithInstallContextV1 `
                    -SubstrateExe $substrateExe `
                    -InstallContext $InstallContext `
                    -Arguments @('--shim-deploy') | Out-Null
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
                Write-Log "[dry-run] & '$warmScript' -DistroName '$DistroName' -ProjectPath '$versionDir' -InstallPrefix '$($InstallContext.SelectedHostPrefix)' -InstallBootstrapContextV1 <redacted>"
            } else {
                try {
                    & $warmScript `
                        -DistroName $DistroName `
                        -ProjectPath $versionDir `
                        -InstallPrefix $InstallContext.SelectedHostPrefix `
                        -InstallBootstrapContextV1 $InstallContext.EncodedCarrier
                } catch {
                    $sanitizedProvisioningError = $_.Exception.Message
                    if (-not [string]::IsNullOrEmpty($sanitizedProvisioningError)) {
                        $sanitizedProvisioningError = $sanitizedProvisioningError.Replace($InstallContext.EncodedCarrier, '<redacted>')
                    }
                    throw "World provisioning reported an error: $sanitizedProvisioningError"
                }
            }
        }
    } else {
        Write-Log "Skipping world provisioning (--NoWorld)"
    }

    if (-not $dry) {
        try {
            Write-Log "Running substrate world doctor"
            Invoke-SubstrateWithInstallContextV1 `
                -SubstrateExe $substrateExe `
                -InstallContext $InstallContext `
                -Arguments @('world', 'doctor', '--json') | Out-Null
        } catch {
            Write-Warn "world doctor reported issues: $($_.Exception.Message)"
        }
    }

    $configPath = Join-Path $Prefix 'config.yaml'
    $legacyConfigPath = Join-Path $Prefix 'config.toml'
    if (Test-Path $legacyConfigPath) {
        Write-Warn "Legacy TOML config detected at $legacyConfigPath. YAML config is now required ($configPath). Delete the TOML file and run 'substrate config init --force' to regenerate defaults."
    }
    if (-not (Test-Path $configPath)) {
        Write-Warn "Config metadata missing at $configPath. Run 'substrate config init' after installation to create defaults."
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
