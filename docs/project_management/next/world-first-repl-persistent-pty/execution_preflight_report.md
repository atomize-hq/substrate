# Execution Preflight Gate Report — world-first-repl-persistent-pty

Date (UTC): 2026-01-26

Standard:
- `docs/project_management/standards/EXECUTION_PREFLIGHT_GATE_STANDARD.md`

Feature directory:
- `docs/project_management/next/world-first-repl-persistent-pty/`

## Recommendation

RECOMMENDATION: **ACCEPT**

Rule:
- The `F0-exec-preflight` task must replace `NOT RUN` with exactly one of: `ACCEPT` or `REVISE`.

## Inputs Reviewed

- [x] Planning quality gate is `ACCEPT` (`docs/project_management/next/world-first-repl-persistent-pty/quality_gate_report.md`)
- [x] ADR accepted and still matches intent (`docs/project_management/next/ADR-0016-world-first-repl-persistent-pty.md`)
- [x] Planning Pack complete (`plan.md`, `tasks.json`, `session_log.md`, specs, kickoff prompts)
- [x] Triad sizing is appropriate (each slice is one behavior delta; no “grab bag” slices)
- [x] Required planning artifacts exist: `integration_map.md`, `manual_testing_playbook.md`
- [x] Cross-platform plan is explicit (tasks.json meta: behavior + CI parity platforms)

Commands run during preflight (record exit codes):
- `FEATURE_DIR="docs/project_management/next/world-first-repl-persistent-pty"; jq -e . "$FEATURE_DIR/tasks.json" >/dev/null` → exit `0`
- `jq -e . docs/project_management/next/sequencing.json >/dev/null` → exit `0`
- `make planning-lint FEATURE_DIR="$FEATURE_DIR"` → exit `0`

Notes:
- Orchestration branch HEAD at preflight time: `405532bb91d8b117cc997eb8475118e3b5cf7a92` (docs-only start-of-preflight status/log updates on top of the Planning Pack).
- ADR-0016 status is `Draft`, but planning-lint’s ADR drift check is green (executive summary hash matches) and the Planning Pack quality gate recommendation is `ACCEPT`.

## 0) Slice Sizing (one behavior delta each)

- Slices reviewed: `C0`, `C1`, `C2`, `C3`, `C4`, `C5`
- Required splits before starting execution: `NONE`

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
- [x] Smoke scripts run the same commands/workflows as the manual testing playbook (minimal viable subset)
- [x] Smoke scripts validate exit codes and key output (at least the critical assertions)

Notes:
- Linux smoke is non-toy: it asserts world backend reachability (`substrate world doctor --json | jq -e '.ok == true'`), runs slice-scoped integration tests (`cargo test ... --test ...`), and includes an observable world-vs-host assertion (`substrate -c 'mkdir -p .wf_world_only_dir'` then assert the directory is not present on the host filesystem).
- macOS smoke asserts backend reachability and then runs the same Linux-equivalent checks (via Lima-backed world).
- Windows smoke is explicitly a no-op for this feature, matching the Planning Pack’s “CI parity-only (no behavioral assertions)” statement for Windows.

## 3) CI Dispatch Path Is Runnable

Expected dispatch commands (validate they are correct and runnable; record evidence):
- CI compile parity:
  - `make ci-compile-parity CI_WORKFLOW_REF="feat/world-first-repl-persistent-pty" CI_REMOTE=origin CI_CLEANUP=1`
- Feature smoke (behavior platforms):
  - `make feature-smoke FEATURE_DIR="docs/project_management/next/world-first-repl-persistent-pty" PLATFORM=behavior SMOKE_SLICE_ID="<slice>" RUNNER_KIND=self-hosted WORKFLOW_REF="feat/world-first-repl-persistent-pty" REMOTE=origin CLEANUP=1 RUN_INTEG_CHECKS=1`

Runner readiness (self-hosted labels per standard; record what is verified):
- Linux: workflow supports `[self-hosted, Linux, linux-host]` (`.github/workflows/feature-smoke.yml`)
- macOS: workflow supports `[self-hosted, macOS]` (`.github/workflows/feature-smoke.yml`)
- Windows: workflow supports `[self-hosted, Windows]` (`.github/workflows/feature-smoke.yml`)
- WSL: workflow supports `[self-hosted, Linux, wsl]` (not required for this feature)

Run ids and URLs (if executed during preflight):
- CI compile parity:
- Feature smoke (behavior):

Validation performed:
- `make -n ci-compile-parity CI_WORKFLOW_REF="feat/world-first-repl-persistent-pty" CI_REMOTE=origin CI_CLEANUP=1` → `PASS` (exit `0`)
- `make -n feature-smoke FEATURE_DIR="docs/project_management/next/world-first-repl-persistent-pty" PLATFORM=behavior SMOKE_SLICE_ID="C0" RUNNER_KIND=self-hosted WORKFLOW_REF="feat/world-first-repl-persistent-pty" REMOTE=origin CLEANUP=1 RUN_INTEG_CHECKS=1` → `PASS` (exit `0`)

## Required Fixes Before Starting C0

- None.
