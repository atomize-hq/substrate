#!/usr/bin/env pwsh
param(
    [string]$PipePath = '\\.\pipe\substrate-agent',
    [string]$Method = 'GET',
    [string]$Path = '/v1/capabilities',
    [hashtable]$Headers = @{},
    [string]$Body = '',
    [int]$TimeoutSeconds = 10,
    [int]$ExpectStatus = 200
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Write-Info($Message){ Write-Host "[INFO] $Message" -ForegroundColor Cyan }
function Write-Fail($Message){ Write-Host "[FAIL] $Message" -ForegroundColor Red }

if (-not ($Path.StartsWith('/'))) { $Path = '/' + $Path }

# Tolerant pipe-name extraction: works for \\.\pipe\name, .\pipe\name, or just name
if ($PipePath -match '\\pipe\\(?<n>[^\\]+)$') {
    $pipeName = $Matches['n']
} else {
    $pipeName = $PipePath
}
Write-Info ("Using pipe name '{0}' from '{1}'" -f $pipeName, $PipePath)

$client = [System.IO.Pipes.NamedPipeClientStream]::new('.', $pipeName, [System.IO.Pipes.PipeDirection]::InOut, [System.IO.Pipes.PipeOptions]::None)
$client.Connect([Math]::Max(1000, $TimeoutSeconds * 1000))

$enc = [System.Text.Encoding]::ASCII
$writer = New-Object System.IO.StreamWriter($client, $enc, 1024, $true)
$writer.NewLine = "`r`n"
$writer.AutoFlush = $true

# Build request
$req = "{0} {1} HTTP/1.1`r`nHost: localhost`r`nConnection: close`r`nUser-Agent: SubstratePipeProbe/1.0" -f $Method.ToUpperInvariant(), $Path
if ($Body.Length -gt 0) {
    $Headers['Content-Length'] = [Text.Encoding]::UTF8.GetByteCount($Body)
}
foreach ($k in $Headers.Keys) { $req += ("`r`n{0}: {1}" -f $k, $Headers[$k]) }
$req += "`r`n`r`n"

$writer.Write($req)
if ($Body.Length -gt 0) { $writer.Write($Body) }
$writer.Flush()

# Read response with real timeout (header-first, then optional body)
$deadline = [DateTime]::UtcNow.AddSeconds([Math]::Max(1, $TimeoutSeconds))

# Tighten IO timeouts on the underlying stream
try { $client.ReadTimeout = 1000; $client.WriteTimeout = 2000 } catch {}

$enc   = [System.Text.Encoding]::ASCII
$reader = New-Object System.IO.StreamReader($client, $enc, $false, 1024, $true)

# 1) Read status line + headers
$statusLine = $null
$respHeaders = New-Object System.Collections.Generic.List[string]
while ([DateTime]::UtcNow -lt $deadline) {
    try { $line = $reader.ReadLine() } catch [System.IO.IOException] { continue }
    if ($null -eq $line) { continue }
    if ($statusLine -eq $null) { $statusLine = $line; continue }
    if ($line -eq '') { break }
    [void]$respHeaders.Add($line)
}
if (-not $statusLine) {
    Write-Fail "No status line received before timeout"
    $client.Dispose(); exit 2
}

# 2) Parse content-length (optional)
$cl = $null
foreach ($h in $respHeaders) { if ($h -match '^[Cc]ontent-[Ll]ength:\s*(\d+)\s*$') { $cl = [int]$matches[1]; break } }

# 3) Read body up to content-length (or until stream closes / deadline)
$bodyBuilder = New-Object System.Text.StringBuilder
if ($cl -ne $null -and $cl -gt 0) {
    $remaining = $cl
    $buf = New-Object byte[] 8192
    while ($remaining -gt 0 -and [DateTime]::UtcNow -lt $deadline) {
        try { $n = $reader.BaseStream.Read($buf, 0, [Math]::Min($buf.Length, $remaining)) } catch [System.IO.IOException] { continue }
        if ($n -le 0) { break }
        [void]$bodyBuilder.Append([Text.Encoding]::UTF8.GetString($buf,0,$n))
        $remaining -= $n
    }
} else {
    # No length -> attempt a single slice within timeout
    $buf = New-Object byte[] 8192
    try { $n = $reader.BaseStream.Read($buf, 0, $buf.Length) } catch [System.IO.IOException] { $n = 0 }
    if ($n -gt 0) { [void]$bodyBuilder.Append([Text.Encoding]::UTF8.GetString($buf,0,$n)) }
}

# 4) Print and enforce expected status
Write-Output ("Status: {0}" -f $statusLine)
if ($respHeaders.Count -gt 0) {
    Write-Output "Headers:"
    ($respHeaders -join "`r`n") | Write-Output
}
$bodyText = $bodyBuilder.ToString()
Write-Output "Body:"
Write-Output ($bodyText.Substring(0, [Math]::Min(400, $bodyText.Length)))

# Enforce expected status if requested
try { $code = [int](([regex]::Match($statusLine, 'HTTP/\d\.\d\s+(\d+)').Groups[1].Value)) } catch { $code = 0 }
$client.Dispose()
if ($ExpectStatus -gt 0 -and $code -ne $ExpectStatus) { exit 3 } else { exit 0 }
