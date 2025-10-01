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

$pipeName = ($PipePath -replace '^\\\\\.\\pipe\\', '')
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

# Read response with deadline
$deadline = [DateTime]::UtcNow.AddSeconds($TimeoutSeconds)
$ms = New-Object System.IO.MemoryStream
$buf = New-Object byte[] 8192
while ([DateTime]::UtcNow -lt $deadline) {
    if (-not ($client.CanRead -and $client.IsConnected)) { break }
    $n = $client.Read($buf, 0, $buf.Length)
    if ($n -gt 0) {
        $ms.Write($buf, 0, $n)
        continue
    }
    Start-Sleep -Milliseconds 50
}
$client.Dispose()

$bytes = $ms.ToArray()
if ($bytes.Length -eq 0) { Write-Fail 'No response'; exit 2 }
$text = [Text.Encoding]::UTF8.GetString($bytes)

# Split headers/body
$sep = "`r`n`r`n"
$i = $text.IndexOf($sep)
if ($i -lt 0) { $i = $text.IndexOf("`n`n") }
if ($i -lt 0) { $i = $text.Length }
$rawHeaders = $text.Substring(0, [Math]::Min($i, $text.Length))
$body = if ($i -lt $text.Length) { $text.Substring($i + $sep.Length) } else { '' }
$statusLine = ($rawHeaders -split "`r?`n")[0]

Write-Output "Status: $statusLine"
Write-Output "Headers:\n$rawHeaders"
Write-Output "Body:\n$($body.Substring(0, [Math]::Min(400, $body.Length)))"

# Enforce expected status if requested
try {
    $code = [int](([regex]::Match($statusLine, 'HTTP/\d\.\d\s+(\d+)').Groups[1].Value))
} catch { $code = 0 }
if ($ExpectStatus -gt 0 -and $code -ne $ExpectStatus) { exit 3 }
exit 0
