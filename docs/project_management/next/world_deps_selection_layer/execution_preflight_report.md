# Execution Preflight Gate Report — world_deps_selection_layer

Date (UTC): 2026-01-11

Standard:
- `docs/project_management/standards/EXECUTION_PREFLIGHT_GATE_STANDARD.md`

Feature directory:
- `docs/project_management/next/world_deps_selection_layer`

## Recommendation

RECOMMENDATION: **ACCEPT**

Rule:
- The `F0-exec-preflight` task must replace `NOT RUN` with exactly one of: `ACCEPT` or `REVISE`.

## Inputs Reviewed

- Planning quality gate is `ACCEPT` (`docs/project_management/next/world_deps_selection_layer/quality_gate_report.md`): VERIFIED
- ADR accepted and still matches intent (`docs/project_management/next/ADR-0002-world-deps-install-classes-and-world-provisioning.md`): VERIFIED
- Planning Pack complete (`plan.md`, `tasks.json`, `session_log.md`, specs, kickoff prompts): VERIFIED
- Triad sizing is appropriate (each slice is one behavior delta; no “grab bag” slices): VERIFIED
- Required planning artifacts exist: `integration_map.md`, `manual_testing_playbook.md`: VERIFIED
- Cross-platform plan is explicit (tasks.json meta: behavior + CI parity platforms): VERIFIED

## 0) Slice Sizing (one behavior delta each)

- Slices reviewed: `WDL0` (selection config + UX), `WDL1` (install classes), `WDL2` (provisioning-time system packages)
- Required splits before starting execution: NONE

## 1) Cross-Platform Coverage

Expected from `docs/project_management/next/world_deps_selection_layer/tasks.json` meta:
- Behavior platforms (smoke required): `["linux","macos","windows"]`
- CI parity platforms (compile parity required): `["linux","macos","windows"]`
- WSL required: `false` (by omission)

Verified:
- `schema_version=3`, behavior platforms, and CI parity platforms match the expected sets.
- `automation.enabled=true` and `automation.orchestration_branch="feat/world_deps_selection_layer"`.

## 2) Smoke Scripts Are Not “Toy” Checks

Manual playbook:
- `docs/project_management/next/world_deps_selection_layer/manual_testing_playbook.md`

Smoke scripts:
- Linux: `docs/project_management/next/world_deps_selection_layer/smoke/linux-smoke.sh`
- macOS: `docs/project_management/next/world_deps_selection_layer/smoke/macos-smoke.sh`
- Windows: `docs/project_management/next/world_deps_selection_layer/smoke/windows-smoke.ps1`

Required parity mapping (must be recorded during preflight):
- Manual steps validated by smoke: VERIFIED (the smoke scripts implement sections `1` and `2` of the manual playbook, and capability-gate the WDL1/WDL2 steps per spec).
- Critical assertions validated by smoke (exit codes + key substrings): VERIFIED

S2 “Automation hooks (required)” coverage (verified by inspection of the smoke scripts):
- Environment isolation: each smoke script uses an isolated `SUBSTRATE_HOME` and a temporary workspace directory.
- “No world-agent calls” proof for no-op paths: each smoke script sets `SUBSTRATE_WORLD_SOCKET` to a non-existent path and asserts `status|sync|install` (and `provision` when present) exit `0`.
- Backend prerequisites handling: macOS and Windows smoke fail with explicit remediation pointing to `substrate world doctor --json` plus `scripts/mac/lima-warm.sh` / `scripts/windows/wsl-warm.ps1` (no auto-provision).
- Capability-gated design: WDL1/WDL2 assertions are gated on presence of `install_class` and `provision` in `substrate world deps` output.

## 3) CI Dispatch Path Is Runnable

Expected dispatch commands (must be executed or explicitly validated during preflight):
- CI compile parity:
  - `make ci-compile-parity CI_WORKFLOW_REF="feat/world_deps_selection_layer" CI_REMOTE=origin CI_CLEANUP=1`
- Feature smoke:
  - `make feature-smoke FEATURE_DIR="docs/project_management/next/world_deps_selection_layer" PLATFORM=behavior RUNNER_KIND=self-hosted WORKFLOW_REF="feat/world_deps_selection_layer" REMOTE=origin CLEANUP=1 RUN_INTEG_CHECKS=1`

Runner readiness (self-hosted labels per standard):
- Linux: `[self-hosted, Linux, linux-host]` — VERIFIED (workflow uses these labels in `.github/workflows/feature-smoke.yml`; runner availability not validated in this preflight)
- macOS: `[self-hosted, macOS]` — VERIFIED (workflow uses these labels in `.github/workflows/feature-smoke.yml`; runner availability not validated in this preflight)
- Windows: `[self-hosted, Windows]` — VERIFIED (workflow uses these labels in `.github/workflows/feature-smoke.yml`; runner availability not validated in this preflight)

Run ids/URLs (if executed during preflight):
- CI compile parity: NOT EXECUTED (validated with `make -n ...` resolving to `scripts/ci/dispatch_ci_testing.sh`)
- Feature smoke (behavior): NOT EXECUTED (validated with `make -n ...` resolving to `scripts/ci/dispatch_feature_smoke.sh`)

Validation performed:
- `make -n ci-compile-parity CI_WORKFLOW_REF="feat/world_deps_selection_layer" CI_REMOTE=origin CI_CLEANUP=1` → exit `0`
- `make -n feature-smoke FEATURE_DIR="docs/project_management/next/world_deps_selection_layer" PLATFORM=behavior RUNNER_KIND=self-hosted WORKFLOW_REF="feat/world_deps_selection_layer" REMOTE=origin CLEANUP=1 RUN_INTEG_CHECKS=1` → exit `0`

## 4) Required Fixes Before Starting WDL0

- None.
