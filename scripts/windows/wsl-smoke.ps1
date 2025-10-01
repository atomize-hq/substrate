#!/usr/bin/env pwsh
param(
    [string]$DistroName = 'substrate-wsl',
    # Anchor to repo root relative to this script
    [string]$ProjectPath = $(Convert-Path (Join-Path $PSScriptRoot '..\..')),
    [switch]$SkipWarm
)

function Convert-ToWslPath {
    param([Parameter(Mandatory=$true)][string]$WindowsPath)
    $p = (Resolve-Path -LiteralPath $WindowsPath).Path
    $drive = $p.Substring(0,1).ToLower()
    $rest  = $p.Substring(2) -replace '\\','/'
    "/mnt/$drive/$rest"
}
function Get-TraceCandidates {
    $c = @()

    # Prefer LocalAppData\Substrate
    if ($env:LOCALAPPDATA) {
        $root = Join-Path $env:LOCALAPPDATA 'Substrate'
        if (Test-Path $root) {
            # common fixed name
            $c += (Join-Path $root 'trace.jsonl')
            # rotated or nested traces
            $c += (Get-ChildItem -Path $root -Filter 'trace*.jsonl' -Recurse -ErrorAction SilentlyContinue |
                   Sort-Object LastWriteTime -Descending |
                   Select-Object -ExpandProperty FullName)
        }
    }

    # Also check legacy/UserProfile location
    if ($env:USERPROFILE) {
        $root = Join-Path $env:USERPROFILE '.substrate'
        if (Test-Path $root) {
            $c += (Join-Path $root 'trace.jsonl')
            $c += (Get-ChildItem -Path $root -Filter 'trace*.jsonl' -Recurse -ErrorAction SilentlyContinue |
                   Sort-Object LastWriteTime -Descending |
                   Select-Object -ExpandProperty FullName)
        }
    }

    # unique, prefer newest first
    $c | Where-Object { $_ } | Select-Object -Unique
}

function Get-LastTraceEntry {
    param(
        [int]$WaitSeconds = 3,
        [Nullable[datetime]]$Since = $null
    )
    $deadline = (Get-Date).AddSeconds([Math]::Max(0, $WaitSeconds))
    $result = $null

    do {
        foreach ($p in Get-TraceCandidates) {
            if (-not (Test-Path -LiteralPath $p)) { continue }

            # If caller provided a "since", skip files older than that moment
            if ($Since) {
                try {
                    $fi = Get-Item -LiteralPath $p -ErrorAction SilentlyContinue
                    if ($fi -and $fi.LastWriteTimeUtc -lt $Since.Value.ToUniversalTime()) { continue }
                } catch {}
            }

            $line = Get-Content -LiteralPath $p -Tail 1 -ErrorAction SilentlyContinue
            if (-not $line) { continue }

            try { $obj = $line | ConvertFrom-Json } catch { $obj = $null }
            if ($obj) {
                $result = @{ Entry = $obj; Path = $p }
                break
            }
        }

        if (-not $result) { Start-Sleep -Milliseconds 200 }
    } while (-not $result -and (Get-Date) -lt $deadline)

    return $result
}

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Invoke-Step {
    param([string]$Name, [scriptblock]$Block)
    Write-Host "[STEP] $Name" -ForegroundColor Cyan
    try { & $Block; Write-Host "[PASS] $Name" -ForegroundColor Green }
    catch { Write-Host "[FAIL] $Name - $_" -ForegroundColor Red; throw }
}

function Resolve-SubstrateExe {
    param([string]$BaseDir)

    # 0) PATH wins
    $cmd = Get-Command substrate -ErrorAction SilentlyContinue
    if ($cmd) { return $cmd.Path }

    # 1) Explicit override
    if ($env:SUBSTRATE_EXE -and (Test-Path -LiteralPath $env:SUBSTRATE_EXE)) {
        return (Resolve-Path -LiteralPath $env:SUBSTRATE_EXE).Path
    }

    # Build candidate roots: user-provided (if valid) + script-anchored repo root
    $roots = @()
    if ($BaseDir -and (Test-Path -LiteralPath $BaseDir)) {
        $roots += (Convert-Path -LiteralPath $BaseDir)
    }
    $roots += (Convert-Path (Join-Path $PSScriptRoot '..\..'))
    $roots = $roots | Select-Object -Unique

    foreach ($root in $roots) {
        $candidates = @(
            (Join-Path $root 'target\release\substrate.exe'),
            (Join-Path $root 'target\debug\substrate.exe')
        )
        foreach ($p in $candidates) {
            if (Test-Path -LiteralPath $p) { return (Resolve-Path -LiteralPath $p).Path }
        }
        $hit = Get-ChildItem -Path (Join-Path $root 'target') -Filter 'substrate.exe' -Recurse -ErrorAction SilentlyContinue | Select-Object -First 1
        if ($hit) { return $hit.FullName }
    }

    $hint = @(
        "substrate CLI not found.",
        "Checked roots:",
        ($roots | ForEach-Object { "  - $_" }),
        "Set `$env:SUBSTRATE_EXE to the full path, or ensure the binary exists under <repo>\target\{debug|release}\substrate.exe."
    ) -join "`n"
    throw $hint
}


