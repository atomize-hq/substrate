#!/usr/bin/env pwsh
param(
    [string]$PipePath = '\\.\pipe\substrate-agent',
    [string]$DistroName = 'substrate-wsl',
    [string]$Method = 'GET',
    [string]$Path = '/v1/capabilities',
    [int]$TimeoutSeconds = 8,
    [int]$ExpectStatus = 200,
    [switch]$TraceParse,
    [string]$InstallPrefix,
    [string]$InstallBootstrapContextV1,
    [string]$PlatformBootstrapMappingV1
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Write-Info($Message){ Write-Host "[INFO] $Message" -ForegroundColor Cyan }
function Write-Fail($Message){ Write-Host "[FAIL] $Message" -ForegroundColor Red }

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

    $identity = [System.Security.Principal.WindowsIdentity]::GetCurrent()
    if ($null -eq $identity) {
        throw 'Unable to resolve the current Windows principal.'
    }

    try {
        $account = $identity.Name
        $sid = if ($null -ne $identity.User) { $identity.User.Value } else { $null }
        if ([string]::IsNullOrWhiteSpace($account) -or [string]::IsNullOrWhiteSpace($sid) -or $sid -notmatch '^S-[0-9]+(?:-[0-9]+)+$' -or $sid -ceq 'S-1-5-7') {
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

        return [pscustomobject]@{
            CanonicalAccount                = $account
            CanonicalSid                    = $sid
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

    $canonicalPipePath = '\\.\pipe\' + (Get-PipeNameFromPath -Path $PipePath)
    $frame = @(
        'domain=substrate.windows_forwarder_scope'
        'version=1'
        ('windows_sid={0}' -f ([Convert]::ToBase64String([System.Text.Encoding]::UTF8.GetBytes($WindowsSid)).TrimEnd('=').Replace('+', '-').Replace('/', '_')))
        ('distro_name={0}' -f ([Convert]::ToBase64String([System.Text.Encoding]::UTF8.GetBytes($DistroName)).TrimEnd('=').Replace('+', '-').Replace('/', '_')))
        "guest_machine_id=$GuestMachineId"
        ('pipe_path={0}' -f ([Convert]::ToBase64String([System.Text.Encoding]::UTF8.GetBytes($canonicalPipePath)).TrimEnd('=').Replace('+', '-').Replace('/', '_')))
    ) -join "`n"
    $frame = "$frame`n"
    $hashBytes = [System.Security.Cryptography.SHA256]::HashData([System.Text.Encoding]::ASCII.GetBytes($frame))
    ([System.BitConverter]::ToString($hashBytes).Replace('-', '').ToLowerInvariant())
}

function Get-PipeNameFromPath {
    param([Parameter(Mandatory = $true)][string]$Path)

    $prefix = '\\.\pipe\'
    if ([string]::IsNullOrEmpty($Path) -or -not $Path.StartsWith($prefix, [System.StringComparison]::Ordinal)) {
        throw "Invalid pipe path: $Path"
    }
    $name = $Path.Substring($prefix.Length)
    $asciiOnly = $true
    foreach ($byte in [System.Text.Encoding]::UTF8.GetBytes($name)) {
        if ($byte -ge 128) {
            $asciiOnly = $false
            break
        }
    }
    if ([string]::IsNullOrEmpty($name) -or $name.Length -gt 128 -or (-not $asciiOnly) -or $name -notmatch '^[A-Za-z0-9._-]+$') {
        throw "Invalid pipe path: $Path"
    }
    return $name.ToLowerInvariant()
}

function Get-HexBytes([string]$s) {
    -join (([Text.Encoding]::UTF8.GetBytes($s) | ForEach-Object { $_.ToString('X2') }) -join ' ')
}

function Resolve-PlatformBootstrapMappingV1 {
    param(
        [string]$DeclaredPrefix,
        [string]$InstallBootstrapContextV1,
        [string]$PlatformBootstrapMappingV1,
        [Parameter(Mandatory = $true)][string]$DeclaredDistroName,
        [Parameter(Mandatory = $true)][string]$DeclaredPipePath
    )

    $currentPrincipal = Assert-CurrentWindowsPrincipalV1
    $validPrincipalText = {
        param([string]$Value)
        return (-not [string]::IsNullOrEmpty($Value)) -and
            ($Value.IndexOf([char]0) -lt 0) -and
            (-not $Value.Contains("`n")) -and
            (-not $Value.Contains("`r"))
    }
    $encodeBase64Url = {
        param([string]$Value)
        [Convert]::ToBase64String([System.Text.Encoding]::UTF8.GetBytes($Value)).TrimEnd('=').Replace('+', '-').Replace('/', '_')
    }
    $decodeBase64Url = {
        param([string]$Value)
        if ([string]::IsNullOrEmpty($Value) -or $Value -notmatch '^[A-Za-z0-9_-]+$') {
            throw 'invalid platform bootstrap mapping'
        }
        $padding = (4 - ($Value.Length % 4)) % 4
        $bytes = [Convert]::FromBase64String($Value.Replace('-', '+').Replace('_', '/') + ('=' * $padding))
        $canonical = [Convert]::ToBase64String($bytes).TrimEnd('=').Replace('+', '-').Replace('/', '_')
        if ($canonical -cne $Value) {
            throw 'invalid platform bootstrap mapping'
        }
        [System.Text.Encoding]::UTF8.GetString($bytes)
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
        if ($normalizedSeparators.Length -lt 4 -or (-not [char]::IsLetter($normalizedSeparators[0])) -or $normalizedSeparators[1] -cne ':' -or $normalizedSeparators[2] -cne '\') {
            throw 'invalid platform bootstrap mapping'
        }
        return ('{0}:\{1}' -f [char]::ToUpperInvariant($normalizedSeparators[0]), ($components -join '\'))
    }
    $normalizeUnixPath = {
        param([string]$RawPath)
        if ([string]::IsNullOrEmpty($RawPath) -or $RawPath -eq '/' -or -not $RawPath.StartsWith('/') -or $RawPath.StartsWith('//') -or $RawPath.IndexOf([char]0) -ge 0) {
            throw 'invalid platform bootstrap mapping'
        }
        $parts = @($RawPath.Substring(1).Split('/') | Where-Object { $_ -ne '' })
        if ($parts.Count -eq 0 -or $parts -contains '.' -or $parts -contains '..') {
            throw 'invalid platform bootstrap mapping'
        }
        return '/' + ($parts -join '/')
    }
    $normalizeWslName = {
        param([string]$Value)
        if (-not $Value) { return $Value }
        return $Value
    }
    $newInstallContext = {
        param([string]$SelectedPrefix, [string]$Account, [string]$Sid)
        $selectedPrefix = & $normalizeWindowsPath $SelectedPrefix
        $accountEncoded = & $encodeBase64Url $Account
        $sidEncoded = & $encodeBase64Url $Sid
        $selectedEncoded = & $encodeBase64Url $selectedPrefix
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
        $commitmentBytes = [System.Security.Cryptography.SHA256]::HashData([System.Text.Encoding]::ASCII.GetBytes($commitmentInput))
        $hostContextCommitment = ([System.BitConverter]::ToString($commitmentBytes).Replace('-', '').ToLowerInvariant())
        $contextRecord = @(
            'domain=substrate.install_bootstrap_context'
            'version=1'
            "selected_host_prefix=$selectedEncoded"
            "host_substrate_home=$selectedEncoded"
            "host_substrate_root=$selectedEncoded"
            'principal_kind=windows'
            "principal_account=$accountEncoded"
            "principal_sid=$sidEncoded"
            "host_context_commitment=$hostContextCommitment"
        ) -join "`n"
        $contextRecord = "$contextRecord`n"
        return [pscustomobject]@{
            SelectedHostPrefix    = $selectedPrefix
            HostSubstrateHome     = $selectedPrefix
            HostSubstrateRoot     = $selectedPrefix
            CanonicalAccount      = $Account
            CanonicalSid          = $Sid
            HostContextCommitment = $hostContextCommitment
            EncodedCarrier        = (& $encodeBase64Url $contextRecord)
        }
    }
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

    $installContext = if ([string]::IsNullOrEmpty($InstallBootstrapContextV1)) {
        $selectedPrefix = if ([string]::IsNullOrEmpty($DeclaredPrefix)) {
            & $normalizeWindowsPath ("{0}\Substrate" -f $currentPrincipal.KnownFolderLocalApplicationData)
        } else {
            & $normalizeWindowsPath $DeclaredPrefix
        }
        & $newInstallContext $selectedPrefix $currentPrincipal.CanonicalAccount $currentPrincipal.CanonicalSid
    } else {
        $record = & $decodeBase64Url $InstallBootstrapContextV1
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
        $hostSubstrateRoot = & $decodeBase64Url $parsed['host_substrate_root']
        $boundAccount = & $decodeBase64Url $parsed['principal_account']
        $boundSid = & $decodeBase64Url $parsed['principal_sid']
        $context = & $newInstallContext $selectedPrefix $boundAccount $boundSid
        if (
            $context.HostSubstrateHome -cne $hostSubstrateHome -or
            $context.HostSubstrateRoot -cne $hostSubstrateRoot -or
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

    $canonicalPipePath = '\\.\pipe\' + (Get-PipeNameFromPath -Path $DeclaredPipePath)
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

    if ([string]::IsNullOrEmpty($PlatformBootstrapMappingV1)) {
        throw 'platform bootstrap mapping is unavailable or incoherent'
    }

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

    $guestMachineId = $parsed['guest_machine_id']
    $guestAccount = & $decodeBase64Url $parsed['realized_principal_account']
    $guestUid = $parsed['realized_principal_uid']
    $controlRoot = & $decodeBase64Url $parsed['host_platform_control_root']
    $guestSubstrateHome = & $decodeBase64Url $parsed['realized_substrate_home']
    $decodedDistroName = & $decodeBase64Url $parsed['instance_name']
    $decodedPipePath = & $decodeBase64Url $parsed['transport_host']
    $decodedGuestSocket = & $decodeBase64Url $parsed['transport_guest_socket']
    if ($guestMachineId -notmatch '^[0-9a-f]{32}$' -or -not (& $validPrincipalText $decodedDistroName) -or -not (& $validPrincipalText $guestAccount) -or $guestUid -notmatch '^(0|[1-9][0-9]*)$' -or $decodedDistroName -cne $exactDistroName -or $decodedPipePath -cne $canonicalPipePath -or $decodedGuestSocket -cne '/run/substrate.sock') {
        throw 'invalid platform bootstrap mapping'
    }
    if (-not $guestSubstrateHome.EndsWith('/.substrate')) {
        throw 'invalid platform bootstrap mapping'
    }
    $guestHome = & $normalizeUnixPath ($guestSubstrateHome.Substring(0, $guestSubstrateHome.Length - '/.substrate'.Length))
    $observation = & $observeGuestIdentity
    $guestMachineId = $observation.GuestMachineId
    $guestAccount = $observation.GuestAccount
    $guestUid = $observation.GuestUid
    $guestHome = & $normalizeUnixPath $observation.GuestHome
    $scope = Get-WindowsForwarderScopeV1 -WindowsSid $currentPrincipal.CanonicalSid -DistroName $exactDistroName -GuestMachineId $guestMachineId -PipePath $canonicalPipePath
    $expectedControlRoot = & $normalizeWindowsPath (Join-Path $currentPrincipal.KnownFolderLocalApplicationData ("Substrate\forwarder\{0}" -f $scope))
    $guestSubstrateHome = & $normalizeUnixPath (($guestHome.TrimEnd('/')) + '/.substrate')
    $expectedRecord = @(
        'domain=substrate.platform_bootstrap_mapping'
        'version=1'
        ('host_context_commitment={0}' -f $installContext.HostContextCommitment)
        'platform_kind=wsl'
        ('instance_name={0}' -f (& $encodeBase64Url $exactDistroName))
        ('guest_machine_id={0}' -f $guestMachineId)
        ('host_platform_control_root={0}' -f (& $encodeBase64Url $expectedControlRoot))
        ('realized_substrate_home={0}' -f (& $encodeBase64Url $guestSubstrateHome))
        ('realized_principal_account={0}' -f (& $encodeBase64Url $guestAccount))
        ('realized_principal_uid={0}' -f ([uint32]$guestUid))
        'transport_kind=wsl'
        ('transport_host={0}' -f (& $encodeBase64Url $canonicalPipePath))
        ('transport_guest_socket={0}' -f (& $encodeBase64Url '/run/substrate.sock'))
    ) -join "`n"
    $expectedRecord = "$expectedRecord`n"
    $encodedMapping = & $encodeBase64Url $expectedRecord
    if (
        $decodedDistroName -cne $exactDistroName -or
        $controlRoot -cne $expectedControlRoot -or
        $decodedPipePath -cne $canonicalPipePath -or
        $decodedGuestSocket -cne '/run/substrate.sock' -or
        $encodedMapping -cne $PlatformBootstrapMappingV1
    ) {
        throw 'invalid platform bootstrap mapping'
    }

    [pscustomobject]@{
        InstallPrefix              = $installContext.SelectedHostPrefix
        DistroName                 = $exactDistroName
        GuestMachineId             = $guestMachineId
        GuestAccount               = $guestAccount
        GuestUid                   = [string]([uint32]$guestUid)
        GuestHome                  = $guestHome
        GuestSocket                = '/run/substrate.sock'
        PipePath                   = $canonicalPipePath
        PipeName                   = Get-PipeNameFromPath -Path $canonicalPipePath
        HostPlatformControlRoot    = $expectedControlRoot
        WindowsForwarderScopeV1    = $scope
        ForwarderConfigPath        = & $normalizeWindowsPath (Join-Path $installContext.SelectedHostPrefix 'forwarder\forwarder.toml')
        ForwarderLogDir            = & $normalizeWindowsPath (Join-Path $installContext.SelectedHostPrefix 'forwarder\logs')
        SharedForwarderPidPath     = & $normalizeWindowsPath (Join-Path $expectedControlRoot 'forwarder.pid')
        PlatformBootstrapMappingV1 = $encodedMapping
    }
}

function Assert-PlatformBootstrapMappingV1 {
    param(
        [Parameter(Mandatory = $true)][pscustomobject]$MappingState,
        [string]$DeclaredPipePath,
        [string]$DeclaredDistroName
    )

    if ($MappingState.PipeName -cne (Get-PipeNameFromPath -Path $DeclaredPipePath)) {
        throw 'declared pipe does not match platform bootstrap mapping'
    }
    if ($MappingState.DistroName -ine $DeclaredDistroName) {
        throw 'declared distro does not match platform bootstrap mapping'
    }
}

if (-not ($Path.StartsWith('/'))) { $Path = '/' + $Path }

$mappingState = Resolve-PlatformBootstrapMappingV1 `
    -DeclaredPrefix $InstallPrefix `
    -InstallBootstrapContextV1 $InstallBootstrapContextV1 `
    -PlatformBootstrapMappingV1 $PlatformBootstrapMappingV1 `
    -DeclaredDistroName $DistroName `
    -DeclaredPipePath $PipePath
Assert-PlatformBootstrapMappingV1 -MappingState $mappingState -DeclaredPipePath $PipePath -DeclaredDistroName $DistroName

$pipeName = Get-PipeNameFromPath -Path $mappingState.PipePath
if ($TraceParse) {
    Write-Info ("Normalized pipe: {0}" -f $mappingState.PipePath)
    Write-Info ("Resolved distro: {0}; machine-id: {1}" -f $mappingState.DistroName, $mappingState.GuestMachineId)
    Write-Info ("Guest account: {0}; uid: {1}; home: {2}" -f $mappingState.GuestAccount, $mappingState.GuestUid, $mappingState.GuestHome)
    Write-Info ("Control root: {0}; scope: {1}" -f $mappingState.HostPlatformControlRoot, $mappingState.WindowsForwarderScopeV1)
    Write-Info ("Derived config/log/pid: {0} | {1} | {2}" -f $mappingState.ForwarderConfigPath, $mappingState.ForwarderLogDir, $mappingState.SharedForwarderPidPath)
}
Write-Info ("Connecting to named pipe '{0}' from '{1}'" -f $pipeName, $mappingState.PipePath)

$client = [System.IO.Pipes.NamedPipeClientStream]::new('.', $pipeName, [System.IO.Pipes.PipeDirection]::InOut, [System.IO.Pipes.PipeOptions]::None)
$client.Connect([Math]::Max(1000, $TimeoutSeconds * 1000))
try { $client.ReadTimeout = 500; $client.WriteTimeout = 2000 } catch {}

# Build minimal HTTP request
$ascii = [System.Text.Encoding]::ASCII
$writer = New-Object System.IO.StreamWriter($client, $ascii, 1024, $true)
$writer.NewLine = "`r`n"
$writer.AutoFlush = $true
$req = "{0} {1} HTTP/1.1`r`nHost: localhost`r`nConnection: close`r`n`r`n" -f $Method.ToUpperInvariant(), $Path
$writer.Write($req)
$writer.Flush()

# Read only the status line from the raw stream, within deadline
$deadline = [DateTime]::UtcNow.AddSeconds([Math]::Max(1, $TimeoutSeconds))
$buf = New-Object byte[] 1024
$acc = New-Object System.Collections.Generic.List[byte]
$crlfFound = $false
while ([DateTime]::UtcNow -lt $deadline -and -not $crlfFound) {
    try {
        $n = $client.Read($buf, 0, $buf.Length)
    } catch [System.IO.IOException] {
        continue
    }
    if ($n -le 0) { continue }
    for ($i = 0; $i -lt $n; $i++) { [void]$acc.Add($buf[$i]) }
    for ($j = 1; $j -lt $acc.Count; $j++) {
        if ($acc[$j-1] -eq 13 -and $acc[$j] -eq 10) { $crlfFound = $true; break }
    }
}

if (-not $crlfFound) {
    Write-Fail "No status line received before timeout"
    $client.Dispose(); exit 2
}

# Extract bytes up to CRLF and decode
$k = 1
for (; $k -lt $acc.Count; $k++) { if ($acc[$k-1] -eq 13 -and $acc[$k] -eq 10) { break } }
$statusBytes = $acc[0..($k-2)]
$statusLine = $ascii.GetString($statusBytes)
Write-Output ("Status: {0}" -f $statusLine)

$code = 0
try { $code = [int](([regex]::Match($statusLine, 'HTTP/\d\.\d\s+(\d+)').Groups[1].Value)) } catch { $code = 0 }
$client.Dispose()
if ($ExpectStatus -gt 0 -and $code -ne $ExpectStatus) { exit 3 } else { exit 0 }
