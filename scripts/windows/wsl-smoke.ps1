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

# ----- Helpers for replay routing -----
function Test-IsLinuxishSpan {
    param($Span)
    try {
        $vals = @()
        foreach ($k in 'exe','image_path','command','path','cwd','argv') {
            if ($Span.PSObject.Properties.Name -contains $k) {
                $v = $Span.$k
                if ($v -is [System.Array]) { $vals += ($v -join ' ') } else { $vals += $v }
            }
        }
        $joined = ($vals -join "`n")
        # Heuristics: absolute POSIX paths or common Linux locations
        return ($joined -match '(?m)(^|\s)/((bin|usr|home|etc)/|[^\\])')
    } catch { return $false }
}

function Invoke-Replay-Windows {
    param([string]$SpanId, [string]$Exe)
    $tmpOut = Join-Path $env:TEMP ("substrate-replay-" + [guid]::NewGuid().ToString() + ".out")
    $tmpErr = Join-Path $env:TEMP ("substrate-replay-" + [guid]::NewGuid().ToString() + ".err")
    try {
        $args = @('--replay', ('"{0}"' -f $SpanId))
        $proc = Start-Process -FilePath $Exe -ArgumentList $args -NoNewWindow -RedirectStandardOutput $tmpOut -RedirectStandardError $tmpErr -PassThru
        if (-not $proc.WaitForExit(30000)) {
            try { $proc.Kill() } catch {}
            $eo = (Get-Content -LiteralPath $tmpOut -Raw -ErrorAction SilentlyContinue)
            $ee = (Get-Content -LiteralPath $tmpErr -Raw -ErrorAction SilentlyContinue)
            throw "Replay timed out. out=$eo; err=$ee"
        }
        if ($proc.ExitCode -ne 0) {
            $eo = (Get-Content -LiteralPath $tmpOut -Raw -ErrorAction SilentlyContinue)
            $ee = (Get-Content -LiteralPath $tmpErr -Raw -ErrorAction SilentlyContinue)
            throw "Replay failed: out=$eo; err=$ee"
        }
    } finally {
        Remove-Item -LiteralPath $tmpOut,$tmpErr -ErrorAction SilentlyContinue | Out-Null
    }
}

function Invoke-Replay-WSL {
    param([string]$SpanId, [string]$Distro)
    $tmpOut = Join-Path $env:TEMP ("substrate-replay-" + [guid]::NewGuid().ToString() + ".out")
    $tmpErr = Join-Path $env:TEMP ("substrate-replay-" + [guid]::NewGuid().ToString() + ".err")
    try {
        # Try substrate, fall back to substrate-agent
        $linuxCmd = "if command -v substrate >/dev/null 2>&1; then substrate --replay '$SpanId'; " +
                    "elif command -v substrate-agent >/dev/null 2>&1; then substrate-agent --replay '$SpanId'; " +
                    "else echo ERR_NO_AGENT 1>&2; exit 127; fi"

        $proc = Start-Process -FilePath 'wsl.exe' -ArgumentList @('-d', $Distro, '--', 'sh', '-lc', $linuxCmd) -NoNewWindow -RedirectStandardOutput $tmpOut -RedirectStandardError $tmpErr -PassThru
        if (-not $proc.WaitForExit(30000)) {
            try { $proc.Kill() } catch {}
            $eo = (Get-Content -LiteralPath $tmpOut -Raw -ErrorAction SilentlyContinue)
            $ee = (Get-Content -LiteralPath $tmpErr -Raw -ErrorAction SilentlyContinue)
            throw "WSL replay timed out. out=$eo; err=$ee"
        }
        if ($proc.ExitCode -ne 0) {
            $eo = (Get-Content -LiteralPath $tmpOut -Raw -ErrorAction SilentlyContinue)
            $ee = (Get-Content -LiteralPath $tmpErr -Raw -ErrorAction SilentlyContinue)
            if ($ee -match 'ERR_NO_AGENT') {
                throw "WSL replay failed: Substrate CLI/agent not found in distro '$Distro'. Install it or expose replay via the forwarder. stderr=$ee"
            }
            throw "WSL replay failed: out=$eo; err=$ee"
        }
    } finally {
        Remove-Item -LiteralPath $tmpOut,$tmpErr -ErrorAction SilentlyContinue | Out-Null
    }
}

