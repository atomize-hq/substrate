#!/usr/bin/env pwsh
param(
    [string]$DistroName = 'substrate-wsl',
    [string]$PipePath = '\\.\pipe\substrate-agent',
    [switch]$Json,
    [string]$InstallPrefix,
    [string]$InstallBootstrapContextV1,
    [string]$PlatformBootstrapMappingV1
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
        if ($normalizedSeparators.Length -lt 4 -or (-not [char]::IsLetter($normalizedSeparators[0])) -or $normalizedSeparators[1] -cne ':' -or $normalizedSeparators[2] -cne '\') {
            throw 'invalid platform bootstrap mapping'
        }
        return ('{0}:\{1}' -f [char]::ToUpperInvariant($normalizedSeparators[0]), ($components -join '\'))
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

    $pipeName = if ($PipePath.StartsWith('\\.\pipe\', [System.StringComparison]::Ordinal)) { $PipePath.Substring('\\.\pipe\'.Length).ToLowerInvariant() } else { throw 'invalid platform bootstrap mapping' }
    $frame = @(
        'domain=substrate.windows_forwarder_scope'
        'version=1'
        ('windows_sid={0}' -f ([Convert]::ToBase64String([System.Text.Encoding]::UTF8.GetBytes($WindowsSid)).TrimEnd('=').Replace('+', '-').Replace('/', '_')))
        ('distro_name={0}' -f ([Convert]::ToBase64String([System.Text.Encoding]::UTF8.GetBytes($DistroName)).TrimEnd('=').Replace('+', '-').Replace('/', '_')))
        "guest_machine_id=$GuestMachineId"
        ('pipe_path={0}' -f ([Convert]::ToBase64String([System.Text.Encoding]::UTF8.GetBytes(('\\.\pipe\' + $pipeName))).TrimEnd('=').Replace('+', '-').Replace('/', '_')))
    ) -join "`n"
    $hashBytes = [System.Security.Cryptography.SHA256]::HashData([System.Text.Encoding]::ASCII.GetBytes("$frame`n"))
    ([System.BitConverter]::ToString($hashBytes).Replace('-', '').ToLowerInvariant())
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
    $pipeName = if ($DeclaredPipePath.StartsWith('\\.\pipe\', [System.StringComparison]::Ordinal)) { $DeclaredPipePath.Substring('\\.\pipe\'.Length).ToLowerInvariant() } else { throw 'invalid platform bootstrap mapping' }
    if ([string]::IsNullOrEmpty($pipeName) -or $pipeName.Length -gt 128 -or $pipeName -notmatch '^[A-Za-z0-9._-]+$') {
        throw 'invalid platform bootstrap mapping'
    }
    $canonicalPipePath = '\\.\pipe\' + $pipeName
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

    $registeredDistros = Get-InstalledWslDistros
    $exactDistro = @($registeredDistros | Where-Object { $_ -and ((Normalize-WSLName $_) -ieq (Normalize-WSLName $DeclaredDistroName)) })
    if ($exactDistro.Count -eq 0) {
        throw 'registered WSL distro not found'
    }
    if ($exactDistro.Count -gt 1) {
        throw 'registered WSL distro is ambiguous'
    }
    $exactDistroName = [string]$exactDistro[0]

    $runningDistros = & $collectRunningDistros $registeredDistros

    $observeGuestIdentity = {
        $runningExactDistro = @($runningDistros | Where-Object { $_ -and ((Normalize-WSLName $_) -ieq (Normalize-WSLName $exactDistroName)) })
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
        PipeName                   = $pipeName
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

    $declaredName = if ($DeclaredPipePath.StartsWith('\\.\pipe\', [System.StringComparison]::Ordinal)) { $DeclaredPipePath.Substring('\\.\pipe\'.Length).ToLowerInvariant() } else { throw 'invalid platform bootstrap mapping' }
    if ($MappingState.PipeName -cne $declaredName) {
        throw 'declared pipe does not match platform bootstrap mapping'
    }
    if ($MappingState.DistroName -ine $DeclaredDistroName) {
        throw 'declared distro does not match platform bootstrap mapping'
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
    return $s
}

function Get-InstalledWslDistros {
    $names = @()
    try {
        $q = & wsl.exe -l -q 2>$null
        if ($q) { $names += $q }
    } catch {}
    $seen = New-Object 'System.Collections.Generic.HashSet[string]' ([System.StringComparer]::Ordinal)
    $result = New-Object System.Collections.Generic.List[string]
    foreach ($name in $names) {
        if ($null -eq $name) {
            continue
        }
        $trimmed = $name.Trim()
        if (-not $trimmed) {
            continue
        }
        if ($seen.Add($trimmed)) {
            [void]$result.Add($trimmed)
        }
    }
    @($result.ToArray())
}

function Test-NamedPipe {
    param(
        [Parameter(Mandatory = $true)][string]$Path,
        [int]$TimeoutMs = 2000
    )
    if (-not $Path.StartsWith('\\.\pipe\', [System.StringComparison]::Ordinal)) {
        throw "Invalid pipe path: $Path"
    }
    $pipeName = $Path.Substring('\\.\pipe\'.Length).ToLowerInvariant()
    if ([string]::IsNullOrEmpty($pipeName) -or $pipeName.Length -gt 128 -or $pipeName -notmatch '^[A-Za-z0-9._-]+$') {
        throw "Invalid pipe path: $Path"
    }
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

    $expectedUds = $script:MappingState.GuestSocket

    if ($ConfigPath -and (Test-Path $ConfigPath)) {
        $content  = Get-Content -Path $ConfigPath -Raw
        $mode     = ([regex]::Match($content, '(?im)^\s*mode\s*=\s*"(?<mode>[^"]+)"')).Groups['mode'].Value.ToLowerInvariant()
        if (-not $mode) { throw 'forwarder target is incoherent' }
        $udsMatch = [regex]::Match($content, '(?im)^\s*uds_path\s*=\s*"(?<path>[^"]+)"')
        $path = if ($udsMatch.Success) { $udsMatch.Groups['path'].Value } else { $expectedUds }
        if ($mode -notin @('uds', 'unix', 'unix_socket') -or $path -cne $expectedUds) {
            throw 'forwarder target conflicts with platform bootstrap mapping'
        }
        return [PSCustomObject]@{ Mode='uds'; Endpoint=$expectedUds; Source="config ($ConfigPath)" }
    }

    throw 'forwarder target is unavailable'
}

# ---------- checks ----------

$script:MappingState = Resolve-PlatformBootstrapMappingV1 `
    -DeclaredPrefix $InstallPrefix `
    -InstallBootstrapContextV1 $InstallBootstrapContextV1 `
    -PlatformBootstrapMappingV1 $PlatformBootstrapMappingV1 `
    -DeclaredDistroName $DistroName `
    -DeclaredPipePath $PipePath
Assert-PlatformBootstrapMappingV1 -MappingState $script:MappingState -DeclaredPipePath $PipePath -DeclaredDistroName $DistroName

$results = @()
$forwarderConfigPath = $script:MappingState.ForwarderConfigPath
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
    $found   = @($distros | Where-Object { $_ -ceq $script:MappingState.DistroName })
    if ($found.Count -ne 1) { throw ("Distro not found or ambiguous (saw: {0})" -f ($distros -join ', ')) }
    "Installed as $($script:MappingState.DistroName)"
} 'Import or create the distro outside this fail-closed slice, then rerun doctor'

# Host C: mounted under /mnt/c
$results += Invoke-Check 'WSL Mount (/mnt/c)' {
    $output = & wsl -d $script:MappingState.DistroName -- bash -lc 'mount | grep "/mnt/c"'
    if ($LASTEXITCODE -ne 0) { throw 'Host C: drive not mounted under /mnt/c' }
    ($output -split "`n" | Select-Object -First 1).Trim()
} 'Ensure /mnt/c is mounted inside the distro (T-012)'

# Forwarder Pipe
$results += Invoke-Check 'Forwarder Pipe' {
    $res = Test-NamedPipe -Path $script:MappingState.PipePath -TimeoutMs 2000
    $script:PipeOk = $true
    $res
} 'Restart the forwarder with scripts/windows/start-forwarder.ps1 and validated mapping inputs'


# Forwarder PID (SKIP if pipe is good but no PID/log yet)
$results += Invoke-Check 'Forwarder PID' {
    $pidFile = $script:MappingState.SharedForwarderPidPath

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
} 'Run scripts/windows/start-forwarder.ps1 with validated mapping inputs to launch the forwarder'

# Forwarder Target
$results += Invoke-Check 'Forwarder Target' {
    try {
        $info = Get-ForwarderTargetInfo -ConfigPath $forwarderConfigPath
        "Mode=$($info.Mode); Endpoint=$($info.Endpoint); Source=$($info.Source); Scope=$($script:MappingState.WindowsForwarderScopeV1)"
    } catch {
        if ($script:PipeOk -and $_.Exception.Message -eq 'forwarder target is unavailable') {
            return @{ Status='SKIP'; Detail='Target evidence unavailable after child-only projection' }
        }
        throw
    }
} 'Set an explicit [target] mode = "uds" in forwarder.toml or relaunch the forwarder with validated mapping inputs'

# Forwarder Log (SKIP if pipe ok but no logs)
$results += Invoke-Check 'Forwarder Log' {
    $logDir = $script:MappingState.ForwarderLogDir
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
} ("Inspect {0} for forwarder errors" -f $script:MappingState.ForwarderLogDir)

# Agent socket
$results += Invoke-Check 'Agent Socket' {
    & wsl -d $script:MappingState.DistroName -- bash -lc 'test -S /run/substrate.sock'
    if ($LASTEXITCODE -ne 0) { throw '/run/substrate.sock missing' }
    '/run/substrate.sock present'
} 'Verify substrate-world-service systemd service is running'

# Agent capabilities
$results += Invoke-Check 'Agent Capabilities' {
    $output = & wsl -d $script:MappingState.DistroName -- bash -lc "curl --unix-socket /run/substrate.sock -s http://localhost/v1/capabilities"
    if ($LASTEXITCODE -ne 0) { throw $output }
    $json = $output | ConvertFrom-Json
    "version=$($json.version) features=$($json.features -join ',')"
} 'Inspect agent logs via journalctl -u substrate-world-service'

# nftables
$results += Invoke-Check 'nftables' {
    $output = & wsl -d $script:MappingState.DistroName -- bash -lc 'nft list tables'
    if ($LASTEXITCODE -ne 0) { throw $output }
    ($output -split "`n" | Select-Object -First 5) -join '; '
} 'Install nftables package inside WSL distro'

# disk root
$results += Invoke-Check 'Disk (/)' {
    $output = & wsl -d $script:MappingState.DistroName -- bash -lc 'df -h /'
    if ($LASTEXITCODE -ne 0) { throw $output }
    ($output -split "`n" | Select-Object -Last 1).Trim()
} 'Free disk space or expand WSL virtual disk'

# agent logs tail
$results += Invoke-Check 'Agent Logs' {
    $output = & wsl -d $script:MappingState.DistroName -- bash -lc 'journalctl -u substrate-world-service -n 20 --no-pager'
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
