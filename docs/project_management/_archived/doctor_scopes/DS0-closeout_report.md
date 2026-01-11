# Slice Closeout Gate Report — doctor_scopes / DS0

Date (UTC): 2026-01-09

Standards:
- `docs/project_management/standards/SLICE_CLOSEOUT_GATE_STANDARD.md`
- `docs/project_management/standards/EXECUTIVE_SUMMARY_STANDARD.md` (behavior delta format)

Feature directory:
- `docs/project_management/_archived/doctor_scopes/`

Slice spec:
- `docs/project_management/_archived/doctor_scopes/DS0-spec.md`

## Status

COMPLETED.

## Behavior Delta (Existing → New → Why)

- Existing behavior: `substrate world doctor` is host-oriented on macOS and mixes host/world facts on Linux without an explicit scope split.
- New behavior: introduce `substrate host doctor`; redefine `substrate world doctor` to include `host` + `world` blocks with world facts sourced from the world-agent endpoint `GET /v1/doctor/world`.
- Why: operators need a single, authoritative answer for “is isolation enforceable right now?” across Linux and macOS, including guest-kernel facts on macOS.
- Links:
  - `docs/project_management/next/ADR-0007-host-and-world-doctor-scopes.md`
  - `docs/project_management/_archived/doctor_scopes/DS0-spec.md`

## Spec Parity (No Drift)

- Acceptance criteria satisfied: `YES`
- Spec changes during the slice: `NONE EXPECTED (spec is authoritative; drift must be justified in writing if it occurs)`

## Checks Run (Evidence)

- `cargo fmt`: `PASS` (via `make integ-checks`)
- `cargo clippy --workspace --all-targets -- -D warnings`: `PASS` (via `make integ-checks`)
- Relevant tests: `PASS` (`cargo test --workspace --all-targets` via `make integ-checks`)
- `make integ-checks`: `PASS` (exit `0`)

## Cross-Platform Smoke

Record run ids/URLs for required platforms:
- CI compile parity (linux/macos/windows):
  - RUN_ID: `20861321448`
  - RUN_URL: `https://github.com/atomize-hq/substrate/actions/runs/20861321448`
  - Result: `success` (`ubuntu-24.04`, `macos-14`, `windows-2022`)
- Linux smoke:
  - RUN_ID: `20861533380`
  - RUN_URL: `https://github.com/atomize-hq/substrate/actions/runs/20861533380`
  - Result: `success`
- macOS smoke:
  - RUN_ID: `20861578627`
  - RUN_URL: `https://github.com/atomize-hq/substrate/actions/runs/20861578627`
  - Result: `success`

Platform-fix work summary (must be explicit; use `NONE` if nothing was needed):
- What failed:
  - macOS smoke runners can have a provisioned Lima world-agent that predates `GET /v1/doctor/world`, causing world doctor requests to fail and smoke to fail.
  - Linux self-hosted runners can have a world-agent that predates `GET /v1/doctor/world`, causing smoke to fail.
  - `make ci-compile-parity` dispatch can fail when `.github/workflows/ci-compile-parity.yml` is not registered on the default branch.
- What was changed:
  - Linux: treat `GET /v1/doctor/world` `HTTP 404` as “legacy agent”; fall back to a lightweight world probe via `/v1/execute` that validates the DS0 contract (Landlock + enumeration probe) without requiring runner reprovisioning.
  - macOS: probe doctor endpoint via Lima guest; fall back to a VM-side probe when the endpoint is missing; harden Landlock ABI detection; improve limactl error capture; stabilize CI log noise.
  - CI: dispatch `ci-compile-parity` via `ci-testing` when the workflow path is not dispatchable from the default branch.
- Why the change is safe (guards, cfg, feature flags):
  - All fallbacks are opt-in by observed conditions (e.g., `HTTP 404` on `/v1/doctor/world` or missing guest paths) and do not change the steady-state path when the endpoint exists and is healthy.
  - Platform behavior is confined to per-platform modules (`linux.rs`, `macos.rs`) and does not introduce new configuration keys or env toggles.

## Smoke ↔ Manual Parity

- Smoke scripts mirror manual playbook: `YES` (`docs/project_management/_archived/doctor_scopes/manual_testing_playbook.md` ↔ `docs/project_management/_archived/doctor_scopes/smoke/`)
- Smoke scripts validate exit codes + key output: `YES` (jq assertions on DS0 v1 JSON contracts)
