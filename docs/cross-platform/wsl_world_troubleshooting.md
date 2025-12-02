# Windows WSL Troubleshooting Catalogue

Use this catalogue when the doctor or smoke scripts report failures. Each entry
lists the observed symptom, likely root cause, recommended remediation, and a
verification command. Reference the entry ID in the evidence log whenever you
apply one of these fixes.

## Index

- [T-001 Virtualization disabled](#t-001-virtualization-disabled)
- [T-002 Optional features missing](#t-002-optional-features-missing)
- [T-003 WSL image download fails](#t-003-wsl-image-download-fails)
- [T-004 WSL import errors](#t-004-wsl-import-errors)
- [T-005 Forwarder pipe unavailable](#t-005-forwarder-pipe-unavailable)
- [T-006 Agent service not running](#t-006-agent-service-not-running)
- [T-007 Doctor nftables failure](#t-007-doctor-nftables-failure)
- [T-008 ConPTY or PTY stream failure](#t-008-conpty-or-pty-stream-failure)
- [T-009 Path translation mismatch](#t-009-path-translation-mismatch)
- [T-010 Forwarder log stale](#t-010-forwarder-log-stale)
- [T-011 Pipe name already in use](#t-011-pipe-name-already-in-use)

## Manager parity quick check

Run the aggregated health command whenever the guest tools diverge from the
Windows host. The text output calls out host-only vs world-only managers, while
the JSON payload exposes explicit lists for telemetry.

```powershell
PS C:\> substrate.exe health --json `
  | ConvertFrom-Json `
  | Select-Object -ExpandProperty summary `
  | Select-Object attention_required_managers, world_only_managers

attention_required_managers : {asdf}
world_only_managers        : {bun}
```

`attention_required_managers` lists host-present/world-missing managers (fix by
running `substrate world deps sync --all` once WSL is healthy). `world_only_managers`
shows tools that exist only in the guest (install them on Windows via
`substrate shim repair --manager <name> --yes` or your preferred package
manager).

Inspect per-manager guidance with:

```powershell
PS C:\> substrate.exe health --json `
  | ConvertFrom-Json `
  | Select-Object -ExpandProperty summary `
  | Select-Object -ExpandProperty manager_states `
  | Format-Table name, parity, recommendation -AutoSize
```

`parity` values (`host_only`, `world_only`, `absent`, `synced`, `unknown`) help
pin down whether the fix belongs on Windows or inside the guest. Keep the JSON
snippet in evidence logs whenever you close a catalogue entry.

### T-001 Virtualization disabled

- **Symptom**: Doctor reports `Virtualization disabled` or `systeminfo` shows
  `No` for virtualization capabilities.
- **Likely cause**: Firmware virtualization disabled or
  Virtualization-Based Security (VBS) is consuming VT-x.
- **Remediation**:

  1. Enable virtualization in UEFI or BIOS.
  1. Disable Credential Guard / VBS if it holds VT-x (`msinfo32` ? Device Guard).
  1. Reboot the host and rerun doctor.

- **Verify**:

```powershell
systeminfo | Select-String "Virtualization"
```

### T-002 Optional features missing

- **Symptom**: Doctor shows FAIL for `WSL Feature` or `VirtualMachinePlatform`.
- **Likely cause**: Required optional features are disabled or the host has not
  rebooted yet.
- **Remediation**:

  1. Enable the features:

  ```powershell
  Enable-WindowsOptionalFeature `
    -Online `
    -FeatureName Microsoft-Windows-Subsystem-Linux `
    -NoRestart
  Enable-WindowsOptionalFeature `
    -Online `
    -FeatureName VirtualMachinePlatform `
    -NoRestart
  ```

  1. Reboot and rerun the doctor script.

- **Verify**:

```powershell
Get-WindowsOptionalFeature -Online -FeatureName Microsoft-Windows-Subsystem-Linux
Get-WindowsOptionalFeature -Online -FeatureName VirtualMachinePlatform
```

### T-003 WSL image download fails

- **Symptom**: `wsl-warm.ps1` fails while downloading the `.wsl` image (HTTP
  404, TLS error, or proxy failure).
- **Likely cause**: Network egress blocked, stale CDN cache, or corporate proxy
  interception.
- **Remediation**:

  1. Confirm `https://cdimage.ubuntu.com/` is reachable.
  1. Configure proxy credentials if HTTPS is intercepted, then retry.
  1. Manually download the `.wsl` file into `%TEMP%` and rerun warm.

- **Verify**:

```powershell
$dest = Join-Path $env:TEMP 'noble.sha'
$uri = 'https://cdimage.ubuntu.com/ubuntu-wsl/noble/daily-live/current/' +
  'SHA256SUMS'
Invoke-WebRequest -Uri $uri -OutFile $dest
Get-Content $dest | Select-String noble-wsl
```

### T-004 WSL import errors

- **Symptom**: `wsl --import` returns access errors or `The process cannot
  access the file`.
- **Likely cause**: Target directory locked, low disk space, or antivirus holds
  the `.wsl` file.
- **Remediation**:

  1. Delete `%LOCALAPPDATA%\substrate\wsl` if it exists.
  1. Ensure at least 10 GB free on the system drive.
  1. Temporarily pause antivirus real-time protection.
  1. Rerun `scripts/windows/wsl-warm.ps1`.
  1. For manual debugging, run `scripts/windows/start-forwarder.ps1`
     to keep a five-minute timeout guard on the forwarder process.

     ```powershell
     pwsh -File scripts/windows/start-forwarder.ps1 `
       -DistroName substrate-wsl
     ```

- **Verify**:

```powershell
wsl -l -v | Select-String substrate-wsl
```

### T-005 Forwarder pipe unavailable

- **Symptom**: Doctor shows FAIL for `Forwarder Pipe` or `Forwarder PID`.
- **Likely cause**: Forwarder crashed, PID file stale, or warm script was
  interrupted.
- **Remediation**:

  1. Run `scripts/windows/wsl-stop.ps1` to clear PID and kill stray processes.
  1. Delete `%LOCALAPPDATA%\Substrate\forwarder.pid` and the forwarder log if
     they look corrupted.
  1. Rerun `scripts/windows/wsl-warm.ps1`.

- **Verify**:

```powershell
$client = [System.IO.Pipes.NamedPipeClientStream]::new(
    '.',
    'substrate-agent',
    [System.IO.Pipes.PipeDirection]::InOut
)
$client.Connect(3000)
$client.Dispose()
Get-Content "$env:LOCALAPPDATA\Substrate\logs\forwarder.log" -Tail 20
```

### T-006 Agent service not running

- **Symptom**: Doctor shows FAIL for `Agent Socket` or `Agent Capabilities`.
- **Likely cause**: `substrate-world-agent` service crashed or the binary is
  outdated.
- **Remediation**:

  1. Inspect logs inside WSL:

  ```powershell
  wsl -d substrate-wsl -- bash -lc 'journalctl -u substrate-world-agent -n 200'
  ```

  1. Rebuild `world-agent` on Windows and rerun warm if the binary changed.
  1. Restart the service inside WSL:

  ```powershell
  wsl -d substrate-wsl -- bash -lc 'sudo systemctl restart substrate-world-agent'
  ```

- **Verify**:

```powershell
wsl -d substrate-wsl -- bash -lc 'systemctl is-active substrate-world-agent'
```

### T-007 Doctor nftables failure

- **Symptom**: Doctor reports FAIL for `nftables` or `nft list tables` errors.
- **Likely cause**: `nftables` package missing or provisioning script aborted.
- **Remediation**:

  1. Reinstall the package inside WSL:

  ```powershell
  wsl -d substrate-wsl -- bash -lc 'sudo apt-get update'
  wsl -d substrate-wsl -- bash -lc 'sudo apt-get install -y nftables'
  ```

  1. If the failure returns, rerun `wsl-warm.ps1` to reprovision.

- **Verify**:

```powershell
wsl -d substrate-wsl -- bash -lc 'nft list tables'
```

### T-008 ConPTY or PTY stream failure

- **Symptom**: Smoke script PTY step fails or `substrate --pty` prints
  `PTY bridge unavailable`.
- **Likely cause**: Forwarder offline, ConPTY unsupported on host SKU, or agent
  lacks PTY capability.
- **Remediation**:

  1. Review forwarder log for connection attempts.
  1. Ensure warm and doctor succeed before running smoke.
  1. Capture PTY logs inside WSL:

  ```powershell
  wsl -d substrate-wsl -- bash -lc 'journalctl -u substrate-world-agent -n 200' |
    Select-String -Pattern pty
  ```

  1. If ConPTY is unsupported, use non-PTY commands and escalate to the plan
     owner.

- **Verify**:

```powershell
substrate --pty -c "bash -lc 'echo conpty-ok'"
```

### T-009 Path translation mismatch

- **Symptom**: Telemetry shows incorrect Windows paths or replay cannot locate
  files created inside WSL.
- **Likely cause**: Project mount missing, mixed casing, or UNC paths outside
  the repository root.
- **Remediation**:

  1. Confirm the project is mounted via `/mnt/c`:

  ```powershell
  wsl -d substrate-wsl -- bash -lc `
  'mount | grep /mnt/c'
  ```

  1. Avoid UNC paths or symlinks outside the repository root.
  1. If replay still fails, capture the conflicting paths and file an issue with
     reproduction steps.

- **Verify**:

```powershell
substrate -c "python - <<'PY'
import pathlib
print(pathlib.Path('README.md').resolve())
PY"
```

### T-010 Forwarder log stale

- **Symptom**: Doctor reports FAIL for `Forwarder Log` or the latest
  `forwarder.log` timestamp is older than expected.
- **Likely cause**: The forwarder exited, cannot write to
  `%LOCALAPPDATA%\Substrate\logs`, or the system clock jumped backward.
- **Remediation**:
  1. Run `scripts/windows/wsl-stop.ps1` to terminate any stale forwarder
     processes.
  2. Remove `%LOCALAPPDATA%\Substrate\logs\forwarder*.log*` if the
     directory is read-only and rerun warm.
  3. Relaunch the forwarder via `scripts/windows/wsl-warm.ps1` and monitor
     the new log.
- **Verify**:

```powershell
Get-ChildItem "$env:LOCALAPPDATA\Substrate\logs" `
  -Filter 'forwarder*.log*' |
  Sort-Object LastWriteTime -Descending |
  Select-Object -First 1
```

### T-011 WSL CLI missing

- **Symptom**: Doctor fails `WSL CLI` check or `Get-Command wsl` returns an
  error.
- **Likely cause**: Windows Subsystem for Linux binaries were removed, WSL
  optional feature disabled, or PATH misconfigured.
- **Remediation**:

  1. Ensure the `Microsoft-Windows-Subsystem-Linux` optional feature is enabled.
  1. Install the WSL package via `wsl --install` or the Microsoft Store if
     it was uninstalled.
  1. Reboot and rerun the doctor script.

- **Verify**:

```powershell
Get-Command wsl
wsl --status
```

### T-012 Host drive not mounted

- **Symptom**: Doctor reports FAIL for `WSL Mount (/mnt/c)` or path
  translation tests show Windows paths leaking into telemetry.
- **Likely cause**: The WSL distro is configured with custom mounts, the
  `drvfs` mount failed, or `/etc/wsl.conf` disables automount.
- **Remediation**:

  1. Inspect `/etc/wsl.conf` inside the distro and ensure `[automount] enabled=true`.
  1. Restart the distro (`wsl --terminate <distro>`), or reboot the host to
     rebuild mounts.
  1. If automount is disabled intentionally, update scripts to mount the
     project path under `/mnt/c`.

- **Verify**:

```powershell
wsl -d substrate-wsl -- bash -lc `
  'mount | grep /mnt/c'
```

### T-011 Pipe name already in use

- Symptom: Forwarder exits immediately with `Access is denied. (os error 5)`
  right after logging "listening on named pipe"; `Test-Path \\.`\`\pipe\substrate-agent`
  returns `True` even when no forwarder is running.
- Likely cause: Another process owns `\\.\pipe\substrate-agent` with a different
  ACL, or a previously launched forwarder (service) is still running under a
  different session. `CreateNamedPipe` fails with `ERROR_ACCESS_DENIED` when the
  name already exists.
- Remediation:
  1. Run `scripts/windows/wsl-stop.ps1` to terminate known forwarders.
  2. Check for the pipe’s existence and look for owners (Sysinternals `handle.exe`):

     ```powershell
     Test-Path \\.\pipe\substrate-agent
     # Optional if Sysinternals is available
     handle.exe substrate-agent | findstr /i ":\\pipe\\substrate-agent"
     ```

  3. As a temporary workaround, rerun warm with a unique pipe name:

     ```powershell
     pwsh -File scripts/windows/wsl-warm.ps1 -PipePath \\.\pipe\substrate-agent-$env:USERNAME
     ```

  4. Once ownership is identified, stop the conflicting process or reconfigure
     it to use a different name; then revert to the default path.
- Verify:

```powershell
Test-Path \\.\pipe\substrate-agent # should be False after the conflict is removed
```
