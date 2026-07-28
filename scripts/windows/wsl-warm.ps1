#!/usr/bin/env pwsh
param(
    [string]$DistroName = 'substrate-wsl',
    [string]$ProjectPath = (Resolve-Path '..\\..' | Select-Object -ExpandProperty Path),
    [string]$PipePath = '\\.\pipe\substrate-agent',
    [switch]$WhatIf,
    [string]$InstallPrefix,
    [string]$InstallBootstrapContextV1,
    [string]$PlatformBootstrapMappingV1
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Write-Info($Message) { Write-Host "[INFO] $Message" -ForegroundColor Cyan }
function Write-Warn($Message) { Write-Host "[WARN] $Message" -ForegroundColor Yellow }
function Write-ErrorAndExit($Message, [int]$Code = 1) { Write-Host "[FAIL] $Message" -ForegroundColor Red; exit $Code }

function Convert-ToWslPathFragment {
    param([string]$Path)
    $withoutDrive = ($Path -replace '^[A-Za-z]:\\', '')
    ($withoutDrive -replace '\\', '/').TrimStart('/')
}

function Quote-ForBash {
    param([string]$Value)
    if ($null -eq $Value) { return "''" }
    # Bash single-quote escape sequence: close-quote, insert '"'"', reopen.
    $singleQuoteEscape = "'" + '"' + "'" + '"' + "'"
    return "'" + ($Value -replace "'", $singleQuoteEscape) + "'"
}

function Test-Truthy {
    param([string]$Value)
    if (-not $Value) { return $false }
    $normalized = $Value.Trim().ToLowerInvariant()
    return $normalized -in @('1', 'true', 'yes', 'y', 'on')
}

function Test-GuestExecutablePresent {
    param(
        [string]$DistroName,
        [string]$Path
    )

    $quotedPath = Quote-ForBash $Path
    & wsl -d $DistroName -- bash -lc "test -x ${quotedPath}"
    return ($LASTEXITCODE -eq 0)
}

function Install-GuestWorldBinaries {
    param(
        [string]$DistroName,
        [string]$ProjectPathWsl,
        [bool]$ProjectHasCargo,
        [string]$ProjectPath
    )

    if ($ProjectHasCargo) {
        Write-Info "Building world-service and substrate-gateway (release) inside WSL"
        $projectPathQuoted = Quote-ForBash $ProjectPathWsl
        $buildScript = @"
set -euo pipefail
if [ -f ~/.cargo/env ]; then
  . ~/.cargo/env
fi
cd $projectPathQuoted
cargo build -p world-service -p substrate-gateway --release
sudo install -m755 target/release/world-service /usr/local/bin/substrate-world-service
sudo install -m755 target/release/substrate-gateway /usr/local/bin/substrate-gateway
sudo systemctl restart substrate-world-service.service
"@
        $buildScript = $buildScript -replace "`r", ""
        & wsl -d $DistroName -- bash -lc $buildScript
        if ($LASTEXITCODE -ne 0) {
            Write-ErrorAndExit "Failed to build/install world-service and substrate-gateway inside WSL"
        }
        return
    }

    Write-Info "Installing packaged world-service and substrate-gateway into WSL"
    $agentFragment = Convert-ToWslPathFragment (Join-Path $ProjectPath 'bin\\linux\\world-service')
    $gatewayFragment = Convert-ToWslPathFragment (Join-Path $ProjectPath 'bin\\linux\\substrate-gateway')
    $agentPath = Quote-ForBash "/mnt/c/$agentFragment"
    $gatewayPath = Quote-ForBash "/mnt/c/$gatewayFragment"
    & wsl -d $DistroName -- bash -lc "set -euo pipefail; sudo install -m755 ${agentPath} /usr/local/bin/substrate-world-service; sudo install -m755 ${gatewayPath} /usr/local/bin/substrate-gateway; sudo systemctl restart substrate-world-service.service"
    if ($LASTEXITCODE -ne 0) {
        Write-ErrorAndExit "Failed to install packaged world-service and substrate-gateway"
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
    if ([string]::IsNullOrEmpty($pipeName) -or $pipeName.Length -gt 128 -or $pipeName -notmatch '^[A-Za-z0-9._-]+$') {
        throw 'invalid platform bootstrap mapping'
    }
    if ([string]::IsNullOrEmpty($WindowsSid) -or $WindowsSid -notmatch '^S-[0-9]+(?:-[0-9]+)+$' -or [string]::IsNullOrEmpty($DistroName) -or $DistroName.Contains("`n") -or $DistroName.Contains("`r")) {
        throw 'invalid platform bootstrap mapping'
    }
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
    $exactDistro = @($registeredDistros | Where-Object { $_ -and ($_.Trim()) -and ($_.Trim() -ieq $DeclaredDistroName) })
    if ($exactDistro.Count -eq 0) {
        throw 'registered WSL distro not found'
    }
    if ($exactDistro.Count -gt 1) {
        throw 'registered WSL distro is ambiguous'
    }
    $exactDistroName = [string]$exactDistro[0]

    $observeGuestIdentity = {
        $runningExactDistro = @($runningDistros | Where-Object { $_ -and ($_.Trim() -ieq $exactDistroName) })
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

    if ($MappingState.PipeName -cne (if ($DeclaredPipePath.StartsWith('\\.\pipe\', [System.StringComparison]::Ordinal)) { $DeclaredPipePath.Substring('\\.\pipe\'.Length).ToLowerInvariant() } else { throw 'invalid platform bootstrap mapping' })) {
        throw 'declared pipe does not match platform bootstrap mapping'
    }
    if ($MappingState.DistroName -ine $DeclaredDistroName) {
        throw 'declared distro does not match platform bootstrap mapping'
    }
}

Write-Info "Starting wsl-warm for distro '$DistroName'"

$projectPath = Resolve-Path $ProjectPath | Select-Object -ExpandProperty Path
$mappingState = Resolve-PlatformBootstrapMappingV1 `
    -DeclaredPrefix $InstallPrefix `
    -InstallBootstrapContextV1 $InstallBootstrapContextV1 `
    -PlatformBootstrapMappingV1 $PlatformBootstrapMappingV1 `
    -DeclaredDistroName $DistroName `
    -DeclaredPipePath $PipePath
Assert-PlatformBootstrapMappingV1 -MappingState $mappingState -DeclaredPipePath $PipePath -DeclaredDistroName $DistroName
Write-Info ("Validated pipe {0} for selected prefix {1}" -f $mappingState.PipePath, $mappingState.InstallPrefix)
if ($mappingState.WindowsForwarderScopeV1) {
    Write-Info ("Validated control root {0} and scope {1}" -f $mappingState.HostPlatformControlRoot, $mappingState.WindowsForwarderScopeV1)
}
Write-Info "Project path: $projectPath"
Write-ErrorAndExit "WSL world provisioning is intentionally fail-closed in this slice because the WSL helper path is not aligned with the Linux/macOS placement contract for SUBSTRATE_HOME placement, socket/group ownership, and runtime artifact access. Use Linux host-native provisioning, macOS Lima provisioning, or a CLI-only WSL install with --no-world instead." 4

$projectHasCargo = Test-Path (Join-Path $projectPath 'Cargo.toml')
$packagedWorldAgent = Join-Path $projectPath 'bin\\linux\\world-service'
$packagedGateway = Join-Path $projectPath 'bin\\linux\\substrate-gateway'
$usesBundledArtifacts = -not $projectHasCargo

if (-not $projectHasCargo -and (-not (Test-Path $packagedWorldAgent) -or -not (Test-Path $packagedGateway))) {
    Write-ErrorAndExit "Project path must contain Cargo.toml or packaged bin\\linux\\world-service and bin\\linux\\substrate-gateway artifacts"
}

$cargoCandidates = @()
if ($env:SUBSTRATE_WINDOWS_CARGO_EXE) {
    $cargoCandidates += $env:SUBSTRATE_WINDOWS_CARGO_EXE
}
if ($env:CARGO -and $env:CARGO.Trim().ToLowerInvariant().EndsWith('cargo.exe')) {
    $cargoCandidates += $env:CARGO
}
if ($env:CARGO_HOME) {
    $cargoCandidates += (Join-Path $env:CARGO_HOME 'bin\cargo.exe')
}
if ($env:SUBSTRATE_HOST_USERPROFILE) {
    $cargoCandidates += (Join-Path $env:SUBSTRATE_HOST_USERPROFILE '.cargo\bin\cargo.exe')
}
if ($env:USERPROFILE) {
    $cargoCandidates += (Join-Path $env:USERPROFILE '.cargo\bin\cargo.exe')
}
$cargoCmd = Get-Command cargo -ErrorAction SilentlyContinue
if ($cargoCmd) {
    $cargoCandidates += $cargoCmd.Path
}
$cargoExe = $cargoCandidates | Where-Object { $_ -and (Test-Path $_) } | Select-Object -First 1
if (-not $usesBundledArtifacts -and -not $cargoExe) {
    Write-ErrorAndExit "cargo.exe not found via SUBSTRATE_WINDOWS_CARGO_EXE, SUBSTRATE_HOST_USERPROFILE, USERPROFILE, or PATH. Install Rust on the Windows host."
}
$cargoToolchain = $env:RUST_TOOLCHAIN
if (-not $cargoToolchain -and $env:RUSTUP_TOOLCHAIN) {
    $cargoToolchain = $env:RUSTUP_TOOLCHAIN
}
$rustupExe = $null
$cargoResolvedViaRustup = $false
if ($cargoExe) {
    $candidateRustupExe = Join-Path (Split-Path -Parent $cargoExe) 'rustup.exe'
    if (Test-Path $candidateRustupExe) {
        $rustupExe = $candidateRustupExe
    }
}
if ($cargoToolchain -and $rustupExe) {
    $resolvedCargoExe = (& $rustupExe which cargo --toolchain $cargoToolchain 2>$null | Select-Object -Last 1)
    if ($LASTEXITCODE -eq 0 -and $resolvedCargoExe) {
        $resolvedCargoExe = $resolvedCargoExe.Trim()
        if ($resolvedCargoExe -and (Test-Path $resolvedCargoExe)) {
            $cargoExe = $resolvedCargoExe
            $cargoResolvedViaRustup = $true
        }
    }
}

# Ensure WSL installed
$wslStatus = & wsl --status 2>$null
if ($LASTEXITCODE -ne 0) {
    Write-ErrorAndExit "WSL not available. Run 'wsl --install' first."
}

# Import distro if missing
$distroList = & wsl -l -v | Out-String
$distroListClean = $distroList -replace "`0", ""
if ($distroListClean -notmatch [regex]::Escape($DistroName)) {
    Write-Info "Importing distro '$DistroName'"

    if ($WhatIf) {
        Write-Warn "WhatIf mode enabled - skipping provisioning"
        return
    }

    $baseUrl = 'https://cdimage.ubuntu.com/ubuntu-wsl/noble/daily-live/current'
    $arch = $null
    try {
        $arch = [System.Runtime.InteropServices.RuntimeInformation]::ProcessArchitecture
    } catch {
        $envArch = $env:PROCESSOR_ARCHITECTURE
        if ($envArch) {
            if ($envArch -match 'ARM64') {
                $arch = 'Arm64'
            } else {
                $arch = 'X64'
            }
        }
    }

    if (-not $arch) {
        Write-Warn "Unable to detect architecture via RuntimeInformation; defaulting to x64 WSL image"
        $arch = 'X64'
    }

    if (($arch -is [string] -and $arch -ieq 'Arm64') -or ($arch -is [System.Runtime.InteropServices.Architecture] -and $arch -eq [System.Runtime.InteropServices.Architecture]::Arm64)) {
        $imageName = 'noble-wsl-arm64.wsl'
    } else {
        $imageName = 'noble-wsl-amd64.wsl'
    }

    $imagePath = Join-Path $env:TEMP $imageName
    $shaPath = Join-Path $env:TEMP 'noble-wsl-SHA256SUMS'

    Write-Info "Downloading Ubuntu WSL image manifest"
    Invoke-WebRequest -Uri "$baseUrl/SHA256SUMS" -OutFile $shaPath
    $expectedLine = Get-Content $shaPath | Where-Object { $_ -match "\*$imageName$" }
    if (-not $expectedLine) {
        Write-ErrorAndExit "SHA256SUMS does not list $imageName"
    }
    $expectedHash = (($expectedLine -split ' \*')[0]).Trim()

    Write-Info "Downloading Ubuntu WSL image ($imageName)"
    Invoke-WebRequest -Uri "$baseUrl/$imageName" -OutFile $imagePath

    $fileHash = (Get-FileHash -Path $imagePath -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($fileHash -ne ($expectedHash.ToLowerInvariant())) {
        Remove-Item $imagePath -ErrorAction SilentlyContinue
        Write-ErrorAndExit "Hash mismatch for $imageName (expected $expectedHash, got $fileHash)"
    }

    $installDir = Join-Path $env:LOCALAPPDATA 'substrate\\wsl'
    New-Item -ItemType Directory -Force $installDir | Out-Null
    & wsl --import $DistroName $installDir $imagePath --version 2
    Remove-Item $imagePath -ErrorAction SilentlyContinue
    Remove-Item $shaPath -ErrorAction SilentlyContinue
}

if ($WhatIf) {
    Write-Warn "WhatIf mode enabled - skipping provisioning"
    return
}

# Copy provisioning script and run
$hostProvisionPath = Join-Path $projectPath 'scripts\\wsl\\provision.sh'
if (-not (Test-Path $hostProvisionPath)) {
    Write-ErrorAndExit "Provisioning script not found at $hostProvisionPath"
}

$projectPathFragment = Convert-ToWslPathFragment $projectPath
$projectPathWsl = "/mnt/c/$projectPathFragment"

$guestWorldAgentInstalled = $false
$guestGatewayInstalled = $false
try { $guestWorldAgentInstalled = Test-GuestExecutablePresent -DistroName $DistroName -Path '/usr/local/bin/substrate-world-service' } catch {}
try { $guestGatewayInstalled = Test-GuestExecutablePresent -DistroName $DistroName -Path '/usr/local/bin/substrate-gateway' } catch {}

Write-Info "Preflight agent health check"
$isHealthy = $false
try {
    $status = & wsl -d $DistroName -- bash -lc "curl -s -o /dev/null -w '%{http_code}' http://127.0.0.1:61337/v1/capabilities || true"
    if ($status -eq '200') { $isHealthy = $true }
} catch {}

$forceRebuild = Test-Truthy $env:SUBSTRATE_WSL_WARM_FORCE_REBUILD

if (-not $isHealthy -or $forceRebuild) {
    if ($forceRebuild -and $isHealthy) {
        Write-Warn "SUBSTRATE_WSL_WARM_FORCE_REBUILD enabled; reprovisioning even though agent reports HTTP 200"
    }
    Write-Info "Updating package cache and running provision script"
    $provisionScript = Quote-ForBash "$projectPathWsl/scripts/wsl/provision.sh"
    & wsl -d $DistroName -- bash -lc "set -euo pipefail; cp ${provisionScript} /tmp/provision.sh && sed -i 's/\r$//' /tmp/provision.sh && chmod +x /tmp/provision.sh && sudo /tmp/provision.sh"
    if ($LASTEXITCODE -ne 0) {
        Write-ErrorAndExit "Provision script failed"
    }

    Install-GuestWorldBinaries -DistroName $DistroName -ProjectPathWsl $projectPathWsl -ProjectHasCargo:$projectHasCargo -ProjectPath $projectPath

    # Ensure systemd units are enabled
    Write-Info "Ensuring substrate-world-service service and socket are enabled"
    & wsl -d $DistroName -- bash -lc "sudo systemctl daemon-reload && sudo systemctl enable substrate-world-service.service && sudo systemctl enable --now substrate-world-service.socket && sudo systemctl restart substrate-world-service.service"
    if ($LASTEXITCODE -ne 0) {
        Write-ErrorAndExit "Failed to enable/restart agent units"
    }
} else {
    Write-Info "Agent reports HTTP 200; skipping provision/build/restart"

    # CI safety: even if the agent is reachable, ensure the guest binaries match the checked-out
    # ref so transport and gateway-runtime fixes take effect on self-hosted runners.
    $rebuildGuestBinaries = (-not $guestWorldAgentInstalled) -or (-not $guestGatewayInstalled)
    if ($projectHasCargo -and (Test-Truthy $env:GITHUB_ACTIONS -or Test-Truthy $env:SUBSTRATE_WSL_WARM_FORCE_AGENT_REBUILD -or Test-Truthy $env:SUBSTRATE_WSL_WARM_FORCE_GATEWAY_REBUILD)) {
        $rebuildGuestBinaries = $true
    }
    if ($rebuildGuestBinaries) {
        Write-Info "Refreshing guest world binaries inside WSL"
        Install-GuestWorldBinaries -DistroName $DistroName -ProjectPathWsl $projectPathWsl -ProjectHasCargo:$projectHasCargo -ProjectPath $projectPath
    }
}

# Build forwarder if needed or use packaged binary
if ($projectHasCargo) {
    $forwarderHostPath = Join-Path $projectPath 'target\\release\\substrate-forwarder.exe'
    $forceForwarderRebuild = $forceRebuild -or (Test-Truthy $env:SUBSTRATE_WSL_WARM_FORCE_FORWARDER_REBUILD) -or (Test-Truthy $env:GITHUB_ACTIONS)
    if ($forceForwarderRebuild -or -not (Test-Path $forwarderHostPath)) {
        Write-Info "Building substrate-forwarder (release)"
        Push-Location $projectPath
        try {
            $forwarderBuildArgs = @('build', '-p', 'substrate-forwarder', '--release')
            if ($cargoToolchain -and -not $cargoResolvedViaRustup) {
                $env:RUSTUP_TOOLCHAIN = $cargoToolchain
            }
            & $cargoExe @forwarderBuildArgs
        } finally {
            Pop-Location
        }
        if ($LASTEXITCODE -ne 0) {
            Write-ErrorAndExit "Failed to build substrate-forwarder.exe"
        }
        if (-not (Test-Path $forwarderHostPath)) {
            Write-ErrorAndExit "substrate-forwarder.exe missing after build at $forwarderHostPath"
        }
    }
} else {
    $forwarderHostPath = Join-Path $projectPath 'bin\\substrate-forwarder.exe'
    if (-not (Test-Path $forwarderHostPath)) {
        Write-ErrorAndExit "Packaged substrate-forwarder.exe not found at $forwarderHostPath"
    }
}

# Launch forwarder
Write-Info "Launching forwarder"
$logDir = Join-Path $env:LOCALAPPDATA 'Substrate\\logs'
New-Item -ItemType Directory -Force $logDir | Out-Null
$pipePath = $PipePath
$pidFile = Join-Path $env:LOCALAPPDATA 'Substrate\\forwarder.pid'
if (Test-Path $pidFile) {
    Write-Warn "Forwarder PID file exists; attempting cleanup"
    $existingPid = Get-Content $pidFile
    Stop-Process -Id $existingPid -ErrorAction SilentlyContinue
    Remove-Item $pidFile -ErrorAction SilentlyContinue
}

# Optional host TCP bridge (opt-in via environment)
$tcpBridge = $null
if ($env:SUBSTRATE_FORWARDER_TCP_ADDR) {
    $tcpBridge = $env:SUBSTRATE_FORWARDER_TCP_ADDR
} elseif ($env:SUBSTRATE_FORWARDER_TCP) {
    $flag = $env:SUBSTRATE_FORWARDER_TCP.Trim().ToLower()
    if ($flag -in @('1','true','yes')) {
        $port = 17788
        if ($env:SUBSTRATE_FORWARDER_TCP_PORT) { [void][int]::TryParse($env:SUBSTRATE_FORWARDER_TCP_PORT, [ref]$port) }
        $tcpBridge = "127.0.0.1:$port"
    }
}

# Ensure the forwarder targets the agent TCP listener inside WSL unless explicitly overridden.
# This avoids named-pipe-to-UDS permission and socket-activation edge cases on CI runners.
if (-not $env:SUBSTRATE_FORWARDER_TARGET) {
    $env:SUBSTRATE_FORWARDER_TARGET = 'tcp:61337'
}
if (-not $env:SUBSTRATE_FORWARDER_CONNECT_TIMEOUT_S) {
    $env:SUBSTRATE_FORWARDER_CONNECT_TIMEOUT_S = '2'
}
if (-not $env:SUBSTRATE_FORWARDER_CONNECT_DEADLINE_S) {
    $env:SUBSTRATE_FORWARDER_CONNECT_DEADLINE_S = '10'
}
if (-not $env:SUBSTRATE_FORWARDER_IDLE_AFTER_STDIN_CLOSE_S) {
    $env:SUBSTRATE_FORWARDER_IDLE_AFTER_STDIN_CLOSE_S = '2'
}

$args = @("--distro", $DistroName, "--pipe", $pipePath, "--log-dir", $logDir, "--run-as-service")
if ($tcpBridge) { $args += @("--tcp-bridge", $tcpBridge) }
$forwarderProcess = Start-Process -FilePath $forwarderHostPath -ArgumentList $args -WindowStyle Hidden -PassThru
Set-Content $pidFile -Value $forwarderProcess.Id

# Wait for pipe using an actual client probe with retries
Write-Info "Probing forwarder pipe $pipePath"
$stopwatch = [System.Diagnostics.Stopwatch]::StartNew()
if ($pipePath -match '\\pipe\\(?<n>[^\\]+)$') {
    $pipeName = $Matches['n']
} else {
    $pipeName = $pipePath
}
Write-Info ("Using pipe name '{0}' from '{1}'" -f $pipeName, $pipePath)
Add-Type -TypeDefinition @"
using System;
using System.Runtime.InteropServices;
namespace Native {
  public static class Win32 {
    [DllImport("kernel32.dll", SetLastError=true, CharSet=CharSet.Unicode)]
    public static extern bool WaitNamedPipe(string name, uint timeout);
  }
}
"@

# First wait for the server to create and pend connect
if (-not ([Native.Win32]::WaitNamedPipe($pipePath, 30000))) {
    # Fallback: poll for path existence for up to 30s
    $deadline = [DateTime]::UtcNow.AddSeconds(30)
    while (-not (Test-Path $pipePath)) {
        if ([DateTime]::UtcNow -ge $deadline) {
            $stopwatch.Stop()
            $err = [Runtime.InteropServices.Marshal]::GetLastWin32Error()
            Write-ErrorAndExit ("Forwarder pipe probe failed after {0:N0} ms: WaitNamedPipe error {1}; also path never appeared" -f $stopwatch.Elapsed.TotalMilliseconds, $err)
        }
        Start-Sleep -Milliseconds 250
    }
}

# Then do a quick client connect/close to fully validate
$client = [System.IO.Pipes.NamedPipeClientStream]::new('.', $pipeName, [System.IO.Pipes.PipeDirection]::InOut, [System.IO.Pipes.PipeOptions]::None)
$client.Connect(2000)
$client.Dispose()
$stopwatch.Stop()
Write-Info ("Forwarder pipe accepted probe in {0:N0} ms" -f $stopwatch.Elapsed.TotalMilliseconds)
Write-Info "Forwarder pipe ready"

# Validate agent round-trip through the forwarder (not just pipe reachability).
Write-Info "Probing agent capabilities via forwarder"
$probeTimeoutMs = 10000
if ($tcpBridge) {
    try {
        $uri = "http://$tcpBridge/v1/capabilities"
        $res = Invoke-WebRequest -Uri $uri -UseBasicParsing -TimeoutSec ([Math]::Ceiling($probeTimeoutMs / 1000)) -ErrorAction Stop
        if ($res.StatusCode -ne 200) {
            Write-ErrorAndExit ("Forwarder TCP probe returned HTTP {0} for {1}" -f $res.StatusCode, $uri)
        }
        Write-Info ("Forwarder TCP probe OK: {0}" -f $uri)
    } catch {
        Write-ErrorAndExit ("Forwarder TCP probe failed for http://{0}/v1/capabilities: {1}" -f $tcpBridge, $_.Exception.Message)
    }
} else {
    $probe = [System.IO.Pipes.NamedPipeClientStream]::new('.', $pipeName, [System.IO.Pipes.PipeDirection]::InOut, [System.IO.Pipes.PipeOptions]::None)
    $probe.Connect($probeTimeoutMs)
    $probe.ReadTimeout = $probeTimeoutMs
    $probe.WriteTimeout = $probeTimeoutMs
    $req = "GET /v1/capabilities HTTP/1.1`r`nHost: localhost`r`nConnection: close`r`n`r`n"
    $bytes = [System.Text.Encoding]::ASCII.GetBytes($req)
    $probe.Write($bytes, 0, $bytes.Length)
    $probe.Flush()

    $buf = New-Object byte[] 4096
    $ms = New-Object System.IO.MemoryStream
    try {
        while ($true) {
            $n = $probe.Read($buf, 0, $buf.Length)
            if ($n -le 0) { break }
            $ms.Write($buf, 0, $n) | Out-Null
        }
    } catch {
        $probe.Dispose()
        Write-ErrorAndExit ("Forwarder named-pipe probe timed out or failed while reading response: {0}" -f $_.Exception.Message)
    }
    $probe.Dispose()

    $text = [System.Text.Encoding]::UTF8.GetString($ms.ToArray())
    if ($text -notmatch '^HTTP/1\\.1 200') {
        $firstLine = ([regex]::Split($text, "\r?\n") | Select-Object -First 1)
        Write-ErrorAndExit ("Forwarder named-pipe probe returned non-200 response (first line): {0}" -f $firstLine)
    }
    Write-Info "Forwarder named-pipe probe OK: /v1/capabilities"
}

Write-Info "Warm complete"
