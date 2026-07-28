#!/usr/bin/env pwsh
param(
    [string]$ProjectPath = (Resolve-Path "$PSScriptRoot/../.." | Select-Object -ExpandProperty Path),
    [string]$DistroName = 'substrate-wsl',
    [string]$PipePath = '\\.\pipe\substrate-agent',
    [int]$TimeoutSeconds = 300,
    [string]$RustLog = 'info',
    [string[]]$AdditionalArgs = @(),
    [switch]$WaitForExit,
    [int]$ReadyTimeoutSeconds = 30,
    [string]$TcpBridge = $null,
    [string]$InstallPrefix,
    [string]$InstallBootstrapContextV1,
    [string]$PlatformBootstrapMappingV1
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Write-Info($Message) {
    Write-Host "[INFO] $Message" -ForegroundColor Cyan
}

function Write-Warn($Message) {
    Write-Host "[WARN] $Message" -ForegroundColor Yellow
}

function Assert-CurrentWindowsPrincipalV1 {
    if (-not [System.Runtime.InteropServices.RuntimeInformation]::IsOSPlatform([System.Runtime.InteropServices.OSPlatform]::Windows)) {
        throw 'Windows platform bootstrap mapping requires a Windows host.'
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

    $normalizeWindowsPath = {
        param([string]$RawPath)
        if ([string]::IsNullOrEmpty($RawPath) -or $RawPath.IndexOf([char]0) -ge 0) {
            throw 'invalid platform bootstrap mapping'
        }

        $normalizedSeparators = $RawPath.Replace('/', '\')
        $lowercase = $normalizedSeparators.ToLowerInvariant()
        if ($lowercase.StartsWith('\\?\') -or $lowercase.StartsWith('\\.\')) {
            throw 'invalid platform bootstrap mapping'
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

        $components = @($normalizedSeparators.Substring($(if ($normalizedSeparators.StartsWith('\\')) { 2 } else { 3 })) -split '\\+' | Where-Object { $_ -ne '' })
        if ($components.Count -eq 0) {
            throw 'invalid platform bootstrap mapping'
        }
        foreach ($component in $components) {
            if (-not (& $validWindowsComponent $component)) {
                throw 'invalid platform bootstrap mapping'
            }
        }
        if ($normalizedSeparators.StartsWith('\\')) {
            if ($components.Count -lt 3) {
                throw 'invalid platform bootstrap mapping'
            }
            return ('\\' + ($components -join '\'))
        }
        if (
            $normalizedSeparators.Length -lt 4 -or
            (-not [char]::IsLetter($normalizedSeparators[0])) -or
            $normalizedSeparators[1] -cne ':' -or
            $normalizedSeparators[2] -cne '\'
        ) {
            throw 'invalid platform bootstrap mapping'
        }
        $drive = [char]::ToUpperInvariant($normalizedSeparators[0])
        return ('{0}:\{1}' -f $drive, ($components -join '\'))
    }

    $validPrincipalText = {
        param([string]$Value)
        return (-not [string]::IsNullOrEmpty($Value)) -and
            ($Value.IndexOf([char]0) -lt 0) -and
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

    $identity = [System.Security.Principal.WindowsIdentity]::GetCurrent()
    if ($null -eq $identity) {
        throw 'Unable to resolve the current Windows principal.'
    }

    try {
        $account = $identity.Name
        $sidObject = $identity.User
        $sid = if ($null -ne $sidObject) { $sidObject.Value } else { $null }
        if (-not (& $validPrincipalText $account) -or -not (& $validWindowsSid $sid) -or $sid -ceq 'S-1-5-7') {
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
            CanonicalAccount               = $account
            CanonicalSid                   = $sid
            KnownFolderLocalApplicationData = (& $normalizeWindowsPath $knownFolder)
        }
    } finally {
        if ($identity -is [System.IDisposable]) {
            $identity.Dispose()
        }
    }
}

function Get-WindowsForwarderScopeV1 {
    param(
        [Parameter(Mandatory = $true)][string]$WindowsSid,
        [Parameter(Mandatory = $true)][string]$DistroName,
        [Parameter(Mandatory = $true)][string]$GuestMachineId,
        [Parameter(Mandatory = $true)][string]$PipePath
    )

    $validPrincipalText = {
        param([string]$Value)
        return (-not [string]::IsNullOrEmpty($Value)) -and
            ($Value.IndexOf([char]0) -lt 0) -and
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
    $encodeBase64Url = {
        param([string]$Value)
        return [Convert]::ToBase64String([System.Text.Encoding]::UTF8.GetBytes($Value)).TrimEnd('=').Replace('+', '-').Replace('/', '_')
    }
    $normalizePipePath = {
        param([string]$RawPath)
        $prefix = '\\.\pipe\'
        if ([string]::IsNullOrEmpty($RawPath) -or -not $RawPath.StartsWith($prefix, [System.StringComparison]::Ordinal)) {
            throw 'invalid platform bootstrap mapping'
        }
        $name = $RawPath.Substring($prefix.Length)
        $asciiOnly = $true
        foreach ($byte in [System.Text.Encoding]::UTF8.GetBytes($name)) {
            if ($byte -ge 128) {
                $asciiOnly = $false
                break
            }
        }
        if (
            [string]::IsNullOrEmpty($name) -or
            $name.Length -gt 128 -or
            (-not $asciiOnly) -or
            $name -notmatch '^[A-Za-z0-9._-]+$'
        ) {
            throw 'invalid platform bootstrap mapping'
        }
        return '\\.\pipe\' + $name.ToLowerInvariant()
    }

    if (-not (& $validWindowsSid $WindowsSid) -or -not (& $validPrincipalText $DistroName) -or $GuestMachineId -notmatch '^[0-9a-f]{32}$') {
        throw 'invalid platform bootstrap mapping'
    }

    $canonicalPipePath = & $normalizePipePath $PipePath
    $frame = @(
        'domain=substrate.windows_forwarder_scope'
        'version=1'
        ('windows_sid={0}' -f (& $encodeBase64Url $WindowsSid))
        ('distro_name={0}' -f (& $encodeBase64Url $DistroName))
        "guest_machine_id=$GuestMachineId"
        ('pipe_path={0}' -f (& $encodeBase64Url $canonicalPipePath))
    ) -join "`n"
    $frame = "$frame`n"
    $hashBytes = [System.Security.Cryptography.SHA256]::HashData([System.Text.Encoding]::ASCII.GetBytes($frame))
    $digest = ([System.BitConverter]::ToString($hashBytes).Replace('-', '').ToLowerInvariant())

    return [pscustomobject]@{
        Digest            = $digest
        CanonicalPipePath = $canonicalPipePath
    }
}

function Resolve-PlatformBootstrapMappingV1 {
    param(
        [string]$DeclaredPrefix,
        [string]$InstallBootstrapContextV1,
        [string]$PlatformBootstrapMappingV1,
        [Parameter(Mandatory = $true)][string]$DeclaredDistroName,
        [Parameter(Mandatory = $true)][string]$DeclaredPipePath,
        [switch]$SkipObservationWhenMappingOmitted
    )

    $currentPrincipal = Assert-CurrentWindowsPrincipalV1

    $validPrincipalText = {
        param([string]$Value)
        return (-not [string]::IsNullOrEmpty($Value)) -and
            ($Value.IndexOf([char]0) -lt 0) -and
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
    $encodeBase64Url = {
        param([string]$Value)
        return [Convert]::ToBase64String([System.Text.Encoding]::UTF8.GetBytes($Value)).TrimEnd('=').Replace('+', '-').Replace('/', '_')
    }
    $decodeBase64Url = {
        param([string]$Value)
        if ([string]::IsNullOrEmpty($Value) -or $Value -notmatch '^[A-Za-z0-9_-]+$') {
            throw 'invalid platform bootstrap mapping'
        }
        $padding = (4 - ($Value.Length % 4)) % 4
        $base64 = $Value.Replace('-', '+').Replace('_', '/') + ('=' * $padding)
        try {
            $bytes = [Convert]::FromBase64String($base64)
        } catch {
            throw 'invalid platform bootstrap mapping'
        }
        $canonical = [Convert]::ToBase64String($bytes).TrimEnd('=').Replace('+', '-').Replace('/', '_')
        if ($canonical -cne $Value) {
            throw 'invalid platform bootstrap mapping'
        }
        return [System.Text.Encoding]::UTF8.GetString($bytes)
    }
    $normalizeWindowsPath = {
        param([string]$RawPath)
        if ([string]::IsNullOrEmpty($RawPath) -or $RawPath.IndexOf([char]0) -ge 0) {
            throw 'invalid platform bootstrap mapping'
        }

        $normalizedSeparators = $RawPath.Replace('/', '\')
        $lowercase = $normalizedSeparators.ToLowerInvariant()
        if ($lowercase.StartsWith('\\?\') -or $lowercase.StartsWith('\\.\')) {
            throw 'invalid platform bootstrap mapping'
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

        $components = @($normalizedSeparators.Substring($(if ($normalizedSeparators.StartsWith('\\')) { 2 } else { 3 })) -split '\\+' | Where-Object { $_ -ne '' })
        if ($components.Count -eq 0) {
            throw 'invalid platform bootstrap mapping'
        }
        foreach ($component in $components) {
            if (-not (& $validWindowsComponent $component)) {
                throw 'invalid platform bootstrap mapping'
            }
        }
        if ($normalizedSeparators.StartsWith('\\')) {
            if ($components.Count -lt 3) {
                throw 'invalid platform bootstrap mapping'
            }
            return ('\\' + ($components -join '\'))
        }
        if (
            $normalizedSeparators.Length -lt 4 -or
            (-not [char]::IsLetter($normalizedSeparators[0])) -or
            $normalizedSeparators[1] -cne ':' -or
            $normalizedSeparators[2] -cne '\'
        ) {
            throw 'invalid platform bootstrap mapping'
        }
        $drive = [char]::ToUpperInvariant($normalizedSeparators[0])
        return ('{0}:\{1}' -f $drive, ($components -join '\'))
    }
    $normalizeUnixPath = {
        param([string]$RawPath)
        if ([string]::IsNullOrEmpty($RawPath) -or $RawPath -eq '/' -or -not $RawPath.StartsWith('/') -or $RawPath.StartsWith('//') -or $RawPath.IndexOf([char]0) -ge 0) {
            throw 'invalid platform bootstrap mapping'
        }
        $parts = New-Object System.Collections.Generic.List[string]
        foreach ($part in $RawPath.Substring(1).Split('/')) {
            if ([string]::IsNullOrEmpty($part)) {
                continue
            }
            if ($part -in @('.', '..')) {
                throw 'invalid platform bootstrap mapping'
            }
            [void]$parts.Add($part)
        }
        if ($parts.Count -eq 0) {
            throw 'invalid platform bootstrap mapping'
        }
        return '/' + ($parts -join '/')
    }
    $normalizePipePath = {
        param([string]$RawPath)
        $prefix = '\\.\pipe\'
        if ([string]::IsNullOrEmpty($RawPath) -or -not $RawPath.StartsWith($prefix, [System.StringComparison]::Ordinal)) {
            throw 'invalid platform bootstrap mapping'
        }
        $name = $RawPath.Substring($prefix.Length)
        $asciiOnly = $true
        foreach ($byte in [System.Text.Encoding]::UTF8.GetBytes($name)) {
            if ($byte -ge 128) {
                $asciiOnly = $false
                break
            }
        }
        if (
            [string]::IsNullOrEmpty($name) -or
            $name.Length -gt 128 -or
            (-not $asciiOnly) -or
            $name -notmatch '^[A-Za-z0-9._-]+$'
        ) {
            throw 'invalid platform bootstrap mapping'
        }
        return '\\.\pipe\' + $name.ToLowerInvariant()
    }
    $parseCanonicalU32 = {
        param([string]$Value)
        if ([string]::IsNullOrEmpty($Value) -or $Value -notmatch '^[0-9]+$' -or ($Value.Length -gt 1 -and $Value.StartsWith('0'))) {
            throw 'invalid platform bootstrap mapping'
        }
        try {
            return [uint32]$Value
        } catch {
            throw 'invalid platform bootstrap mapping'
        }
    }
    $normalizeWslName = {
        param([string]$Value)
        if (-not $Value) {
            return $Value
        }
        return $Value
    }

    $newInstallContext = {
        param([string]$SelectedPrefix, [string]$Account, [string]$Sid)
        $selected = & $normalizeWindowsPath $SelectedPrefix
        if (-not (& $validPrincipalText $Account) -or -not (& $validWindowsSid $Sid)) {
            throw 'invalid install bootstrap carrier'
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
        $hashBytes = [System.Security.Cryptography.SHA256]::HashData([System.Text.Encoding]::ASCII.GetBytes($commitmentInput))
        $commitment = ([System.BitConverter]::ToString($hashBytes).Replace('-', '').ToLowerInvariant())
        $record = "${commitmentInput}host_context_commitment=$commitment`n"
        [pscustomobject]@{
            SelectedHostPrefix    = $selected
            HostSubstrateHome     = $selected
            HostSubstrateRoot     = $selected
            CanonicalAccount      = $Account
            CanonicalSid          = $Sid
            HostContextCommitment = $commitment
            EncodedCarrier        = (& $encodeBase64Url $record)
        }
    }

    $installContext = if ([string]::IsNullOrEmpty($InstallBootstrapContextV1)) {
        $selectedPrefix = if ([string]::IsNullOrEmpty($DeclaredPrefix)) {
            & $normalizeWindowsPath ("{0}\Substrate" -f $currentPrincipal.KnownFolderLocalApplicationData)
        } else {
            & $normalizeWindowsPath $DeclaredPrefix
        }
        & $newInstallContext $selectedPrefix $currentPrincipal.CanonicalAccount $currentPrincipal.CanonicalSid
    } else {
        $record = & $decodeBase64Url $InstallBootstrapContextV1
        if ($record.Contains("`r") -or $record.IndexOf([char]0) -ge 0) {
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
        $hostSubstrateHome = & $decodeBase64Url $parsed['host_substrate_home']
        $root = & $decodeBase64Url $parsed['host_substrate_root']
        $account = & $decodeBase64Url $parsed['principal_account']
        $sid = & $decodeBase64Url $parsed['principal_sid']
        $context = & $newInstallContext $selectedPrefix $account $sid
        if (
            $context.HostSubstrateHome -cne $hostSubstrateHome -or
            $context.HostSubstrateRoot -cne $root -or
            $context.HostContextCommitment -cne $parsed['host_context_commitment'] -or
            $context.EncodedCarrier -cne $InstallBootstrapContextV1
        ) {
            throw 'invalid install bootstrap carrier'
        }
        if ($context.CanonicalAccount -cne $currentPrincipal.CanonicalAccount -or $context.CanonicalSid -cne $currentPrincipal.CanonicalSid) {
            throw 'install bootstrap principal does not match the current Windows principal'
        }
        if (-not [string]::IsNullOrEmpty($DeclaredPrefix)) {
            $normalizedDeclaredPrefix = & $normalizeWindowsPath $DeclaredPrefix
            if ($normalizedDeclaredPrefix -cne $context.SelectedHostPrefix) {
                throw 'declared install prefix does not match the install bootstrap carrier'
            }
        }
        foreach ($entry in @{
                SUBSTRATE_HOME                            = $context.HostSubstrateHome
                SUBSTRATE_ROOT                            = $context.HostSubstrateRoot
                SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT = $context.HostContextCommitment
                SUBSTRATE_INSTALL_PRIMARY_USER            = $context.CanonicalAccount
                SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1    = $context.EncodedCarrier
            }.GetEnumerator()) {
            $current = [Environment]::GetEnvironmentVariable($entry.Key)
            if ($null -ne $current -and $current -cne $entry.Value) {
                throw 'install bootstrap environment projection is conflicting'
            }
        }
        $context
    }

    $canonicalPipePath = & $normalizePipePath $DeclaredPipePath
    $collectDistinctNames = {
        param([string[]]$Values)
        $seen = New-Object 'System.Collections.Generic.HashSet[string]' ([System.StringComparer]::Ordinal)
        $result = New-Object System.Collections.Generic.List[string]
        foreach ($value in $Values) {
            if ($null -eq $value) {
                continue
            }
            $trimmed = $value.Trim()
            if (-not $trimmed) {
                continue
            }
            if ($seen.Add($trimmed)) {
                [void]$result.Add($trimmed)
            }
        }
        return @($result.ToArray())
    }
    $registeredDistros = @()
    $runningDistros = @()
    $collectRunningDistros = {
        param([string[]]$RegisteredDistroNames)
        $orderedNames = @($RegisteredDistroNames |
                Where-Object { $_ -and $_.Trim() } |
                Sort-Object -Property @{ Expression = { $_.Length }; Descending = $true }, @{ Expression = { $_ }; Descending = $false })
        $seen = New-Object 'System.Collections.Generic.HashSet[string]' ([System.StringComparer]::Ordinal)
        $result = New-Object System.Collections.Generic.List[string]
        try {
            & wsl.exe -l -v 2>$null | Select-Object -Skip 1 | ForEach-Object {
                $line = ($_ -replace '^\s*\*\s*', '').Trim()
                if (-not $line) {
                    return
                }
                foreach ($registeredName in $orderedNames) {
                    if (-not $line.StartsWith($registeredName, [System.StringComparison]::Ordinal)) {
                        continue
                    }
                    $remainder = $line.Substring($registeredName.Length)
                    if ($remainder -match '^\s{2,}(?<state>\S+)(?:\s{2,}\S.*)?$' -and $Matches['state'].Equals('Running', [System.StringComparison]::OrdinalIgnoreCase)) {
                        if ($seen.Add($registeredName)) {
                            [void]$result.Add($registeredName)
                        }
                    }
                    return
                }
            }
        } catch {}
        return @($result.ToArray())
    }
    try {
        $registeredDistros += (& wsl.exe -l -q 2>$null)
    } catch {}
    $registeredDistros = & $collectDistinctNames $registeredDistros
    $runningDistros = & $collectRunningDistros $registeredDistros
    $exactDistro = @($registeredDistros | Where-Object { $_ -and ((& $normalizeWslName $_) -ieq (& $normalizeWslName $DeclaredDistroName)) })
    if ($exactDistro.Count -eq 0) {
        throw 'registered WSL distro not found'
    }
    if ($exactDistro.Count -gt 1) {
        throw 'registered WSL distro is ambiguous'
    }
    $exactDistroName = [string]$exactDistro[0]

    $observeGuestIdentity = {
        $runningExactDistro = @($runningDistros | Where-Object { $_ -and ((& $normalizeWslName $_) -ieq (& $normalizeWslName $exactDistroName)) })
        if ($runningExactDistro.Count -ne 1) {
            throw 'registered WSL distro is not already running'
        }
        $machineIdScript = 'cat /etc/machine-id | tr -d ''\r\n'''
        $machineId = (& wsl.exe -d $exactDistroName -- bash -lc $machineIdScript 2>$null | Out-String).Trim()
        $machineIdAgain = (& wsl.exe -d $exactDistroName -- bash -lc $machineIdScript 2>$null | Out-String).Trim()
        $identityScript = @'
set -euo pipefail
account=$(id -un | tr -d '\r\n')
uid=$(id -u | tr -d '\r\n')
name_entry=$(getent passwd "$account" | tr -d '\r')
uid_entry=$(getent passwd "$uid" | tr -d '\r')
printf '%s\n%s\n%s\n%s\n' "$account" "$uid" "$name_entry" "$uid_entry"
'@
        $identityLines = @(& wsl.exe -d $exactDistroName -- bash -lc $identityScript 2>$null)
        if ($machineId -cne $machineIdAgain -or $identityLines.Count -lt 4) {
            throw 'guest identity observation is invalid'
        }
        $account = ($identityLines[0] | Out-String).Trim()
        $uid = ($identityLines[1] | Out-String).Trim()
        $nameEntry = ($identityLines[2] | Out-String).Trim()
        $uidEntry = ($identityLines[3] | Out-String).Trim()
        if ([string]::IsNullOrEmpty($account) -or [string]::IsNullOrEmpty($uid) -or [string]::IsNullOrEmpty($nameEntry) -or $nameEntry -cne $uidEntry) {
            throw 'guest identity observation is invalid'
        }
        $entry = $nameEntry.Split(':')
        if ($entry.Length -lt 6 -or $entry[0] -cne $account -or $entry[2] -cne $uid -or [string]::IsNullOrEmpty($entry[5])) {
            throw 'guest identity observation is invalid'
        }
        [pscustomobject]@{
            GuestMachineId = $machineId
            GuestAccount   = $account
            GuestUid       = $uid
            GuestHome      = $entry[5]
        }
    }

    $buildMapping = {
        param(
            [string]$GuestMachineId,
            [string]$GuestAccount,
            [string]$GuestUid,
            [string]$GuestHome
        )
        $canonicalGuestMachineId = $GuestMachineId.Trim()
        if ($canonicalGuestMachineId -notmatch '^[0-9a-f]{32}$' -or -not (& $validPrincipalText $GuestAccount)) {
            throw 'invalid platform bootstrap mapping'
        }
        $canonicalGuestUid = & $parseCanonicalU32 $GuestUid
        $canonicalGuestHome = & $normalizeUnixPath $GuestHome
        $guestSubstrateHome = & $normalizeUnixPath (($canonicalGuestHome.TrimEnd('/')) + '/.substrate')
        $scope = Get-WindowsForwarderScopeV1 -WindowsSid $currentPrincipal.CanonicalSid -DistroName $exactDistroName -GuestMachineId $canonicalGuestMachineId -PipePath $canonicalPipePath
        $hostPlatformControlRoot = & $normalizeWindowsPath (Join-Path $currentPrincipal.KnownFolderLocalApplicationData ("Substrate\forwarder\{0}" -f $scope.Digest))
        $record = @(
            'domain=substrate.platform_bootstrap_mapping'
            'version=1'
            ('host_context_commitment={0}' -f $installContext.HostContextCommitment)
            'platform_kind=wsl'
            ('instance_name={0}' -f (& $encodeBase64Url $exactDistroName))
            ('guest_machine_id={0}' -f $canonicalGuestMachineId)
            ('host_platform_control_root={0}' -f (& $encodeBase64Url $hostPlatformControlRoot))
            ('realized_substrate_home={0}' -f (& $encodeBase64Url $guestSubstrateHome))
            ('realized_principal_account={0}' -f (& $encodeBase64Url $GuestAccount))
            ('realized_principal_uid={0}' -f $canonicalGuestUid)
            'transport_kind=wsl'
            ('transport_host={0}' -f (& $encodeBase64Url $canonicalPipePath))
            ('transport_guest_socket={0}' -f (& $encodeBase64Url '/run/substrate.sock'))
        ) -join "`n"
        $record = "$record`n"
        [pscustomobject]@{
            InstallContext             = $installContext
            DistroName                 = $exactDistroName
            GuestMachineId             = $canonicalGuestMachineId
            GuestAccount               = $GuestAccount
            GuestUid                   = [string]$canonicalGuestUid
            GuestHome                  = $canonicalGuestHome
            RealizedSubstrateHome      = $guestSubstrateHome
            PipePath                   = $canonicalPipePath
            PipeName                   = $canonicalPipePath.Substring('\\.\pipe\'.Length)
            GuestSocket                = '/run/substrate.sock'
            HostPlatformControlRoot    = $hostPlatformControlRoot
            WindowsForwarderScopeV1    = $scope.Digest
            ForwarderConfigPath        = & $normalizeWindowsPath (Join-Path $installContext.SelectedHostPrefix 'forwarder\forwarder.toml')
            ForwarderLogDir            = & $normalizeWindowsPath (Join-Path $installContext.SelectedHostPrefix 'forwarder\logs')
            SharedForwarderPidPath     = & $normalizeWindowsPath (Join-Path $hostPlatformControlRoot 'forwarder.pid')
            PlatformBootstrapMappingV1 = (& $encodeBase64Url $record)
        }
    }

    $mappingState = if ([string]::IsNullOrEmpty($PlatformBootstrapMappingV1)) {
        if ($SkipObservationWhenMappingOmitted.IsPresent) {
            [pscustomobject]@{
                InstallContext             = $installContext
                DistroName                 = $exactDistroName
                GuestMachineId             = $null
                GuestAccount               = $null
                GuestUid                   = $null
                GuestHome                  = $null
                RealizedSubstrateHome      = $null
                PipePath                   = $canonicalPipePath
                PipeName                   = $canonicalPipePath.Substring('\\.\pipe\'.Length)
                GuestSocket                = '/run/substrate.sock'
                HostPlatformControlRoot    = $null
                WindowsForwarderScopeV1    = $null
                ForwarderConfigPath        = & $normalizeWindowsPath (Join-Path $installContext.SelectedHostPrefix 'forwarder\forwarder.toml')
                ForwarderLogDir            = & $normalizeWindowsPath (Join-Path $installContext.SelectedHostPrefix 'forwarder\logs')
                SharedForwarderPidPath     = $null
                PlatformBootstrapMappingV1 = $null
            }
        } else {
            $observation = & $observeGuestIdentity
            & $buildMapping $observation.GuestMachineId $observation.GuestAccount $observation.GuestUid $observation.GuestHome
        }
    } else {
        $record = & $decodeBase64Url $PlatformBootstrapMappingV1
        if ($record.Contains("`r") -or $record.IndexOf([char]0) -ge 0 -or -not $record.EndsWith("`n")) {
            throw 'invalid platform bootstrap mapping'
        }
        $lines = $record.Substring(0, $record.Length - 1).Split("`n")
        if ($lines.Length -ne 13) {
            throw 'invalid platform bootstrap mapping'
        }
        $expectedKeys = @(
            'domain',
            'version',
            'host_context_commitment',
            'platform_kind',
            'instance_name',
            'guest_machine_id',
            'host_platform_control_root',
            'realized_substrate_home',
            'realized_principal_account',
            'realized_principal_uid',
            'transport_kind',
            'transport_host',
            'transport_guest_socket'
        )
        $parsed = [ordered]@{}
        for ($index = 0; $index -lt $expectedKeys.Count; $index++) {
            $line = $lines[$index]
            $delimiter = $line.IndexOf('=')
            if ($delimiter -lt 1) {
                throw 'invalid platform bootstrap mapping'
            }
            $key = $line.Substring(0, $delimiter)
            $value = $line.Substring($delimiter + 1)
            if ($key -cne $expectedKeys[$index] -or $parsed.Contains($key)) {
                throw 'invalid platform bootstrap mapping'
            }
            $parsed[$key] = $value
        }
        if (
            $parsed['domain'] -cne 'substrate.platform_bootstrap_mapping' -or
            $parsed['version'] -cne '1' -or
            $parsed['host_context_commitment'] -cne $installContext.HostContextCommitment -or
            $parsed['platform_kind'] -cne 'wsl' -or
            $parsed['transport_kind'] -cne 'wsl'
        ) {
            throw 'invalid platform bootstrap mapping'
        }
        $decodedDistroName = & $decodeBase64Url $parsed['instance_name']
        $decodedGuestMachineId = $parsed['guest_machine_id']
        $decodedGuestAccount = & $decodeBase64Url $parsed['realized_principal_account']
        $decodedGuestUid = $parsed['realized_principal_uid']
        $decodedControlRoot = & $decodeBase64Url $parsed['host_platform_control_root']
        $decodedGuestSubstrateHome = & $decodeBase64Url $parsed['realized_substrate_home']
        $decodedPipePath = & $decodeBase64Url $parsed['transport_host']
        $decodedGuestSocket = & $decodeBase64Url $parsed['transport_guest_socket']
        if (
            -not (& $validPrincipalText $decodedDistroName) -or
            -not (& $validPrincipalText $decodedGuestAccount) -or
            $decodedDistroName -cne $exactDistroName -or
            $decodedGuestMachineId -notmatch '^[0-9a-f]{32}$' -or
            $decodedGuestUid -notmatch '^(0|[1-9][0-9]*)$' -or
            $decodedPipePath -cne $canonicalPipePath -or
            $decodedGuestSocket -cne '/run/substrate.sock' -or
            (& $normalizeWindowsPath $decodedControlRoot) -ne $decodedControlRoot
        ) {
            throw 'invalid platform bootstrap mapping'
        }
        $decodedSubstrateHome = & $normalizeUnixPath $decodedGuestSubstrateHome
        if (-not $decodedSubstrateHome.EndsWith('/.substrate')) {
            throw 'invalid platform bootstrap mapping'
        }
        $decodedGuestHome = & $normalizeUnixPath ($decodedSubstrateHome.Substring(0, $decodedSubstrateHome.Length - '/.substrate'.Length))
        $observation = & $observeGuestIdentity
        $state = & $buildMapping `
            $observation.GuestMachineId `
            $observation.GuestAccount `
            $observation.GuestUid `
            $observation.GuestHome
        if (
            $decodedDistroName -cne $exactDistroName -or
            $decodedGuestMachineId -cne $state.GuestMachineId -or
            $decodedGuestAccount -cne $state.GuestAccount -or
            $decodedGuestUid -cne $state.GuestUid -or
            $decodedControlRoot -cne $state.HostPlatformControlRoot -or
            $decodedSubstrateHome -cne $state.RealizedSubstrateHome -or
            $decodedPipePath -cne $state.PipePath -or
            $decodedGuestSocket -cne $state.GuestSocket -or
            $state.PlatformBootstrapMappingV1 -cne $PlatformBootstrapMappingV1
        ) {
            throw 'invalid platform bootstrap mapping'
        }
        $state
    }

    return $mappingState
}

function Assert-PlatformBootstrapMappingV1 {
    param(
        [Parameter(Mandatory = $true)][pscustomobject]$MappingState,
        [string]$DeclaredPrefix,
        [string]$DeclaredDistroName,
        [string]$DeclaredPipePath
    )

    $normalizeWindowsPath = {
        param([string]$RawPath)
        if ([string]::IsNullOrEmpty($RawPath) -or $RawPath.IndexOf([char]0) -ge 0) {
            throw 'declared install prefix does not match the install bootstrap carrier'
        }

        $normalizedSeparators = $RawPath.Replace('/', '\')
        $lowercase = $normalizedSeparators.ToLowerInvariant()
        if ($lowercase.StartsWith('\\?\') -or $lowercase.StartsWith('\\.\')) {
            throw 'declared install prefix does not match the install bootstrap carrier'
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

        $components = @($normalizedSeparators.Substring($(if ($normalizedSeparators.StartsWith('\\')) { 2 } else { 3 })) -split '\\+' | Where-Object { $_ -ne '' })
        if ($components.Count -eq 0) {
            throw 'declared install prefix does not match the install bootstrap carrier'
        }
        foreach ($component in $components) {
            if (-not (& $validWindowsComponent $component)) {
                throw 'declared install prefix does not match the install bootstrap carrier'
            }
        }
        if ($normalizedSeparators.StartsWith('\\')) {
            if ($components.Count -lt 3) {
                throw 'declared install prefix does not match the install bootstrap carrier'
            }
            return ('\\' + ($components -join '\'))
        }
        if (
            $normalizedSeparators.Length -lt 4 -or
            (-not [char]::IsLetter($normalizedSeparators[0])) -or
            $normalizedSeparators[1] -cne ':' -or
            $normalizedSeparators[2] -cne '\'
        ) {
            throw 'declared install prefix does not match the install bootstrap carrier'
        }
        $drive = [char]::ToUpperInvariant($normalizedSeparators[0])
        return ('{0}:\{1}' -f $drive, ($components -join '\'))
    }

    if (-not [string]::IsNullOrEmpty($DeclaredPrefix)) {
        $normalizedDeclaredPrefix = & $normalizeWindowsPath $DeclaredPrefix
        if ($MappingState.InstallContext.SelectedHostPrefix -cne $normalizedDeclaredPrefix) {
            throw 'declared install prefix does not match the install bootstrap carrier'
        }
    }
    if (-not [string]::IsNullOrEmpty($DeclaredDistroName) -and $MappingState.DistroName -ne $DeclaredDistroName -and $MappingState.DistroName -ine $DeclaredDistroName) {
        throw 'declared distro does not match platform bootstrap mapping'
    }
    if (-not [string]::IsNullOrEmpty($DeclaredPipePath)) {
        $candidate = $DeclaredPipePath.Replace('/', '\')
        if (-not $candidate.StartsWith('\\.\pipe\', [System.StringComparison]::OrdinalIgnoreCase)) {
            throw 'declared pipe does not match platform bootstrap mapping'
        }
        if ($candidate.Substring('\\.\pipe\'.Length).ToLowerInvariant() -cne $MappingState.PipeName) {
            throw 'declared pipe does not match platform bootstrap mapping'
        }
    }
}

$resolvedProject = Resolve-Path $ProjectPath | Select-Object -ExpandProperty Path
if (-not (Test-Path (Join-Path $resolvedProject 'Cargo.toml'))) {
    throw "Project path $resolvedProject does not contain Cargo.toml"
}

$forwarderRelease = Join-Path $resolvedProject 'target/release/substrate-forwarder.exe'
$forwarderDebug = Join-Path $resolvedProject 'target/debug/substrate-forwarder.exe'
if (Test-Path $forwarderRelease) {
    $forwarderExe = $forwarderRelease
} elseif (Test-Path $forwarderDebug) {
    Write-Warn 'Release binary not found, using debug build'
    $forwarderExe = $forwarderDebug
} else {
    throw "substrate-forwarder binary not found. Build it with 'cargo build -p substrate-forwarder --release'"
}

$mappingState = Resolve-PlatformBootstrapMappingV1 `
    -DeclaredPrefix $InstallPrefix `
    -InstallBootstrapContextV1 $InstallBootstrapContextV1 `
    -PlatformBootstrapMappingV1 $PlatformBootstrapMappingV1 `
    -DeclaredDistroName $DistroName `
    -DeclaredPipePath $PipePath
Assert-PlatformBootstrapMappingV1 `
    -MappingState $mappingState `
    -DeclaredPrefix $InstallPrefix `
    -DeclaredDistroName $DistroName `
    -DeclaredPipePath $PipePath

$logDir = $mappingState.ForwarderLogDir
New-Item -ItemType Directory -Force $logDir | Out-Null

$argumentList = @(
    '--distro',
    $mappingState.DistroName,
    '--pipe',
    $mappingState.PipePath,
    '--config',
    $mappingState.ForwarderConfigPath,
    '--log-dir',
    $mappingState.ForwarderLogDir
)

if ($TcpBridge) {
    Write-Info ("Enabling host TCP bridge at {0}" -f $TcpBridge)
    $argumentList += @('--tcp-bridge', $TcpBridge)
}
if ($AdditionalArgs.Length -gt 0) {
    $reservedForwarderArgs = @('--distro', '--pipe', '--config', '--log-dir', '--tcp-bridge')
    foreach ($arg in $AdditionalArgs) {
        if ($null -eq $arg) {
            continue
        }
        $candidate = $arg.Trim()
        if (-not $candidate.StartsWith('--', [System.StringComparison]::Ordinal)) {
            continue
        }
        $delimiterIndex = $candidate.IndexOf('=')
        $flag = if ($delimiterIndex -ge 0) { $candidate.Substring(0, $delimiterIndex) } else { $candidate }
        if ($reservedForwarderArgs -contains $flag.ToLowerInvariant()) {
            throw "AdditionalArgs may not override authenticated forwarder selector $flag"
        }
    }
    $argumentList += $AdditionalArgs
}

$psi = New-Object System.Diagnostics.ProcessStartInfo
$psi.FileName = $forwarderExe
foreach ($arg in $argumentList) {
    $null = $psi.ArgumentList.Add($arg)
}
$psi.UseShellExecute = $false
$psi.RedirectStandardOutput = $true
$psi.RedirectStandardError = $true
$psi.Environment['RUST_LOG'] = $RustLog

$childEnvironment = @{
    SUBSTRATE_HOME                            = $mappingState.InstallContext.HostSubstrateHome
    SUBSTRATE_ROOT                            = $mappingState.InstallContext.HostSubstrateRoot
    SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT = $mappingState.InstallContext.HostContextCommitment
    SUBSTRATE_INSTALL_PRIMARY_USER            = $mappingState.InstallContext.CanonicalAccount
    SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1    = $mappingState.InstallContext.EncodedCarrier
    SUBSTRATE_FORWARDER_TARGET                = ('uds:{0}' -f $mappingState.GuestSocket)
    LOCALAPPDATA                              = $null
    USERPROFILE                               = $null
    WSLENV                                    = $null
    SUBSTRATE_FORWARDER_TARGET_MODE           = $null
    SUBSTRATE_FORWARDER_TARGET_HOST           = $null
    SUBSTRATE_FORWARDER_TARGET_PORT           = $null
    SUBSTRATE_FORWARDER_TARGET_ENDPOINT       = $null
    SUBSTRATE_FORWARDER_PIPE                  = $null
    SUBSTRATE_FORWARDER_TCP                   = $null
    SUBSTRATE_FORWARDER_TCP_ADDR              = $null
    SUBSTRATE_FORWARDER_TCP_HOST              = $null
    SUBSTRATE_FORWARDER_TCP_PORT              = $null
}
foreach ($entry in $childEnvironment.GetEnumerator()) {
    if ($null -eq $entry.Value) {
        $psi.Environment.Remove($entry.Key)
    } else {
        $psi.Environment[$entry.Key] = [string]$entry.Value
    }
}

Write-Info ("Resolved host commitment {0}, distro {1}, machine-id {2}, pipe {3}" -f $mappingState.InstallContext.HostContextCommitment, $mappingState.DistroName, $mappingState.GuestMachineId, $mappingState.PipePath)
Write-Info ("Using control root {0} and scope {1}" -f $mappingState.HostPlatformControlRoot, $mappingState.WindowsForwarderScopeV1)
Write-Info ("Launching substrate-forwarder (guard {0}s)" -f $TimeoutSeconds)
$process = [System.Diagnostics.Process]::Start($psi)

# Default: return once ready. Use -WaitForExit only for CI/service.
if (-not $WaitForExit) {
    # Probe readiness and return immediately
    Write-Info ("Waiting up to {0}s for pipe readiness" -f $ReadyTimeoutSeconds)
    if ($PipePath -match '\\pipe\\(?<n>[^\\]+)$') {
        $pipeName = $Matches['n']
    } else {
        $pipeName = $PipePath
    }
    Write-Info ("Using pipe name '{0}' from '{1}'" -f $pipeName, $PipePath)
    $deadline = [DateTime]::UtcNow.AddSeconds([Math]::Max(1, $ReadyTimeoutSeconds))
    $ready = $false
    while ([DateTime]::UtcNow -lt $deadline) {
        try {
            $client = [System.IO.Pipes.NamedPipeClientStream]::new('.', $pipeName, [System.IO.Pipes.PipeDirection]::InOut, [System.IO.Pipes.PipeOptions]::None)
            $client.Connect(1000)
            $client.Dispose()
            $ready = $true
            break
        } catch {
            Start-Sleep -Milliseconds 200
        }
    }
    if (-not $ready) {
        Write-Warn "Pipe did not become ready within the readiness window; forwarder left running under guard"
    } else {
        Write-Info "Pipe is ready; returning without waiting for forwarder exit"
    }
    return
} else {
    $stopwatch = [System.Diagnostics.Stopwatch]::StartNew()
    if (-not $process.WaitForExit($TimeoutSeconds * 1000)) {
        Write-Warn "Forwarder exceeded timeout. Terminating process."
        try {
            $process.Kill()
            $process.WaitForExit()
        } catch {
            Write-Warn "Failed to terminate forwarder process: $_"
        }
        $stopwatch.Stop()
        throw "substrate-forwarder exceeded timeout of $TimeoutSeconds seconds"
    }
    $stopwatch.Stop()

    $stdout = $process.StandardOutput.ReadToEnd()
    $stderr = $process.StandardError.ReadToEnd()
    if ($stdout) { Write-Output $stdout.TrimEnd() }
    if ($stderr) { Write-Warn $stderr.TrimEnd() }

    Write-Info ("Forwarder exited in {0:F1}s with code {1}" -f $stopwatch.Elapsed.TotalSeconds, $process.ExitCode)
    if ($process.ExitCode -ne 0) {
        throw "substrate-forwarder exited with code $($process.ExitCode)"
    }
}
