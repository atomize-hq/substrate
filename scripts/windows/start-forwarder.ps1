#!/usr/bin/env pwsh
param(
    [string]$ProjectPath = (Resolve-Path "$PSScriptRoot/../.." | Select-Object -ExpandProperty Path),
    [string]$DistroName = 'substrate-wsl',
    [string]$PipePath = '\\.\pipe\substrate-agent',
    [int]$TimeoutSeconds = 300,
    [string]$RustLog = 'info',
    [string[]]$AdditionalArgs = @(),
    [switch]$WaitForExit,
    [int]$ReadyTimeoutSeconds = 30,
    [string]$TcpBridge = $null
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Write-Info($Message) {
    Write-Host "[INFO] $Message" -ForegroundColor Cyan
}

function Write-Warn($Message) {
    Write-Host "[WARN] $Message" -ForegroundColor Yellow
}

$resolvedProject = Resolve-Path $ProjectPath | Select-Object -ExpandProperty Path
if (-not (Test-Path (Join-Path $resolvedProject 'Cargo.toml'))) {
    throw "Project path $resolvedProject does not contain Cargo.toml"
}

$forwarderRelease = Join-Path $resolvedProject 'target/release/substrate-forwarder.exe'
$forwarderDebug = Join-Path $resolvedProject 'target/debug/substrate-forwarder.exe'
if (Test-Path $forwarderRelease) {
    $forwarderExe = $forwarderRelease
} elseif (Test-Path $forwarderDebug) {
    Write-Warn 'Release binary not found, using debug build'
    $forwarderExe = $forwarderDebug
} else {
    throw "substrate-forwarder binary not found. Build it with 'cargo build -p substrate-forwarder --release'"
}

$logDir = Join-Path $env:LOCALAPPDATA 'Substrate/logs'
New-Item -ItemType Directory -Force $logDir | Out-Null

$argumentList = @('--distro', $DistroName, '--pipe', $PipePath, '--log-dir', $logDir)
# Optional host TCP bridge (opt-in via parameter or environment)
if (-not $TcpBridge) {
    $envFlag = ($env:SUBSTRATE_FORWARDER_TCP -as [string])
    $envAddr = ($env:SUBSTRATE_FORWARDER_TCP_ADDR -as [string])
    if ($envAddr) {
        $TcpBridge = $envAddr
    } elseif ($envFlag -and ($envFlag.Trim().ToLower() -in @('1','true','yes'))) {
        $port = 17788
        if ($env:SUBSTRATE_FORWARDER_TCP_PORT) { [void][int]::TryParse($env:SUBSTRATE_FORWARDER_TCP_PORT, [ref]$port) }
        $TcpBridge = "127.0.0.1:$port"
    }
}
if ($TcpBridge) {
    Write-Info ("Enabling host TCP bridge at {0}" -f $TcpBridge)
    $argumentList += @('--tcp-bridge', $TcpBridge)
}
if ($AdditionalArgs.Length -gt 0) {
    $argumentList += $AdditionalArgs
}

$psi = New-Object System.Diagnostics.ProcessStartInfo
$psi.FileName = $forwarderExe
foreach ($arg in $argumentList) {
    $null = $psi.ArgumentList.Add($arg)
}
$psi.UseShellExecute = $false
$psi.RedirectStandardOutput = $true
$psi.RedirectStandardError = $true
$psi.Environment['RUST_LOG'] = $RustLog

Write-Info ("Launching substrate-forwarder (guard {0}s)" -f $TimeoutSeconds)
$process = [System.Diagnostics.Process]::Start($psi)

# Default: return once ready. Use -WaitForExit only for CI/service.
if (-not $WaitForExit) {
    # Probe readiness and return immediately
    Write-Info ("Waiting up to {0}s for pipe readiness" -f $ReadyTimeoutSeconds)
    if ($PipePath -match '\\pipe\\(?<n>[^\\]+)$') {
        $pipeName = $Matches['n']
    } else {
        $pipeName = $PipePath
    }
    Write-Info ("Using pipe name '{0}' from '{1}'" -f $pipeName, $PipePath)
    $deadline = [DateTime]::UtcNow.AddSeconds([Math]::Max(1, $ReadyTimeoutSeconds))
    $ready = $false
    while ([DateTime]::UtcNow -lt $deadline) {
        try {
            $client = [System.IO.Pipes.NamedPipeClientStream]::new('.', $pipeName, [System.IO.Pipes.PipeDirection]::InOut, [System.IO.Pipes.PipeOptions]::None)
            $client.Connect(1000)
            $client.Dispose()
            $ready = $true
            break
        } catch {
            Start-Sleep -Milliseconds 200
        }
    }
    if (-not $ready) {
        Write-Warn "Pipe did not become ready within the readiness window; forwarder left running under guard"
    } else {
        Write-Info "Pipe is ready; returning without waiting for forwarder exit"
    }
    return
} else {
    $stopwatch = [System.Diagnostics.Stopwatch]::StartNew()
    if (-not $process.WaitForExit($TimeoutSeconds * 1000)) {
        Write-Warn "Forwarder exceeded timeout. Terminating process."
        try {
            $process.Kill()
            $process.WaitForExit()
        } catch {
            Write-Warn "Failed to terminate forwarder process: $_"
        }
        $stopwatch.Stop()
        throw "substrate-forwarder exceeded timeout of $TimeoutSeconds seconds"
    }
    $stopwatch.Stop()

    $stdout = $process.StandardOutput.ReadToEnd()
    $stderr = $process.StandardError.ReadToEnd()
    if ($stdout) { Write-Output $stdout.TrimEnd() }
    if ($stderr) { Write-Warn $stderr.TrimEnd() }

    Write-Info ("Forwarder exited in {0:F1}s with code {1}" -f $stopwatch.Elapsed.TotalSeconds, $process.ExitCode)
    if ($process.ExitCode -ne 0) {
        throw "substrate-forwarder exited with code $($process.ExitCode)"
    }
}