# ----- Warm or preflight -----
if (-not $SkipWarm) {
    Invoke-Step "Warm environment" {
        pwsh -File scripts/windows/wsl-warm.ps1 -DistroName $DistroName -ProjectPath $ProjectPath | Out-Host
    }
}
else {
    # Preflight: ensure forwarder is up when skipping warm
    try {
        $probe = pwsh -File scripts/windows/pipe-status.ps1 -PipePath '\\.\pipe\substrate-agent' -TimeoutSeconds 3 -ExpectStatus 200 2>&1
        $ok = $LASTEXITCODE -eq 0
    } catch { $ok = $false }
    if (-not $ok) {
        Write-Host "[INFO] Forwarder not ready; starting it (SkipWarm preflight)" -ForegroundColor Yellow
        pwsh -File scripts/windows/start-forwarder.ps1 -DistroName $DistroName -PipePath '\\.\pipe\substrate-agent' -ReadyTimeoutSeconds 20 -WaitForExit:$false | Out-Host
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

Invoke-Step "Non-PTY command executes (stdout-only)" {
  $marker = "nonpty-smoke-" + ([guid]::NewGuid().ToString())
  $tmpOut = Join-Path $env:TEMP ("substrate-nonpty-" + [guid]::NewGuid().ToString() + ".out")
  $tmpErr = Join-Path $env:TEMP ("substrate-nonpty-" + [guid]::NewGuid().ToString() + ".err")
  try {
    $cmd = "echo $marker"
    $args = @('--command', ('"{0}"' -f $cmd))
    $proc = Start-Process -FilePath $SubstrateExe `
            -ArgumentList $args -NoNewWindow `
            -RedirectStandardOutput $tmpOut -RedirectStandardError $tmpErr -PassThru
    if (-not $proc.WaitForExit(20000)) { try { $proc.Kill() } catch {}; throw "substrate non-pty timed out" }
    if ($proc.ExitCode -ne 0) { $err = (Get-Content -LiteralPath $tmpErr -Raw -EA SilentlyContinue); throw "substrate exec failed: $err" }
    $out = Get-Content -LiteralPath $tmpOut -Raw
    if ($out -notmatch [regex]::Escape($marker)) { throw "Marker missing in stdout" }
  } finally {
    Remove-Item -LiteralPath $tmpOut,$tmpErr -EA SilentlyContinue | Out-Null
  }
}

Invoke-Step "PTY command" {
  $tmpOut = Join-Path $env:TEMP ("substrate-pty-" + [guid]::NewGuid().ToString() + ".out")
  $tmpErr = Join-Path $env:TEMP ("substrate-pty-" + [guid]::NewGuid().ToString() + ".err")
  try {
    $args = @('--pty','--command','"echo pty-smoke"')
    $proc = Start-Process -FilePath $SubstrateExe `
            -ArgumentList $args -NoNewWindow `
            -RedirectStandardOutput $tmpOut -RedirectStandardError $tmpErr -PassThru
    if (-not $proc.WaitForExit(20000)) { try { $proc.Kill() } catch {}; throw "substrate pty timed out" }
    $output = Get-Content -LiteralPath $tmpOut -Raw -EA SilentlyContinue
    if ($output -notmatch 'pty-smoke') {
      $err = (Get-Content -LiteralPath $tmpErr -Raw -EA SilentlyContinue)
      throw "PTY output missing expected text. stderr=$err"
    }
  } finally {
    Remove-Item -LiteralPath $tmpOut,$tmpErr -EA SilentlyContinue | Out-Null
  }
}

Invoke-Step "Replay (if trace available)" {
    $res = Get-LastTraceEntry -WaitSeconds 0
    if (-not $res) {
        Write-Host "[INFO] No trace available; skipping replay" -ForegroundColor Yellow
        return
    }

    $span = $res.Entry
    if (-not $span.span_id) { throw 'span_id missing' }

    $linuxish = $false
    try { $linuxish = Test-IsLinuxishSpan $span } catch {}

    # Try the most likely executor first, then auto-fallback.
    try {
        if ($linuxish) {
            # Recorded from WSL → replay inside WSL
            Invoke-Replay-WSL -SpanId $span.span_id -Distro $DistroName
        } else {
            # Recorded on Windows (or unknown) → try Windows first
            Invoke-Replay-Windows -SpanId $span.span_id -Exe $SubstrateExe
        }
    } catch {
        $errText = "$_"

        # If Windows replay failed with spawn/path error, it's probably a Linux span → retry in WSL.
        if ($errText -match 'Failed to spawn command' -and ($errText -match 'os error 3' -or $errText -match 'cannot find the path specified')) {
            Write-Host "[INFO] Windows replay couldn't spawn process (likely Linux span). Retrying in WSL..." -ForegroundColor Yellow
            Invoke-Replay-WSL -SpanId $span.span_id -Distro $DistroName
            return
        }

        # If WSL side is missing binaries, treat as best-effort and skip.
        if ($errText -match 'ERR_NO_AGENT') {
            Write-Host "[INFO] WSL replay unavailable (Substrate CLI/agent not found in '$DistroName'). Skipping replay." -ForegroundColor Yellow
            return
        }

        throw
    }
}


Invoke-Step "Forwarder restart resilience" {
    pwsh -File scripts/windows/wsl-stop.ps1 -DistroName $DistroName | Out-Host
    pwsh -File scripts/windows/wsl-warm.ps1 -DistroName $DistroName -ProjectPath $ProjectPath | Out-Host
    & $SubstrateExe --command "echo restart-smoke" | Out-Host
}

Write-Host "Smoke suite completed successfully" -ForegroundColor Green
