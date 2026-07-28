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
    [string]$Prefix,
    [string]$DistroName = 'substrate-wsl',
    [switch]$RemoveWSLDistro,
    [switch]$DryRun,
    [string]$InstallBootstrapContextV1
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

Write-Log "Stopping substrate-world-service inside WSL (if present)"
if ($dry) {
    Write-Log "[dry-run] wsl -d $DistroName -- bash -lc 'sudo systemctl disable --now substrate-world-service.socket substrate-world-service.service'"
} else {
    try {
        & wsl -d $DistroName -- bash -lc "sudo systemctl disable --now substrate-world-service.socket substrate-world-service.service" | Out-Null
    } catch {
        Write-Warn ("Unable to disable substrate-world-service inside {0}: {1}" -f $DistroName, $_.Exception.Message)
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
