#!/usr/bin/env pwsh
param(
    [string]$DistroName = 'substrate-wsl',
    # Anchor to repo root relative to this script
    [string]$ProjectPath = $(Convert-Path (Join-Path $PSScriptRoot '..\..')),
    [switch]$SkipWarm,
    [switch]$WorldDisabledDiagnostics
)

if (-not $IsWindows) {
    Write-Host "SKIP: world-disabled-diagnostics Windows smoke (not Windows)"
    exit 0
}

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

function New-HostGatewaySmokeAuthFixture {
    $homeRoot = $env:HOME
    $cleanupHome = $false
    if (-not $homeRoot) {
        if ($env:USERPROFILE) {
            $homeRoot = $env:USERPROFILE
        } else {
            $homeRoot = Join-Path $env:TEMP ("substrate-gateway-auth-" + [guid]::NewGuid().ToString())
            New-Item -ItemType Directory -Force -Path $homeRoot | Out-Null
            $cleanupHome = $true
        }
    }

    $authDir = Join-Path $homeRoot '.codex'
    $authPath = Join-Path $authDir 'auth.json'
    $created = $false
    if (-not (Test-Path -LiteralPath $authPath)) {
        New-Item -ItemType Directory -Force -Path $authDir | Out-Null
        @'
{
  "account_id": "acct_smoke",
  "access_token": "header.payload.signature"
}
 '@ | Set-Content -LiteralPath $authPath -NoNewline
        $created = $true
    }

    return [pscustomobject]@{
        Home = $homeRoot
        AuthPath = $authPath
        Created = $created
        CleanupHome = $cleanupHome
    }
}

function Remove-HostGatewaySmokeAuth {
    param($Fixture)

    if ($Fixture.Created -and (Test-Path -LiteralPath $Fixture.AuthPath)) {
        Remove-Item -LiteralPath $Fixture.AuthPath -Force
    }
    if ($Fixture.CleanupHome -and (Test-Path -LiteralPath $Fixture.Home)) {
        Remove-Item -LiteralPath $Fixture.Home -Recurse -Force
    }
}

