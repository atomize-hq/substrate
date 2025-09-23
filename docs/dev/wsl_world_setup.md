
# Windows WSL World Setup Guide

This guide explains how to provision and maintain the `substrate-wsl`
distribution that powers the Windows Always World stack. Follow these steps so
every `substrate` command executes inside WSL with full telemetry and
isolation.

## Prerequisites

### Host requirements

1. **Windows build**: Windows 11 22H2 (or later) or Windows 10 22H2 with WSL2.

1. **Virtualization**: Firmware virtualization must be enabled. Verify with:

```powershell
systeminfo | Select-String "Virtualization"
```

   Each reported capability should end with `Yes`.

1. **Optional features**: Enable WSL and VirtualMachinePlatform. Check with:

```powershell
Get-WindowsOptionalFeature -Online -FeatureName Microsoft-Windows-Subsystem-Linux
Get-WindowsOptionalFeature -Online -FeatureName VirtualMachinePlatform
```

   Both commands should report `State : Enabled`. If not, enable and reboot.

1. **WSL status**: Confirm WSL is healthy and the default version is 2:

```powershell
wsl --status
```

1. **PowerShell 7**: Required for the helper scripts. Install if `pwsh` is
   missing:

```powershell
winget install --id Microsoft.PowerShell -e --scope user
```

1. **Developer toolchain**: Ensure Git, Rust, and Python are present:

```powershell
winget install --id Git.Git -e
winget install --id Rustlang.Rustup -e
winget install --id Python.Python.3.12 -e
```

1. **Execution policy**: Allow local scripts for the current user:

```powershell
Set-ExecutionPolicy -Scope CurrentUser -ExecutionPolicy RemoteSigned
```

### Repository location

Clone `substrate` into a path without spaces (the plan references
`C:\workspace\substrate`). Run helper scripts from the repo root or pass
`-ProjectPath` when using a different path.

## First-time setup

### Step 1: Warm the environment

The warm script performs these tasks:

- Downloads the latest Ubuntu 24.04 `.wsl` image from Canonical.
- Imports (or upgrades) the `substrate-wsl` distribution as WSL2.
- Executes `docs/dev/wsl/provision.sh` inside the distro to install packages
  and configure the `substrate-world-agent` service.
- Builds `substrate-forwarder` and `world-agent` if needed, copies the agent
  into WSL, and restarts the service.
- Starts the Windows forwarder and waits for `\.\pipe\substrate-agent`.

Run the script from the repo root:

```powershell
pwsh -File scripts/windows/wsl-warm.ps1 `
  -DistroName substrate-wsl `
  -ProjectPath (Resolve-Path .)
```

Sample success excerpt:

```text
[INFO] Starting wsl-warm for distro 'substrate-wsl'
[INFO] Project path: C:\workspace\substrate
[INFO] Importing distro 'substrate-wsl'
[INFO] Downloading Ubuntu WSL image (noble-wsl-amd64.wsl)
[INFO] Waiting for forwarder pipe \.\pipe\substrate-agent
[INFO] Forwarder pipe ready
[INFO] Warm complete
```

> Tip: Pass `-WhatIf` for dry runs. It validates prerequisites without
> downloading or provisioning.

### Step 2: Validate the distro

Ensure the distro exists after warm finishes:

```powershell
wsl -l -v | Select-String substrate-wsl
```

The state should be `Running` immediately after warm or `Stopped` later on.

### Step 3: Confirm agent and forwarder

Check both sides of the bridge:

```powershell
# inside WSL
wsl -d substrate-wsl -- bash -lc 'systemctl status substrate-world-agent'

# on Windows
Get-Content "$env:LOCALAPPDATA\Substrate\logs\forwarder.log" -Tail 50  # logs rotate daily (5 files, 10 MB each)
```

## Updating the agent binary

When `world-agent` changes, rebuild and rerun warm so the new binary is copied
into WSL:

```powershell
cargo build -p world-agent --release
pwsh -File scripts/windows/wsl-warm.ps1 `
  -DistroName substrate-wsl `
  -ProjectPath (Resolve-Path .)
```

A future plan step adds `-SkipProvision`; until then, rerunning the full warm
script is safe and idempotent.

## Running the doctor script

Use the doctor script to verify host prerequisites, distro health, forwarder,
and agent endpoints:

```powershell
pwsh -File scripts/windows/wsl-doctor.ps1 -DistroName substrate-wsl
pwsh -File scripts/windows/wsl-doctor.ps1 -DistroName substrate-wsl -Json `
  | ConvertFrom-Json | Format-Table
```

Expected PASS output (Appendix I):

```text
[substrate/windows doctor]
Virtualization: PASS (Enabled)
WSL status: PASS (Default version 2, kernel 5.15.133.1)
Distro substrate-wsl: PASS (Running)
Forwarder pipe: PASS (\.\pipe\substrate-agent)
Agent socket: PASS (/run/substrate.sock exists)
Agent capabilities: PASS (features: execute, pty_streaming, trace_retrieval)
Nftables: PASS
Logs: PASS (no errors in last 100 lines)
```

If a check fails, read the remediation column in the table and consult
`docs/dev/wsl_world_troubleshooting.md` for detailed guidance.

## Smoke test

After Phase 5 scripts are in place, run the smoke suite for end-to-end
validation:

```powershell
pwsh -File scripts/windows/wsl-smoke.ps1 `
  -DistroName substrate-wsl `
  -ProjectPath (Resolve-Path .)
```

The script performs:

1. Warm (unless `-SkipWarm` is supplied).
1. Doctor checks.
1. Non-PTY span verification to ensure the latest trace entry includes
   `world_id` and `fs_diff`.
1. PTY execution test through ConPTY and WebSocket.
1. Replay of the most recent span.
1. Forwarder restart resilience.

Record the console output in the evidence log after each run.

## Common operations

- **Stop distro and forwarder**

  ```powershell
  pwsh -File scripts/windows/wsl-stop.ps1 -DistroName substrate-wsl
  ```

- **Launch an interactive shell**

  ```powershell
  wsl -d substrate-wsl -- bash
  ```

- **View agent logs**

  ```powershell
  wsl -d substrate-wsl -- bash -lc 'journalctl -u substrate-world-agent -n 200'
  ```

- **Tail forwarder log**

  ```powershell
  Get-Content "$env:LOCALAPPDATA\Substrate\logs\forwarder.log" -Tail 200 -Wait
  ```

## Troubleshooting

A dedicated catalogue lives at `docs/dev/wsl_world_troubleshooting.md`. It
covers scenarios such as:

- Virtualization disabled or blocked by VBS.
- Optional feature enable failures.
- WSL import errors or stale `.wsl` images.
- Forwarder pipe permissions or orphaned PID files.
- Agent provisioning failures within the distro.
- ConPTY or WebSocket connectivity issues.
- Windows/WSL path translation mismatches.

Use the catalogue when doctor or smoke checks fail, and include the relevant
entry ID in the evidence log when documenting remediation.

## Maintenance checklist

1. Keep the helper scripts aligned with the plan appendices.
1. Run the doctor script after OS, firmware, or WSL updates.
1. Re-run the smoke script after modifying `world-agent` or forwarder code.
1. For releases, capture doctor and smoke outputs and attach them to the
   Windows evidence log.

## Support

- Team channel: `#substrate-windows`
- Plan owner: Substrate Core

Follow the Phase 5 guardrails: execute steps in order, capture evidence after
Each check, and stop immediately if a command fails.

