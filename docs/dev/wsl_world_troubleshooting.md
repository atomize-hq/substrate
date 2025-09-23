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
Test-Path "\\.\pipe\substrate-agent"
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
  wsl -d substrate-wsl -- bash -lc 'mount | grep /mnt/c'
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