function Invoke-GatewayLifecycleProof {
    param(
        [string]$Exe,
        [string]$DistroName
    )

    $fixture = New-HostGatewaySmokeAuthFixture
    $previousHome = $env:HOME
    $previousUserProfile = $env:USERPROFILE
    $env:HOME = $fixture.Home
    $env:USERPROFILE = $fixture.Home
    try {
        $syncOutput = & $Exe world gateway sync 2>&1 | Out-String
        if ($LASTEXITCODE -ne 0) {
            throw "gateway sync failed: $syncOutput"
        }

        $statusText = & $Exe world gateway status --json 2>&1 | Out-String
        if ($LASTEXITCODE -ne 0) {
            throw "gateway status --json failed: $statusText"
        }
        $status = $statusText.Trim() | ConvertFrom-Json
        if ($status.status -ne 'available') {
            throw "gateway status did not report available: $statusText"
        }
        if (-not $status.client_wiring) {
            throw "gateway status omitted client wiring: $statusText"
        }
        if ($status.client_wiring.openai_base_url -ne $status.client_wiring.anthropic_base_url) {
            throw "gateway status wiring diverged unexpectedly: $statusText"
        }

        $restartOutput = & $Exe world gateway restart 2>&1 | Out-String
        if ($LASTEXITCODE -ne 0) {
            throw "gateway restart failed: $restartOutput"
        }

        $statusText = & $Exe world gateway status --json 2>&1 | Out-String
        if ($LASTEXITCODE -ne 0) {
            throw "gateway status --json after restart failed: $statusText"
        }
        $status = $statusText.Trim() | ConvertFrom-Json
        if ($status.status -ne 'available') {
            throw "gateway status after restart did not report available: $statusText"
        }

        $baseUrl = [string]$status.client_wiring.openai_base_url
        $uri = [Uri]$baseUrl
        if ($uri.Host -ne '127.0.0.1' -or $uri.Port -le 0) {
            throw "unexpected gateway base URL: $baseUrl"
        }

        $healthText = & wsl -d $DistroName -- bash -lc "curl -fsS http://127.0.0.1:$($uri.Port)/health" 2>&1 | Out-String
        if ($LASTEXITCODE -ne 0) {
            throw "gateway /health probe failed: $healthText"
        }
        $health = $healthText.Trim() | ConvertFrom-Json
        if ($health.status -ne 'ok' -or $health.service -ne 'substrate-gateway') {
            throw "unexpected gateway /health payload: $healthText"
        }
    } finally {
        if ($null -ne $previousHome) {
            $env:HOME = $previousHome
        } else {
            Remove-Item Env:HOME -ErrorAction SilentlyContinue
        }
        if ($null -ne $previousUserProfile) {
            $env:USERPROFILE = $previousUserProfile
        } else {
            Remove-Item Env:USERPROFILE -ErrorAction SilentlyContinue
        }
        Remove-HostGatewaySmokeAuth -Fixture $fixture
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

$projectHasCargo = Test-Path (Join-Path $ProjectPath 'Cargo.toml')
if (-not $projectHasCargo) {
    Invoke-Step "Packaged guest world artifacts present" {
        $worldAgentArtifact = Join-Path $ProjectPath 'bin\linux\world-agent'
        $gatewayArtifact = Join-Path $ProjectPath 'bin\linux\substrate-gateway'
        if (-not (Test-Path -LiteralPath $worldAgentArtifact)) {
            throw "expected packaged guest world-agent artifact at $worldAgentArtifact"
        }
        if (-not (Test-Path -LiteralPath $gatewayArtifact)) {
            throw "expected packaged guest gateway artifact at $gatewayArtifact"
        }
    }
}

function Invoke-WorldDisabledDiagnosticsSmoke {
    param([string]$Exe)

    $sliceId = if ($env:SUBSTRATE_SMOKE_SLICE_ID -and $env:SUBSTRATE_SMOKE_SLICE_ID.Trim() -ne "") { $env:SUBSTRATE_SMOKE_SLICE_ID } else { "WDD2" }
    if (@("WDD0", "WDD1", "WDD2") -notcontains $sliceId) {
        Write-Error "FAIL: unsupported SUBSTRATE_SMOKE_SLICE_ID=$sliceId (expected WDD0, WDD1, or WDD2)"
        exit 2
    }

    function New-TempDir([string]$Prefix) {
        $dir = Join-Path ([System.IO.Path]::GetTempPath()) ($Prefix + "-" + [guid]::NewGuid().ToString("N"))
        New-Item -ItemType Directory -Path $dir | Out-Null
        return $dir
    }

    function Invoke-Substrate {
        param(
            [string]$Label,
            [int]$ExpectedExit,
            [string[]]$Args,
            [hashtable]$Env = @{}
        )

        $stdout = New-TemporaryFile
        $stderr = New-TemporaryFile
        $old = @{}

        foreach ($k in $Env.Keys) {
            $current = Get-Item -Path "Env:$k" -ErrorAction SilentlyContinue
            $old[$k] = if ($current) { $current.Value } else { $null }
            Set-Item -Path "Env:$k" -Value $Env[$k]
        }

        try {
            & $Exe @Args 1> $stdout.FullName 2> $stderr.FullName
            $rc = $LASTEXITCODE
        } finally {
            foreach ($k in $Env.Keys) {
                if ($null -eq $old[$k]) {
                    Remove-Item -Path "Env:$k" -ErrorAction SilentlyContinue
                } else {
                    Set-Item -Path "Env:$k" -Value $old[$k]
                }
            }
        }

        $out = Get-Content -Raw -Path $stdout.FullName
        $err = Get-Content -Raw -Path $stderr.FullName
        Remove-Item -Force $stdout.FullName, $stderr.FullName -ErrorAction SilentlyContinue

        if ($rc -ne $ExpectedExit) {
            Write-Error "FAIL: $Label expected exit=$ExpectedExit, got=$rc`nSTDOUT:`n$out`nSTDERR:`n$err"
            exit 1
        }

        return [pscustomobject]@{
            ExitCode = $rc
            Stdout   = $out
            Stderr   = $err
        }
    }

    function Require-Contains([string]$Haystack, [string]$Needle) {
        if ($Haystack -notlike "*$Needle*") {
            Write-Error "FAIL: missing expected substring: $Needle"
            exit 1
        }
    }

    function Require-NotContains([string]$Haystack, [string]$Needle) {
        if ($Haystack -like "*$Needle*") {
            Write-Error "FAIL: found forbidden substring: $Needle"
            exit 1
        }
    }

    function Test-Wdd0ConfigResolutionError {
        $home = New-TempDir "wdd-home"
        Set-Content -Encoding utf8 -Path (Join-Path $home "config.yaml") -Value "world: [`n"

        $r = Invoke-Substrate -Label "shim doctor invalid config" -ExpectedExit 2 -Args @("shim", "doctor") -Env @{ SUBSTRATE_HOME = $home }
        Require-Contains ($r.Stdout + $r.Stderr) "config.yaml"

        $r = Invoke-Substrate -Label "shim doctor --json invalid config" -ExpectedExit 2 -Args @("shim", "doctor", "--json") -Env @{ SUBSTRATE_HOME = $home }
        Require-Contains ($r.Stdout + $r.Stderr) "config.yaml"

        $r = Invoke-Substrate -Label "health invalid config" -ExpectedExit 2 -Args @("health") -Env @{ SUBSTRATE_HOME = $home }
        Require-Contains ($r.Stdout + $r.Stderr) "config.yaml"

        $r = Invoke-Substrate -Label "health --json invalid config" -ExpectedExit 2 -Args @("health", "--json") -Env @{ SUBSTRATE_HOME = $home }
        Require-Contains ($r.Stdout + $r.Stderr) "config.yaml"

        Remove-Item -Recurse -Force $home -ErrorAction SilentlyContinue
    }

    function Test-Wdd1ShimDoctorDisabledAndBroken {
        $home = New-TempDir "wdd-home"

        $r = Invoke-Substrate -Label "shim doctor disabled" -ExpectedExit 0 -Args @("shim", "doctor") -Env @{ SUBSTRATE_HOME = $home; SUBSTRATE_OVERRIDE_WORLD = "disabled" }
        Require-Contains $r.Stdout 'World backend:'
        Require-Contains $r.Stdout '  Status: disabled'
        Require-Contains $r.Stdout '  Next: run `substrate world enable` to provision'
        Require-Contains $r.Stdout 'World deps:'
        Require-Contains $r.Stdout '  Status: skipped (world disabled)'
        Require-NotContains $r.Stdout '  Error:'

        $r = Invoke-Substrate -Label "shim doctor --json disabled" -ExpectedExit 0 -Args @("shim", "doctor", "--json") -Env @{ SUBSTRATE_HOME = $home; SUBSTRATE_OVERRIDE_WORLD = "disabled" }
        $d = $r.Stdout | ConvertFrom-Json
        if ($d.world.status -ne "disabled") { Write-Error "FAIL: expected .world.status=disabled"; exit 1 }
        if ($d.world_deps.status -ne "skipped_disabled") { Write-Error "FAIL: expected .world_deps.status=skipped_disabled"; exit 1 }
        foreach ($k in @("error", "stderr", "exit_code", "details")) {
            if ($d.world.PSObject.Properties.Name -contains $k) { Write-Error "FAIL: world.$k must be omitted"; exit 1 }
        }
        foreach ($k in @("error", "report")) {
            if ($d.world_deps.PSObject.Properties.Name -contains $k) { Write-Error "FAIL: world_deps.$k must be omitted"; exit 1 }
        }

        $brokenPipe = "\\\\.\\pipe\\substrate-agent-wdd-broken-$([guid]::NewGuid())"
        $r = Invoke-Substrate -Label "shim doctor enabled-but-broken" -ExpectedExit 0 -Args @("--world", "shim", "doctor") -Env @{
            SUBSTRATE_HOME           = $home
            SUBSTRATE_FORWARDER_PIPE = $brokenPipe
            SUBSTRATE_FORWARDER_TCP  = "0"
        }
        Require-Contains $r.Stdout 'World backend:'
        Require-Contains $r.Stdout '  Status: needs attention'
        Require-Contains $r.Stdout '  Details:'
        Require-Contains $r.Stdout '  Applied:'
        Require-NotContains $r.Stdout '  Status: disabled'

        $r = Invoke-Substrate -Label "shim doctor --json enabled-but-broken" -ExpectedExit 0 -Args @("--world", "shim", "doctor", "--json") -Env @{
            SUBSTRATE_HOME           = $home
            SUBSTRATE_FORWARDER_PIPE = $brokenPipe
            SUBSTRATE_FORWARDER_TCP  = "0"
        }
        $d = $r.Stdout | ConvertFrom-Json
        if ($d.world.status -ne "needs_attention") { Write-Error "FAIL: expected .world.status=needs_attention"; exit 1 }
        if (-not $d.world.details) { Write-Error "FAIL: expected world.details object"; exit 1 }
        if ($d.world_deps.status -ne "error") { Write-Error "FAIL: expected .world_deps.status=error"; exit 1 }
        if (-not $d.world_deps.report) { Write-Error "FAIL: expected world_deps.report object"; exit 1 }

        Remove-Item -Recurse -Force $home -ErrorAction SilentlyContinue
    }

    function Test-Wdd2HealthDisabledAndBroken {
        $home = New-TempDir "wdd-home"

        $r = Invoke-Substrate -Label "health disabled" -ExpectedExit 0 -Args @("health") -Env @{ SUBSTRATE_HOME = $home; SUBSTRATE_OVERRIDE_WORLD = "disabled" }
        Require-Contains $r.Stdout 'World backend: disabled'
        Require-Contains $r.Stdout '  Next: run `substrate world enable` to provision'
        Require-Contains $r.Stdout 'World deps: skipped (world disabled)'
        Require-NotContains $r.Stdout 'substrate world deps current'

        $r = Invoke-Substrate -Label "health --json disabled" -ExpectedExit 0 -Args @("health", "--json") -Env @{ SUBSTRATE_HOME = $home; SUBSTRATE_OVERRIDE_WORLD = "disabled" }
        $h = $r.Stdout | ConvertFrom-Json
        if ($h.shim.world.status -ne "disabled") { Write-Error "FAIL: expected .shim.world.status=disabled"; exit 1 }
        if ($h.shim.world_deps.status -ne "skipped_disabled") { Write-Error "FAIL: expected .shim.world_deps.status=skipped_disabled"; exit 1 }
        if ($null -ne $h.summary.world_ok) { Write-Error "FAIL: expected summary.world_ok=null"; exit 1 }
        if ($h.summary.PSObject.Properties.Name -contains "world_error") { Write-Error "FAIL: summary.world_error must be omitted"; exit 1 }
        if ($h.summary.PSObject.Properties.Name -contains "world_deps_error") { Write-Error "FAIL: summary.world_deps_error must be omitted"; exit 1 }
        if (($h.summary.world_deps_missing | ConvertTo-Json -Compress) -ne "[]") { Write-Error "FAIL: expected summary.world_deps_missing=[]"; exit 1 }
        if (($h.summary.world_deps_blocked | ConvertTo-Json -Compress) -ne "[]") { Write-Error "FAIL: expected summary.world_deps_blocked=[]"; exit 1 }

        $brokenPipe = "\\\\.\\pipe\\substrate-agent-wdd-broken-$([guid]::NewGuid())"
        $r = Invoke-Substrate -Label "health enabled-but-broken" -ExpectedExit 0 -Args @("--world", "health") -Env @{
            SUBSTRATE_HOME           = $home
            SUBSTRATE_FORWARDER_PIPE = $brokenPipe
            SUBSTRATE_FORWARDER_TCP  = "0"
        }
        Require-Contains $r.Stdout 'World backend: needs attention'
        Require-Contains $r.Stdout 'World deps: unavailable'
        Require-Contains $r.Stdout 'Overall status: attention required'
        Require-Contains $r.Stdout '  - world backend health check failed'
        Require-NotContains $r.Stdout 'World backend: disabled'

        $r = Invoke-Substrate -Label "health --json enabled-but-broken" -ExpectedExit 0 -Args @("--world", "health", "--json") -Env @{
            SUBSTRATE_HOME           = $home
            SUBSTRATE_FORWARDER_PIPE = $brokenPipe
            SUBSTRATE_FORWARDER_TCP  = "0"
        }
        $h = $r.Stdout | ConvertFrom-Json
        if ($h.shim.world.status -ne "needs_attention") { Write-Error "FAIL: expected .shim.world.status=needs_attention"; exit 1 }
        if ($h.summary.world_ok -ne $false) { Write-Error "FAIL: expected summary.world_ok=false"; exit 1 }
        if ($h.shim.world_deps.status -ne "error") { Write-Error "FAIL: expected .shim.world_deps.status=error"; exit 1 }
        if (-not $h.summary.world_deps_error -or $h.summary.world_deps_error.Trim() -eq "") { Write-Error "FAIL: expected non-empty summary.world_deps_error"; exit 1 }

        Remove-Item -Recurse -Force $home -ErrorAction SilentlyContinue
    }

    Write-Host "INFO: world-disabled-diagnostics Windows smoke slice=$sliceId"
    Test-Wdd0ConfigResolutionError
    if ($sliceId -eq "WDD0") {
        Write-Host "OK: world-disabled-diagnostics Windows smoke ($sliceId)"
        return
    }

    Test-Wdd1ShimDoctorDisabledAndBroken
    if ($sliceId -eq "WDD1") {
        Write-Host "OK: world-disabled-diagnostics Windows smoke ($sliceId)"
        return
    }

    Test-Wdd2HealthDisabledAndBroken
    Write-Host "OK: world-disabled-diagnostics Windows smoke ($sliceId)"
}

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

# ----- Disabled-diagnostics conformance -----
if ($WorldDisabledDiagnostics) {
    Invoke-WorldDisabledDiagnosticsSmoke -Exe $SubstrateExe
    exit 0
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

Invoke-Step "Gateway lifecycle proof" {
    Invoke-GatewayLifecycleProof -Exe $SubstrateExe -DistroName $DistroName
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