$SubstrateExe = Resolve-SubstrateExe -BaseDir $ProjectPath
Write-Host "[INFO] Using substrate.exe at: $SubstrateExe" -ForegroundColor Yellow



function Invoke-Step {
    param(
        [string]$Name,
        [scriptblock]$Block
    )
    Write-Host "[STEP] $Name" -ForegroundColor Cyan
    try {
        & $Block
        Write-Host "[PASS] $Name" -ForegroundColor Green
    } catch {
        Write-Host "[FAIL] $Name - $_" -ForegroundColor Red
        throw
    }
}

if (-not $SkipWarm) {
    Invoke-Step "Warm environment" {
        pwsh -File scripts/windows/wsl-warm.ps1 -DistroName $DistroName -ProjectPath $ProjectPath | Out-Host
    }
}

Invoke-Step "Forwarder pipe capabilities (HTTP 200)" {
    $resp = pwsh -File scripts/windows/pipe-status.ps1 `
        -PipePath '\\.\pipe\substrate-agent' `
        -TimeoutSeconds 8 `
        -ExpectStatus 200 2>&1

    if ($LASTEXITCODE -ne 0) { throw ($resp -join "`n") }

    # Show only the Status: line, ignore any [INFO] lines from the child process
    $respText   = [string]::Join("`n", @($resp))
    $statusLine = ($respText -split "`r?`n") |
                  Where-Object { $_ -match '^Status:\s' } |
                  Select-Object -First 1
    if (-not $statusLine) { $statusLine = ($respText -split "`r?`n")[-1] }
    $statusLine | Out-Host
}


Invoke-Step "Doctor checks" {
    $resp = pwsh -File scripts/windows/wsl-doctor.ps1 -DistroName $DistroName 2>&1
    if ($LASTEXITCODE -ne 0) { throw ($resp -join "`n") }
    # success: stay quiet to avoid log pollution
}


Invoke-Step "Non-PTY command produces world span" {
    $marker  = [guid]::NewGuid().ToString()

    # Write into the repo root so we can verify from Windows
    $winPath = Join-Path $ProjectPath 'win_smoke.txt'
    $wslPath = Convert-ToWslPath $winPath

    # Build a bash one-liner safely:
    # result: bash -lc 'printf %s "MARKER" > "/mnt/c/.../win_smoke.txt"'
    $bashInner = "printf %s `"$marker`" > `"$wslPath`""
    $cmd       = "bash -lc '$bashInner'"

    $out = & $SubstrateExe -c $cmd
    if ($LASTEXITCODE -ne 0) { throw "substrate exec failed: $out" }

    # Wait briefly for the file to appear, then assert contents
    $deadline = (Get-Date).AddSeconds(3)
    while (-not (Test-Path -LiteralPath $winPath) -and (Get-Date) -lt $deadline) {
        Start-Sleep -Milliseconds 150
    }
    if (-not (Test-Path -LiteralPath $winPath)) {
        throw "win_smoke.txt not created at $winPath"
    }
    $content = Get-Content -LiteralPath $winPath -Raw
    if ($content -ne $marker) { throw "win_smoke.txt content mismatch" }
}





Invoke-Step "PTY command" {
    $output = & $SubstrateExe --pty -c "bash -lc 'echo pty-smoke'"
    if ($output -notmatch 'pty-smoke') { throw 'PTY output missing expected text' }
}

Invoke-Step "Replay (if trace available)" {
    $res = Get-LastTraceEntry -WaitSeconds 0
    if (-not $res) {
        Write-Host "[INFO] No trace available; skipping replay" -ForegroundColor Yellow
        return
    }
    $span = $res.Entry
    if (-not $span.span_id) { throw 'span_id missing' }
    $replay = & $SubstrateExe replay $span.span_id 2>&1
    if ($LASTEXITCODE -ne 0) { throw "Replay failed: $replay" }
}



Invoke-Step "Forwarder restart resilience" {
    pwsh -File scripts/windows/wsl-stop.ps1 -DistroName $DistroName | Out-Host
    pwsh -File scripts/windows/wsl-warm.ps1 -DistroName $DistroName -ProjectPath $ProjectPath | Out-Host
    & $SubstrateExe -c "echo restart-smoke" | Out-Host
}


Write-Host "Smoke suite completed successfully" -ForegroundColor Green
