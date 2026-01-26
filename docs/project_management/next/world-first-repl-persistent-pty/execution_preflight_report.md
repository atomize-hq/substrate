# Execution Preflight Gate Report — world-first-repl-persistent-pty

Date (UTC): 2026-01-26

Standard:
- `docs/project_management/standards/EXECUTION_PREFLIGHT_GATE_STANDARD.md`

Feature directory:
- `docs/project_management/next/world-first-repl-persistent-pty/`

## Recommendation

RECOMMENDATION: **NOT RUN**

Rule:
- The `F0-exec-preflight` task must replace `NOT RUN` with exactly one of: `ACCEPT` or `REVISE`.

## Inputs Reviewed

- [ ] Planning quality gate is `ACCEPT` (`docs/project_management/next/world-first-repl-persistent-pty/quality_gate_report.md`)
- [ ] ADR accepted and still matches intent (`docs/project_management/next/ADR-0016-world-first-repl-persistent-pty.md`)
- [ ] Planning Pack complete (`plan.md`, `tasks.json`, `session_log.md`, specs, kickoff prompts)
- [ ] Triad sizing is appropriate (each slice is one behavior delta; no “grab bag” slices)
- [ ] Required planning artifacts exist: `integration_map.md`, `manual_testing_playbook.md`
- [ ] Cross-platform plan is explicit (tasks.json meta: behavior + CI parity platforms)

Commands run during preflight (record exit codes):
- `FEATURE_DIR="docs/project_management/next/world-first-repl-persistent-pty"; jq -e . "$FEATURE_DIR/tasks.json" >/dev/null`
- `jq -e . docs/project_management/next/sequencing.json >/dev/null`
- `make planning-lint FEATURE_DIR="$FEATURE_DIR"`

## 0) Slice Sizing (one behavior delta each)

- Slices reviewed: `C0`, `C1`, `C2`, `C3`, `C4`, `C5`
- Required splits before starting execution: `NONE` | `<list slice ids + exact split required>`

## 1) Cross-Platform Coverage

Expected from `docs/project_management/next/world-first-repl-persistent-pty/tasks.json` meta:
- Behavior platforms (smoke required): `["linux","macos"]`
- CI parity platforms (compile parity required): `["linux","macos","windows"]`
- WSL required: `false` (by omission)

Verified:
- `schema_version=3`, behavior platforms, and CI parity platforms match the expected sets.
- `automation.enabled=true` and `automation.orchestration_branch="feat/world-first-repl-persistent-pty"`.
- Platform-fix integration structure exists per slice (`X-integ-core`, `X-integ-<platform>`, `X-integ`).

## 2) Smoke Scripts Are Not “Toy” Checks

Manual playbook:
- `docs/project_management/next/world-first-repl-persistent-pty/manual_testing_playbook.md`

Smoke scripts:
- Linux: `docs/project_management/next/world-first-repl-persistent-pty/smoke/linux-smoke.sh`
- macOS: `docs/project_management/next/world-first-repl-persistent-pty/smoke/macos-smoke.sh`
- Windows: `docs/project_management/next/world-first-repl-persistent-pty/smoke/windows-smoke.ps1`

Smoke ↔ manual parity checklist:
- [ ] Smoke scripts run the same commands/workflows as the manual testing playbook (minimal viable subset)
- [ ] Smoke scripts validate exit codes and key output (at least the critical assertions)

Notes:
-

## 3) CI Dispatch Path Is Runnable

Expected dispatch commands (validate they are correct and runnable; record evidence):
- CI compile parity:
  - `make ci-compile-parity CI_WORKFLOW_REF="feat/world-first-repl-persistent-pty" CI_REMOTE=origin CI_CLEANUP=1`
- Feature smoke (behavior platforms):
  - `make feature-smoke FEATURE_DIR="docs/project_management/next/world-first-repl-persistent-pty" PLATFORM=behavior SMOKE_SLICE_ID="<slice>" RUNNER_KIND=self-hosted WORKFLOW_REF="feat/world-first-repl-persistent-pty" REMOTE=origin CLEANUP=1 RUN_INTEG_CHECKS=1`

Runner readiness (self-hosted labels per standard; record what is verified):
- Linux:
- macOS:
- Windows:
- WSL:

Run ids and URLs (if executed during preflight):
- CI compile parity:
- Feature smoke (behavior):

Validation performed:
- `make -n ci-compile-parity ...` → `PASS|FAIL`
- `make -n feature-smoke ...` → `PASS|FAIL`

## Required Fixes Before Starting C0

- None.
