#!/usr/bin/env pwsh

param(
    [switch]$W2Only,
    [switch]$W3Only,
    [switch]$W5Only
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Assert-True {
    param(
        [bool]$Condition,
        [string]$Message
    )

    if (-not $Condition) {
        throw $Message
    }
}

function Assert-Equal {
    param(
        $Actual,
        $Expected,
        [string]$Message
    )

    if ($Actual -cne $Expected) {
        throw ("{0}`nExpected: {1}`nActual:   {2}" -f $Message, $Expected, $Actual)
    }
}

function Assert-Match {
    param(
        [string]$Text,
        [string]$Pattern,
        [string]$Message
    )

    if ($Text -notmatch $Pattern) {
        throw $Message
    }
}

function Assert-Throws {
    param(
        [scriptblock]$Operation,
        [string]$Message
    )

    $threw = $false
    try {
        & $Operation
    } catch {
        $threw = $true
    }

    if (-not $threw) {
        throw $Message
    }
}

function Get-RepoRoot {
    return (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
}

function Get-ScriptText {
    param([string]$ScriptName)
    return [System.IO.File]::ReadAllText((Join-Path $PSScriptRoot $ScriptName)).Replace("`r`n", "`n")
}

function Get-FunctionSnippet {
    param(
        [string]$Text,
        [string]$StartMarker,
        [string]$EndMarker
    )

    $start = $Text.IndexOf($StartMarker, [System.StringComparison]::Ordinal)
    if ($start -lt 0) {
        throw "Unable to find marker: $StartMarker"
    }

    $end = if ([string]::IsNullOrEmpty($EndMarker)) {
        $Text.Length
    } else {
        $Text.IndexOf($EndMarker, $start, [System.StringComparison]::Ordinal)
    }
    if ($end -lt 0) {
        throw "Unable to find marker: $EndMarker"
    }

    return $Text.Substring($start, $end - $start).Trim()
}

function Get-TextBetweenMarkers {
    param(
        [string]$Text,
        [string]$StartMarker,
        [string]$EndMarker
    )

    $start = $Text.IndexOf($StartMarker, [System.StringComparison]::Ordinal)
    if ($start -lt 0) {
        throw "Unable to find start marker: $StartMarker"
    }

    $end = $Text.IndexOf($EndMarker, $start, [System.StringComparison]::Ordinal)
    if ($end -lt 0) {
        throw "Unable to find end marker: $EndMarker"
    }

    return $Text.Substring($start, $end - $start)
}

function Get-TailFromMarker {
    param(
        [string]$Text,
        [string]$StartMarker
    )

    $start = $Text.LastIndexOf($StartMarker, [System.StringComparison]::Ordinal)
    if ($start -lt 0) {
        throw "Unable to find start marker: $StartMarker"
    }

    return $Text.Substring($start).TrimEnd()
}

function Normalize-FunctionSnippetForComparison {
    param([string]$Snippet)

    $normalizedLines = foreach ($line in $Snippet.Replace("`r`n", "`n").Split("`n")) {
        ($line -replace ' {2,}=', ' =' -replace '= {2,}', '= ').TrimEnd()
    }

    return ($normalizedLines -join "`n").Trim()
}

function ConvertTo-Base64UrlUtf8 {
    param([string]$Value)
    return [Convert]::ToBase64String([System.Text.Encoding]::UTF8.GetBytes($Value)).TrimEnd('=').Replace('+', '-').Replace('/', '_')
}

function ConvertFrom-CanonicalBase64UrlUtf8 {
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

function Test-ValidWindowsComponent {
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

function Normalize-WindowsInstallBootstrapPathTest {
    param([string]$RawPath)

    if ([string]::IsNullOrEmpty($RawPath) -or $RawPath.IndexOf([char]0) -ge 0) {
        throw 'invalid install bootstrap path'
    }

    $normalizedSeparators = $RawPath.Replace('/', '\')
    $lowercase = $normalizedSeparators.ToLowerInvariant()
    if ($lowercase.StartsWith('\\?\') -or $lowercase.StartsWith('\\.\')) {
        throw 'invalid install bootstrap path'
    }

    if ($normalizedSeparators.StartsWith('\\')) {
        $components = @($normalizedSeparators.Substring(2) -split '\\+' | Where-Object { $_ -ne '' })
        if ($components.Count -lt 3) {
            throw 'invalid install bootstrap path'
        }
        foreach ($component in $components) {
            if (-not (Test-ValidWindowsComponent $component)) {
                throw 'invalid install bootstrap path'
            }
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

    $components = @($normalizedSeparators.Substring(3) -split '\\+' | Where-Object { $_ -ne '' })
    if ($components.Count -eq 0) {
        throw 'invalid install bootstrap path'
    }
    foreach ($component in $components) {
        if (-not (Test-ValidWindowsComponent $component)) {
            throw 'invalid install bootstrap path'
        }
    }

    $drive = [char]::ToUpperInvariant($normalizedSeparators[0])
    return ('{0}:\{1}' -f $drive, ($components -join '\'))
}

function New-WindowsInstallBootstrapContextTest {
    param(
        [string]$SelectedPrefix,
        [string]$Account,
        [string]$Sid
    )

    $selected = Normalize-WindowsInstallBootstrapPathTest $SelectedPrefix
    if ([string]::IsNullOrEmpty($Account) -or [string]::IsNullOrEmpty($Sid)) {
        throw 'invalid install bootstrap principal'
    }

    $selectedEncoded = ConvertTo-Base64UrlUtf8 $selected
    $accountEncoded = ConvertTo-Base64UrlUtf8 $Account
    $sidEncoded = ConvertTo-Base64UrlUtf8 $Sid
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
    $carrier = ConvertTo-Base64UrlUtf8 $record

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

function Get-Sha256Hex {
    param([string]$Value)

    $hashBytes = [System.Security.Cryptography.SHA256]::HashData([System.Text.Encoding]::UTF8.GetBytes($Value))
    return ([System.BitConverter]::ToString($hashBytes).Replace('-', '').ToLowerInvariant())
}

function Resolve-WindowsInstallBootstrapContextTest {
    param(
        [string]$DeclaredPrefix,
        [string]$EncodedCarrier,
        [string]$CurrentAccount,
        [string]$CurrentSid,
        [string]$KnownFolder,
        [hashtable]$InheritedProjection
    )

    if ([string]::IsNullOrEmpty($EncodedCarrier)) {
        $selectedPrefix = if ([string]::IsNullOrEmpty($DeclaredPrefix)) {
            Normalize-WindowsInstallBootstrapPathTest ("{0}\Substrate" -f (Normalize-WindowsInstallBootstrapPathTest $KnownFolder))
        } else {
            Normalize-WindowsInstallBootstrapPathTest $DeclaredPrefix
        }

        return (New-WindowsInstallBootstrapContextTest -SelectedPrefix $selectedPrefix -Account $CurrentAccount -Sid $CurrentSid)
    }

    $record = ConvertFrom-CanonicalBase64UrlUtf8 $EncodedCarrier
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

    $selectedPrefix = ConvertFrom-CanonicalBase64UrlUtf8 $parsed['selected_host_prefix']
    $hostSubstrateHome = ConvertFrom-CanonicalBase64UrlUtf8 $parsed['host_substrate_home']
    $root = ConvertFrom-CanonicalBase64UrlUtf8 $parsed['host_substrate_root']
    $account = ConvertFrom-CanonicalBase64UrlUtf8 $parsed['principal_account']
    $sid = ConvertFrom-CanonicalBase64UrlUtf8 $parsed['principal_sid']
    $context = New-WindowsInstallBootstrapContextTest -SelectedPrefix $selectedPrefix -Account $account -Sid $sid

    if (
        $context.HostSubstrateHome -cne $hostSubstrateHome -or
        $context.HostSubstrateRoot -cne $root -or
        $context.HostContextCommitment -cne $parsed['host_context_commitment'] -or
        $context.EncodedCarrier -cne $EncodedCarrier
    ) {
        throw 'invalid install bootstrap carrier'
    }

    if ($context.CanonicalAccount -cne $CurrentAccount -or $context.CanonicalSid -cne $CurrentSid) {
        throw 'install bootstrap principal does not match the current Windows principal'
    }

    if (-not [string]::IsNullOrEmpty($DeclaredPrefix)) {
        $normalizedDeclaredPrefix = Normalize-WindowsInstallBootstrapPathTest $DeclaredPrefix
        if ($normalizedDeclaredPrefix -cne $context.SelectedHostPrefix) {
            throw 'declared install prefix does not match the install bootstrap carrier'
        }
    }

    $projection = if ($null -ne $InheritedProjection) { $InheritedProjection } else { @{} }
    $expectedProjection = @{
        SUBSTRATE_HOME                            = $context.HostSubstrateHome
        SUBSTRATE_ROOT                            = $context.HostSubstrateRoot
        SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT = $context.HostContextCommitment
        SUBSTRATE_INSTALL_PRIMARY_USER            = $context.CanonicalAccount
        SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1    = $context.EncodedCarrier
    }
    foreach ($entry in $expectedProjection.GetEnumerator()) {
        if ($projection.ContainsKey($entry.Key) -and $projection[$entry.Key] -cne $entry.Value) {
            throw 'install bootstrap environment projection is conflicting'
        }
    }

    return $context
}

function Invoke-InstallContextProjectionHarness {
    param(
        [pscustomobject]$InstallContext,
        [hashtable]$AdditionalEnvironment,
        [scriptblock]$Action
    )

    $additional = if ($null -ne $AdditionalEnvironment) { $AdditionalEnvironment } else { @{} }
    $required = @{
        SUBSTRATE_HOME                            = $InstallContext.HostSubstrateHome
        SUBSTRATE_ROOT                            = $InstallContext.HostSubstrateRoot
        SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT = $InstallContext.HostContextCommitment
        SUBSTRATE_INSTALL_PRIMARY_USER            = $InstallContext.CanonicalAccount
        SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1    = $InstallContext.EncodedCarrier
    }
    $trackedKeys = @($required.Keys + $additional.Keys | Select-Object -Unique)
    $previous = @{}
    foreach ($key in $trackedKeys) {
        $current = Get-Item -Path "Env:$key" -ErrorAction SilentlyContinue
        $previous[$key] = [pscustomobject]@{
            Present = ($null -ne $current)
            Value   = if ($null -ne $current) { $current.Value } else { $null }
        }
    }

    try {
        foreach ($entry in $required.GetEnumerator()) {
            Set-Item -Path "Env:$($entry.Key)" -Value $entry.Value
        }
        foreach ($entry in $additional.GetEnumerator()) {
            Set-Item -Path "Env:$($entry.Key)" -Value ([string]$entry.Value)
        }

        & $Action
    } finally {
        foreach ($key in $trackedKeys) {
            $prior = $previous[$key]
            if ($prior.Present) {
                Set-Item -Path "Env:$key" -Value $prior.Value
            } else {
                Remove-Item -Path "Env:$key" -ErrorAction SilentlyContinue
            }
        }
    }
}

function Test-ValidPrincipalText {
    param([string]$Value)
    return (-not [string]::IsNullOrEmpty($Value)) -and
        ($Value.IndexOf([char]0) -lt 0) -and
        (-not $Value.Contains("`n")) -and
        (-not $Value.Contains("`r"))
}

function Test-ValidWindowsSid {
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

function Normalize-UnixInstallBootstrapPathTest {
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

function Normalize-WindowsPipePathTest {
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

function Parse-CanonicalU32Test {
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

function Normalize-WSLNameComparisonTest {
    param([string]$Value)
    if (-not $Value) { return $Value }
    return $Value
}

function Resolve-WslRegisteredDistroTest {
    param(
        [string]$DeclaredDistroName,
        [string[]]$RegisteredDistros
    )

    $want = Normalize-WSLNameComparisonTest $DeclaredDistroName
    $matches = @(
        $RegisteredDistros |
            Where-Object {
                $_ -and ((Normalize-WSLNameComparisonTest $_) -ieq $want)
            }
    )

    if ($matches.Count -eq 0) {
        throw 'registered WSL distro not found'
    }
    if ($matches.Count -gt 1) {
        throw 'registered WSL distro is ambiguous'
    }

    return [string]$matches[0]
}

function Get-WslVerboseRunningDistroNamesTest {
    param(
        [string[]]$Lines,
        [string[]]$RegisteredDistros
    )

    $names = New-Object System.Collections.Generic.List[string]
    $seen = New-Object 'System.Collections.Generic.HashSet[string]' ([System.StringComparer]::Ordinal)
    $orderedNames = @($RegisteredDistros |
            Where-Object { $_ -and $_.Trim() } |
            Sort-Object -Property @{ Expression = { $_.Length }; Descending = $true }, @{ Expression = { $_ }; Descending = $false })
    for ($index = 1; $index -lt $Lines.Count; $index++) {
        $line = $Lines[$index]
        $normalized = ($line -replace '^\s*\*\s*', '').Trim()
        if (-not $normalized) {
            continue
        }
        foreach ($registeredName in $orderedNames) {
            if (-not $normalized.StartsWith($registeredName, [System.StringComparison]::Ordinal)) {
                continue
            }
            $remainder = $normalized.Substring($registeredName.Length)
            if ($remainder -match '^\s{2,}(?<state>\S+)(?:\s{2,}\S.*)?$' -and $Matches['state'].Equals('Running', [System.StringComparison]::OrdinalIgnoreCase)) {
                if ($seen.Add($registeredName)) {
                    [void]$names.Add($registeredName)
                }
            }
            break
        }
    }
    @($names.ToArray())
}

function Get-WindowsForwarderScopeV1Test {
    param(
        [string]$WindowsSid,
        [string]$DistroName,
        [string]$GuestMachineId,
        [string]$PipePath
    )

    if (-not (Test-ValidWindowsSid $WindowsSid) -or -not (Test-ValidPrincipalText $DistroName) -or $GuestMachineId -notmatch '^[0-9a-f]{32}$') {
        throw 'invalid platform bootstrap mapping'
    }

    $canonicalPipePath = Normalize-WindowsPipePathTest $PipePath
    $frame = @(
        'domain=substrate.windows_forwarder_scope'
        'version=1'
        ('windows_sid={0}' -f (ConvertTo-Base64UrlUtf8 $WindowsSid))
        ('distro_name={0}' -f (ConvertTo-Base64UrlUtf8 $DistroName))
        "guest_machine_id=$GuestMachineId"
        ('pipe_path={0}' -f (ConvertTo-Base64UrlUtf8 $canonicalPipePath))
    ) -join "`n"
    $frame = "$frame`n"
    $digest = Get-Sha256Hex $frame

    return [pscustomobject]@{
        Digest            = $digest
        Frame             = $frame
        CanonicalPipePath = $canonicalPipePath
    }
}

function New-PlatformBootstrapMappingV1Test {
    param(
        [pscustomobject]$InstallContext,
        [string]$DistroName,
        [string]$GuestMachineId,
        [string]$HostPlatformControlRoot,
        [string]$GuestSubstrateHome,
        [string]$GuestAccount,
        [string]$GuestUid,
        [string]$PipePath,
        [string]$GuestSocket
    )

    if ($GuestMachineId -notmatch '^[0-9a-f]{32}$' -or -not (Test-ValidPrincipalText $DistroName) -or -not (Test-ValidPrincipalText $GuestAccount)) {
        throw 'invalid platform bootstrap mapping'
    }

    $canonicalControlRoot = Normalize-WindowsInstallBootstrapPathTest $HostPlatformControlRoot
    $canonicalGuestHome = Normalize-UnixInstallBootstrapPathTest $GuestSubstrateHome
    $canonicalPipePath = Normalize-WindowsPipePathTest $PipePath
    $canonicalGuestSocket = Normalize-UnixInstallBootstrapPathTest $GuestSocket
    $canonicalUid = Parse-CanonicalU32Test $GuestUid

    $frame = @(
        'domain=substrate.platform_bootstrap_mapping'
        'version=1'
        ('host_context_commitment={0}' -f $InstallContext.HostContextCommitment)
        'platform_kind=wsl'
        ('instance_name={0}' -f (ConvertTo-Base64UrlUtf8 $DistroName))
        "guest_machine_id=$GuestMachineId"
        ('host_platform_control_root={0}' -f (ConvertTo-Base64UrlUtf8 $canonicalControlRoot))
        ('realized_substrate_home={0}' -f (ConvertTo-Base64UrlUtf8 $canonicalGuestHome))
        ('realized_principal_account={0}' -f (ConvertTo-Base64UrlUtf8 $GuestAccount))
        ('realized_principal_uid={0}' -f $canonicalUid)
        'transport_kind=wsl'
        ('transport_host={0}' -f (ConvertTo-Base64UrlUtf8 $canonicalPipePath))
        ('transport_guest_socket={0}' -f (ConvertTo-Base64UrlUtf8 $canonicalGuestSocket))
    ) -join "`n"
    $frame = "$frame`n"
    $encoded = ConvertTo-Base64UrlUtf8 $frame

    return [pscustomobject]@{
        HostContextCommitment  = $InstallContext.HostContextCommitment
        DistroName             = $DistroName
        GuestMachineId         = $GuestMachineId
        HostPlatformControlRoot = $canonicalControlRoot
        RealizedSubstrateHome  = $canonicalGuestHome
        GuestAccount           = $GuestAccount
        GuestUid               = [string]$canonicalUid
        PipePath               = $canonicalPipePath
        GuestSocket            = $canonicalGuestSocket
        EncodedMapping         = $encoded
        Frame                  = $frame
    }
}

function ConvertFrom-PlatformBootstrapMappingV1Test {
    param(
        [string]$EncodedMapping,
        [pscustomobject]$InstallContext
    )

    $record = ConvertFrom-CanonicalBase64UrlUtf8 $EncodedMapping
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
        $parsed['host_context_commitment'] -cne $InstallContext.HostContextCommitment -or
        $parsed['platform_kind'] -cne 'wsl' -or
        $parsed['transport_kind'] -cne 'wsl'
    ) {
        throw 'invalid platform bootstrap mapping'
    }

    $mapping = New-PlatformBootstrapMappingV1Test `
        -InstallContext $InstallContext `
        -DistroName (ConvertFrom-CanonicalBase64UrlUtf8 $parsed['instance_name']) `
        -GuestMachineId $parsed['guest_machine_id'] `
        -HostPlatformControlRoot (ConvertFrom-CanonicalBase64UrlUtf8 $parsed['host_platform_control_root']) `
        -GuestSubstrateHome (ConvertFrom-CanonicalBase64UrlUtf8 $parsed['realized_substrate_home']) `
        -GuestAccount (ConvertFrom-CanonicalBase64UrlUtf8 $parsed['realized_principal_account']) `
        -GuestUid $parsed['realized_principal_uid'] `
        -PipePath (ConvertFrom-CanonicalBase64UrlUtf8 $parsed['transport_host']) `
        -GuestSocket (ConvertFrom-CanonicalBase64UrlUtf8 $parsed['transport_guest_socket'])

    if ($mapping.EncodedMapping -cne $EncodedMapping) {
        throw 'invalid platform bootstrap mapping'
    }

    return $mapping
}

function Resolve-PlatformBootstrapMappingV1Test {
    param(
        [string]$DeclaredPrefix,
        [string]$InstallBootstrapContextV1,
        [string]$PlatformBootstrapMappingV1,
        [string]$DeclaredDistroName,
        [string]$DeclaredPipePath,
        [string]$CurrentAccount,
        [string]$CurrentSid,
        [string]$KnownFolder,
        [string[]]$RegisteredDistros,
        [string[]]$RunningDistros,
        [pscustomobject]$GuestObservation,
        [hashtable]$InheritedProjection,
        [switch]$SkipObservationWhenMappingOmitted
    )

    $normalizedKnownFolder = Normalize-WindowsInstallBootstrapPathTest $KnownFolder
    $context = Resolve-WindowsInstallBootstrapContextTest `
        -DeclaredPrefix $DeclaredPrefix `
        -EncodedCarrier $InstallBootstrapContextV1 `
        -CurrentAccount $CurrentAccount `
        -CurrentSid $CurrentSid `
        -KnownFolder $normalizedKnownFolder `
        -InheritedProjection $InheritedProjection

    $canonicalPipePath = Normalize-WindowsPipePathTest $DeclaredPipePath
    $exactDistroName = Resolve-WslRegisteredDistroTest -DeclaredDistroName $DeclaredDistroName -RegisteredDistros $RegisteredDistros

    if ([string]::IsNullOrEmpty($PlatformBootstrapMappingV1)) {
        if ($SkipObservationWhenMappingOmitted.IsPresent) {
            return [pscustomobject]@{
                InstallContext            = $context
                DistroName                = $exactDistroName
                PipePath                  = $canonicalPipePath
                HostPlatformControlRoot   = $null
                WindowsForwarderScopeV1   = $null
                ForwarderConfigPath       = Normalize-WindowsInstallBootstrapPathTest (Join-Path $context.SelectedHostPrefix 'forwarder\forwarder.toml')
                ForwarderLogDir           = Normalize-WindowsInstallBootstrapPathTest (Join-Path $context.SelectedHostPrefix 'forwarder\logs')
                SharedForwarderPidPath    = $null
                PlatformBootstrapMappingV1 = $null
            }
        }

        if ($null -eq $GuestObservation) {
            throw 'platform bootstrap mapping observation unavailable'
        }

        $runningNames = if ($null -eq $RunningDistros) { $RegisteredDistros } else { $RunningDistros }
        $runningMatch = @($runningNames | Where-Object { $_ -and ((Normalize-WSLNameComparisonTest $_) -ieq (Normalize-WSLNameComparisonTest $exactDistroName)) })
        if ($runningMatch.Count -ne 1 -or [string]$runningMatch[0] -cne $exactDistroName) {
            throw 'registered WSL distro is not already running'
        }

        $machineId = [string]$GuestObservation.MachineId
        $account = [string]$GuestObservation.Account
        $uid = [string]$GuestObservation.Uid
        $nameEntry = [string]$GuestObservation.NameEntry
        $uidEntry = [string]$GuestObservation.UidEntry
        if ($machineId -notmatch '^[0-9a-f]{32}$' -or -not (Test-ValidPrincipalText $account) -or $uid -notmatch '^(0|[1-9][0-9]*)$' -or [string]::IsNullOrEmpty($nameEntry) -or $nameEntry -cne $uidEntry) {
            throw 'guest identity observation is invalid'
        }

        $fields = $nameEntry.Split(':')
        if ($fields.Length -lt 6 -or $fields[0] -cne $account -or $fields[2] -cne $uid -or [string]::IsNullOrEmpty($fields[5])) {
            throw 'guest identity observation is invalid'
        }

        $guestHome = Normalize-UnixInstallBootstrapPathTest $fields[5]
        if ($GuestObservation.PSObject.Properties.Name -contains 'Home') {
            $declaredGuestHome = Normalize-UnixInstallBootstrapPathTest ([string]$GuestObservation.Home)
            if ($declaredGuestHome -cne $guestHome) {
                throw 'guest identity observation is invalid'
            }
        }

        $guestSubstrateHome = Normalize-UnixInstallBootstrapPathTest (($guestHome.TrimEnd('/')) + '/.substrate')
        $scope = Get-WindowsForwarderScopeV1Test -WindowsSid $CurrentSid -DistroName $exactDistroName -GuestMachineId $machineId -PipePath $canonicalPipePath
        $controlRoot = Normalize-WindowsInstallBootstrapPathTest (Join-Path $normalizedKnownFolder ("Substrate\forwarder\{0}" -f $scope.Digest))
        $mapping = New-PlatformBootstrapMappingV1Test `
            -InstallContext $context `
            -DistroName $exactDistroName `
            -GuestMachineId $machineId `
            -HostPlatformControlRoot $controlRoot `
            -GuestSubstrateHome $guestSubstrateHome `
            -GuestAccount $account `
            -GuestUid $uid `
            -PipePath $canonicalPipePath `
            -GuestSocket '/run/substrate.sock'
    } else {
        $decoded = ConvertFrom-PlatformBootstrapMappingV1Test -EncodedMapping $PlatformBootstrapMappingV1 -InstallContext $context
        Assert-Equal $decoded.DistroName $exactDistroName 'mapping lost exact registered distro spelling'
        Assert-Equal $decoded.PipePath $canonicalPipePath 'mapping pipe drifted from declared selector'
        Assert-Equal $decoded.GuestSocket '/run/substrate.sock' 'mapping guest socket drifted from fixed contract'

        if ($null -eq $GuestObservation) {
            throw 'guest identity observation unavailable'
        }

        $runningNames = if ($null -eq $RunningDistros) { $RegisteredDistros } else { $RunningDistros }
        $runningMatch = @($runningNames | Where-Object { $_ -and ((Normalize-WSLNameComparisonTest $_) -ieq (Normalize-WSLNameComparisonTest $exactDistroName)) })
        if ($runningMatch.Count -ne 1 -or [string]$runningMatch[0] -cne $exactDistroName) {
            throw 'registered WSL distro is not already running'
        }

        $machineId = [string]$GuestObservation.MachineId
        $account = [string]$GuestObservation.Account
        $uid = [string]$GuestObservation.Uid
        $nameEntry = [string]$GuestObservation.NameEntry
        $uidEntry = [string]$GuestObservation.UidEntry
        if ($machineId -notmatch '^[0-9a-f]{32}$' -or -not (Test-ValidPrincipalText $account) -or $uid -notmatch '^(0|[1-9][0-9]*)$' -or [string]::IsNullOrEmpty($nameEntry) -or $nameEntry -cne $uidEntry) {
            throw 'guest identity observation is invalid'
        }

        $fields = $nameEntry.Split(':')
        if ($fields.Length -lt 6 -or $fields[0] -cne $account -or $fields[2] -cne $uid -or [string]::IsNullOrEmpty($fields[5])) {
            throw 'guest identity observation is invalid'
        }

        $guestHome = Normalize-UnixInstallBootstrapPathTest $fields[5]
        if ($GuestObservation.PSObject.Properties.Name -contains 'Home') {
            $declaredGuestHome = Normalize-UnixInstallBootstrapPathTest ([string]$GuestObservation.Home)
            if ($declaredGuestHome -cne $guestHome) {
                throw 'guest identity observation is invalid'
            }
        }

        $guestSubstrateHome = Normalize-UnixInstallBootstrapPathTest (($guestHome.TrimEnd('/')) + '/.substrate')
        $scope = Get-WindowsForwarderScopeV1Test -WindowsSid $CurrentSid -DistroName $exactDistroName -GuestMachineId $machineId -PipePath $canonicalPipePath
        $expectedControlRoot = Normalize-WindowsInstallBootstrapPathTest (Join-Path $normalizedKnownFolder ("Substrate\forwarder\{0}" -f $scope.Digest))
        $mapping = New-PlatformBootstrapMappingV1Test `
            -InstallContext $context `
            -DistroName $exactDistroName `
            -GuestMachineId $machineId `
            -HostPlatformControlRoot $expectedControlRoot `
            -GuestSubstrateHome $guestSubstrateHome `
            -GuestAccount $account `
            -GuestUid $uid `
            -PipePath $canonicalPipePath `
            -GuestSocket '/run/substrate.sock'
        Assert-Equal $mapping.EncodedMapping $PlatformBootstrapMappingV1 'mapping live revalidation drifted from observed guest state'
    }

    $configPath = Normalize-WindowsInstallBootstrapPathTest (Join-Path $context.SelectedHostPrefix 'forwarder\forwarder.toml')
    $logDir = Normalize-WindowsInstallBootstrapPathTest (Join-Path $context.SelectedHostPrefix 'forwarder\logs')
    $pidPath = Normalize-WindowsInstallBootstrapPathTest (Join-Path $mapping.HostPlatformControlRoot 'forwarder.pid')

    return [pscustomobject]@{
        InstallContext             = $context
        DistroName                 = $mapping.DistroName
        GuestMachineId             = $mapping.GuestMachineId
        GuestAccount               = $mapping.GuestAccount
        GuestUid                   = $mapping.GuestUid
        RealizedSubstrateHome      = $mapping.RealizedSubstrateHome
        PipePath                   = $mapping.PipePath
        GuestSocket                = $mapping.GuestSocket
        HostPlatformControlRoot    = $mapping.HostPlatformControlRoot
        WindowsForwarderScopeV1    = $scope.Digest
        ForwarderConfigPath        = $configPath
        ForwarderLogDir            = $logDir
        SharedForwarderPidPath     = $pidPath
        PlatformBootstrapMappingV1 = $mapping.EncodedMapping
        PlatformBootstrapFrame     = $mapping.Frame
        WindowsForwarderScopeFrame = $scope.Frame
    }
}

function Get-ForwarderTargetInfoTest {
    param(
        [pscustomobject]$MappingState,
        [string]$ConfigText
    )

    if (-not [string]::IsNullOrEmpty($ConfigText)) {
        $mode = ([regex]::Match($ConfigText, '(?im)^\s*mode\s*=\s*"(?<mode>[^"]+)"')).Groups['mode'].Value.ToLowerInvariant()
        if (-not $mode) { throw 'forwarder target is incoherent' }
        $udsMatch = [regex]::Match($ConfigText, '(?im)^\s*uds_path\s*=\s*"(?<path>[^"]+)"')
        $udsPath = if ($udsMatch.Success) { $udsMatch.Groups['path'].Value } else { $MappingState.GuestSocket }
        if ($mode -notin @('uds', 'unix', 'unix_socket') -or $udsPath -cne $MappingState.GuestSocket) {
            throw 'forwarder target conflicts with platform bootstrap mapping'
        }
        return [pscustomobject]@{
            Mode      = 'uds'
            Endpoint  = $MappingState.GuestSocket
            Source    = 'config (match-only)'
            MatchOnly = $true
        }
    }

    throw 'forwarder target is unavailable'
}

function Invoke-ForwarderTargetCheckTest {
    param(
        [pscustomobject]$MappingState,
        [string]$ConfigText,
        [string]$EnvironmentOverride,
        [switch]$PipeOk
    )

    $runCheck = {
        try {
            $info = Get-ForwarderTargetInfoTest -MappingState $MappingState -ConfigText $ConfigText
            [pscustomobject]@{
                Status = 'PASS'
                Detail = "Mode=$($info.Mode); Endpoint=$($info.Endpoint); Source=$($info.Source); Scope=$($MappingState.WindowsForwarderScopeV1)"
            }
        } catch {
            if ($PipeOk.IsPresent -and $_.Exception.Message -eq 'forwarder target is unavailable') {
                [pscustomobject]@{
                    Status = 'SKIP'
                    Detail = 'Target evidence unavailable after child-only projection'
                }
            } else {
                throw
            }
        }
    }

    if ($PSBoundParameters.ContainsKey('EnvironmentOverride')) {
        Invoke-EnvironmentOverrideHarness -Overrides @{ SUBSTRATE_FORWARDER_TARGET = $EnvironmentOverride } -Action $runCheck
    } else {
        & $runCheck
    }
}

function Get-ForwarderLaunchProjectionTest {
    param(
        [pscustomobject]$MappingState,
        [string]$RustLog,
        [string]$TcpBridge,
        [string[]]$AdditionalArgs = @()
    )

    $args = New-Object System.Collections.Generic.List[string]
    foreach ($arg in @(
            '--distro',
            $MappingState.DistroName,
            '--pipe',
            $MappingState.PipePath,
            '--install-bootstrap-context-v1',
            $MappingState.InstallContext.EncodedCarrier,
            '--platform-bootstrap-mapping-v1',
            $MappingState.PlatformBootstrapMappingV1,
            '--config',
            $MappingState.ForwarderConfigPath,
            '--log-dir',
            $MappingState.ForwarderLogDir
        )) {
        [void]$args.Add($arg)
    }
    if (-not [string]::IsNullOrEmpty($TcpBridge)) {
        try {
            $tcpBridgeUri = [System.Uri]("tcp://$TcpBridge")
        } catch {
            throw 'TcpBridge must be a loopback tcp endpoint'
        }
        if (
            (-not $tcpBridgeUri.IsAbsoluteUri) -or
            (-not $tcpBridgeUri.IsLoopback) -or
            ($tcpBridgeUri.Port -lt 1) -or
            ($tcpBridgeUri.Port -gt 65535)
        ) {
            throw 'TcpBridge must be a loopback tcp endpoint'
        }
        [void]$args.Add('--tcp-bridge')
        [void]$args.Add($TcpBridge)
    }
    if ($AdditionalArgs.Length -gt 0) {
        $reservedForwarderArgs = @(
            '--distro',
            '--pipe',
            '--install-bootstrap-context-v1',
            '--platform-bootstrap-mapping-v1',
            '--config',
            '--log-dir',
            '--tcp-bridge'
        )
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
        foreach ($arg in $AdditionalArgs) {
            [void]$args.Add($arg)
        }
    }

    return [pscustomobject]@{
        Arguments = @($args.ToArray())
        Environment = @{
            RUST_LOG                                 = $RustLog
            SUBSTRATE_HOME                           = $MappingState.InstallContext.HostSubstrateHome
            SUBSTRATE_ROOT                           = $MappingState.InstallContext.HostSubstrateRoot
            SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT = $MappingState.InstallContext.HostContextCommitment
            SUBSTRATE_INSTALL_PRIMARY_USER           = $MappingState.InstallContext.CanonicalAccount
            SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1   = $MappingState.InstallContext.EncodedCarrier
            SUBSTRATE_FORWARDER_TARGET               = ('uds:{0}' -f $MappingState.GuestSocket)
            LOCALAPPDATA                             = $null
            USERPROFILE                              = $null
            WSLENV                                   = $null
            SUBSTRATE_FORWARDER_TARGET_MODE          = $null
            SUBSTRATE_FORWARDER_TARGET_HOST          = $null
            SUBSTRATE_FORWARDER_TARGET_PORT          = $null
            SUBSTRATE_FORWARDER_TARGET_ENDPOINT      = $null
            SUBSTRATE_FORWARDER_PIPE                 = $null
            SUBSTRATE_FORWARDER_TCP                  = $null
            SUBSTRATE_FORWARDER_TCP_ADDR             = $null
            SUBSTRATE_FORWARDER_TCP_HOST             = $null
            SUBSTRATE_FORWARDER_TCP_PORT             = $null
        }
    }
}

function Invoke-EnvironmentOverrideHarness {
    param(
        [hashtable]$Overrides,
        [scriptblock]$Action
    )

    $tracked = @($Overrides.Keys | Sort-Object -Unique)
    $previous = @{}
    foreach ($key in $tracked) {
        $current = Get-Item -Path "Env:$key" -ErrorAction SilentlyContinue
        $previous[$key] = [pscustomobject]@{
            Present = ($null -ne $current)
            Value   = if ($null -ne $current) { $current.Value } else { $null }
        }
    }

    try {
        foreach ($key in $tracked) {
            if ($null -eq $Overrides[$key]) {
                Remove-Item -Path "Env:$key" -ErrorAction SilentlyContinue
            } else {
                Set-Item -Path "Env:$key" -Value ([string]$Overrides[$key])
            }
        }
        & $Action
    } finally {
        foreach ($key in $tracked) {
            $state = $previous[$key]
            if ($state.Present) {
                Set-Item -Path "Env:$key" -Value $state.Value
            } else {
                Remove-Item -Path "Env:$key" -ErrorAction SilentlyContinue
            }
        }
    }
}

function Get-W2SampleMappingState {
    $guestObservation = Get-W2SampleGuestObservation
    $context = New-WindowsInstallBootstrapContextTest `
        -SelectedPrefix 'C:\Users\Alice\AppData\Local\Substrate' `
        -Account 'ACME\Alice' `
        -Sid 'S-1-5-21-1000'

    return Resolve-PlatformBootstrapMappingV1Test `
        -InstallBootstrapContextV1 $context.EncodedCarrier `
        -DeclaredDistroName 'substrate-wsl' `
        -DeclaredPipePath '\\.\pipe\Substrate-Agent' `
        -CurrentAccount 'ACME\Alice' `
        -CurrentSid 'S-1-5-21-1000' `
        -KnownFolder 'C:\Users\Alice\AppData\Local' `
        -RegisteredDistros @('Substrate-WSL') `
        -RunningDistros @('Substrate-WSL') `
        -GuestObservation $guestObservation
}

function Get-W2SampleGuestObservation {
    [pscustomobject]@{
        MachineId = 'abcdef0123456789abcdef0123456789'
        Account   = 'bob'
        Uid       = '1001'
        Home      = '/home/bob'
        NameEntry = 'bob:x:1001:1001:Bob:/home/bob:/bin/bash'
        UidEntry  = 'bob:x:1001:1001:Bob:/home/bob:/bin/bash'
    }
}

function Invoke-W2WarmEntryFlowModelTest {
    param(
        [scriptblock]$ResolveMapping,
        [scriptblock]$AssertMapping,
        [scriptblock]$Guard,
        [scriptblock]$Provision
    )

    $trace = New-Object System.Collections.Generic.List[string]
    try {
        [void]$trace.Add('start')
        $mappingState = & $ResolveMapping
        [void]$trace.Add('resolved')
        & $AssertMapping $mappingState
        [void]$trace.Add('asserted')
        [void]$trace.Add('validated-pipe')
        if ($mappingState.WindowsForwarderScopeV1) {
            [void]$trace.Add('validated-scope')
        }
        [void]$trace.Add('project-path')
        & $Guard
        [void]$trace.Add('guard-returned')
        & $Provision
        [void]$trace.Add('provisioned')
        return [pscustomobject]@{
            Trace = @($trace.ToArray())
            Error = $null
        }
    } catch {
        return [pscustomobject]@{
            Trace = @($trace.ToArray())
            Error = $_.Exception.Message
        }
    }
}

function Test-GoldenVector {
    $context = New-WindowsInstallBootstrapContextTest `
        -SelectedPrefix 'C:\Users\Alice\AppData\Local\Substrate' `
        -Account 'ACME\Alice' `
        -Sid 'S-1-5-21-1000'

    Assert-Equal $context.HostContextCommitment '3e1e71b325e92b16f5bfc0f3d875fd15f04a1615ac90b1439eb37afdf90a5ac7' 'golden commitment drifted'
    Assert-Equal $context.EncodedCarrier 'ZG9tYWluPXN1YnN0cmF0ZS5pbnN0YWxsX2Jvb3RzdHJhcF9jb250ZXh0CnZlcnNpb249MQpzZWxlY3RlZF9ob3N0X3ByZWZpeD1RenBjVlhObGNuTmNRV3hwWTJWY1FYQndSR0YwWVZ4TWIyTmhiRnhUZFdKemRISmhkR1UKaG9zdF9zdWJzdHJhdGVfaG9tZT1RenBjVlhObGNuTmNRV3hwWTJWY1FYQndSR0YwWVZ4TWIyTmhiRnhUZFdKemRISmhkR1UKaG9zdF9zdWJzdHJhdGVfcm9vdD1RenBjVlhObGNuTmNRV3hwWTJWY1FYQndSR0YwWVZ4TWIyTmhiRnhUZFdKemRISmhkR1UKcHJpbmNpcGFsX2tpbmQ9d2luZG93cwpwcmluY2lwYWxfYWNjb3VudD1RVU5OUlZ4QmJHbGpaUQpwcmluY2lwYWxfc2lkPVV5MHhMVFV0TWpFdE1UQXdNQQpob3N0X2NvbnRleHRfY29tbWl0bWVudD0zZTFlNzFiMzI1ZTkyYjE2ZjViZmMwZjNkODc1ZmQxNWYwNGExNjE1YWM5MGIxNDM5ZWIzN2FmZGY5MGE1YWM3Cg' 'golden carrier drifted'
}

function Test-PathNormalization {
    Assert-Equal (Normalize-WindowsInstallBootstrapPathTest 'c:/users/alice/appdata/local/substrate') 'C:\users\alice\appdata\local\substrate' 'drive path normalization drifted'
    Assert-Equal (Normalize-WindowsInstallBootstrapPathTest '\\server\share\Substrate') '\\server\share\Substrate' 'UNC normalization drifted'

    foreach ($invalid in @(
            'relative\path',
            'C:\',
            '\\?\C:\Users\Alice\AppData\Local\Substrate',
            '\\.\C:\Users\Alice\AppData\Local\Substrate',
            'C:\Users\Alice\AppData\Local\Substrate:ads',
            'C:\Users\Alice\AppData\Local\.\Substrate',
            'C:\Users\Alice\AppData\Local\..\Substrate',
            ([string]::Concat('C:\Users\Alice\AppData\Local\Substrate', [char]0xD800)),
            'C:\Users\Alice\AppData\Local\NUL',
            "C:\Users\Alice\AppData\Local`n\Substrate",
            'C:\Users\Alice\AppData\Local\trailingspace ',
            'C:\Users\Alice\AppData\Local\trailingdot.',
            '\\server\share',
            '\\server'
        )) {
        Assert-Throws { Normalize-WindowsInstallBootstrapPathTest $invalid } "expected invalid path rejection for $invalid"
    }
}

function Test-PublicConstructionAndAmbientNonAuthority {
    $knownFolder = 'C:\Users\Alice\AppData\Local'
    $publicDefault = Resolve-WindowsInstallBootstrapContextTest `
        -CurrentAccount 'ACME\Alice' `
        -CurrentSid 'S-1-5-21-1000' `
        -KnownFolder $knownFolder `
        -InheritedProjection @{
            SUBSTRATE_HOME = 'D:\Ambient'
            SUBSTRATE_ROOT = 'D:\Ambient'
            SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT = '0' * 64
            SUBSTRATE_INSTALL_PRIMARY_USER = 'OTHER\User'
            SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1 = 'ambient'
        }
    Assert-Equal $publicDefault.SelectedHostPrefix 'C:\Users\Alice\AppData\Local\Substrate' 'default known-folder prefix drifted'

    $publicDeclared = Resolve-WindowsInstallBootstrapContextTest `
        -DeclaredPrefix 'D:\Tools\Substrate' `
        -CurrentAccount 'ACME\Alice' `
        -CurrentSid 'S-1-5-21-1000' `
        -KnownFolder $knownFolder `
        -InheritedProjection @{
            SUBSTRATE_HOME = 'E:\Ambient'
            SUBSTRATE_ROOT = 'E:\Ambient'
        }
    Assert-Equal $publicDeclared.SelectedHostPrefix 'D:\Tools\Substrate' 'declared prefix lost authority'

    $repeat = Resolve-WindowsInstallBootstrapContextTest `
        -DeclaredPrefix 'D:\Tools\Substrate' `
        -CurrentAccount 'ACME\Alice' `
        -CurrentSid 'S-1-5-21-1000' `
        -KnownFolder $knownFolder
    Assert-Equal $publicDeclared.EncodedCarrier $repeat.EncodedCarrier 'repeat public construction drifted'
}

function Test-CarrierRejection {
    $context = New-WindowsInstallBootstrapContextTest `
        -SelectedPrefix 'C:\Users\Alice\AppData\Local\Substrate' `
        -Account 'ACME\Alice' `
        -Sid 'S-1-5-21-1000'

    $projection = @{
        SUBSTRATE_HOME                            = $context.HostSubstrateHome
        SUBSTRATE_ROOT                            = $context.HostSubstrateRoot
        SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT = $context.HostContextCommitment
        SUBSTRATE_INSTALL_PRIMARY_USER            = $context.CanonicalAccount
        SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1    = $context.EncodedCarrier
    }

    $bound = Resolve-WindowsInstallBootstrapContextTest `
        -EncodedCarrier $context.EncodedCarrier `
        -CurrentAccount 'ACME\Alice' `
        -CurrentSid 'S-1-5-21-1000' `
        -KnownFolder 'C:\Users\Alice\AppData\Local' `
        -InheritedProjection $projection
    Assert-Equal $bound.EncodedCarrier $context.EncodedCarrier 'bound carrier changed unexpectedly'

    $tamperedRecord = (ConvertFrom-CanonicalBase64UrlUtf8 $context.EncodedCarrier).Replace('principal_kind=windows', 'principal_kind=linux')
    $tamperedCarrier = ConvertTo-Base64UrlUtf8 $tamperedRecord
    Assert-Throws {
        Resolve-WindowsInstallBootstrapContextTest `
            -EncodedCarrier $tamperedCarrier `
            -CurrentAccount 'ACME\Alice' `
            -CurrentSid 'S-1-5-21-1000' `
            -KnownFolder 'C:\Users\Alice\AppData\Local'
    } 'tampered carrier should reject'

    $duplicateRecord = "domain=substrate.install_bootstrap_context`nversion=1`ndomain=substrate.install_bootstrap_context`n"
    $duplicateCarrier = ConvertTo-Base64UrlUtf8 $duplicateRecord
    Assert-Throws {
        Resolve-WindowsInstallBootstrapContextTest `
            -EncodedCarrier $duplicateCarrier `
            -CurrentAccount 'ACME\Alice' `
            -CurrentSid 'S-1-5-21-1000' `
            -KnownFolder 'C:\Users\Alice\AppData\Local'
    } 'duplicate-key carrier should reject'

    $reorderedRecord = (ConvertFrom-CanonicalBase64UrlUtf8 $context.EncodedCarrier) -replace "^domain=substrate\.install_bootstrap_context`nversion=1`n", "version=1`ndomain=substrate.install_bootstrap_context`n"
    $reorderedCarrier = ConvertTo-Base64UrlUtf8 $reorderedRecord
    Assert-Throws {
        Resolve-WindowsInstallBootstrapContextTest `
            -EncodedCarrier $reorderedCarrier `
            -CurrentAccount 'ACME\Alice' `
            -CurrentSid 'S-1-5-21-1000' `
            -KnownFolder 'C:\Users\Alice\AppData\Local'
    } 'reordered carrier should reject'

    Assert-Throws {
        Resolve-WindowsInstallBootstrapContextTest `
            -EncodedCarrier ($context.EncodedCarrier + '=') `
            -CurrentAccount 'ACME\Alice' `
            -CurrentSid 'S-1-5-21-1000' `
            -KnownFolder 'C:\Users\Alice\AppData\Local'
    } 'noncanonical base64url should reject'

    Assert-Throws {
        Resolve-WindowsInstallBootstrapContextTest `
            -DeclaredPrefix 'D:\Elsewhere\Substrate' `
            -EncodedCarrier $context.EncodedCarrier `
            -CurrentAccount 'ACME\Alice' `
            -CurrentSid 'S-1-5-21-1000' `
            -KnownFolder 'C:\Users\Alice\AppData\Local'
    } 'wrong declared prefix should reject'

    Assert-Throws {
        Resolve-WindowsInstallBootstrapContextTest `
            -EncodedCarrier $context.EncodedCarrier `
            -CurrentAccount 'ACME\Bob' `
            -CurrentSid 'S-1-5-21-1000' `
            -KnownFolder 'C:\Users\Alice\AppData\Local'
    } 'forged account should reject'

    Assert-Throws {
        Resolve-WindowsInstallBootstrapContextTest `
            -EncodedCarrier $context.EncodedCarrier `
            -CurrentAccount 'ACME\Alice' `
            -CurrentSid 'S-1-5-21-9999' `
            -KnownFolder 'C:\Users\Alice\AppData\Local'
    } 'forged sid should reject'

    Assert-Throws {
        Resolve-WindowsInstallBootstrapContextTest `
            -EncodedCarrier $context.EncodedCarrier `
            -CurrentAccount 'ACME\Alice' `
            -CurrentSid 'S-1-5-21-1000' `
            -KnownFolder 'C:\Users\Alice\AppData\Local' `
            -InheritedProjection @{ SUBSTRATE_ROOT = 'D:\Ambient' }
    } 'conflicting inherited projection should reject'
}

function Test-ProjectionHarnessRestoresEnvironment {
    $context = New-WindowsInstallBootstrapContextTest `
        -SelectedPrefix 'C:\Users\Alice\AppData\Local\Substrate' `
        -Account 'ACME\Alice' `
        -Sid 'S-1-5-21-1000'

    $script:snapshot = $null
    $priorPath = [Environment]::GetEnvironmentVariable('PATH')
    $priorMarker = [Environment]::GetEnvironmentVariable('SUBSTRATE_INSTALL_PRIMARY_USER')
    $priorTemp = [Environment]::GetEnvironmentVariable('R2_3W1_TEST_TEMP')
    try {
        [Environment]::SetEnvironmentVariable('SUBSTRATE_INSTALL_PRIMARY_USER', 'AMBIENT\User')
        [Environment]::SetEnvironmentVariable('R2_3W1_TEST_TEMP', $null)
        Invoke-InstallContextProjectionHarness `
            -InstallContext $context `
            -AdditionalEnvironment @{ R2_3W1_TEST_TEMP = 'set-inside-child' } `
            -Action {
                $script:snapshot = [pscustomobject]@{
                    SUBSTRATE_HOME                            = [Environment]::GetEnvironmentVariable('SUBSTRATE_HOME')
                    SUBSTRATE_ROOT                            = [Environment]::GetEnvironmentVariable('SUBSTRATE_ROOT')
                    SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT = [Environment]::GetEnvironmentVariable('SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT')
                    SUBSTRATE_INSTALL_PRIMARY_USER            = [Environment]::GetEnvironmentVariable('SUBSTRATE_INSTALL_PRIMARY_USER')
                    SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1    = [Environment]::GetEnvironmentVariable('SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1')
                    R2_3W1_TEST_TEMP                          = [Environment]::GetEnvironmentVariable('R2_3W1_TEST_TEMP')
                }
            }

        Assert-Equal $script:snapshot.SUBSTRATE_HOME $context.HostSubstrateHome 'projection harness lost SUBSTRATE_HOME'
        Assert-Equal $script:snapshot.SUBSTRATE_ROOT $context.HostSubstrateRoot 'projection harness lost SUBSTRATE_ROOT'
        Assert-Equal $script:snapshot.SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT $context.HostContextCommitment 'projection harness lost commitment'
        Assert-Equal $script:snapshot.SUBSTRATE_INSTALL_PRIMARY_USER $context.CanonicalAccount 'projection harness lost account'
        Assert-Equal $script:snapshot.SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1 $context.EncodedCarrier 'projection harness lost carrier'
        Assert-Equal $script:snapshot.R2_3W1_TEST_TEMP 'set-inside-child' 'projection harness lost child-only env'
        Assert-Equal ([Environment]::GetEnvironmentVariable('PATH')) $priorPath 'projection harness failed to restore PATH'
        Assert-Equal ([Environment]::GetEnvironmentVariable('SUBSTRATE_INSTALL_PRIMARY_USER')) 'AMBIENT\User' 'projection harness failed to restore prior value'
        Assert-True ([string]::IsNullOrEmpty([Environment]::GetEnvironmentVariable('R2_3W1_TEST_TEMP'))) 'projection harness failed to restore prior absence'
    } finally {
        [Environment]::SetEnvironmentVariable('PATH', $priorPath)
        [Environment]::SetEnvironmentVariable('SUBSTRATE_INSTALL_PRIMARY_USER', $priorMarker)
        [Environment]::SetEnvironmentVariable('R2_3W1_TEST_TEMP', $priorTemp)
    }
}

function Test-W2PipeAndScopeGoldenVectors {
    Assert-Equal (Normalize-WindowsPipePathTest '\\.\pipe\Substrate-Agent') '\\.\pipe\substrate-agent' 'pipe canonicalization drifted'
    Assert-Equal (Normalize-WindowsPipePathTest '\\.\pipe\SUBSTRATE-AGENT') '\\.\pipe\substrate-agent' 'pipe lowercase alias canonicalization drifted'

    foreach ($invalid in @(
            '',
            '\\.\pipe\',
            '//./pipe/substrate-agent',
            '\\server\pipe\substrate-agent',
            '\\.\PIPE\substrate-agent',
            '\\.\pipe\nested\name',
            '\\.\pipe\has space',
            "\\.\pipe\line`nbreak",
            '\\.\pipe\name$',
            ('\\.\pipe\' + ('a' * 129)),
            ('\\.\pipe\caf' + [char]0xE9)
        )) {
        Assert-Throws { Normalize-WindowsPipePathTest $invalid } "invalid pipe should reject: $invalid"
    }

    $scope = Get-WindowsForwarderScopeV1Test `
        -WindowsSid 'S-1-5-21-1000' `
        -DistroName 'Substrate-WSL' `
        -GuestMachineId 'abcdef0123456789abcdef0123456789' `
        -PipePath '\\.\pipe\Substrate-Agent'
    Assert-Equal $scope.Digest '3b3405b2cf309c050f4ba7acb5f43a2348babf18d6be3c426d066ed058a5e75a' 'forwarder scope digest drifted'
    Assert-Equal $scope.Frame "domain=substrate.windows_forwarder_scope`nversion=1`nwindows_sid=Uy0xLTUtMjEtMTAwMA`ndistro_name=U3Vic3RyYXRlLVdTTA`nguest_machine_id=abcdef0123456789abcdef0123456789`npipe_path=XFwuXHBpcGVcc3Vic3RyYXRlLWFnZW50`n" 'forwarder scope frame drifted'
}

function Test-W2PlatformMappingGoldenVectorAndRevalidation {
    $mappingState = Get-W2SampleMappingState
    $guestObservation = Get-W2SampleGuestObservation
    Assert-Equal $mappingState.PlatformBootstrapMappingV1 'ZG9tYWluPXN1YnN0cmF0ZS5wbGF0Zm9ybV9ib290c3RyYXBfbWFwcGluZwp2ZXJzaW9uPTEKaG9zdF9jb250ZXh0X2NvbW1pdG1lbnQ9M2UxZTcxYjMyNWU5MmIxNmY1YmZjMGYzZDg3NWZkMTVmMDRhMTYxNWFjOTBiMTQzOWViMzdhZmRmOTBhNWFjNwpwbGF0Zm9ybV9raW5kPXdzbAppbnN0YW5jZV9uYW1lPVUzVmljM1J5WVhSbExWZFRUQQpndWVzdF9tYWNoaW5lX2lkPWFiY2RlZjAxMjM0NTY3ODlhYmNkZWYwMTIzNDU2Nzg5Cmhvc3RfcGxhdGZvcm1fY29udHJvbF9yb290PVF6cGNWWE5sY25OY1FXeHBZMlZjUVhCd1JHRjBZVnhNYjJOaGJGeFRkV0p6ZEhKaGRHVmNabTl5ZDJGeVpHVnlYRE5pTXpRd05XSXlZMll6TURsak1EVXdaalJpWVRkaFkySTFaalF6WVRJek5EaGlZV0ptTVRoa05tSmxNMk0wTWpaa01EWTJaV1F3TlRoaE5XVTNOV0UKcmVhbGl6ZWRfc3Vic3RyYXRlX2hvbWU9TDJodmJXVXZZbTlpTHk1emRXSnpkSEpoZEdVCnJlYWxpemVkX3ByaW5jaXBhbF9hY2NvdW50PVltOWkKcmVhbGl6ZWRfcHJpbmNpcGFsX3VpZD0xMDAxCnRyYW5zcG9ydF9raW5kPXdzbAp0cmFuc3BvcnRfaG9zdD1YRnd1WEhCcGNHVmNjM1ZpYzNSeVlYUmxMV0ZuWlc1MAp0cmFuc3BvcnRfZ3Vlc3Rfc29ja2V0PUwzSjFiaTl6ZFdKemRISmhkR1V1YzI5amF3Cg' 'platform bootstrap mapping carrier drifted'
    Assert-Equal $mappingState.HostPlatformControlRoot 'C:\Users\Alice\AppData\Local\Substrate\forwarder\3b3405b2cf309c050f4ba7acb5f43a2348babf18d6be3c426d066ed058a5e75a' 'host control root drifted'
    Assert-Equal $mappingState.RealizedSubstrateHome '/home/bob/.substrate' 'guest substrate home drifted'
    Assert-Equal $mappingState.ForwarderConfigPath 'C:\Users\Alice\AppData\Local\Substrate\forwarder\forwarder.toml' 'forwarder config projection drifted'
    Assert-Equal $mappingState.ForwarderLogDir 'C:\Users\Alice\AppData\Local\Substrate\forwarder\logs' 'forwarder log projection drifted'
    Assert-Equal $mappingState.SharedForwarderPidPath 'C:\Users\Alice\AppData\Local\Substrate\forwarder\3b3405b2cf309c050f4ba7acb5f43a2348babf18d6be3c426d066ed058a5e75a\forwarder.pid' 'forwarder pid projection drifted'
    Assert-Equal $mappingState.WindowsForwarderScopeV1 '3b3405b2cf309c050f4ba7acb5f43a2348babf18d6be3c426d066ed058a5e75a' 'scope digest drifted'

    $revalidated = Resolve-PlatformBootstrapMappingV1Test `
        -InstallBootstrapContextV1 $mappingState.InstallContext.EncodedCarrier `
        -PlatformBootstrapMappingV1 $mappingState.PlatformBootstrapMappingV1 `
        -DeclaredDistroName 'substrate-wsl' `
        -DeclaredPipePath '\\.\pipe\substrate-agent' `
        -CurrentAccount 'ACME\Alice' `
        -CurrentSid 'S-1-5-21-1000' `
        -KnownFolder 'C:\Users\Alice\AppData\Local' `
        -RegisteredDistros @('Substrate-WSL') `
        -RunningDistros @('Substrate-WSL') `
        -GuestObservation $guestObservation
    Assert-Equal $revalidated.PlatformBootstrapMappingV1 $mappingState.PlatformBootstrapMappingV1 'revalidated mapping drifted'
}

function Test-W2MappingAndObservationRejections {
    $mappingState = Get-W2SampleMappingState
    $guestObservation = Get-W2SampleGuestObservation

    Assert-Throws {
        Resolve-PlatformBootstrapMappingV1Test `
            -InstallBootstrapContextV1 $mappingState.InstallContext.EncodedCarrier `
            -PlatformBootstrapMappingV1 $mappingState.PlatformBootstrapMappingV1 `
            -DeclaredDistroName 'substrate-wsl' `
            -DeclaredPipePath '\\.\pipe\other' `
            -CurrentAccount 'ACME\Alice' `
            -CurrentSid 'S-1-5-21-1000' `
            -KnownFolder 'C:\Users\Alice\AppData\Local' `
            -RegisteredDistros @('Substrate-WSL') `
            -RunningDistros @('Substrate-WSL') `
            -GuestObservation $guestObservation
    } 'wrong pipe should reject mapping revalidation'

    Assert-Throws {
        Resolve-PlatformBootstrapMappingV1Test `
            -InstallBootstrapContextV1 $mappingState.InstallContext.EncodedCarrier `
            -PlatformBootstrapMappingV1 $mappingState.PlatformBootstrapMappingV1 `
            -DeclaredDistroName 'substrate-wsl' `
            -DeclaredPipePath '\\.\pipe\substrate-agent' `
            -CurrentAccount 'ACME\Alice' `
            -CurrentSid 'S-1-5-21-1001' `
            -KnownFolder 'C:\Users\Alice\AppData\Local' `
            -RegisteredDistros @('Substrate-WSL') `
            -RunningDistros @('Substrate-WSL') `
            -GuestObservation $guestObservation
    } 'wrong current sid should reject mapping revalidation'

    Assert-Throws {
        Resolve-PlatformBootstrapMappingV1Test `
            -InstallBootstrapContextV1 $mappingState.InstallContext.EncodedCarrier `
            -DeclaredDistroName 'substrate-wsl' `
            -DeclaredPipePath '\\.\pipe\substrate-agent' `
            -CurrentAccount 'ACME\Alice' `
            -CurrentSid 'S-1-5-21-1000' `
            -KnownFolder 'C:\Users\Alice\AppData\Local' `
            -RegisteredDistros @('Substrate-WSL', 'substrate-wsl') `
            -GuestObservation ([pscustomobject]@{
                    MachineId = 'abcdef0123456789abcdef0123456789'
                    Account   = 'bob'
                    Uid       = '1001'
                    Home      = '/home/bob'
                    NameEntry = 'bob:x:1001:1001:Bob:/home/bob:/bin/bash'
                    UidEntry  = 'bob:x:1001:1001:Bob:/home/bob:/bin/bash'
                })
    } 'ambiguous distro spelling should reject'

    Assert-Throws {
        Resolve-PlatformBootstrapMappingV1Test `
            -InstallBootstrapContextV1 $mappingState.InstallContext.EncodedCarrier `
            -DeclaredDistroName 'substrate-wsl' `
            -DeclaredPipePath '\\.\pipe\substrate-agent' `
            -CurrentAccount 'ACME\Alice' `
            -CurrentSid 'S-1-5-21-1000' `
            -KnownFolder 'C:\Users\Alice\AppData\Local' `
            -RegisteredDistros @('Substrate-WSL (Default)') `
            -GuestObservation ([pscustomobject]@{
                    MachineId = 'abcdef0123456789abcdef0123456789'
                    Account   = 'bob'
                    Uid       = '1001'
                    Home      = '/home/bob'
                    NameEntry = 'bob:x:1001:1001:Bob:/home/bob:/bin/bash'
                    UidEntry  = 'bob:x:1001:1001:Bob:/home/bob:/bin/bash'
                })
    } 'literal default suffix should not match a different declared distro name'

    Assert-Throws {
        Resolve-PlatformBootstrapMappingV1Test `
            -InstallBootstrapContextV1 $mappingState.InstallContext.EncodedCarrier `
            -DeclaredDistroName 'Substrate WSL' `
            -DeclaredPipePath '\\.\pipe\substrate-agent' `
            -CurrentAccount 'ACME\Alice' `
            -CurrentSid 'S-1-5-21-1000' `
            -KnownFolder 'C:\Users\Alice\AppData\Local' `
            -RegisteredDistros @('Substrate  WSL') `
            -GuestObservation ([pscustomobject]@{
                    MachineId = 'abcdef0123456789abcdef0123456789'
                    Account   = 'bob'
                    Uid       = '1001'
                    Home      = '/home/bob'
                    NameEntry = 'bob:x:1001:1001:Bob:/home/bob:/bin/bash'
                    UidEntry  = 'bob:x:1001:1001:Bob:/home/bob:/bin/bash'
                })
    } 'repeated whitespace should not collapse into the declared distro name'

    Assert-Throws {
        Resolve-PlatformBootstrapMappingV1Test `
            -InstallBootstrapContextV1 $mappingState.InstallContext.EncodedCarrier `
            -DeclaredDistroName 'substrate-wsl' `
            -DeclaredPipePath '\\.\pipe\substrate-agent' `
            -CurrentAccount 'ACME\Alice' `
            -CurrentSid 'S-1-5-21-1000' `
            -KnownFolder 'C:\Users\Alice\AppData\Local' `
            -RegisteredDistros @('Substrate-WSL') `
            -RunningDistros @('Substrate-WSL') `
            -GuestObservation ([pscustomobject]@{
                    MachineId = 'abcdef0123456789abcdef0123456789'
                    Account   = 'bob'
                    Uid       = '1001'
                    Home      = '/home/guessed'
                    NameEntry = 'bob:x:1001:1001:Bob:/home/bob:/bin/bash'
                    UidEntry  = 'bob:x:1001:1001:Bob:/home/bob:/bin/bash'
                })
    } 'guessed guest home should reject'

    Assert-Throws {
        Resolve-PlatformBootstrapMappingV1Test `
            -InstallBootstrapContextV1 $mappingState.InstallContext.EncodedCarrier `
            -DeclaredDistroName 'substrate-wsl' `
            -DeclaredPipePath '\\.\pipe\substrate-agent' `
            -CurrentAccount 'ACME\Alice' `
            -CurrentSid 'S-1-5-21-1000' `
            -KnownFolder 'C:\Users\Alice\AppData\Local' `
            -RegisteredDistros @('Substrate-WSL') `
            -RunningDistros @() `
            -GuestObservation ([pscustomobject]@{
                    MachineId = 'abcdef0123456789abcdef0123456789'
                    Account   = 'bob'
                    Uid       = '1001'
                    Home      = '/home/bob'
                    NameEntry = 'bob:x:1001:1001:Bob:/home/bob:/bin/bash'
                    UidEntry  = 'bob:x:1001:1001:Bob:/home/bob:/bin/bash'
                })
    } 'non-running distro should reject guest observation'

    $tamperedRecord = (ConvertFrom-CanonicalBase64UrlUtf8 $mappingState.PlatformBootstrapMappingV1).Replace(
        'host_context_commitment=3e1e71b325e92b16f5bfc0f3d875fd15f04a1615ac90b1439eb37afdf90a5ac7',
        'host_context_commitment=0000000000000000000000000000000000000000000000000000000000000000'
    )
    Assert-Throws {
        Resolve-PlatformBootstrapMappingV1Test `
            -InstallBootstrapContextV1 $mappingState.InstallContext.EncodedCarrier `
            -PlatformBootstrapMappingV1 (ConvertTo-Base64UrlUtf8 $tamperedRecord) `
            -DeclaredDistroName 'substrate-wsl' `
            -DeclaredPipePath '\\.\pipe\substrate-agent' `
            -CurrentAccount 'ACME\Alice' `
            -CurrentSid 'S-1-5-21-1000' `
            -KnownFolder 'C:\Users\Alice\AppData\Local' `
            -RegisteredDistros @('Substrate-WSL') `
            -RunningDistros @('Substrate-WSL') `
            -GuestObservation $guestObservation
    } 'tampered mapping commitment should reject'

    $staleMachineId = [pscustomobject]@{
        MachineId = '00000000000000000000000000000000'
        Account   = $guestObservation.Account
        Uid       = $guestObservation.Uid
        Home      = $guestObservation.Home
        NameEntry = $guestObservation.NameEntry
        UidEntry  = $guestObservation.UidEntry
    }
    Assert-Throws {
        Resolve-PlatformBootstrapMappingV1Test `
            -InstallBootstrapContextV1 $mappingState.InstallContext.EncodedCarrier `
            -PlatformBootstrapMappingV1 $mappingState.PlatformBootstrapMappingV1 `
            -DeclaredDistroName 'substrate-wsl' `
            -DeclaredPipePath '\\.\pipe\substrate-agent' `
            -CurrentAccount 'ACME\Alice' `
            -CurrentSid 'S-1-5-21-1000' `
            -KnownFolder 'C:\Users\Alice\AppData\Local' `
            -RegisteredDistros @('Substrate-WSL') `
            -RunningDistros @('Substrate-WSL') `
            -GuestObservation $staleMachineId
    } 'stale live machine-id should reject mapping revalidation'

    $badGuestPrincipalRecord = (ConvertFrom-CanonicalBase64UrlUtf8 $mappingState.PlatformBootstrapMappingV1).Replace(
        'realized_principal_account=Ym9i',
        ('realized_principal_account={0}' -f (ConvertTo-Base64UrlUtf8 "bob`n"))
    )
    Assert-Throws {
        Resolve-PlatformBootstrapMappingV1Test `
            -InstallBootstrapContextV1 $mappingState.InstallContext.EncodedCarrier `
            -PlatformBootstrapMappingV1 (ConvertTo-Base64UrlUtf8 $badGuestPrincipalRecord) `
            -DeclaredDistroName 'substrate-wsl' `
            -DeclaredPipePath '\\.\pipe\substrate-agent' `
            -CurrentAccount 'ACME\Alice' `
            -CurrentSid 'S-1-5-21-1000' `
            -KnownFolder 'C:\Users\Alice\AppData\Local' `
            -RegisteredDistros @('Substrate-WSL') `
            -RunningDistros @('Substrate-WSL') `
            -GuestObservation $guestObservation
    } 'malformed guest principal should reject mapping revalidation'

    $badGuestHomeRecord = (ConvertFrom-CanonicalBase64UrlUtf8 $mappingState.PlatformBootstrapMappingV1).Replace(
        ('realized_substrate_home={0}' -f (ConvertTo-Base64UrlUtf8 $mappingState.RealizedSubstrateHome)),
        ('realized_substrate_home={0}' -f (ConvertTo-Base64UrlUtf8 '/home/bob/../evil/.substrate'))
    )
    Assert-Throws {
        Resolve-PlatformBootstrapMappingV1Test `
            -InstallBootstrapContextV1 $mappingState.InstallContext.EncodedCarrier `
            -PlatformBootstrapMappingV1 (ConvertTo-Base64UrlUtf8 $badGuestHomeRecord) `
            -DeclaredDistroName 'substrate-wsl' `
            -DeclaredPipePath '\\.\pipe\substrate-agent' `
            -CurrentAccount 'ACME\Alice' `
            -CurrentSid 'S-1-5-21-1000' `
            -KnownFolder 'C:\Users\Alice\AppData\Local' `
            -RegisteredDistros @('Substrate-WSL') `
            -RunningDistros @('Substrate-WSL') `
            -GuestObservation $guestObservation
    } 'noncanonical guest substrate home should reject mapping revalidation'

    Assert-Throws {
        Resolve-PlatformBootstrapMappingV1Test `
            -InstallBootstrapContextV1 $mappingState.InstallContext.EncodedCarrier `
            -PlatformBootstrapMappingV1 $mappingState.PlatformBootstrapMappingV1 `
            -DeclaredDistroName 'ubuntu-24.04' `
            -DeclaredPipePath '\\.\pipe\substrate-agent' `
            -CurrentAccount 'ACME\Alice' `
            -CurrentSid 'S-1-5-21-1000' `
            -KnownFolder 'C:\Users\Alice\AppData\Local' `
            -RegisteredDistros @('Substrate-WSL', 'Ubuntu-24.04') `
            -RunningDistros @('Ubuntu-24.04') `
            -GuestObservation $guestObservation
    } 'declared distro mismatch should reject before guest observation'
}

function Test-W2ForwarderTargetAndLaunchProjection {
    $mappingState = Get-W2SampleMappingState
    $target = Get-ForwarderTargetInfoTest `
        -MappingState $mappingState `
        -ConfigText "[target]`nmode = ""uds""`nuds_path = ""/run/substrate.sock""`n"
    Assert-Equal $target.Mode 'uds' 'forwarder target mode drifted'
    Assert-Equal $target.Endpoint '/run/substrate.sock' 'forwarder target endpoint drifted'

    $configDefaultUdsTarget = Get-ForwarderTargetInfoTest `
        -MappingState $mappingState `
        -ConfigText "[target]`nmode = ""uds""`n"
    Assert-Equal $configDefaultUdsTarget.Endpoint '/run/substrate.sock' 'forwarder target config default UDS path drifted'

    $skipWithoutPersistentEvidence = Invoke-ForwarderTargetCheckTest `
        -MappingState $mappingState `
        -EnvironmentOverride 'uds:/run/substrate.sock' `
        -PipeOk
    Assert-Equal $skipWithoutPersistentEvidence.Status 'SKIP' 'doctor target check should skip when only child-process target evidence exists'
    Assert-Throws {
        Invoke-EnvironmentOverrideHarness -Overrides @{ SUBSTRATE_FORWARDER_TARGET = 'uds:/run/substrate.sock' } -Action {
            Get-ForwarderTargetInfoTest -MappingState $mappingState
        }
    } 'doctor target evidence helper should ignore ambient SUBSTRATE_FORWARDER_TARGET without persistent config'

    Assert-Throws {
        Get-ForwarderTargetInfoTest -MappingState $mappingState -ConfigText "[target]`nmode = ""uds""`nuds_path = ""/run/other.sock""`n"
    } 'conflicting config target should reject'
    $skipWithoutConfig = Invoke-ForwarderTargetCheckTest -MappingState $mappingState -PipeOk
    Assert-Equal $skipWithoutConfig.Status 'SKIP' 'doctor target check should skip implicit TCP defaults when the pipe is healthy'
    Assert-Throws {
        Get-ForwarderTargetInfoTest -MappingState $mappingState -ConfigText "[target]`nuds_path = ""/run/substrate.sock""`n"
    } 'config without explicit UDS mode should reject implicit TCP default'
    Assert-Throws {
        Invoke-ForwarderTargetCheckTest -MappingState $mappingState -ConfigText "[target]`nuds_path = ""/run/substrate.sock""`n" -PipeOk
    } 'doctor target check should fail closed for malformed persisted config even when the pipe is healthy'

    $launch = Get-ForwarderLaunchProjectionTest -MappingState $mappingState -RustLog 'info' -AdditionalArgs @('--run-as-service')
    $expectedLaunchArgs = (
        '--distro|Substrate-WSL|--pipe|\\.\pipe\substrate-agent|--install-bootstrap-context-v1|{0}|--platform-bootstrap-mapping-v1|{1}|--config|C:\Users\Alice\AppData\Local\Substrate\forwarder\forwarder.toml|--log-dir|C:\Users\Alice\AppData\Local\Substrate\forwarder\logs|--run-as-service' -f
        $mappingState.InstallContext.EncodedCarrier,
        $mappingState.PlatformBootstrapMappingV1
    )
    Assert-Equal ($launch.Arguments -join '|') $expectedLaunchArgs 'forwarder child argv drifted'
    Assert-Throws {
        Get-ForwarderLaunchProjectionTest -MappingState $mappingState -RustLog 'info' -AdditionalArgs @('--pipe', '\\.\pipe\other')
    } 'AdditionalArgs should not override the authenticated pipe selector'
    Assert-Throws {
        Get-ForwarderLaunchProjectionTest -MappingState $mappingState -RustLog 'info' -AdditionalArgs @('--install-bootstrap-context-v1=forged')
    } 'AdditionalArgs should not override the authenticated install bootstrap carrier'
    Assert-Throws {
        Get-ForwarderLaunchProjectionTest -MappingState $mappingState -RustLog 'info' -AdditionalArgs @('--platform-bootstrap-mapping-v1=forged')
    } 'AdditionalArgs should not override the authenticated platform bootstrap mapping'
    Assert-Throws {
        Get-ForwarderLaunchProjectionTest -MappingState $mappingState -RustLog 'info' -AdditionalArgs @('--config=C:\Other\forwarder.toml')
    } 'AdditionalArgs should not override the authenticated config selector'
    Assert-Throws {
        Get-ForwarderLaunchProjectionTest -MappingState $mappingState -RustLog 'info' -AdditionalArgs @('--tcp-bridge=127.0.0.1:5000')
    } 'AdditionalArgs should not override the authenticated tcp bridge selector'
    Assert-Throws {
        Get-ForwarderLaunchProjectionTest -MappingState $mappingState -RustLog 'info' -TcpBridge '0.0.0.0:5000'
    } 'non-loopback diagnostic tcp bridge should reject'
    $ambientTcpLaunch = Invoke-EnvironmentOverrideHarness -Overrides @{
        SUBSTRATE_FORWARDER_TCP      = '1'
        SUBSTRATE_FORWARDER_TCP_ADDR = '127.0.0.1:5000'
        SUBSTRATE_FORWARDER_TCP_HOST = '127.0.0.1'
        SUBSTRATE_FORWARDER_TCP_PORT = '5000'
    } -Action {
        Get-ForwarderLaunchProjectionTest -MappingState $mappingState -RustLog 'info'
    }
    Assert-True (-not (@($ambientTcpLaunch.Arguments) -contains '--tcp-bridge')) 'forwarder launch should ignore ambient TCP bridge selectors'
    Assert-Equal ((@($launch.Environment.Keys | Sort-Object) -join '|')) 'LOCALAPPDATA|RUST_LOG|SUBSTRATE_FORWARDER_PIPE|SUBSTRATE_FORWARDER_TARGET|SUBSTRATE_FORWARDER_TARGET_ENDPOINT|SUBSTRATE_FORWARDER_TARGET_HOST|SUBSTRATE_FORWARDER_TARGET_MODE|SUBSTRATE_FORWARDER_TARGET_PORT|SUBSTRATE_FORWARDER_TCP|SUBSTRATE_FORWARDER_TCP_ADDR|SUBSTRATE_FORWARDER_TCP_HOST|SUBSTRATE_FORWARDER_TCP_PORT|SUBSTRATE_HOME|SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1|SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT|SUBSTRATE_INSTALL_PRIMARY_USER|SUBSTRATE_ROOT|USERPROFILE|WSLENV' 'forwarder child environment key set drifted'
    Assert-Equal $launch.Environment.SUBSTRATE_HOME $mappingState.InstallContext.HostSubstrateHome 'forwarder child SUBSTRATE_HOME drifted'
    Assert-Equal $launch.Environment.SUBSTRATE_ROOT $mappingState.InstallContext.HostSubstrateRoot 'forwarder child SUBSTRATE_ROOT drifted'
    Assert-Equal $launch.Environment.SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT $mappingState.InstallContext.HostContextCommitment 'forwarder child host commitment drifted'
    Assert-Equal $launch.Environment.SUBSTRATE_INSTALL_PRIMARY_USER $mappingState.InstallContext.CanonicalAccount 'forwarder child primary user drifted'
    Assert-Equal $launch.Environment.SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1 $mappingState.InstallContext.EncodedCarrier 'forwarder child bootstrap carrier drifted'
    Assert-Equal $launch.Environment.SUBSTRATE_FORWARDER_TARGET 'uds:/run/substrate.sock' 'forwarder child target env drifted'
    Assert-True ($null -eq $launch.Environment.LOCALAPPDATA) 'forwarder child should scrub LOCALAPPDATA'
    Assert-True ($null -eq $launch.Environment.USERPROFILE) 'forwarder child should scrub USERPROFILE'
    Assert-True ($null -eq $launch.Environment.WSLENV) 'forwarder child should scrub WSLENV'
    Assert-True ($null -eq $launch.Environment.SUBSTRATE_FORWARDER_TARGET_MODE) 'forwarder child should scrub target mode'
    Assert-True ($null -eq $launch.Environment.SUBSTRATE_FORWARDER_TARGET_HOST) 'forwarder child should scrub target host'
    Assert-True ($null -eq $launch.Environment.SUBSTRATE_FORWARDER_TARGET_PORT) 'forwarder child should scrub target port'
    Assert-True ($null -eq $launch.Environment.SUBSTRATE_FORWARDER_TARGET_ENDPOINT) 'forwarder child should scrub target endpoint'
    Assert-True ($null -eq $launch.Environment.SUBSTRATE_FORWARDER_PIPE) 'forwarder child should scrub ambient pipe selector'
    Assert-True ($null -eq $launch.Environment.SUBSTRATE_FORWARDER_TCP) 'forwarder child should scrub ambient TCP bridge enablement'
    Assert-True ($null -eq $launch.Environment.SUBSTRATE_FORWARDER_TCP_ADDR) 'forwarder child should scrub ambient TCP bridge address'
    Assert-True ($null -eq $launch.Environment.SUBSTRATE_FORWARDER_TCP_HOST) 'forwarder child should scrub ambient TCP bridge host'
    Assert-True ($null -eq $launch.Environment.SUBSTRATE_FORWARDER_TCP_PORT) 'forwarder child should scrub ambient TCP bridge port'

    $priorTarget = [Environment]::GetEnvironmentVariable('SUBSTRATE_FORWARDER_TARGET')
    $priorLocalAppData = [Environment]::GetEnvironmentVariable('LOCALAPPDATA')
    $priorUserProfile = [Environment]::GetEnvironmentVariable('USERPROFILE')
    $priorWslEnv = [Environment]::GetEnvironmentVariable('WSLENV')
    $priorHome = [Environment]::GetEnvironmentVariable('SUBSTRATE_HOME')
    $priorRoot = [Environment]::GetEnvironmentVariable('SUBSTRATE_ROOT')
    $priorCommitment = [Environment]::GetEnvironmentVariable('SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT')
    $priorPrimaryUser = [Environment]::GetEnvironmentVariable('SUBSTRATE_INSTALL_PRIMARY_USER')
    $priorCarrier = [Environment]::GetEnvironmentVariable('SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1')
    $priorTargetMode = [Environment]::GetEnvironmentVariable('SUBSTRATE_FORWARDER_TARGET_MODE')
    $priorTargetHost = [Environment]::GetEnvironmentVariable('SUBSTRATE_FORWARDER_TARGET_HOST')
    $priorTargetPort = [Environment]::GetEnvironmentVariable('SUBSTRATE_FORWARDER_TARGET_PORT')
    $priorTargetEndpoint = [Environment]::GetEnvironmentVariable('SUBSTRATE_FORWARDER_TARGET_ENDPOINT')
    $priorPipe = [Environment]::GetEnvironmentVariable('SUBSTRATE_FORWARDER_PIPE')
    try {
        [Environment]::SetEnvironmentVariable('SUBSTRATE_FORWARDER_TARGET', 'ambient')
        [Environment]::SetEnvironmentVariable('LOCALAPPDATA', 'D:\Ambient')
        [Environment]::SetEnvironmentVariable('USERPROFILE', 'D:\AmbientUser')
        [Environment]::SetEnvironmentVariable('WSLENV', 'AMBIENT/u')
        [Environment]::SetEnvironmentVariable('SUBSTRATE_HOME', 'D:\AmbientHome')
        [Environment]::SetEnvironmentVariable('SUBSTRATE_ROOT', 'D:\AmbientRoot')
        [Environment]::SetEnvironmentVariable('SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT', 'ambient-commitment')
        [Environment]::SetEnvironmentVariable('SUBSTRATE_INSTALL_PRIMARY_USER', 'AMBIENT\User')
        [Environment]::SetEnvironmentVariable('SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1', 'ambient-carrier')
        [Environment]::SetEnvironmentVariable('SUBSTRATE_FORWARDER_TARGET_MODE', 'tcp')
        [Environment]::SetEnvironmentVariable('SUBSTRATE_FORWARDER_TARGET_HOST', '127.0.0.1')
        [Environment]::SetEnvironmentVariable('SUBSTRATE_FORWARDER_TARGET_PORT', '61337')
        [Environment]::SetEnvironmentVariable('SUBSTRATE_FORWARDER_TARGET_ENDPOINT', 'tcp://127.0.0.1:61337')
        [Environment]::SetEnvironmentVariable('SUBSTRATE_FORWARDER_PIPE', '\\.\pipe\ambient')
        $script:snapshot = $null
        Invoke-EnvironmentOverrideHarness -Overrides $launch.Environment -Action {
            $script:snapshot = [pscustomobject]@{
                SUBSTRATE_HOME                         = [Environment]::GetEnvironmentVariable('SUBSTRATE_HOME')
                SUBSTRATE_ROOT                         = [Environment]::GetEnvironmentVariable('SUBSTRATE_ROOT')
                SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT = [Environment]::GetEnvironmentVariable('SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT')
                SUBSTRATE_INSTALL_PRIMARY_USER         = [Environment]::GetEnvironmentVariable('SUBSTRATE_INSTALL_PRIMARY_USER')
                SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1 = [Environment]::GetEnvironmentVariable('SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1')
                SUBSTRATE_FORWARDER_TARGET             = [Environment]::GetEnvironmentVariable('SUBSTRATE_FORWARDER_TARGET')
                LOCALAPPDATA                           = [Environment]::GetEnvironmentVariable('LOCALAPPDATA')
                USERPROFILE                            = [Environment]::GetEnvironmentVariable('USERPROFILE')
                WSLENV                                 = [Environment]::GetEnvironmentVariable('WSLENV')
                SUBSTRATE_FORWARDER_TARGET_MODE        = [Environment]::GetEnvironmentVariable('SUBSTRATE_FORWARDER_TARGET_MODE')
                SUBSTRATE_FORWARDER_TARGET_HOST        = [Environment]::GetEnvironmentVariable('SUBSTRATE_FORWARDER_TARGET_HOST')
                SUBSTRATE_FORWARDER_TARGET_PORT        = [Environment]::GetEnvironmentVariable('SUBSTRATE_FORWARDER_TARGET_PORT')
                SUBSTRATE_FORWARDER_TARGET_ENDPOINT    = [Environment]::GetEnvironmentVariable('SUBSTRATE_FORWARDER_TARGET_ENDPOINT')
                SUBSTRATE_FORWARDER_PIPE               = [Environment]::GetEnvironmentVariable('SUBSTRATE_FORWARDER_PIPE')
            }
        }
        Assert-Equal $script:snapshot.SUBSTRATE_HOME $mappingState.InstallContext.HostSubstrateHome 'SUBSTRATE_HOME should carry the selected prefix'
        Assert-Equal $script:snapshot.SUBSTRATE_ROOT $mappingState.InstallContext.HostSubstrateRoot 'SUBSTRATE_ROOT should carry the selected prefix'
        Assert-Equal $script:snapshot.SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT $mappingState.InstallContext.HostContextCommitment 'host commitment should survive the child projection'
        Assert-Equal $script:snapshot.SUBSTRATE_INSTALL_PRIMARY_USER $mappingState.InstallContext.CanonicalAccount 'primary user should survive the child projection'
        Assert-Equal $script:snapshot.SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1 $mappingState.InstallContext.EncodedCarrier 'bootstrap carrier should survive the child projection'
        Assert-Equal $script:snapshot.SUBSTRATE_FORWARDER_TARGET 'uds:/run/substrate.sock' 'forwarder child env projection drifted'
        Assert-True ([string]::IsNullOrEmpty($script:snapshot.LOCALAPPDATA)) 'LOCALAPPDATA should be absent inside child projection'
        Assert-True ([string]::IsNullOrEmpty($script:snapshot.USERPROFILE)) 'USERPROFILE should be absent inside child projection'
        Assert-True ([string]::IsNullOrEmpty($script:snapshot.WSLENV)) 'WSLENV should be absent inside child projection'
        Assert-True ([string]::IsNullOrEmpty($script:snapshot.SUBSTRATE_FORWARDER_TARGET_MODE)) 'target mode should be absent inside child projection'
        Assert-True ([string]::IsNullOrEmpty($script:snapshot.SUBSTRATE_FORWARDER_TARGET_HOST)) 'target host should be absent inside child projection'
        Assert-True ([string]::IsNullOrEmpty($script:snapshot.SUBSTRATE_FORWARDER_TARGET_PORT)) 'target port should be absent inside child projection'
        Assert-True ([string]::IsNullOrEmpty($script:snapshot.SUBSTRATE_FORWARDER_TARGET_ENDPOINT)) 'target endpoint should be absent inside child projection'
        Assert-True ([string]::IsNullOrEmpty($script:snapshot.SUBSTRATE_FORWARDER_PIPE)) 'ambient pipe selector should be absent inside child projection'
        Assert-Equal ([Environment]::GetEnvironmentVariable('SUBSTRATE_HOME')) 'D:\AmbientHome' 'forwarder env harness failed to restore prior SUBSTRATE_HOME'
        Assert-Equal ([Environment]::GetEnvironmentVariable('SUBSTRATE_ROOT')) 'D:\AmbientRoot' 'forwarder env harness failed to restore prior SUBSTRATE_ROOT'
        Assert-Equal ([Environment]::GetEnvironmentVariable('SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT')) 'ambient-commitment' 'forwarder env harness failed to restore prior host commitment'
        Assert-Equal ([Environment]::GetEnvironmentVariable('SUBSTRATE_INSTALL_PRIMARY_USER')) 'AMBIENT\User' 'forwarder env harness failed to restore prior primary user'
        Assert-Equal ([Environment]::GetEnvironmentVariable('SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1')) 'ambient-carrier' 'forwarder env harness failed to restore prior carrier'
        Assert-Equal ([Environment]::GetEnvironmentVariable('SUBSTRATE_FORWARDER_TARGET')) 'ambient' 'forwarder env harness failed to restore prior target'
        Assert-Equal ([Environment]::GetEnvironmentVariable('LOCALAPPDATA')) 'D:\Ambient' 'forwarder env harness failed to restore prior LOCALAPPDATA'
        Assert-Equal ([Environment]::GetEnvironmentVariable('USERPROFILE')) 'D:\AmbientUser' 'forwarder env harness failed to restore prior USERPROFILE'
        Assert-Equal ([Environment]::GetEnvironmentVariable('WSLENV')) 'AMBIENT/u' 'forwarder env harness failed to restore prior WSLENV'
        Assert-Equal ([Environment]::GetEnvironmentVariable('SUBSTRATE_FORWARDER_TARGET_MODE')) 'tcp' 'forwarder env harness failed to restore prior target mode'
        Assert-Equal ([Environment]::GetEnvironmentVariable('SUBSTRATE_FORWARDER_TARGET_HOST')) '127.0.0.1' 'forwarder env harness failed to restore prior target host'
        Assert-Equal ([Environment]::GetEnvironmentVariable('SUBSTRATE_FORWARDER_TARGET_PORT')) '61337' 'forwarder env harness failed to restore prior target port'
        Assert-Equal ([Environment]::GetEnvironmentVariable('SUBSTRATE_FORWARDER_TARGET_ENDPOINT')) 'tcp://127.0.0.1:61337' 'forwarder env harness failed to restore prior target endpoint'
        Assert-Equal ([Environment]::GetEnvironmentVariable('SUBSTRATE_FORWARDER_PIPE')) '\\.\pipe\ambient' 'forwarder env harness failed to restore prior pipe selector'
    } finally {
        [Environment]::SetEnvironmentVariable('SUBSTRATE_HOME', $priorHome)
        [Environment]::SetEnvironmentVariable('SUBSTRATE_ROOT', $priorRoot)
        [Environment]::SetEnvironmentVariable('SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT', $priorCommitment)
        [Environment]::SetEnvironmentVariable('SUBSTRATE_INSTALL_PRIMARY_USER', $priorPrimaryUser)
        [Environment]::SetEnvironmentVariable('SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1', $priorCarrier)
        [Environment]::SetEnvironmentVariable('SUBSTRATE_FORWARDER_TARGET', $priorTarget)
        [Environment]::SetEnvironmentVariable('LOCALAPPDATA', $priorLocalAppData)
        [Environment]::SetEnvironmentVariable('USERPROFILE', $priorUserProfile)
        [Environment]::SetEnvironmentVariable('WSLENV', $priorWslEnv)
        [Environment]::SetEnvironmentVariable('SUBSTRATE_FORWARDER_TARGET_MODE', $priorTargetMode)
        [Environment]::SetEnvironmentVariable('SUBSTRATE_FORWARDER_TARGET_HOST', $priorTargetHost)
        [Environment]::SetEnvironmentVariable('SUBSTRATE_FORWARDER_TARGET_PORT', $priorTargetPort)
        [Environment]::SetEnvironmentVariable('SUBSTRATE_FORWARDER_TARGET_ENDPOINT', $priorTargetEndpoint)
        [Environment]::SetEnvironmentVariable('SUBSTRATE_FORWARDER_PIPE', $priorPipe)
    }
}

function Test-W2WarmGuardEntryFlow {
    $missingResult = Invoke-W2WarmEntryFlowModelTest `
        -ResolveMapping { throw 'platform bootstrap mapping is unavailable or incoherent' } `
        -AssertMapping { throw 'assert should not run when mapping resolution fails' } `
        -Guard { throw 'guard should not run when mapping resolution fails' } `
        -Provision { throw 'provision should not run when mapping resolution fails' }
    Assert-Equal ($missingResult.Trace -join '|') 'start' 'missing mapping should stop before assert/guard/provision'
    Assert-Equal $missingResult.Error 'platform bootstrap mapping is unavailable or incoherent' 'missing mapping should surface the mapping failure'

    $missingStateResult = Invoke-W2WarmEntryFlowModelTest `
        -ResolveMapping {
            [pscustomobject]@{
                PipePath                 = '\\.\pipe\substrate-agent'
                InstallPrefix            = 'C:\Users\Alice\AppData\Local\Substrate'
                WindowsForwarderScopeV1  = $null
                PlatformBootstrapMappingV1 = $null
            }
        } `
        -AssertMapping {
            param($MappingState)
            if ([string]::IsNullOrEmpty($MappingState.PlatformBootstrapMappingV1)) {
                throw 'platform bootstrap mapping is unavailable or incoherent'
            }
        } `
        -Guard { throw 'guard should not run when mapping state is unavailable' } `
        -Provision { throw 'provision should not run when mapping state is unavailable' }
    Assert-Equal ($missingStateResult.Trace -join '|') 'start|resolved' 'missing mapping state should stop before guard/provision'
    Assert-Equal $missingStateResult.Error 'platform bootstrap mapping is unavailable or incoherent' 'missing mapping state should surface the mapping failure'

    $guardedResult = Invoke-W2WarmEntryFlowModelTest `
        -ResolveMapping {
            [pscustomobject]@{
                PipePath                 = '\\.\pipe\substrate-agent'
                InstallPrefix            = 'C:\Users\Alice\AppData\Local\Substrate'
                WindowsForwarderScopeV1  = '3b3405b2cf309c050f4ba7acb5f43a2348babf18d6be3c426d066ed058a5e75a'
                HostPlatformControlRoot = 'C:\Users\Alice\AppData\Local\Substrate\forwarder\3b3405b2cf309c050f4ba7acb5f43a2348babf18d6be3c426d066ed058a5e75a'
            }
        } `
        -AssertMapping { param($MappingState) } `
        -Guard { throw 'guarded fail-closed' } `
        -Provision { throw 'provision should not run after the guard' }
    Assert-Equal ($guardedResult.Trace -join '|') 'start|resolved|asserted|validated-pipe|validated-scope|project-path' 'warm entry flow should hit the guard before any provisioning'
    Assert-Equal $guardedResult.Error 'guarded fail-closed' 'warm entry flow should terminate at the guard'
}

function Test-W2DistroFallbackAndRemediation {
    $verboseNames = Get-WslVerboseRunningDistroNamesTest -RegisteredDistros @(
        'Substrate  WSL',
        'Substrate',
        'Ubuntu-24.04'
    ) -Lines @(
        '  NAME              STATE           VERSION',
        '* Substrate  WSL    Running         2',
        '  Substrate         Stopped         2',
        '  Ubuntu-24.04      Stopped         2',
        '  Ubuntu-24.04      Running         2',
        '  Substrate  WSL    Running         2'
    )
    Assert-Equal ($verboseNames -join '|') 'Substrate  WSL|Ubuntu-24.04' 'verbose running parser drifted'

    $aliasNames = Get-WslVerboseRunningDistroNamesTest -RegisteredDistros @('Substrate  WSL') -Lines @(
        '  NAME              STATE           VERSION',
        '  Substrate         Running         2'
    )
    Assert-Equal (@($aliasNames).Count) 0 'verbose running parser accepted a fabricated spaced-name alias'
}

function Test-W2StaticScriptShape {
    $startForwarder = Get-ScriptText 'start-forwarder.ps1'
    $pipeStatus = Get-ScriptText 'pipe-status.ps1'
    $wslWarm = Get-ScriptText 'wsl-warm.ps1'
    $wslDoctor = Get-ScriptText 'wsl-doctor.ps1'
    $wslDoctorInstalledDistros = Get-TextBetweenMarkers $wslDoctor 'function Get-InstalledWslDistros {' 'function Test-NamedPipe {'

    foreach ($entry in @(
            @{ Name = 'start-forwarder.ps1'; Text = $startForwarder },
            @{ Name = 'pipe-status.ps1'; Text = $pipeStatus },
            @{ Name = 'wsl-warm.ps1'; Text = $wslWarm },
            @{ Name = 'wsl-doctor.ps1'; Text = $wslDoctor }
        )) {
        Assert-Match $entry.Text 'function Resolve-PlatformBootstrapMappingV1' "$($entry.Name) is missing Resolve-PlatformBootstrapMappingV1"
        Assert-Match $entry.Text 'function Assert-PlatformBootstrapMappingV1' "$($entry.Name) is missing Assert-PlatformBootstrapMappingV1"
        Assert-Match $entry.Text 'function Get-WindowsForwarderScopeV1' "$($entry.Name) is missing Get-WindowsForwarderScopeV1"
        Assert-Match $entry.Text 'function Assert-CurrentWindowsPrincipalV1' "$($entry.Name) is missing Assert-CurrentWindowsPrincipalV1"
        Assert-Match $entry.Text '\[string\]\$InstallPrefix' "$($entry.Name) is missing InstallPrefix intake"
        Assert-Match $entry.Text '\[string\]\$InstallBootstrapContextV1' "$($entry.Name) is missing InstallBootstrapContextV1 intake"
        Assert-Match $entry.Text '\[string\]\$PlatformBootstrapMappingV1' "$($entry.Name) is missing PlatformBootstrapMappingV1 intake"
    }

    Assert-Match $pipeStatus '\[string\]\$DistroName = ''substrate-wsl''' 'pipe-status.ps1 is missing DistroName intake'
    Assert-Match $pipeStatus 'function Get-HexBytes' 'pipe-status.ps1 lost Get-HexBytes'
    Assert-Match $wslWarm 'function Convert-ToWslPathFragment' 'wsl-warm.ps1 lost Convert-ToWslPathFragment'
    Assert-Match $wslWarm 'function Quote-ForBash' 'wsl-warm.ps1 lost Quote-ForBash'
    Assert-Match $wslWarm 'function Test-Truthy' 'wsl-warm.ps1 lost Test-Truthy'
    Assert-Match $wslWarm 'function Test-GuestExecutablePresent' 'wsl-warm.ps1 lost Test-GuestExecutablePresent'
    Assert-Match $wslWarm 'function Install-GuestWorldBinaries' 'wsl-warm.ps1 lost Install-GuestWorldBinaries'
    Assert-True ($startForwarder -notmatch 'Join-Path \$env:LOCALAPPDATA ''Substrate/logs''') 'start-forwarder.ps1 still selects logs from LOCALAPPDATA'
    Assert-True ($startForwarder -notmatch '\$envFlag = \(\$env:SUBSTRATE_FORWARDER_TCP') 'start-forwarder.ps1 still selects tcp bridge from ambient SUBSTRATE_FORWARDER_TCP'
    Assert-True ($startForwarder -notmatch '\$TcpBridge = \$env:SUBSTRATE_FORWARDER_TCP_ADDR') 'start-forwarder.ps1 still selects tcp bridge from ambient SUBSTRATE_FORWARDER_TCP_ADDR'
    Assert-True ($wslDoctor -notmatch 'Join-Path \$env:LOCALAPPDATA ''Substrate\\forwarder\.toml''') 'wsl-doctor.ps1 still selects config from LOCALAPPDATA'
    Assert-True ($wslDoctor -notmatch '\$pipePath = "\\\\\.\\pipe\\substrate-agent"') 'wsl-doctor.ps1 still hardcodes the runtime pipe selector'
    Assert-Match $startForwarder 'registered WSL distro is not already running' 'start-forwarder.ps1 lost the running-distro gate'
    Assert-Match $startForwarder '\$line\.StartsWith\(\$registeredName, \[System\.StringComparison\]::Ordinal\)' 'start-forwarder.ps1 must anchor verbose rows to exact registered distro names'
    Assert-Match $startForwarder '\$normalizedDeclaredPrefix = & \$normalizeWindowsPath \$DeclaredPrefix' 'start-forwarder.ps1 must normalize explicit prefixes before the final assert'
    Assert-Match $startForwarder '\$decodedGuestAccount = & \$decodeBase64Url \$parsed\[''realized_principal_account''\]' 'start-forwarder.ps1 must decode guest account before live observation'
    Assert-Match $startForwarder '\$decodedGuestUid = \$parsed\[''realized_principal_uid''\]' 'start-forwarder.ps1 must decode guest uid before live observation'
    Assert-Match $startForwarder '\$decodedControlRoot = & \$decodeBase64Url \$parsed\[''host_platform_control_root''\]' 'start-forwarder.ps1 must decode control root before live observation'
    Assert-Match $startForwarder '\$decodedSubstrateHome = & \$normalizeUnixPath \$decodedGuestSubstrateHome' 'start-forwarder.ps1 must canonicalize guest substrate home before live observation'
    Assert-Match $startForwarder '\$observation = & \$observeGuestIdentity' 'start-forwarder.ps1 must still perform live guest observation'
    $mappingPresentObservationIndex = $startForwarder.LastIndexOf('$observation = & $observeGuestIdentity', [System.StringComparison]::Ordinal)
    Assert-True (($startForwarder.IndexOf('$decodedGuestAccount = & $decodeBase64Url $parsed[''realized_principal_account'']', [System.StringComparison]::Ordinal)) -lt $mappingPresentObservationIndex) 'start-forwarder.ps1 must decode guest account before live observation'
    Assert-True (($startForwarder.IndexOf('$decodedGuestUid = $parsed[''realized_principal_uid'']', [System.StringComparison]::Ordinal)) -lt $mappingPresentObservationIndex) 'start-forwarder.ps1 must decode guest uid before live observation'
    Assert-True (($startForwarder.IndexOf('$decodedControlRoot = & $decodeBase64Url $parsed[''host_platform_control_root'']', [System.StringComparison]::Ordinal)) -lt $mappingPresentObservationIndex) 'start-forwarder.ps1 must decode control root before live observation'
    Assert-True (($startForwarder.IndexOf('$decodedSubstrateHome = & $normalizeUnixPath $decodedGuestSubstrateHome', [System.StringComparison]::Ordinal)) -lt $mappingPresentObservationIndex) 'start-forwarder.ps1 must canonicalize guest substrate home before live observation'
    Assert-Match $pipeStatus 'platform bootstrap mapping is unavailable or incoherent' 'pipe-status.ps1 must reject missing mappings'
    Assert-Match $wslDoctor 'platform bootstrap mapping is unavailable or incoherent' 'wsl-doctor.ps1 must reject missing mappings'
    Assert-Match $wslWarm 'platform bootstrap mapping is unavailable or incoherent' 'wsl-warm.ps1 must reject missing mappings'
    Assert-Match $wslWarm 'KnownFolderLocalApplicationData = \(& \$normalizeWindowsPath \$knownFolder\)' 'wsl-warm.ps1 must canonicalize the Known Folder binding'
    Assert-Match $pipeStatus '\$line\.StartsWith\(\$registeredName, \[System\.StringComparison\]::Ordinal\)' 'pipe-status.ps1 must anchor verbose rows to exact registered distro names'
    Assert-Match $wslWarm '\$line\.StartsWith\(\$registeredName, \[System\.StringComparison\]::Ordinal\)' 'wsl-warm.ps1 must anchor verbose rows to exact registered distro names'
    Assert-Match $wslDoctor 'Import or create the distro outside this fail-closed slice, then rerun doctor' 'wsl-doctor.ps1 must not point distro remediation at wsl-warm.ps1'
    Assert-Match $wslDoctor 'scripts/windows/start-forwarder\.ps1' 'wsl-doctor.ps1 must point forwarder remediation at start-forwarder.ps1'
    Assert-Match $startForwarder '--config' 'start-forwarder.ps1 is missing explicit --config projection'
    Assert-Match $startForwarder '--log-dir' 'start-forwarder.ps1 is missing explicit --log-dir projection'
    Assert-Match $startForwarder '--install-bootstrap-context-v1' 'start-forwarder.ps1 is missing explicit install bootstrap carrier argv'
    Assert-Match $startForwarder '--platform-bootstrap-mapping-v1' 'start-forwarder.ps1 is missing explicit platform bootstrap mapping argv'
    Assert-Match $startForwarder '\[System\.Uri\]\("tcp://\$TcpBridge"\)' 'start-forwarder.ps1 must parse explicit tcp bridge endpoints before launch'
    Assert-Match $startForwarder '\$tcpBridgeUri\.IsLoopback' 'start-forwarder.ps1 must keep diagnostic tcp bridge loopback-only'
    Assert-Match $startForwarder 'SUBSTRATE_FORWARDER_TCP\s*=\s*\$null' 'start-forwarder.ps1 must scrub ambient TCP bridge enablement for the child'
    Assert-Match $startForwarder 'SUBSTRATE_FORWARDER_TCP_ADDR\s*=\s*\$null' 'start-forwarder.ps1 must scrub ambient TCP bridge address for the child'
    Assert-Match $startForwarder 'SUBSTRATE_FORWARDER_TCP_HOST\s*=\s*\$null' 'start-forwarder.ps1 must scrub ambient TCP bridge host for the child'
    Assert-Match $startForwarder 'SUBSTRATE_FORWARDER_TCP_PORT\s*=\s*\$null' 'start-forwarder.ps1 must scrub ambient TCP bridge port for the child'
    Assert-Match $startForwarder 'SUBSTRATE_FORWARDER_TARGET\s*=\s*\(''uds:\{0\}'' -f \$mappingState\.GuestSocket\)' 'start-forwarder.ps1 is missing explicit target projection'
    Assert-Match $startForwarder 'LOCALAPPDATA\s*=\s*\$null' 'start-forwarder.ps1 must scrub LOCALAPPDATA for the child'
    Assert-Match $startForwarder 'USERPROFILE\s*=\s*\$null' 'start-forwarder.ps1 must scrub USERPROFILE for the child'
    Assert-Match $startForwarder 'WSLENV\s*=\s*\$null' 'start-forwarder.ps1 must scrub WSLENV for the child'
    Assert-Match $wslWarm 'Resolve-PlatformBootstrapMappingV1' 'wsl-warm.ps1 must validate mapping before the guard'
    Assert-Match $wslWarm 'Write-ErrorAndExit "WSL world provisioning is intentionally fail-closed' 'wsl-warm.ps1 lost the fail-closed guard'
    Assert-Match $wslDoctor '\$script:MappingState = Resolve-PlatformBootstrapMappingV1' 'wsl-doctor.ps1 must resolve mapping once at entry'
    Assert-Match $wslDoctor '\$line\.StartsWith\(\$registeredName, \[System\.StringComparison\]::Ordinal\)' 'wsl-doctor.ps1 must anchor verbose rows to exact registered distro names'
    Assert-Match $wslDoctorInstalledDistros '& wsl\.exe -l -q 2>\$null' 'wsl-doctor.ps1 must enumerate registered distros via wsl -l -q'
    Assert-True ($wslDoctorInstalledDistros -notmatch 'wsl\.exe -l -v') 'wsl-doctor.ps1 must not infer registered distros from the verbose table'
    Assert-Match $wslDoctor '\$forwarderConfigPath = \$script:MappingState\.ForwarderConfigPath' 'wsl-doctor.ps1 must use mapping-derived config path'
    Assert-Match $wslDoctor '\$pidFile = \$script:MappingState\.SharedForwarderPidPath' 'wsl-doctor.ps1 must use mapping-derived pid path'
    Assert-Match $wslDoctor '\$logDir = \$script:MappingState\.ForwarderLogDir' 'wsl-doctor.ps1 must use mapping-derived log path'
    Assert-Match $startForwarder '\$forwarderRelease = Join-Path \$resolvedProject ''target/release/substrate-forwarder\.exe''' 'start-forwarder.ps1 must still prefer the release forwarder binary'
    Assert-Match $startForwarder "Write-Warn 'Release binary not found, using debug build'" 'start-forwarder.ps1 must still disclose debug-binary fallback'
    Assert-Match $startForwarder "Build it with 'cargo build -p substrate-forwarder --release'" 'start-forwarder.ps1 must still disclose the release build instruction'
    Assert-True ($startForwarder -notmatch 'SharedForwarderPidPath.*(Remove-Item|Set-Content|Out-File|WriteAllText)') 'start-forwarder.ps1 must not take ownership of the shared pid projection'
    Assert-True ($startForwarder -notmatch 'Remove-Item[^\n]*forwarder\.pid') 'start-forwarder.ps1 must not delete the shared forwarder pid path'
    foreach ($entry in @(
            @{ Name = 'start-forwarder.ps1'; Text = $startForwarder },
            @{ Name = 'pipe-status.ps1'; Text = $pipeStatus },
            @{ Name = 'wsl-warm.ps1'; Text = $wslWarm },
            @{ Name = 'wsl-doctor.ps1'; Text = $wslDoctor }
        )) {
        Assert-True ($entry.Text -notmatch 'Write-(Info|Warn|Fail|Diag)[^\r\n]*(CanonicalSid|InstallBootstrapContextV1|PlatformBootstrapMappingV1|EncodedCarrier)') "$($entry.Name) leaks a secret carrier or SID in diagnostics"
    }
}

function Test-W2FrozenBlockHashes {
    $startForwarder = Get-ScriptText 'start-forwarder.ps1'
    $pipeStatus = Get-ScriptText 'pipe-status.ps1'
    $wslWarm = Get-ScriptText 'wsl-warm.ps1'

    $startForwarderReadyBlock = Get-TextBetweenMarkers $startForwarder '# Default: return once ready. Use -WaitForExit only for CI/service.' '    return'
    Assert-Equal (Get-Sha256Hex $startForwarderReadyBlock) 'b2d0956be084b52b98111510db7473851817816b85ab9794fd6a3fc10a174dd5' 'start-forwarder readiness block drifted'

    $startForwarderBlock = Get-TextBetweenMarkers $startForwarder '    $stopwatch = [System.Diagnostics.Stopwatch]::StartNew()' '    $stdout = $process.StandardOutput.ReadToEnd()'
    Assert-Equal (Get-Sha256Hex $startForwarderBlock) '062f7f803f1c350a7cb25ba1a8eab395be713d44fa07b09729da2e98a0318497' 'start-forwarder timeout block drifted'

    $pipeStatusReadBlock = Get-TextBetweenMarkers $pipeStatus '$client = [System.IO.Pipes.NamedPipeClientStream]::new(''.'', $pipeName, [System.IO.Pipes.PipeDirection]::InOut, [System.IO.Pipes.PipeOptions]::None)' 'Write-Output ("Status: {0}" -f $statusLine)'
    Assert-Equal (Get-Sha256Hex $pipeStatusReadBlock) '7d985c7a693b2cdb776a706cbd88fc05fc47670af36fa47290dd8fffa1dfc7f9' 'pipe-status raw status-line parser block drifted'

    $wslWarmTail = Get-TailFromMarker $wslWarm 'Write-ErrorAndExit "WSL world provisioning is intentionally fail-closed in this slice because the WSL helper path is not aligned with the Linux/macOS placement contract for SUBSTRATE_HOME placement, socket/group ownership, and runtime artifact access. Use Linux host-native provisioning, macOS Lima provisioning, or a CLI-only WSL install with --no-world instead." 4'
    Assert-Equal (Get-Sha256Hex $wslWarmTail) '6c17a3b8925cd2c3b240d27e98bfa6330ffbc3335d2481adef19cd033a6288ba' 'wsl-warm guard tail drifted'

    $wslWarmEntry = Get-TextBetweenMarkers $wslWarm 'Write-Info "Starting wsl-warm for distro ''$DistroName''"' '$projectHasCargo = Test-Path (Join-Path $projectPath ''Cargo.toml'')'
    Assert-Equal (Get-Sha256Hex $wslWarmEntry) '2ed96a06122ae11a850f0d700757aae9ee0fa49aadb3d0a0c90aa27075129946' 'wsl-warm entry/guard block drifted'
}

function Test-StaticScriptShape {
    $devInstall = Get-ScriptText 'dev-install-substrate.ps1'
    $devUninstall = Get-ScriptText 'dev-uninstall-substrate.ps1'
    $releaseInstall = Get-ScriptText 'install-substrate.ps1'
    $releaseUninstall = Get-ScriptText 'uninstall-substrate.ps1'

    foreach ($entry in @(
            @{ Name = 'dev-install-substrate.ps1'; Text = $devInstall },
            @{ Name = 'dev-uninstall-substrate.ps1'; Text = $devUninstall },
            @{ Name = 'install-substrate.ps1'; Text = $releaseInstall },
            @{ Name = 'uninstall-substrate.ps1'; Text = $releaseUninstall }
        )) {
        Assert-Match $entry.Text 'function Resolve-InstallBootstrapContextV1 \{' "$($entry.Name) is missing Resolve-InstallBootstrapContextV1"
        Assert-Match $entry.Text 'function Assert-InstallBootstrapContextV1 \{' "$($entry.Name) is missing Assert-InstallBootstrapContextV1"
        Assert-Match $entry.Text 'function Invoke-SubstrateWithInstallContextV1 \{' "$($entry.Name) is missing Invoke-SubstrateWithInstallContextV1"
        Assert-Match $entry.Text '\$InstallContext = Resolve-InstallBootstrapContextV1 -DeclaredPrefix \$Prefix -EncodedCarrier \$InstallBootstrapContextV1' "$($entry.Name) does not resolve install context at entry"
    }

    Assert-True ($devInstall -notmatch '\[string\]\$Prefix = \(Join-Path \$env:LOCALAPPDATA ''Substrate''\)') 'dev install still defaults Prefix from LOCALAPPDATA'
    Assert-True ($devUninstall -notmatch '\[string\]\$Prefix = \(Join-Path \$env:LOCALAPPDATA ''Substrate''\)') 'dev uninstall still defaults Prefix from LOCALAPPDATA'
    Assert-True ($releaseInstall -notmatch '\[string\]\$Prefix = \(Join-Path \$env:LOCALAPPDATA ''Substrate''\)') 'release install still defaults Prefix from LOCALAPPDATA'
    Assert-True ($releaseUninstall -notmatch '\[string\]\$Prefix = \(Join-Path \$env:LOCALAPPDATA ''Substrate''\)') 'release uninstall still defaults Prefix from LOCALAPPDATA'

    Assert-Match $devInstall '\$env:SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT = ''\$commitmentLiteral''' 'dev install profile is missing commitment projection'
    Assert-Match $devInstall '\$env:SUBSTRATE_INSTALL_PRIMARY_USER = ''\$accountLiteral''' 'dev install profile is missing account projection'
    Assert-Match $devInstall '\$env:SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1 = ''\$carrierLiteral''' 'dev install profile is missing carrier projection'
    Assert-Match $releaseInstall '\$env:SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT = ''\$commitmentLiteral''' 'release install profile is missing commitment projection'
    Assert-Match $releaseInstall '\$env:SUBSTRATE_INSTALL_PRIMARY_USER = ''\$accountLiteral''' 'release install profile is missing account projection'
    Assert-Match $releaseInstall '\$env:SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1 = ''\$carrierLiteral''' 'release install profile is missing carrier projection'

    Assert-Match $devInstall 'Invoke-SubstrateWithInstallContextV1\s+`?\s*-SubstrateExe \$substrateBin\s+`?\s*-InstallContext \$InstallContext\s+`?\s*-Arguments @\(''--shim-deploy''\)' 'dev install shim deploy lost explicit carrier wrapper'
    Assert-Match $devUninstall 'Invoke-SubstrateWithInstallContextV1\s+`?\s*-SubstrateExe \$substrateBin\s+`?\s*-InstallContext \$InstallContext\s+`?\s*-Arguments @\(''--no-world'', ''--shim-remove''\)' 'dev uninstall shim-remove lost explicit carrier wrapper'
    Assert-Match $releaseInstall 'Invoke-SubstrateWithInstallContextV1\s+`?\s*-SubstrateExe \$substrateExe\s+`?\s*-InstallContext \$InstallContext\s+`?\s*-Arguments @\(''--shim-deploy''\)' 'release install shim deploy lost explicit carrier wrapper'
    Assert-Match $releaseInstall 'Invoke-SubstrateWithInstallContextV1\s+`?\s*-SubstrateExe \$substrateExe\s+`?\s*-InstallContext \$InstallContext\s+`?\s*-Arguments @\(''world'', ''doctor'', ''--json''\)' 'release install world doctor lost explicit carrier wrapper'
    Assert-Match $releaseInstall '-InstallPrefix \$InstallContext\.SelectedHostPrefix' 'release install lost explicit selected prefix propagation to wsl-warm'
    Assert-Match $releaseInstall '-InstallBootstrapContextV1 \$InstallContext\.EncodedCarrier' 'release install lost explicit carrier propagation to wsl-warm'
    Assert-Match $releaseInstall '\[dry-run\] & ''\$warmScript'' -DistroName ''\$DistroName'' -ProjectPath ''\$versionDir'' -InstallPrefix ''\$\(\$InstallContext\.SelectedHostPrefix\)'' -InstallBootstrapContextV1 <redacted>' 'release install dry-run log must redact carrier'
    Assert-Match $releaseInstall 'throw "World provisioning reported an error: \$sanitizedProvisioningError"' 'release install must fail closed when guarded wsl-warm intake rejects the new parameters'
    Assert-Match $releaseInstall '\$substrateExe = Join-Path \$binDir ''substrate\.exe''' 'release install stopped using the selected installed witness path'

    $resolveReference = Normalize-FunctionSnippetForComparison (Get-FunctionSnippet $devInstall 'function Resolve-InstallBootstrapContextV1 {' 'function Assert-InstallBootstrapContextV1 {')
    $assertReference = Normalize-FunctionSnippetForComparison (Get-FunctionSnippet $devInstall 'function Assert-InstallBootstrapContextV1 {' 'function Invoke-SubstrateWithInstallContextV1 {')
    $invokeReference = Normalize-FunctionSnippetForComparison (Get-FunctionSnippet $devInstall 'function Invoke-SubstrateWithInstallContextV1 {' '$InstallContext = Resolve-InstallBootstrapContextV1')
    foreach ($text in @($devUninstall, $releaseInstall, $releaseUninstall)) {
        Assert-Equal (Normalize-FunctionSnippetForComparison (Get-FunctionSnippet $text 'function Resolve-InstallBootstrapContextV1 {' 'function Assert-InstallBootstrapContextV1 {')) $resolveReference 'Resolve helper drifted across entry surfaces'
        Assert-Equal (Normalize-FunctionSnippetForComparison (Get-FunctionSnippet $text 'function Assert-InstallBootstrapContextV1 {' 'function Invoke-SubstrateWithInstallContextV1 {')) $assertReference 'Assert helper drifted across entry surfaces'
        Assert-Equal (Normalize-FunctionSnippetForComparison (Get-FunctionSnippet $text 'function Invoke-SubstrateWithInstallContextV1 {' '$InstallContext = Resolve-InstallBootstrapContextV1')) $invokeReference 'Invoke helper drifted across entry surfaces'
    }
}

function Test-FrozenBlockHashes {
    $devUninstall = Get-ScriptText 'dev-uninstall-substrate.ps1'
    $releaseInstall = Get-ScriptText 'install-substrate.ps1'
    $releaseUninstall = Get-ScriptText 'uninstall-substrate.ps1'

    $checks = @(
        @{
            Name = 'dev uninstall frozen delete block'
            Text = Get-TextBetweenMarkers $devUninstall '$shimDir = Join-Path $Prefix ''shims''' "Write-Host ''"
            Hash = 'aea6f6c74013558a48d7b6382845f0e864f38d1bc8d3b7975ec0c3ee0491f0a3'
        }
        @{
            Name = 'release install versions/bin block'
            Text = Get-TextBetweenMarkers $releaseInstall '$versionsDir = Join-Path $Prefix ''versions''' '$profileScript = Join-Path $Prefix ''substrate-profile.ps1'''
            Hash = '17a0c147858ebddba5216fed50d3a2f31c26c12c952064eef0912b694aa66d37'
        }
        @{
            Name = 'release install profile snippet block'
            Text = Get-TextBetweenMarkers $releaseInstall '$profileSnippet = @"' '    if ($dry) {`n        Write-Log "[dry-run] Would dot-source $profileScript for this session"'
            Hash = '5abf33efe35dd61132649eb3940d0892f10bfa61063aea66d7fc9fdd05bd85de'
        }
        @{
            Name = 'release install temp cleanup block'
            Text = Get-TailFromMarker $releaseInstall "finally {`n    if (-not \$dry -and (Test-Path \$tempRoot)) {"
            Hash = '9d5ab6914e5d014105916c20bca648c71d454670f169068e19afe33696b96de1'
        }
        @{
            Name = 'release uninstall forwarder/kill block'
            Text = Get-TextBetweenMarkers $releaseUninstall 'Write-Log "Stopping forwarder (if running)"' 'Write-Log "Removing profile snippet"'
            Hash = 'ad6e919429c71c7bee61a00bdfcdf5535b63fce9120f596019c30e806b226fe8'
        }
        @{
            Name = 'release uninstall profile-removal block'
            Text = Get-TextBetweenMarkers $releaseUninstall 'Write-Log "Removing profile snippet"' 'Write-Log "Clearing installation directory: $Prefix"'
            Hash = 'e5f18e0b06ce1e2e78b81f35b0b6e29eca8c1b670cd30fc8c61faaca90b2e41b'
        }
        @{
            Name = 'release uninstall prefix-clear block'
            Text = Get-TextBetweenMarkers $releaseUninstall 'Write-Log "Clearing installation directory: $Prefix"' 'Write-Log "Cleaning shim cache"'
            Hash = 'b6ae5fc06254a3068c5dc2a5361b4035189e2d53e7e4ade5fc2e16edf5727ebd'
        }
        @{
            Name = 'release uninstall shim-cache block'
            Text = Get-TextBetweenMarkers $releaseUninstall 'Write-Log "Cleaning shim cache"' 'Write-Log "Stopping substrate-world-service inside WSL (if present)"'
            Hash = 'e334aa2f923c444659722dfb520538b0fe0c13c690153625f7d262f7ac299a8f'
        }
        @{
            Name = 'release uninstall WSL stop block'
            Text = Get-TextBetweenMarkers $releaseUninstall 'Write-Log "Stopping substrate-world-service inside WSL (if present)"' 'if ($RemoveWSLDistro.IsPresent) {'
            Hash = '1d40e1e45a88dc11b4ff59d97bb47348015e94f8b4ea0ad0ea3e4ad2b16a7e9f'
        }
        @{
            Name = 'release uninstall WSL unregister block'
            Text = Get-TextBetweenMarkers $releaseUninstall 'if ($RemoveWSLDistro.IsPresent) {' 'Write-Log "Uninstall complete. Open a new PowerShell session to refresh PATH."'
            Hash = 'e3ef09f1edbc2c32223746bf9c2bf04c8d122578477f88a40c3e9a37091ea87b'
        }
    )

    foreach ($check in $checks) {
        Assert-Equal (Get-Sha256Hex $check.Text) $check.Hash "$($check.Name) drifted"
    }
}

function Test-W3RustStaticShape {
    $repoRoot = Get-RepoRoot
    $backend = [System.IO.File]::ReadAllText((Join-Path $repoRoot 'crates/world-windows-wsl/src/backend.rs')).Replace("`r`n", "`n")
    $warm = [System.IO.File]::ReadAllText((Join-Path $repoRoot 'crates/world-windows-wsl/src/warm.rs')).Replace("`r`n", "`n")
    $paths = [System.IO.File]::ReadAllText((Join-Path $repoRoot 'crates/world-windows-wsl/src/paths.rs')).Replace("`r`n", "`n")

    Assert-Match $backend 'pub fn new_with_mapping' 'backend.rs is missing new_with_mapping'
    Assert-Match $backend 'validate_wsl_mapping_v1\(&self\.host_carrier, &self\.platform_mapping\)\?;' 'backend.rs must revalidate the stored mapping before readiness'
    Assert-Match $backend 'Transport::NamedPipe \{' 'backend.rs must use named-pipe product transport'
    Assert-Match $backend 'normalize_windows_pipe_path' 'backend.rs must canonicalize the pipe path'
    Assert-Match $backend 'WindowsForwarderScopeV1::derive' 'backend.rs must recompute the Windows forwarder scope'
    Assert-Match $backend 'Command::new\("wsl\.exe"\)' 'backend.rs must perform explicit WSL observation'
    Assert-Match $backend 'Command::new\("pwsh"\)' 'backend.rs must bind validation to the current Windows token and Known Folder'
    Assert-Match $backend 'guest substrate home does not match the live WSL account database' 'backend.rs must reject guest-home drift'
    Assert-True ($backend -notmatch 'DEFAULT_DISTRO') 'backend.rs still selects a default distro'
    Assert-True ($backend -notmatch 'SUBSTRATE_WSL_DISTRO') 'backend.rs still reads ambient SUBSTRATE_WSL_DISTRO'
    Assert-True ($backend -notmatch 'SUBSTRATE_PROJECT_PATH') 'backend.rs still reads ambient SUBSTRATE_PROJECT_PATH'
    Assert-True ($backend -notmatch 'SUBSTRATE_FORWARDER_PIPE') 'backend.rs still selects the runtime pipe from ambient state'
    Assert-True ($backend -notmatch 'forwarder_tcp') 'backend.rs still carries product TCP selection state'

    Assert-Match $warm '-PipePath' 'warm.rs must pass the explicit pipe path'
    Assert-Match $warm '-InstallPrefix' 'warm.rs must pass the explicit install prefix'
    Assert-Match $warm '-InstallBootstrapContextV1' 'warm.rs must pass the explicit install bootstrap carrier'
    Assert-Match $warm '-PlatformBootstrapMappingV1' 'warm.rs must pass the explicit platform bootstrap mapping'
    Assert-Match $warm 'command\.env_remove\("SUBSTRATE_FORWARDER_PIPE"\)' 'warm.rs must scrub ambient pipe selection'
    Assert-Match $warm 'command\.env_remove\("SUBSTRATE_FORWARDER_TCP"\)' 'warm.rs must scrub ambient TCP selection'
    Assert-Match $warm 'command\.env_remove\("LOCALAPPDATA"\)' 'warm.rs must scrub LOCALAPPDATA for the child'
    Assert-Match $warm 'command\.env_remove\("USERPROFILE"\)' 'warm.rs must scrub USERPROFILE for the child'
    Assert-Match $warm 'command\.env_remove\("WSLENV"\)' 'warm.rs must scrub WSLENV for the child'

    Assert-Match $paths '/mnt/unc/' 'paths.rs must preserve explicit UNC conversion'
    Assert-True ($paths -notmatch 'current_dir\(') 'paths.rs must not reach for ambient current_dir'
    Assert-True ($paths -notmatch 'std::env::var') 'paths.rs must not read ambient environment state'
}

function Test-W5RustStaticShape {
    $repoRoot = Get-RepoRoot
    $pipe = [System.IO.File]::ReadAllText((Join-Path $repoRoot 'crates/forwarder/src/pipe.rs')).Replace("`r`n", "`n")
    $bridge = [System.IO.File]::ReadAllText((Join-Path $repoRoot 'crates/forwarder/src/bridge.rs')).Replace("`r`n", "`n")
    $wsl = [System.IO.File]::ReadAllText((Join-Path $repoRoot 'crates/forwarder/src/wsl.rs')).Replace("`r`n", "`n")

    Assert-Match $pipe 'pub fn new\(config: &ForwarderConfig\)' 'pipe.rs must bind listener construction to the verified forwarder configuration'
    Assert-Match $pipe 'bridge::verify_forwarder_projection\(config, None\)' 'pipe.rs must validate the authenticated projection before bind'
    Assert-Match $pipe 'normalize_windows_pipe_path' 'pipe.rs must canonicalize named-pipe identity with the shared Windows grammar'
    Assert-True ($pipe -notmatch 'PipeListener::new\("') 'pipe.rs must not construct listeners from raw string authority'

    Assert-Match $bridge 'config\.validate_mapping\(\)' 'bridge.rs must revalidate the authenticated mapping before session handoff'
    Assert-Match $bridge 'SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1' 'bridge.rs must require the projected install bootstrap carrier'
    Assert-Match $bridge 'SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT' 'bridge.rs must require the projected host commitment'
    Assert-Match $bridge 'host_context_commitment:\s*None' 'bridge.rs must preserve the legacy/non-internal startup path when no authenticated mapping exists'
    Assert-Match $bridge 'wsl::spawn\(\s*&verified\.distro,\s*&verified\.target,\s*verified\.host_context_commitment\.as_deref\(\),' 'bridge.rs must pass exact distro, target, and commitment into wsl::spawn'

    Assert-Match $wsl 'HOST_CONTEXT_COMMITMENT_ENV' 'wsl.rs must project the host commitment into the child'
    Assert-Match $wsl 'WSLENV_EXPORTS' 'wsl.rs must define an exact WSLENV export list'
    Assert-Match $wsl 'GetSystemDirectoryW' 'wsl.rs must resolve wsl.exe from the trusted Windows system directory instead of PATH'
    Assert-Match $wsl 'SUBSTRATE_FORWARDER_TARGET_MODE' 'wsl.rs must explicitly overwrite target mode'
    Assert-Match $wsl 'SUBSTRATE_FORWARDER_TARGET_ENDPOINT' 'wsl.rs must explicitly overwrite target endpoint'
    Assert-Match $wsl 'SUBSTRATE_FORWARDER_CONNECT_TIMEOUT_S' 'wsl.rs must pin the bridge connect timeout'
    Assert-Match $wsl 'SUBSTRATE_FORWARDER_CONNECT_DEADLINE_S' 'wsl.rs must pin the bridge connect deadline'
    Assert-Match $wsl 'SUBSTRATE_FORWARDER_IDLE_AFTER_STDIN_CLOSE_S' 'wsl.rs must pin the bridge stdin-idle timeout'
    Assert-Match $wsl 'SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1' 'wsl.rs must scrub the inherited bootstrap carrier from the guest launch'
    Assert-Match $wsl 'SUBSTRATE_FORWARDER_PIPE' 'wsl.rs must scrub inherited pipe-selector projections from the guest launch'
    Assert-Match $wsl 'build_spawn_spec' 'wsl.rs must build the WSL launch from a deterministic static spec'
}

if ($W5Only) {
    Test-W5RustStaticShape
} elseif ($W3Only) {
    Test-W3RustStaticShape
} elseif ($W2Only) {
    Test-W2PipeAndScopeGoldenVectors
    Test-W2PlatformMappingGoldenVectorAndRevalidation
    Test-W2MappingAndObservationRejections
    Test-W2ForwarderTargetAndLaunchProjection
    Test-W2WarmGuardEntryFlow
    Test-W2DistroFallbackAndRemediation
    Test-W2StaticScriptShape
    Test-W2FrozenBlockHashes
} else {
    Test-GoldenVector
    Test-PathNormalization
    Test-PublicConstructionAndAmbientNonAuthority
    Test-CarrierRejection
    Test-ProjectionHarnessRestoresEnvironment
    Test-W2PipeAndScopeGoldenVectors
    Test-W2PlatformMappingGoldenVectorAndRevalidation
    Test-W2MappingAndObservationRejections
    Test-W2ForwarderTargetAndLaunchProjection
    Test-W2WarmGuardEntryFlow
    Test-W2DistroFallbackAndRemediation
    Test-W2StaticScriptShape
    Test-W3RustStaticShape
    Test-W5RustStaticShape
    Test-StaticScriptShape
    Test-FrozenBlockHashes
    Test-W2FrozenBlockHashes
}

Write-Host 'prefix-mapping-r2-3 tests passed'
