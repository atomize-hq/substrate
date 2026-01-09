# Execution Preflight Gate Report — doctor_scopes

Date (UTC): 2026-01-09

Standard:
- `docs/project_management/standards/EXECUTION_PREFLIGHT_GATE_STANDARD.md`

Feature directory:
- `docs/project_management/next/doctor_scopes/`

## Recommendation

RECOMMENDATION: **ACCEPT**

Rationale:
- `tasks.json` platform declarations match the DS0 spec intent: behavioral smoke on `linux, macos` and CI parity on `linux, macos, windows` (no WSL).
- Smoke scripts are runnable and directly mirror the `manual_testing_playbook.md` assertions for the “happy path” JSON contracts on Linux/macOS; Windows smoke is an explicit no-op as required for CI-parity-only.
- CI dispatch paths referenced by integration tasks are valid (`make ci-compile-parity ...`, `make feature-smoke ...`), and the `feat/doctor-scopes` workflow ref exists on `origin`.

## Inputs Reviewed

- [x] ADR accepted and still matches intent
- [x] Planning Pack complete (`plan.md`, `tasks.json`, `session_log.md`, specs, kickoff prompts)
- [x] Triad sizing is appropriate (each slice is one behavior delta; no “grab bag” slices)
- [x] Cross-platform plan is explicit (`tasks.json` meta: behavior + CI parity platforms)
- [x] `manual_testing_playbook.md` exists and is runnable
- [x] Smoke scripts exist and map to the manual playbook

## Cross-Platform Coverage

- Declared behavior platforms (smoke required): `linux, macos`
- Declared CI parity platforms (parity required): `linux, macos, windows`
- WSL required: `no`

## Smoke ↔ Manual Parity Check

Smoke scripts to validate:
- Linux smoke: `docs/project_management/next/doctor_scopes/smoke/linux-smoke.sh`
- macOS smoke: `docs/project_management/next/doctor_scopes/smoke/macos-smoke.sh`
- Windows smoke: `docs/project_management/next/doctor_scopes/smoke/windows-smoke.ps1` (must be a no-op; Windows is CI-parity-only here)

Manual playbook:
- `docs/project_management/next/doctor_scopes/manual_testing_playbook.md`

Parity notes:
- Linux smoke asserts `substrate host doctor --json` and `substrate world doctor --json` stable fields, matching the Linux playbook contract checks.
- macOS smoke asserts the same stable fields for `host doctor` and `world doctor`, matching the macOS playbook contract checks.
- Windows smoke is intentionally a no-op, matching the playbook’s “CI parity only” stance for DS0.

## CI Dispatch Readiness

The integration tasks in `docs/project_management/next/doctor_scopes/tasks.json` require:
- `make ci-compile-parity CI_WORKFLOW_REF="feat/doctor-scopes" CI_REMOTE=origin CI_CLEANUP=1`
- `make feature-smoke FEATURE_DIR="docs/project_management/next/doctor_scopes" PLATFORM=linux RUNNER_KIND=self-hosted WORKFLOW_REF="feat/doctor-scopes" REMOTE=origin CLEANUP=1 RUN_INTEG_CHECKS=1`
- `make feature-smoke FEATURE_DIR="docs/project_management/next/doctor_scopes" PLATFORM=macos RUNNER_KIND=self-hosted WORKFLOW_REF="feat/doctor-scopes" REMOTE=origin CLEANUP=1 RUN_INTEG_CHECKS=1`

Preflight must confirm:
- The workflow ref exists on the remote (`feat/doctor-scopes` is pushed): `origin/feat/doctor-scopes`.
- Self-hosted runners exist and are online for behavior smoke:
  - Linux: `linux-manjaro-runner` (labels include `self-hosted`, `Linux`, `linux-host`)
  - macOS: `macOS-runner` (labels include `self-hosted`, `macOS`)
- GitHub-hosted CI parity workflow is enabled and runnable (`.github/workflows/ci-compile-parity.yml` is `workflow_dispatch`-capable).

Run ids/URLs (leave blank until preflight is executed):
- CI compile parity:
- Linux smoke:
- macOS smoke:

## Required Fixes Before Starting DS0

- Planning quality gate must be `RECOMMENDATION: ACCEPT` before starting DS0 (this execution preflight gate does not override the planning gate ordering requirement in `EXECUTION_PREFLIGHT_GATE_STANDARD.md`).
