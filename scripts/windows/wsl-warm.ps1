#!/usr/bin/env pwsh
param(
    [string]$DistroName = 'substrate-wsl',
    [string]$ProjectPath = (Resolve-Path '..\\..' | Select-Object -ExpandProperty Path),
    [string]$PipePath = '\\.\pipe\substrate-agent',
    [switch]$WhatIf
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Write-Info($Message) { Write-Host "[INFO] $Message" -ForegroundColor Cyan }
function Write-Warn($Message) { Write-Host "[WARN] $Message" -ForegroundColor Yellow }
function Write-ErrorAndExit($Message) { Write-Host "[FAIL] $Message" -ForegroundColor Red; exit 1 }

function Convert-ToWslPathFragment {
    param([string]$Path)
    $withoutDrive = ($Path -replace '^[A-Za-z]:\\', '')
    ($withoutDrive -replace '\\', '/').TrimStart('/')
}

function Quote-ForBash {
    param([string]$Value)
    if ($null -eq $Value) { return "''" }
    return "'" + ($Value -replace "'", "'\"'\"'") + "'"
}

Write-Info "Starting wsl-warm for distro '$DistroName'"

$projectPath = Resolve-Path $ProjectPath | Select-Object -ExpandProperty Path
Write-Info "Project path: $projectPath"

$projectHasCargo = Test-Path (Join-Path $projectPath 'Cargo.toml')
$packagedWorldAgent = Join-Path $projectPath 'bin\\linux\\world-agent'
$usesBundledArtifacts = -not $projectHasCargo

if (-not $projectHasCargo -and -not (Test-Path $packagedWorldAgent)) {
    Write-ErrorAndExit "Project path must contain Cargo.toml or a packaged bin\\linux\\world-agent"
}

$cargoExe = Join-Path $env:USERPROFILE '.cargo\bin\cargo.exe'
if (-not $usesBundledArtifacts -and -not (Test-Path $cargoExe)) {
    Write-ErrorAndExit "cargo.exe not found at $cargoExe. Install Rust on Windows host."
}

# Ensure WSL installed
$wslStatus = & wsl --status 2>$null
if ($LASTEXITCODE -ne 0) {
    Write-ErrorAndExit "WSL not available. Run 'wsl --install' first."
}

# Import distro if missing
$distroList = & wsl -l -v | Out-String
$distroListClean = $distroList -replace "`0", ""
if ($distroListClean -notmatch [regex]::Escape($DistroName)) {
    Write-Info "Importing distro '$DistroName'"

    if ($WhatIf) {
        Write-Warn "WhatIf mode enabled - skipping provisioning"
        return
    }

    $baseUrl = 'https://cdimage.ubuntu.com/ubuntu-wsl/noble/daily-live/current'
    $arch = $null
    try {
        $arch = [System.Runtime.InteropServices.RuntimeInformation]::ProcessArchitecture
    } catch {
        $envArch = $env:PROCESSOR_ARCHITECTURE
        if ($envArch) {
            if ($envArch -match 'ARM64') {
                $arch = 'Arm64'
            } else {
                $arch = 'X64'
            }
        }
    }

    if (-not $arch) {
        Write-Warn "Unable to detect architecture via RuntimeInformation; defaulting to x64 WSL image"
        $arch = 'X64'
    }

    if (($arch -is [string] -and $arch -ieq 'Arm64') -or ($arch -is [System.Runtime.InteropServices.Architecture] -and $arch -eq [System.Runtime.InteropServices.Architecture]::Arm64)) {
        $imageName = 'noble-wsl-arm64.wsl'
    } else {
        $imageName = 'noble-wsl-amd64.wsl'
    }

    $imagePath = Join-Path $env:TEMP $imageName
    $shaPath = Join-Path $env:TEMP 'noble-wsl-SHA256SUMS'

    Write-Info "Downloading Ubuntu WSL image manifest"
    Invoke-WebRequest -Uri "$baseUrl/SHA256SUMS" -OutFile $shaPath
    $expectedLine = Get-Content $shaPath | Where-Object { $_ -match "\*$imageName$" }
    if (-not $expectedLine) {
        Write-ErrorAndExit "SHA256SUMS does not list $imageName"
    }
    $expectedHash = (($expectedLine -split ' \*')[0]).Trim()

    Write-Info "Downloading Ubuntu WSL image ($imageName)"
    Invoke-WebRequest -Uri "$baseUrl/$imageName" -OutFile $imagePath

    $fileHash = (Get-FileHash -Path $imagePath -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($fileHash -ne ($expectedHash.ToLowerInvariant())) {
        Remove-Item $imagePath -ErrorAction SilentlyContinue
        Write-ErrorAndExit "Hash mismatch for $imageName (expected $expectedHash, got $fileHash)"
    }

    $installDir = Join-Path $env:LOCALAPPDATA 'substrate\\wsl'
    New-Item -ItemType Directory -Force $installDir | Out-Null
    & wsl --import $DistroName $installDir $imagePath --version 2
    Remove-Item $imagePath -ErrorAction SilentlyContinue
    Remove-Item $shaPath -ErrorAction SilentlyContinue
}

if ($WhatIf) {
    Write-Warn "WhatIf mode enabled - skipping provisioning"
    return
}

# Copy provisioning script and run
$hostProvisionPath = Join-Path $projectPath 'scripts\\wsl\\provision.sh'
if (-not (Test-Path $hostProvisionPath)) {
    Write-ErrorAndExit "Provisioning script not found at $hostProvisionPath"
}

Write-Info "Preflight agent health check"
$isHealthy = $false
try {
    $status = & wsl -d $DistroName -- bash -lc "curl -s -o /dev/null -w '%{http_code}' http://127.0.0.1:61337/v1/capabilities || true"
    if ($status -eq '200') { $isHealthy = $true }
} catch {}

if (-not $isHealthy) {
    Write-Info "Updating package cache and running provision script"
    $projectPathFragment = Convert-ToWslPathFragment $projectPath
    $projectPathWsl = "/mnt/c/$projectPathFragment"
    $provisionScript = Quote-ForBash "$projectPathWsl/scripts/wsl/provision.sh"
    & wsl -d $DistroName -- bash -lc "set -euo pipefail; cp ${provisionScript} /tmp/provision.sh && sed -i 's/\r$//' /tmp/provision.sh && chmod +x /tmp/provision.sh && sudo /tmp/provision.sh"
    if ($LASTEXITCODE -ne 0) {
        Write-ErrorAndExit "Provision script failed"
    }

    if ($projectHasCargo) {
        # Build and install world-agent inside WSL
        Write-Info "Building world-agent (release) inside WSL"
        $projectPathQuoted = Quote-ForBash $projectPathWsl
        $buildScript = @"
set -euo pipefail
if [ -f ~/.cargo/env ]; then
  . ~/.cargo/env
fi
cd $projectPathQuoted
cargo build -p world-agent --release
sudo install -m755 target/release/world-agent /usr/local/bin/substrate-world-agent
"@
        $buildScript = $buildScript -replace "`r", ""
        & wsl -d $DistroName -- bash -lc $buildScript
        if ($LASTEXITCODE -ne 0) {
            Write-ErrorAndExit "Failed to build/install world-agent inside WSL"
        }
    } else {
        Write-Info "Installing packaged world-agent into WSL"
        $agentFragment = Convert-ToWslPathFragment (Join-Path $projectPath 'bin\\linux\\world-agent')
        $agentPath = Quote-ForBash "/mnt/c/$agentFragment"
        & wsl -d $DistroName -- bash -lc "set -euo pipefail; sudo install -m755 ${agentPath} /usr/local/bin/substrate-world-agent"
        if ($LASTEXITCODE -ne 0) {
            Write-ErrorAndExit "Failed to install packaged world-agent"
        }
    }

    # Ensure systemd units are enabled
    Write-Info "Ensuring substrate-world-agent service and socket are enabled"
    & wsl -d $DistroName -- bash -lc "sudo systemctl daemon-reload && sudo systemctl enable substrate-world-agent.service && sudo systemctl enable --now substrate-world-agent.socket && sudo systemctl restart substrate-world-agent.service"
    if ($LASTEXITCODE -ne 0) {
        Write-ErrorAndExit "Failed to enable/restart agent units"
    }
} else {
    Write-Info "Agent reports HTTP 200; skipping provision/build/restart"
}

# Build forwarder if needed or use packaged binary
if ($projectHasCargo) {
    $forwarderHostPath = Join-Path $projectPath 'target\\release\\substrate-forwarder.exe'
    if (-not (Test-Path $forwarderHostPath)) {
        Write-Info "Building substrate-forwarder (release)"
        & $cargoExe build -p substrate-forwarder --release
    }
} else {
    $forwarderHostPath = Join-Path $projectPath 'bin\\substrate-forwarder.exe'
    if (-not (Test-Path $forwarderHostPath)) {
        Write-ErrorAndExit "Packaged substrate-forwarder.exe not found at $forwarderHostPath"
    }
}

# Launch forwarder
Write-Info "Launching forwarder"
$logDir = Join-Path $env:LOCALAPPDATA 'Substrate\\logs'
New-Item -ItemType Directory -Force $logDir | Out-Null
$pipePath = $PipePath
$pidFile = Join-Path $env:LOCALAPPDATA 'Substrate\\forwarder.pid'
if (Test-Path $pidFile) {
    Write-Warn "Forwarder PID file exists; attempting cleanup"
    $existingPid = Get-Content $pidFile
    Stop-Process -Id $existingPid -ErrorAction SilentlyContinue
    Remove-Item $pidFile -ErrorAction SilentlyContinue
}

# Optional host TCP bridge (opt-in via environment)
$tcpBridge = $null
if ($env:SUBSTRATE_FORWARDER_TCP_ADDR) {
    $tcpBridge = $env:SUBSTRATE_FORWARDER_TCP_ADDR
} elseif ($env:SUBSTRATE_FORWARDER_TCP) {
    $flag = $env:SUBSTRATE_FORWARDER_TCP.Trim().ToLower()
    if ($flag -in @('1','true','yes')) {
        $port = 17788
        if ($env:SUBSTRATE_FORWARDER_TCP_PORT) { [void][int]::TryParse($env:SUBSTRATE_FORWARDER_TCP_PORT, [ref]$port) }
        $tcpBridge = "127.0.0.1:$port"
    }
}

$args = @("--distro", $DistroName, "--pipe", $pipePath, "--log-dir", $logDir, "--run-as-service")
if ($tcpBridge) { $args += @("--tcp-bridge", $tcpBridge) }
$forwarderProcess = Start-Process -FilePath $forwarderHostPath -ArgumentList $args -WindowStyle Hidden -PassThru
Set-Content $pidFile -Value $forwarderProcess.Id

# Wait for pipe using an actual client probe with retries
Write-Info "Probing forwarder pipe $pipePath"
$stopwatch = [System.Diagnostics.Stopwatch]::StartNew()
if ($pipePath -match '\\pipe\\(?<n>[^\\]+)$') {
    $pipeName = $Matches['n']
} else {
    $pipeName = $pipePath
}
Write-Info ("Using pipe name '{0}' from '{1}'" -f $pipeName, $pipePath)
Add-Type -TypeDefinition @"
using System;
using System.Runtime.InteropServices;
namespace Native {
  public static class Win32 {
    [DllImport("kernel32.dll", SetLastError=true, CharSet=CharSet.Unicode)]
    public static extern bool WaitNamedPipe(string name, uint timeout);
  }
}
"@

# First wait for the server to create and pend connect
if (-not ([Native.Win32]::WaitNamedPipe($pipePath, 30000))) {
    # Fallback: poll for path existence for up to 30s
    $deadline = [DateTime]::UtcNow.AddSeconds(30)
    while (-not (Test-Path $pipePath)) {
        if ([DateTime]::UtcNow -ge $deadline) {
            $stopwatch.Stop()
            $err = [Runtime.InteropServices.Marshal]::GetLastWin32Error()
            Write-ErrorAndExit ("Forwarder pipe probe failed after {0:N0} ms: WaitNamedPipe error {1}; also path never appeared" -f $stopwatch.Elapsed.TotalMilliseconds, $err)
        }
        Start-Sleep -Milliseconds 250
    }
}

# Then do a quick client connect/close to fully validate
$client = [System.IO.Pipes.NamedPipeClientStream]::new('.', $pipeName, [System.IO.Pipes.PipeDirection]::InOut, [System.IO.Pipes.PipeOptions]::None)
$client.Connect(2000)
$client.Dispose()
$stopwatch.Stop()
Write-Info ("Forwarder pipe accepted probe in {0:N0} ms" -f $stopwatch.Elapsed.TotalMilliseconds)
Write-Info "Forwarder pipe ready"

Write-Info "Warm complete"
