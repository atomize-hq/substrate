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
function Write-ErrorAndExit($Message, [int]$Code = 1) { Write-Host "[FAIL] $Message" -ForegroundColor Red; exit $Code }

function Convert-ToWslPathFragment {
    param([string]$Path)
    $withoutDrive = ($Path -replace '^[A-Za-z]:\\', '')
    ($withoutDrive -replace '\\', '/').TrimStart('/')
}

function Quote-ForBash {
    param([string]$Value)
    if ($null -eq $Value) { return "''" }
    # Bash single-quote escape sequence: close-quote, insert '"'"', reopen.
    $singleQuoteEscape = "'" + '"' + "'" + '"' + "'"
    return "'" + ($Value -replace "'", $singleQuoteEscape) + "'"
}

function Test-Truthy {
    param([string]$Value)
    if (-not $Value) { return $false }
    $normalized = $Value.Trim().ToLowerInvariant()
    return $normalized -in @('1', 'true', 'yes', 'y', 'on')
}

function Test-GuestExecutablePresent {
    param(
        [string]$DistroName,
        [string]$Path
    )

    $quotedPath = Quote-ForBash $Path
    & wsl -d $DistroName -- bash -lc "test -x ${quotedPath}"
    return ($LASTEXITCODE -eq 0)
}

function Install-GuestWorldBinaries {
    param(
        [string]$DistroName,
        [string]$ProjectPathWsl,
        [bool]$ProjectHasCargo,
        [string]$ProjectPath
    )

    if ($ProjectHasCargo) {
        Write-Info "Building world-service and substrate-gateway (release) inside WSL"
        $projectPathQuoted = Quote-ForBash $ProjectPathWsl
        $buildScript = @"
set -euo pipefail
if [ -f ~/.cargo/env ]; then
  . ~/.cargo/env
fi
cd $projectPathQuoted
cargo build -p world-service -p substrate-gateway --release
sudo install -m755 target/release/world-service /usr/local/bin/substrate-world-service
sudo install -m755 target/release/substrate-gateway /usr/local/bin/substrate-gateway
sudo systemctl restart substrate-world-service.service
"@
        $buildScript = $buildScript -replace "`r", ""
        & wsl -d $DistroName -- bash -lc $buildScript
        if ($LASTEXITCODE -ne 0) {
            Write-ErrorAndExit "Failed to build/install world-service and substrate-gateway inside WSL"
        }
        return
    }

    Write-Info "Installing packaged world-service and substrate-gateway into WSL"
    $agentFragment = Convert-ToWslPathFragment (Join-Path $ProjectPath 'bin\\linux\\world-service')
    $gatewayFragment = Convert-ToWslPathFragment (Join-Path $ProjectPath 'bin\\linux\\substrate-gateway')
    $agentPath = Quote-ForBash "/mnt/c/$agentFragment"
    $gatewayPath = Quote-ForBash "/mnt/c/$gatewayFragment"
    & wsl -d $DistroName -- bash -lc "set -euo pipefail; sudo install -m755 ${agentPath} /usr/local/bin/substrate-world-service; sudo install -m755 ${gatewayPath} /usr/local/bin/substrate-gateway; sudo systemctl restart substrate-world-service.service"
    if ($LASTEXITCODE -ne 0) {
        Write-ErrorAndExit "Failed to install packaged world-service and substrate-gateway"
    }
}

Write-Info "Starting wsl-warm for distro '$DistroName'"

$projectPath = Resolve-Path $ProjectPath | Select-Object -ExpandProperty Path
Write-Info "Project path: $projectPath"
Write-ErrorAndExit "WSL world provisioning is intentionally fail-closed in this slice because the WSL helper path is not aligned with the Linux/macOS placement contract for SUBSTRATE_HOME placement, socket/group ownership, and runtime artifact access. Use Linux host-native provisioning, macOS Lima provisioning, or a CLI-only WSL install with --no-world instead." 4

