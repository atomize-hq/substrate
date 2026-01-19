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

#### Reruns (chronological)

| Run ID | Run URL | Conclusion | Passed | Failed | Notes |
| --- | --- | --- | --- | --- | --- |
| `21126880019` | https://github.com/atomize-hq/substrate/actions/runs/21126880019 | `failure` | (unknown) | (unknown) | first rerun after initial failures |
| `21126938288` | https://github.com/atomize-hq/substrate/actions/runs/21126938288 | `success` | `ubuntu-24.04`, `macos-14`, `windows-2022` | (none) | PASS (compile parity fixed) |
| `21127600131` | https://github.com/atomize-hq/substrate/actions/runs/21127600131 | `success` | `ubuntu-24.04`, `macos-14`, `windows-2022` | (none) | PASS (post-smoke fixes) |
| `21130938721` | https://github.com/atomize-hq/substrate/actions/runs/21130938721 | `failure` | `ubuntu-24.04`, `macos-14` | `windows-2022` | clippy: `process_agent_stream` unused + needless `return` |
| `21131034869` | https://github.com/atomize-hq/substrate/actions/runs/21131034869 | `success` | `ubuntu-24.04`, `macos-14`, `windows-2022` | (none) | PASS (clippy clean) |

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

#### Reruns (chronological)

| Run ID | Run URL | Conclusion | Passed | Failed | Notes |
| --- | --- | --- | --- | --- | --- |
| `21127001275` | https://github.com/atomize-hq/substrate/actions/runs/21127001275 | `failure` | (unknown) | (unknown) | rerun after adding Windows entrypoint |
| `21127165394` | https://github.com/atomize-hq/substrate/actions/runs/21127165394 | `failure` | (unknown) | (unknown) | rerun (mid-remediation) |
| `21127400575` | https://github.com/atomize-hq/substrate/actions/runs/21127400575 | `failure` | `linux` | `macos`, `windows` | macOS world backend unavailable, Windows trace parsing |
| `21127666844` | https://github.com/atomize-hq/substrate/actions/runs/21127666844 | `failure` | `linux` | `macos`, `windows` | Windows trace-meta strict-mode crash; macOS mount failures |
| `21127760249` | https://github.com/atomize-hq/substrate/actions/runs/21127760249 | `failure` | `linux`, `macos` | `windows` | Windows: strict-mode meta parsing |
| `21127840385` | https://github.com/atomize-hq/substrate/actions/runs/21127840385 | `failure` | `linux`, `macos` | `windows` | Windows: command exits/metadata missing |
| `21128045234` | https://github.com/atomize-hq/substrate/actions/runs/21128045234 | `failure` | `linux`, `macos` | `windows` | Windows: WSL world execution unstable |
| `21128331821` | https://github.com/atomize-hq/substrate/actions/runs/21128331821 | `failure` | `linux`, `macos` | `windows` | Windows: WSL exec timeouts |
| `21128478370` | https://github.com/atomize-hq/substrate/actions/runs/21128478370 | `failure` | `linux`, `macos` | `windows` | Windows: WSL exec timeouts |
| `21128613039` | https://github.com/atomize-hq/substrate/actions/runs/21128613039 | `failure` | `linux`, `macos` | `windows` | Windows: WSL exec timeouts |
| `21128820499` | https://github.com/atomize-hq/substrate/actions/runs/21128820499 | `failure` | `linux`, `macos` | `windows` | Windows: WSL exec timeouts |
| `21128938418` | https://github.com/atomize-hq/substrate/actions/runs/21128938418 | `failure` | `linux`, `macos` | `windows` | Windows: WSL exec timeouts |
| `21129120189` | https://github.com/atomize-hq/substrate/actions/runs/21129120189 | `failure` | `linux`, `macos` | `windows` | Windows: WSL exec timeouts |
| `21129304171` | https://github.com/atomize-hq/substrate/actions/runs/21129304171 | `failure` | `linux`, `macos` | `windows` | Windows: WSL exec timeouts |
| `21129494613` | https://github.com/atomize-hq/substrate/actions/runs/21129494613 | `failure` | `linux`, `macos` | `windows` | Windows: WSL exec timeouts |
| `21129804137` | https://github.com/atomize-hq/substrate/actions/runs/21129804137 | `failure` | `linux`, `macos` | `windows` | Windows: command arg parsing + wait handling issues |
| `21129913653` | https://github.com/atomize-hq/substrate/actions/runs/21129913653 | `failure` | `linux`, `macos` | `windows` | Windows: WSL exec timeouts (stderr showed only tracing setup) |
| `21130117622` | https://github.com/atomize-hq/substrate/actions/runs/21130117622 | `failure` | `linux`, `macos` | `windows` | Windows: WSL exec timeouts |
| `21130336563` | https://github.com/atomize-hq/substrate/actions/runs/21130336563 | `failure` | `linux`, `macos` | `windows` | Windows: WSL exec timeouts |
| `21130549796` | https://github.com/atomize-hq/substrate/actions/runs/21130549796 | `failure` | `linux`, `macos` | `windows` | Windows: WSL exec timeouts (TCP forwarder enabled) |
| `21130859601` | https://github.com/atomize-hq/substrate/actions/runs/21130859601 | `success` | `linux`, `macos`, `windows` | (none) | PASS (Windows smoke gates on doctor OK) |
