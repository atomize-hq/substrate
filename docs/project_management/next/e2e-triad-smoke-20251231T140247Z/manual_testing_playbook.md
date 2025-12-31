# Manual Testing Playbook

This playbook must contain runnable commands and expected exit codes/output.

## CI Smoke Scripts

These are invoked by the Feature Smoke workflow. Keep them deterministic and fast.

- Linux: `bash smoke/linux-smoke.sh` (expected exit: 0)
- macOS: `bash smoke/macos-smoke.sh` (expected exit: 0)
- Windows: `pwsh -File smoke/windows-smoke.ps1` (expected exit: 0)
