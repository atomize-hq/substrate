#!/usr/bin/env pwsh
param(
    [string]$PipePath = '\\.\pipe\substrate-agent',
    [string]$Method = 'GET',
    [string]$Path = '/v1/capabilities',
    [int]$TimeoutSeconds = 8,
    [int]$ExpectStatus = 200,
    [switch]$TraceParse
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Write-Info($Message){ Write-Host "[INFO] $Message" -ForegroundColor Cyan }
function Write-Fail($Message){ Write-Host "[FAIL] $Message" -ForegroundColor Red }

function Get-PipeNameFromPath {
    param([Parameter(Mandatory=$true)][string]$Path)
    # Normalize slashes and trim quotes/whitespace
    $raw  = $Path.Trim().Trim('"',[char]39)                         # FIX: trim " and ' (39), NOT backslash
    $norm = $raw.Replace('/', [char]92)                             # FIX: avoid regex; 92 = '\'

    # Preferred: segment after last '\pipe\'
    $marker = '\pipe\'                                              # FIX: match the real canonical prefix
    $idx = $norm.LastIndexOf($marker)
    if ($idx -ge 0) {
        $name = $norm.Substring($idx + $marker.Length)
        if (-not [string]::IsNullOrWhiteSpace($name)) { return $name }
    }
    # Fallback: last path segment
    $parts = $norm -split '[\\/]'
    if ($parts.Length -gt 0) { return $parts[-1] }
    return $norm
}

function Get-HexBytes([string]$s) {
    -join (([Text.Encoding]::UTF8.GetBytes($s) | ForEach-Object { $_.ToString('X2') }) -join ' ')
}

if (-not ($Path.StartsWith('/'))) { $Path = '/' + $Path }

$pipeName = Get-PipeNameFromPath -Path $PipePath
if ($TraceParse) {
    Write-Info ("PipePath UTF8 hex: {0}" -f (([Text.Encoding]::UTF8.GetBytes($PipePath) | ForEach-Object { $_.ToString('X2') }) -join ' '))
    Write-Info ("Extracted pipe name: '{0}'" -f $pipeName)
}
Write-Info ("Connecting to named pipe '{0}' from '{1}'" -f $pipeName, $PipePath)

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
