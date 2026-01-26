# Manual Testing Playbook

This playbook must contain runnable commands and expected exit codes/output.

## Behavioral Smoke Scripts

These scripts define the behavioral platform contract for this feature. Keep them deterministic and fast.
- Linux: `bash smoke/linux-smoke.sh` (expected exit: 0)
- macOS: `bash smoke/macos-smoke.sh` (expected exit: 0)
- Windows (CI parity-only; no behavioral assertions for this feature): `pwsh -File smoke/windows-smoke.ps1` (expected exit: 0)

## CI Parity (compile/test)

CI parity platforms (can be broader than behavioral scope): `linux,macos,windows`

Required gates:
- `make ci-compile-parity CI_WORKFLOW_REF="feat/full-isolation-landlock-overlayfs-compat" CI_REMOTE=origin CI_CLEANUP=1`
- `scripts/ci/dispatch_ci_testing.sh --workflow-ref "feat/full-isolation-landlock-overlayfs-compat" --remote origin --cleanup`

## Linux manual validation (local)

Preconditions:
- The world backend is provisioned and reachable (`substrate world doctor --json` returns exit `0` and `.ok=true`).
- The kernel supports Landlock (`.world.landlock.supported=true`).
- The world filesystem strategy primary is overlay (`.world.world_fs_strategy.primary=="overlay"`).

Run:
- `bash docs/project_management/_archived/full-isolation-landlock-overlayfs-compat/smoke/linux-smoke.sh`

Expected:
- Exit `0`.
- Output contains:
  - `OK: allowlisted write succeeded`
  - `OK: denied write remained denied`

## macOS manual validation (local)

Preconditions:
- The world backend is provisioned and reachable (`substrate world doctor --json` returns exit `0` and `.ok=true`).
- The kernel supports Landlock (`.world.landlock.supported=true`) in the Lima Linux guest.
- The world filesystem strategy primary is overlay (`.world.world_fs_strategy.primary=="overlay"`).

Run:
- `bash docs/project_management/_archived/full-isolation-landlock-overlayfs-compat/smoke/macos-smoke.sh`

Expected:
- Exit `0`.
- Output contains:
  - `OK: allowlisted write succeeded`
  - `OK: denied write remained denied`

## Windows manual validation (local)

Run:
- `pwsh -NoProfile -File docs/project_management/_archived/full-isolation-landlock-overlayfs-compat/smoke/windows-smoke.ps1`

Expected:
- Exit `0`.
- Output contains `OK: Windows smoke is a no-op for this feature`.

## CI behavioral smoke dispatch (required for Linux + macOS behavior validation)

Run from the integration worktree `HEAD`:
- `make feature-smoke FEATURE_DIR="docs/project_management/_archived/full-isolation-landlock-overlayfs-compat" PLATFORM=behavior SMOKE_SLICE_ID="C0" RUNNER_KIND=self-hosted WORKFLOW_REF="feat/full-isolation-landlock-overlayfs-compat" REMOTE=origin CLEANUP=1 RUN_INTEG_CHECKS=1`

Expected:
- The dispatcher prints `DISPATCH_OK=1` and a `RUN_URL=...`.
- The workflow concludes success for Linux and macOS.