$projectHasCargo = Test-Path (Join-Path $projectPath 'Cargo.toml')
$packagedWorldAgent = Join-Path $projectPath 'bin\\linux\\world-service'
$packagedGateway = Join-Path $projectPath 'bin\\linux\\substrate-gateway'
$usesBundledArtifacts = -not $projectHasCargo

if (-not $projectHasCargo -and (-not (Test-Path $packagedWorldAgent) -or -not (Test-Path $packagedGateway))) {
    Write-ErrorAndExit "Project path must contain Cargo.toml or packaged bin\\linux\\world-service and bin\\linux\\substrate-gateway artifacts"
}

$cargoCandidates = @()
if ($env:SUBSTRATE_WINDOWS_CARGO_EXE) {
    $cargoCandidates += $env:SUBSTRATE_WINDOWS_CARGO_EXE
}
if ($env:CARGO -and $env:CARGO.Trim().ToLowerInvariant().EndsWith('cargo.exe')) {
    $cargoCandidates += $env:CARGO
}
if ($env:CARGO_HOME) {
    $cargoCandidates += (Join-Path $env:CARGO_HOME 'bin\cargo.exe')
}
if ($env:SUBSTRATE_HOST_USERPROFILE) {
    $cargoCandidates += (Join-Path $env:SUBSTRATE_HOST_USERPROFILE '.cargo\bin\cargo.exe')
}
if ($env:USERPROFILE) {
    $cargoCandidates += (Join-Path $env:USERPROFILE '.cargo\bin\cargo.exe')
}
$cargoCmd = Get-Command cargo -ErrorAction SilentlyContinue
if ($cargoCmd) {
    $cargoCandidates += $cargoCmd.Path
}
$cargoExe = $cargoCandidates | Where-Object { $_ -and (Test-Path $_) } | Select-Object -First 1
if (-not $usesBundledArtifacts -and -not $cargoExe) {
    Write-ErrorAndExit "cargo.exe not found via SUBSTRATE_WINDOWS_CARGO_EXE, SUBSTRATE_HOST_USERPROFILE, USERPROFILE, or PATH. Install Rust on the Windows host."
}
$cargoToolchain = $env:RUST_TOOLCHAIN
if (-not $cargoToolchain -and $env:RUSTUP_TOOLCHAIN) {
    $cargoToolchain = $env:RUSTUP_TOOLCHAIN
}
$rustupExe = $null
$cargoResolvedViaRustup = $false
if ($cargoExe) {
    $candidateRustupExe = Join-Path (Split-Path -Parent $cargoExe) 'rustup.exe'
    if (Test-Path $candidateRustupExe) {
        $rustupExe = $candidateRustupExe
    }
}
if ($cargoToolchain -and $rustupExe) {
    $resolvedCargoExe = (& $rustupExe which cargo --toolchain $cargoToolchain 2>$null | Select-Object -Last 1)
    if ($LASTEXITCODE -eq 0 -and $resolvedCargoExe) {
        $resolvedCargoExe = $resolvedCargoExe.Trim()
        if ($resolvedCargoExe -and (Test-Path $resolvedCargoExe)) {
            $cargoExe = $resolvedCargoExe
            $cargoResolvedViaRustup = $true
        }
    }
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

$projectPathFragment = Convert-ToWslPathFragment $projectPath
$projectPathWsl = "/mnt/c/$projectPathFragment"

$guestWorldAgentInstalled = $false
$guestGatewayInstalled = $false
try { $guestWorldAgentInstalled = Test-GuestExecutablePresent -DistroName $DistroName -Path '/usr/local/bin/substrate-world-service' } catch {}
try { $guestGatewayInstalled = Test-GuestExecutablePresent -DistroName $DistroName -Path '/usr/local/bin/substrate-gateway' } catch {}

Write-Info "Preflight agent health check"
$isHealthy = $false
try {
    $status = & wsl -d $DistroName -- bash -lc "curl -s -o /dev/null -w '%{http_code}' http://127.0.0.1:61337/v1/capabilities || true"
    if ($status -eq '200') { $isHealthy = $true }
} catch {}

$forceRebuild = Test-Truthy $env:SUBSTRATE_WSL_WARM_FORCE_REBUILD

if (-not $isHealthy -or $forceRebuild) {
    if ($forceRebuild -and $isHealthy) {
        Write-Warn "SUBSTRATE_WSL_WARM_FORCE_REBUILD enabled; reprovisioning even though agent reports HTTP 200"
    }
    Write-Info "Updating package cache and running provision script"
    $provisionScript = Quote-ForBash "$projectPathWsl/scripts/wsl/provision.sh"
    & wsl -d $DistroName -- bash -lc "set -euo pipefail; cp ${provisionScript} /tmp/provision.sh && sed -i 's/\r$//' /tmp/provision.sh && chmod +x /tmp/provision.sh && sudo /tmp/provision.sh"
    if ($LASTEXITCODE -ne 0) {
        Write-ErrorAndExit "Provision script failed"
    }

    Install-GuestWorldBinaries -DistroName $DistroName -ProjectPathWsl $projectPathWsl -ProjectHasCargo:$projectHasCargo -ProjectPath $projectPath

    # Ensure systemd units are enabled
    Write-Info "Ensuring substrate-world-service service and socket are enabled"
    & wsl -d $DistroName -- bash -lc "sudo systemctl daemon-reload && sudo systemctl enable substrate-world-service.service && sudo systemctl enable --now substrate-world-service.socket && sudo systemctl restart substrate-world-service.service"
    if ($LASTEXITCODE -ne 0) {
        Write-ErrorAndExit "Failed to enable/restart agent units"
    }
} else {
    Write-Info "Agent reports HTTP 200; skipping provision/build/restart"

    # CI safety: even if the agent is reachable, ensure the guest binaries match the checked-out
    # ref so transport and gateway-runtime fixes take effect on self-hosted runners.
    $rebuildGuestBinaries = (-not $guestWorldAgentInstalled) -or (-not $guestGatewayInstalled)
    if ($projectHasCargo -and (Test-Truthy $env:GITHUB_ACTIONS -or Test-Truthy $env:SUBSTRATE_WSL_WARM_FORCE_AGENT_REBUILD -or Test-Truthy $env:SUBSTRATE_WSL_WARM_FORCE_GATEWAY_REBUILD)) {
        $rebuildGuestBinaries = $true
    }
    if ($rebuildGuestBinaries) {
        Write-Info "Refreshing guest world binaries inside WSL"
        Install-GuestWorldBinaries -DistroName $DistroName -ProjectPathWsl $projectPathWsl -ProjectHasCargo:$projectHasCargo -ProjectPath $projectPath
    }
}

# Build forwarder if needed or use packaged binary
if ($projectHasCargo) {
    $forwarderHostPath = Join-Path $projectPath 'target\\release\\substrate-forwarder.exe'
    $forceForwarderRebuild = $forceRebuild -or (Test-Truthy $env:SUBSTRATE_WSL_WARM_FORCE_FORWARDER_REBUILD) -or (Test-Truthy $env:GITHUB_ACTIONS)
    if ($forceForwarderRebuild -or -not (Test-Path $forwarderHostPath)) {
        Write-Info "Building substrate-forwarder (release)"
        Push-Location $projectPath
        try {
            $forwarderBuildArgs = @('build', '-p', 'substrate-forwarder', '--release')
            if ($cargoToolchain -and -not $cargoResolvedViaRustup) {
                $env:RUSTUP_TOOLCHAIN = $cargoToolchain
            }
            & $cargoExe @forwarderBuildArgs
        } finally {
            Pop-Location
        }
        if ($LASTEXITCODE -ne 0) {
            Write-ErrorAndExit "Failed to build substrate-forwarder.exe"
        }
        if (-not (Test-Path $forwarderHostPath)) {
            Write-ErrorAndExit "substrate-forwarder.exe missing after build at $forwarderHostPath"
        }
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

# Ensure the forwarder targets the agent TCP listener inside WSL unless explicitly overridden.
# This avoids named-pipe-to-UDS permission and socket-activation edge cases on CI runners.
if (-not $env:SUBSTRATE_FORWARDER_TARGET) {
    $env:SUBSTRATE_FORWARDER_TARGET = 'tcp:61337'
}
if (-not $env:SUBSTRATE_FORWARDER_CONNECT_TIMEOUT_S) {
    $env:SUBSTRATE_FORWARDER_CONNECT_TIMEOUT_S = '2'
}
if (-not $env:SUBSTRATE_FORWARDER_CONNECT_DEADLINE_S) {
    $env:SUBSTRATE_FORWARDER_CONNECT_DEADLINE_S = '10'
}
if (-not $env:SUBSTRATE_FORWARDER_IDLE_AFTER_STDIN_CLOSE_S) {
    $env:SUBSTRATE_FORWARDER_IDLE_AFTER_STDIN_CLOSE_S = '2'
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

# Validate agent round-trip through the forwarder (not just pipe reachability).
Write-Info "Probing agent capabilities via forwarder"
$probeTimeoutMs = 10000
if ($tcpBridge) {
    try {
        $uri = "http://$tcpBridge/v1/capabilities"
        $res = Invoke-WebRequest -Uri $uri -UseBasicParsing -TimeoutSec ([Math]::Ceiling($probeTimeoutMs / 1000)) -ErrorAction Stop
        if ($res.StatusCode -ne 200) {
            Write-ErrorAndExit ("Forwarder TCP probe returned HTTP {0} for {1}" -f $res.StatusCode, $uri)
        }
        Write-Info ("Forwarder TCP probe OK: {0}" -f $uri)
    } catch {
        Write-ErrorAndExit ("Forwarder TCP probe failed for http://{0}/v1/capabilities: {1}" -f $tcpBridge, $_.Exception.Message)
    }
} else {
    $probe = [System.IO.Pipes.NamedPipeClientStream]::new('.', $pipeName, [System.IO.Pipes.PipeDirection]::InOut, [System.IO.Pipes.PipeOptions]::None)
    $probe.Connect($probeTimeoutMs)
    $probe.ReadTimeout = $probeTimeoutMs
    $probe.WriteTimeout = $probeTimeoutMs
    $req = "GET /v1/capabilities HTTP/1.1`r`nHost: localhost`r`nConnection: close`r`n`r`n"
    $bytes = [System.Text.Encoding]::ASCII.GetBytes($req)
    $probe.Write($bytes, 0, $bytes.Length)
    $probe.Flush()

    $buf = New-Object byte[] 4096
    $ms = New-Object System.IO.MemoryStream
    try {
        while ($true) {
            $n = $probe.Read($buf, 0, $buf.Length)
            if ($n -le 0) { break }
            $ms.Write($buf, 0, $n) | Out-Null
        }
    } catch {
        $probe.Dispose()
        Write-ErrorAndExit ("Forwarder named-pipe probe timed out or failed while reading response: {0}" -f $_.Exception.Message)
    }
    $probe.Dispose()

    $text = [System.Text.Encoding]::UTF8.GetString($ms.ToArray())
    if ($text -notmatch '^HTTP/1\\.1 200') {
        $firstLine = ([regex]::Split($text, "\r?\n") | Select-Object -First 1)
        Write-ErrorAndExit ("Forwarder named-pipe probe returned non-200 response (first line): {0}" -f $firstLine)
    }
    Write-Info "Forwarder named-pipe probe OK: /v1/capabilities"
}

Write-Info "Warm complete"
