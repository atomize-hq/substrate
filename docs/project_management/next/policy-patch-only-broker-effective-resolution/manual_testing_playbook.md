# Manual Testing Playbook — Policy Patch-only + Broker-Canonical Effective Resolution

Exit codes:
- Exit code taxonomy: `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`

Smoke scripts (minimal, runnable subset of this playbook):
- Linux: `docs/project_management/next/policy-patch-only-broker-effective-resolution/smoke/linux-smoke.sh`
- macOS: `docs/project_management/next/policy-patch-only-broker-effective-resolution/smoke/macos-smoke.sh`
- Windows: `docs/project_management/next/policy-patch-only-broker-effective-resolution/smoke/windows-smoke.ps1`

## Preconditions
- Repo is checked out and buildable.
- `cargo` is available on PATH.
- Linux/macOS: `python3` is available on PATH.
- Windows: `pwsh` is available on PATH.

## Shared test setup (all platforms)

Goal:
- Validate policy effective resolution, disabled workspace semantics, and fail-closed behavior on invalid policy patches.

Inputs created in a scratch directory:
- `SUBSTRATE_HOME` points to a scratch home containing `policy.yaml` (global patch).
- A scratch workspace containing:
  - `.substrate/workspace.yaml` (workspace marker; created via `substrate workspace init --force`)
  - `.substrate/policy.yaml` (workspace patch)
  - optional `.substrate/workspace.disabled` (disabled marker)

## Additional required validation (C1 only): shim + world-agent

The smoke scripts run the Substrate CLI via `scripts/dev/substrate_shell_driver`, which injects `--no-world` when neither `--world` nor `--no-world` is provided.
The smoke scripts do not validate world-agent execution.

Run (C1):
- `cargo test -p substrate-shim -- --nocapture`
- `cargo test -p world-agent -- --nocapture`

Expected:
- Exit `0`.

## Linux

Run (C0):
- `SUBSTRATE_SMOKE_SLICE_ID=C0 bash docs/project_management/next/policy-patch-only-broker-effective-resolution/smoke/linux-smoke.sh`

Expected:
- Exit `0`.

Run (C1):
- `SUBSTRATE_SMOKE_SLICE_ID=C1 bash docs/project_management/next/policy-patch-only-broker-effective-resolution/smoke/linux-smoke.sh`

Expected:
- Exit `0`.

## macOS

Run (C0):
- `SUBSTRATE_SMOKE_SLICE_ID=C0 bash docs/project_management/next/policy-patch-only-broker-effective-resolution/smoke/macos-smoke.sh`

Expected:
- Exit `0`.

Run (C1):
- `SUBSTRATE_SMOKE_SLICE_ID=C1 bash docs/project_management/next/policy-patch-only-broker-effective-resolution/smoke/macos-smoke.sh`

Expected:
- Exit `0`.

## Windows

Run (C0):
- `pwsh -NoProfile -Command '$env:SUBSTRATE_SMOKE_SLICE_ID=\"C0\"; pwsh -File docs/project_management/next/policy-patch-only-broker-effective-resolution/smoke/windows-smoke.ps1'`

Expected:
- Exit `0`.

Run (C1):
- `pwsh -NoProfile -Command '$env:SUBSTRATE_SMOKE_SLICE_ID=\"C1\"; pwsh -File docs/project_management/next/policy-patch-only-broker-effective-resolution/smoke/windows-smoke.ps1'`

Expected:
- Exit `0`.
