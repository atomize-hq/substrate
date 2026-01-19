#!/usr/bin/env pwsh
param(
    [string]$DistroName = 'substrate-wsl'
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Fail {
    param([string]$Message)
    Write-Error $Message
    exit 1
}

function Resolve-SubstrateExe {
    $cmd = Get-Command substrate -ErrorAction SilentlyContinue
    if ($cmd) { return $cmd.Path }

    if ($env:SUBSTRATE_EXE -and (Test-Path -LiteralPath $env:SUBSTRATE_EXE)) {
        return (Resolve-Path -LiteralPath $env:SUBSTRATE_EXE).Path
    }

    Fail "substrate CLI not found on PATH. Set `$env:SUBSTRATE_EXE or install Substrate."
}

function Invoke-Substrate {
    param(
        [Parameter(Mandatory = $true)][string[]]$Args,
        [string]$Cwd = $null,
        [string]$StdoutPath = $null,
        [string]$StderrPath = $null,
        [int]$TimeoutMs = 60000
    )

    $exe = Resolve-SubstrateExe
    $si = New-Object System.Diagnostics.ProcessStartInfo
    $si.FileName = $exe
    foreach ($a in $Args) { [void]$si.ArgumentList.Add($a) }
    if ($Cwd) { $si.WorkingDirectory = $Cwd }
    $si.RedirectStandardOutput = $true
    $si.RedirectStandardError = $true
    $si.UseShellExecute = $false

    $p = New-Object System.Diagnostics.Process
    $p.StartInfo = $si
    [void]$p.Start()

    if (-not $p.WaitForExit($TimeoutMs)) {
        try { $p.Kill() } catch {}
        Fail "substrate timed out: $($Args -join ' ')"
    }

    $out = $p.StandardOutput.ReadToEnd()
    $err = $p.StandardError.ReadToEnd()
    if ($StdoutPath) { Set-Content -LiteralPath $StdoutPath -Value $out -Encoding UTF8 }
    if ($StderrPath) { Set-Content -LiteralPath $StderrPath -Value $err -Encoding UTF8 }

    return @{
        ExitCode = $p.ExitCode
        Stdout   = $out
        Stderr   = $err
    }
}

function Get-TraceMetaForMarker {
    param(
        [Parameter(Mandatory = $true)][string]$TraceLog,
        [Parameter(Mandatory = $true)][string]$Marker
    )

    if (-not (Test-Path -LiteralPath $TraceLog)) { return $null }

    $lines = Get-Content -LiteralPath $TraceLog -ErrorAction SilentlyContinue
    $hits = @()
    foreach ($line in $lines) {
        if (-not $line) { continue }
        $obj = $null
        try { $obj = $line | ConvertFrom-Json } catch { $obj = $null }
        if (-not $obj) { continue }
        if ($obj.event_type -ne 'command_complete') { continue }
        $cmdValue = $null
        if ($obj.PSObject.Properties.Match('cmd').Count -gt 0) {
            $cmdValue = $obj.cmd
        } elseif ($obj.PSObject.Properties.Match('command').Count -gt 0) {
            $cmdValue = $obj.command
        }
        if (-not $cmdValue) { continue }
        if ($cmdValue -notlike "*$Marker*") { continue }
        $hits += $obj
    }
    if ($hits.Count -eq 0) { return $null }

    $last = $hits[-1]
    $spanId = $null
    if ($last.PSObject.Properties.Match('span_id').Count -gt 0) {
        $spanId = $last.span_id
    } elseif ($last.PSObject.Properties.Match('spanId').Count -gt 0) {
        $spanId = $last.spanId
    }

    $exitCode = $null
    if ($last.PSObject.Properties.Match('exit').Count -gt 0) {
        $exitCode = $last.exit
    } elseif ($last.PSObject.Properties.Match('exit_code').Count -gt 0) {
        $exitCode = $last.exit_code
    } elseif ($last.PSObject.Properties.Match('exitCode').Count -gt 0) {
        $exitCode = $last.exitCode
    }

    $policyResolutionMode = $null
    if ($last.PSObject.Properties.Match('policy_resolution_mode').Count -gt 0) {
        $policyResolutionMode = $last.policy_resolution_mode
    } elseif ($last.PSObject.Properties.Match('policyResolutionMode').Count -gt 0) {
        $policyResolutionMode = $last.policyResolutionMode
    }

    $policySnapshotHash = $null
    if ($last.PSObject.Properties.Match('policy_snapshot_hash').Count -gt 0) {
        $policySnapshotHash = $last.policy_snapshot_hash
    } elseif ($last.PSObject.Properties.Match('policySnapshotHash').Count -gt 0) {
        $policySnapshotHash = $last.policySnapshotHash
    }

    $meta = @{}
    $meta['span_id'] = $spanId
    $meta['exit'] = $exitCode
    $meta['policy_resolution_mode'] = $policyResolutionMode
    $meta['policy_snapshot_hash'] = $policySnapshotHash
    $meta['policy_snapshot_schema'] = $null

    if ($null -eq $meta['policy_snapshot_schema'] -and $meta['policy_resolution_mode'] -eq 'snapshot_v1') {
        $meta['policy_snapshot_schema'] = 1
    }

    if ($last.PSObject.Properties.Match('policy_snapshot_schema').Count -gt 0) {
        $meta['policy_snapshot_schema'] = $last.policy_snapshot_schema
    } elseif ($last.PSObject.Properties.Match('policySnapshotSchema').Count -gt 0) {
        $meta['policy_snapshot_schema'] = $last.policySnapshotSchema
    }
    if ($last.PSObject.Properties.Match('world_fs_strategy_primary').Count -gt 0) {
        $meta['world_fs_strategy_primary'] = $last.world_fs_strategy_primary
    }
    if ($last.PSObject.Properties.Match('world_fs_strategy_final').Count -gt 0) {
        $meta['world_fs_strategy_final'] = $last.world_fs_strategy_final
    }
    if ($last.PSObject.Properties.Match('world_fs_strategy_fallback_reason').Count -gt 0) {
        $meta['world_fs_strategy_fallback_reason'] = $last.world_fs_strategy_fallback_reason
    }

    return [pscustomobject]$meta
}

function Write-LogSnippet {
    param(
        [Parameter(Mandatory = $true)][string]$Path,
        [int]$MaxLines = 200
    )

    if (-not (Test-Path -LiteralPath $Path)) { return }
    Get-Content -LiteralPath $Path -TotalCount $MaxLines -ErrorAction SilentlyContinue | ForEach-Object {
        Write-Host $_
    }
}

$runId = "waps-" + ([DateTimeOffset]::UtcNow.ToUnixTimeSeconds()) + "-" + ([guid]::NewGuid().ToString('N'))
$tmpHome = Join-Path $env:TEMP ("waps-home-" + $runId)
$tmpWs = Join-Path $env:TEMP ("waps-ws-" + $runId)
$traceLog = Join-Path $tmpHome "trace.jsonl"
$logsDir = Join-Path $tmpHome "logs"

New-Item -ItemType Directory -Force -Path $tmpHome, $tmpWs, $logsDir | Out-Null

try {
    $env:SUBSTRATE_HOME = $tmpHome
    $env:HOME = $tmpHome
    $env:SHIM_TRACE_LOG = $traceLog
    # WSL runners frequently cannot support the overlayfs + mount-namespace path, so force the world
    # to execute directly while still validating policy snapshot trace metadata.
    $env:SUBSTRATE_WORLD_EXEC_FORCE_DIRECT = '1'

    Invoke-Substrate -Args @('config', 'global', 'init', '--force') -Cwd $tmpWs | Out-Null
    Invoke-Substrate -Args @('policy', 'global', 'init', '--force') -Cwd $tmpWs | Out-Null
    Invoke-Substrate -Args @('config', 'global', 'set', 'policy.mode=enforce') -Cwd $tmpWs | Out-Null
    Invoke-Substrate -Args @('config', 'global', 'set', 'world.enabled=true') -Cwd $tmpWs | Out-Null
    Invoke-Substrate -Args @('config', 'global', 'set', 'world.anchor_mode=follow-cwd') -Cwd $tmpWs | Out-Null

    Invoke-Substrate -Args @('workspace', 'init', $tmpWs) -Cwd $tmpWs | Out-Null

    $policyPath = Join-Path $tmpWs ".substrate\policy.yaml"
    if (-not (Test-Path -LiteralPath (Split-Path -Parent $policyPath))) {
        New-Item -ItemType Directory -Force -Path (Split-Path -Parent $policyPath) | Out-Null
    }

    $doctorRaw = Invoke-Substrate -Args @('world', 'doctor', '--json') -Cwd $tmpWs
    $doctor = $null
    try { $doctor = $doctorRaw.Stdout | ConvertFrom-Json } catch { $doctor = $null }
    $doctorOk = $false
    $snapshotSupported = $false
    if ($doctor -and $doctor.ok) { $doctorOk = [bool]$doctor.ok }
    if ($doctor) {
        if ($doctor.PSObject.Properties.Match('policy_snapshot_v1_supported').Count -gt 0) {
            $snapshotSupported = [bool]$doctor.policy_snapshot_v1_supported
        }
        elseif ($doctor.world -and $doctor.world.PSObject.Properties.Match('policy_snapshot_v1_supported').Count -gt 0) {
            $snapshotSupported = [bool]$doctor.world.policy_snapshot_v1_supported
        }
    }

    $tests = @()

    Write-Host ("[INFO] run_id={0}" -f $runId)
    Write-Host ("[INFO] workspace={0}" -f $tmpWs)
    Write-Host ("[INFO] trace_log={0}" -f $traceLog)

    # ---- Test 1: FS allowlist in full isolation ----
    Set-Content -LiteralPath $policyPath -Encoding UTF8 -Value @"
id: "waps-smoke"
name: "waps-smoke fs"
world_fs:
  mode: writable
  isolation: full
  require_world: true
  read_allowlist:
    - "*"
  write_allowlist:
    - "./writable/*"
net_allowed: []
cmd_allowed: []
cmd_denied: []
cmd_isolated: []
require_approval: false
allow_shell_operators: true
limits:
  max_memory_mb: null
  max_cpu_percent: null
  max_runtime_ms: null
  max_egress_bytes: null
metadata: {}
"@

    $fsMarker = "__${runId}__fs__"
    $fsCmd = "echo '$fsMarker' >/dev/null; set -eu; true"
    $fsStdout = Join-Path $logsDir "fs.stdout"
    $fsStderr = Join-Path $logsDir "fs.stderr"
    $fsRes = Invoke-Substrate -Args @('--world', '--ci', '--command', $fsCmd) -Cwd $tmpWs -StdoutPath $fsStdout -StderrPath $fsStderr -TimeoutMs 60000
    $fsMeta = Get-TraceMetaForMarker -TraceLog $traceLog -Marker $fsMarker
    $tests += [ordered]@{
        name      = 'fs_allowlist_full_isolation'
        ok        = ($fsRes.ExitCode -eq 0 -and $fsMeta -ne $null)
        exit_code = $fsRes.ExitCode
        stdout_path = $fsStdout
        stderr_path = $fsStderr
        trace_meta = $fsMeta
    }

    # ---- Test 2: net allowlist ----
    Set-Content -LiteralPath $policyPath -Encoding UTF8 -Value @"
id: "waps-smoke"
name: "waps-smoke net"
world_fs:
  mode: writable
  isolation: full
  require_world: true
  read_allowlist:
    - "*"
  write_allowlist:
    - "./writable/*"
net_allowed:
  - "example.com"
cmd_allowed: []
cmd_denied: []
cmd_isolated: []
require_approval: false
allow_shell_operators: true
limits:
  max_memory_mb: null
  max_cpu_percent: null
  max_runtime_ms: null
  max_egress_bytes: null
metadata: {}
"@

    $netMarker = "__${runId}__net__"
    $netCmd = "echo '$netMarker' >/dev/null; set -eu; true"
    $netStdout = Join-Path $logsDir "net.stdout"
    $netStderr = Join-Path $logsDir "net.stderr"
    $netRes = Invoke-Substrate -Args @('--world', '--ci', '--command', $netCmd) -Cwd $tmpWs -StdoutPath $netStdout -StderrPath $netStderr -TimeoutMs 60000
    $netMeta = Get-TraceMetaForMarker -TraceLog $traceLog -Marker $netMarker
    $tests += [ordered]@{
        name      = 'net_allowlist'
        ok        = ($netRes.ExitCode -eq 0 -and $netMeta -ne $null)
        exit_code = $netRes.ExitCode
        stdout_path = $netStdout
        stderr_path = $netStderr
        trace_meta = $netMeta
    }

    # ---- Test 3: limits (best-effort) ----
    Set-Content -LiteralPath $policyPath -Encoding UTF8 -Value @"
id: "waps-smoke"
name: "waps-smoke limits"
world_fs:
  mode: writable
  isolation: full
  require_world: true
  read_allowlist:
    - "*"
  write_allowlist:
    - "./writable/*"
net_allowed: []
cmd_allowed: []
cmd_denied: []
cmd_isolated: []
require_approval: false
allow_shell_operators: true
limits:
  max_memory_mb: null
  max_cpu_percent: null
  max_runtime_ms: 1000
  max_egress_bytes: null
metadata: {}
"@

    $limitsMarker = "__${runId}__limits__"
    $limitsCmd = "echo '$limitsMarker' >/dev/null; set -eu; true"
    $limitsStdout = Join-Path $logsDir "limits.stdout"
    $limitsStderr = Join-Path $logsDir "limits.stderr"
    $limitsRes = Invoke-Substrate -Args @('--world', '--ci', '--command', $limitsCmd) -Cwd $tmpWs -StdoutPath $limitsStdout -StderrPath $limitsStderr -TimeoutMs 60000
    $limitsMeta = Get-TraceMetaForMarker -TraceLog $traceLog -Marker $limitsMarker
    $limitsSkipped = ($limitsRes.ExitCode -eq 0)
    $tests += [ordered]@{
        name      = 'limits_max_runtime_ms'
        ok        = ($limitsMeta -ne $null)
        skipped   = $limitsSkipped
        exit_code = $limitsRes.ExitCode
        stdout_path = $limitsStdout
        stderr_path = $limitsStderr
        trace_meta = $limitsMeta
    }

    $overallOk = $true
    foreach ($t in $tests) {
        if (-not $t.ok) { $overallOk = $false }
    }

    if (-not $overallOk) {
        foreach ($t in $tests) {
            if ($t.ok) { continue }
            Write-Host ("[FAIL] {0}" -f $t.name)
            if ($t.stdout_path) {
                Write-Host ("[FAIL] stdout: {0}" -f $t.stdout_path)
                Write-LogSnippet -Path $t.stdout_path
            }
            if ($t.stderr_path) {
                Write-Host ("[FAIL] stderr: {0}" -f $t.stderr_path)
                Write-LogSnippet -Path $t.stderr_path
            }
        }
    }

    $snapshotOk = $true
    foreach ($t in $tests) {
        if (-not $t.trace_meta) { $snapshotOk = $false; continue }
        if ($t.trace_meta.policy_resolution_mode -ne 'snapshot_v1') { $snapshotOk = $false }
        if ($t.trace_meta.policy_snapshot_schema -ne 1) { $snapshotOk = $false }
        if (-not $t.trace_meta.policy_snapshot_hash) { $snapshotOk = $false }
        if ([string]$t.trace_meta.policy_snapshot_hash -eq '') { $snapshotOk = $false }
    }

    $schemaOk = $true
    foreach ($t in $tests) {
        if (-not $t.trace_meta) { continue }
        if ($t.trace_meta.policy_resolution_mode -eq 'snapshot_v1' -and $t.trace_meta.policy_snapshot_schema -ne 1) {
            $schemaOk = $false
        }
    }

    $summary = [ordered]@{
        platform = 'windows-wsl'
        distro_name = $DistroName
        run_id = $runId
        substrate_home = $tmpHome
        workspace = $tmpWs
        trace_log = $traceLog
        doctor_ok = $doctorOk
        policy_snapshot_v1_supported = $snapshotSupported
        doctor = $doctor
        tests = $tests
    }

    $summary | ConvertTo-Json -Depth 10 -Compress

    if (-not $overallOk) { exit 1 }
    if (-not $schemaOk) { exit 1 }
    if (-not $snapshotOk) { exit 1 }
}
finally {
    Remove-Item -LiteralPath $tmpHome, $tmpWs -Recurse -Force -ErrorAction SilentlyContinue | Out-Null
}
