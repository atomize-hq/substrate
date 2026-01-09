# Execution Preflight Gate Report — doctor_scopes

Date (UTC): 2026-01-08

Standard:
- `docs/project_management/standards/EXECUTION_PREFLIGHT_GATE_STANDARD.md`

Feature directory:
- `docs/project_management/next/doctor_scopes/`

## Recommendation

RECOMMENDATION: **REVISE**

Rationale:
- This Planning Pack is fully authored, but execution triads must not begin until:
  - the planning quality gate is performed and recorded in `quality_gate_report.md`, and
  - the preflight reviewer validates runner readiness and CI dispatch correctness for the declared platform set.

## Inputs Reviewed

- [ ] ADR accepted and still matches intent
- [ ] Planning Pack complete (`plan.md`, `tasks.json`, `session_log.md`, specs, kickoff prompts)
- [ ] Triad sizing is appropriate (each slice is one behavior delta; no “grab bag” slices)
- [ ] Cross-platform plan is explicit (`tasks.json` meta: behavior + CI parity platforms)
- [ ] `manual_testing_playbook.md` exists and is runnable
- [ ] Smoke scripts exist and map to the manual playbook

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

## CI Dispatch Readiness

The integration tasks in `docs/project_management/next/doctor_scopes/tasks.json` require:
- `make ci-compile-parity CI_WORKFLOW_REF="feat/doctor-scopes" CI_REMOTE=origin CI_CLEANUP=1`
- `make feature-smoke FEATURE_DIR="docs/project_management/next/doctor_scopes" PLATFORM=linux RUNNER_KIND=self-hosted WORKFLOW_REF="feat/doctor-scopes" REMOTE=origin CLEANUP=1 RUN_INTEG_CHECKS=1`
- `make feature-smoke FEATURE_DIR="docs/project_management/next/doctor_scopes" PLATFORM=macos RUNNER_KIND=self-hosted WORKFLOW_REF="feat/doctor-scopes" REMOTE=origin CLEANUP=1 RUN_INTEG_CHECKS=1`

Preflight must confirm:
- The workflow ref exists on the remote (`feat/doctor-scopes` must be pushed before dispatch).
- Self-hosted runners exist for linux and macos behavior smoke.
- GitHub-hosted CI parity workflow is enabled and runnable for the repo.

Run ids/URLs (leave blank until preflight is executed):
- CI compile parity:
- Linux smoke:
- macOS smoke:

## Required Fixes Before Starting DS0

- Run and record the planning quality gate in `docs/project_management/next/doctor_scopes/quality_gate_report.md` (must not be “FLAG FOR HUMAN REVIEW”).
- Re-run this preflight and flip `RECOMMENDATION` to **ACCEPT** before starting DS0.
