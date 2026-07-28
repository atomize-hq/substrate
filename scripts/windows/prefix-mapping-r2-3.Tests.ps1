#!/usr/bin/env pwsh

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

    if ([string]::IsNullOrEmpty($RawPath) -or $RawPath.IndexOf("`0") -ge 0) {
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

    $selectedPrefix = ConvertFrom-CanonicalBase64UrlUtf8 $parsed['selected_host_prefix']
    $home = ConvertFrom-CanonicalBase64UrlUtf8 $parsed['host_substrate_home']
    $root = ConvertFrom-CanonicalBase64UrlUtf8 $parsed['host_substrate_root']
    $account = ConvertFrom-CanonicalBase64UrlUtf8 $parsed['principal_account']
    $sid = ConvertFrom-CanonicalBase64UrlUtf8 $parsed['principal_sid']
    $context = New-WindowsInstallBootstrapContextTest -SelectedPrefix $selectedPrefix -Account $account -Sid $sid

    if (
        $context.HostSubstrateHome -cne $home -or
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

    $snapshot = $null
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
                $script:snapshot = @{
                    SUBSTRATE_HOME                            = [Environment]::GetEnvironmentVariable('SUBSTRATE_HOME')
                    SUBSTRATE_ROOT                            = [Environment]::GetEnvironmentVariable('SUBSTRATE_ROOT')
                    SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT = [Environment]::GetEnvironmentVariable('SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT')
                    SUBSTRATE_INSTALL_PRIMARY_USER            = [Environment]::GetEnvironmentVariable('SUBSTRATE_INSTALL_PRIMARY_USER')
                    SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1    = [Environment]::GetEnvironmentVariable('SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1')
                    R2_3W1_TEST_TEMP                          = [Environment]::GetEnvironmentVariable('R2_3W1_TEST_TEMP')
                }
            }

        Assert-Equal $snapshot.SUBSTRATE_HOME $context.HostSubstrateHome 'projection harness lost SUBSTRATE_HOME'
        Assert-Equal $snapshot.SUBSTRATE_ROOT $context.HostSubstrateRoot 'projection harness lost SUBSTRATE_ROOT'
        Assert-Equal $snapshot.SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT $context.HostContextCommitment 'projection harness lost commitment'
        Assert-Equal $snapshot.SUBSTRATE_INSTALL_PRIMARY_USER $context.CanonicalAccount 'projection harness lost account'
        Assert-Equal $snapshot.SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1 $context.EncodedCarrier 'projection harness lost carrier'
        Assert-Equal $snapshot.R2_3W1_TEST_TEMP 'set-inside-child' 'projection harness lost child-only env'
        Assert-Equal ([Environment]::GetEnvironmentVariable('PATH')) $priorPath 'projection harness failed to restore PATH'
        Assert-Equal ([Environment]::GetEnvironmentVariable('SUBSTRATE_INSTALL_PRIMARY_USER')) 'AMBIENT\User' 'projection harness failed to restore prior value'
        Assert-True ([string]::IsNullOrEmpty([Environment]::GetEnvironmentVariable('R2_3W1_TEST_TEMP'))) 'projection harness failed to restore prior absence'
    } finally {
        [Environment]::SetEnvironmentVariable('PATH', $priorPath)
        [Environment]::SetEnvironmentVariable('SUBSTRATE_INSTALL_PRIMARY_USER', $priorMarker)
        [Environment]::SetEnvironmentVariable('R2_3W1_TEST_TEMP', $priorTemp)
    }
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

Test-GoldenVector
Test-PathNormalization
Test-PublicConstructionAndAmbientNonAuthority
Test-CarrierRejection
Test-ProjectionHarnessRestoresEnvironment
Test-StaticScriptShape
Test-FrozenBlockHashes

Write-Host 'prefix-mapping-r2-3 tests passed'
