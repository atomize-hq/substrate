# CI Runs: World-Agent Policy Snapshot (WAPS)

This file records CI workflow-dispatch runs executed for WAPS cross-platform smoke + compile parity.

## 2026-01-19 (UTC)

- Ref (workflow dispatch ref): `feat/policy-patch-only-broker-effective-resolution`
- Commit (checkout target): `3c99ea6e14c80d98862cb6097ec461934b09a2ee`
- Remote: `origin`

### `make ci-compile-parity` (CI Testing / compile-parity)

- Workflow: `CI Testing` (`.github/workflows/ci-testing.yml`)
- Workflow ref: `feat/policy-patch-only-broker-effective-resolution`
- Checkout ref (temp branch): `tmp/ci-compile-parity/20260119T044534Z`
- Runner kind: `github-hosted` (runs-on: `ubuntu-24.04`, `macos-14`, `windows-2022`)
- Run ID: `21125633215`
- Run URL: https://github.com/atomize-hq/substrate/actions/runs/21125633215
- Platforms executed: `ubuntu-24.04`, `macos-14`, `windows-2022`
- Platforms passed: (none)
- Platforms failed: `ubuntu-24.04`, `macos-14`, `windows-2022`
- Failed jobs: `Lint & Test (macos-14)`, `Lint & Test (ubuntu-24.04)`, `Lint & Test (windows-2022)`
- Final: **FAIL** (`conclusion=failure`)

### `make feature-smoke-all` (Feature Smoke / WAPS)

- Workflow: `Feature Smoke (Planning Pack)` (`.github/workflows/feature-smoke.yml`)
- Workflow ref: `feat/policy-patch-only-broker-effective-resolution`
- Feature dir: `docs/project_management/next/world-agent-policy-snapshot`
- Runner kind: `self-hosted`
- Platform selection: `all`
- Checkout ref (temp branch): `tmp/feature-smoke/world-agent-policy-snapshot/all/20260119T044738Z`
- Run ID: `21125669297`
- Run URL: https://github.com/atomize-hq/substrate/actions/runs/21125669297
- Platforms executed: `linux`, `macos`, `windows`
- Platforms passed: (none)
- Platforms failed: `linux`, `macos`, `windows`
- Smoke run ids:
    - linux: `waps-1768798093-2215825`
    - macos: `waps-1768798142-54970`
    - windows: (not emitted; job failed before running the smoke script: missing `docs/project_management/next/world-agent-policy-snapshot/smoke/windows-smoke.ps1`)
- Final: **FAIL** (`conclusion=failure`)
