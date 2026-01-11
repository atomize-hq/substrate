# Execution Preflight Gate Report — world_deps_selection_layer

Date (UTC): 2026-01-10

Standard:
- `docs/project_management/standards/EXECUTION_PREFLIGHT_GATE_STANDARD.md`

Feature directory:
- `docs/project_management/next/world_deps_selection_layer`

## Recommendation

RECOMMENDATION: **NOT RUN**

Rule:
- The `F0-exec-preflight` task must replace `NOT RUN` with exactly one of: `ACCEPT` or `REVISE`.

## Inputs Reviewed (NOT RUN)

- Planning quality gate is `ACCEPT` (`docs/project_management/next/world_deps_selection_layer/quality_gate_report.md`): NOT RUN
- ADR accepted and still matches intent: NOT RUN
- Planning Pack complete (`plan.md`, `tasks.json`, `session_log.md`, specs, kickoff prompts): NOT RUN
- Triad sizing is appropriate (each slice is one behavior delta; no “grab bag” slices): NOT RUN
- Required planning artifacts exist: `integration_map.md`, `manual_testing_playbook.md`: NOT RUN
- Cross-platform plan is explicit (tasks.json meta: behavior + CI parity platforms): NOT RUN

## 0) Slice Sizing (one behavior delta each) (NOT RUN)

- Slices reviewed: NOT RUN
- Required splits before starting execution: NONE RECORDED (NOT RUN)

## 1) Cross-Platform Coverage (NOT RUN)

Expected from `docs/project_management/next/world_deps_selection_layer/tasks.json` meta:
- Behavior platforms (smoke required): `["linux","macos","windows"]`
- CI parity platforms (compile parity required): `["linux","macos","windows"]`
- WSL required: `false` (by omission)

## 2) Smoke Scripts Are Not “Toy” Checks (NOT RUN)

Manual playbook:
- `docs/project_management/next/world_deps_selection_layer/manual_testing_playbook.md`

Smoke scripts:
- Linux: `docs/project_management/next/world_deps_selection_layer/smoke/linux-smoke.sh`
- macOS: `docs/project_management/next/world_deps_selection_layer/smoke/macos-smoke.sh`
- Windows: `docs/project_management/next/world_deps_selection_layer/smoke/windows-smoke.ps1`

Required parity mapping (must be recorded during preflight):
- Manual steps validated by smoke: NOT RUN
- Critical assertions validated by smoke (exit codes + key substrings): NOT RUN

## 3) CI Dispatch Path Is Runnable (NOT RUN)

Expected dispatch commands (must be executed or explicitly validated during preflight):
- CI compile parity:
  - `make ci-compile-parity CI_WORKFLOW_REF="feat/world_deps_selection_layer" CI_REMOTE=origin CI_CLEANUP=1`
- Feature smoke:
  - `make feature-smoke FEATURE_DIR="docs/project_management/next/world_deps_selection_layer" PLATFORM=behavior RUNNER_KIND=self-hosted WORKFLOW_REF="feat/world_deps_selection_layer" REMOTE=origin CLEANUP=1 RUN_INTEG_CHECKS=1`

Runner readiness (self-hosted labels per standard):
- Linux: `[self-hosted, Linux, linux-host]` — NOT RUN
- macOS: `[self-hosted, macOS]` — NOT RUN
- Windows: `[self-hosted, Windows]` — NOT RUN

Run ids/URLs (if executed during preflight):
- CI compile parity: NOT RUN
- Feature smoke (behavior): NOT RUN

## 4) Required Fixes Before Starting WDL0 (NOT RUN)

- NONE RECORDED (NOT RUN)

