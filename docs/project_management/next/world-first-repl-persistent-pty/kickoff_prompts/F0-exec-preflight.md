# Kickoff: F0-exec-preflight (execution preflight gate)

## Scope
- Run the feature-level start gate before any triad work begins.
- This task is docs-only and must be performed on the orchestration branch (no worktrees).
- Standard: `docs/project_management/standards/EXECUTION_PREFLIGHT_GATE_STANDARD.md`
- Report: `docs/project_management/next/world-first-repl-persistent-pty/execution_preflight_report.md`

## Start Checklist

Do not edit planning docs inside the worktree.

1. Ensure the orchestration branch exists and is checked out:
   - `make triad-orch-ensure FEATURE_DIR="docs/project_management/next/world-first-repl-persistent-pty"`
2. Read end-to-end (authoritative set + supporting proofs):
   - `docs/project_management/next/ADR-0016-world-first-repl-persistent-pty.md`
   - `docs/project_management/next/world-first-repl-persistent-pty/plan.md`
   - `docs/project_management/next/world-first-repl-persistent-pty/tasks.json`
   - `docs/project_management/next/world-first-repl-persistent-pty/session_log.md`
   - `docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md`
   - `docs/project_management/next/world-first-repl-persistent-pty/STATE_MACHINE.md`
   - `docs/project_management/next/world-first-repl-persistent-pty/decision_register.md`
   - `docs/project_management/next/world-first-repl-persistent-pty/integration_map.md`
   - `docs/project_management/next/world-first-repl-persistent-pty/manual_testing_playbook.md`
   - `docs/project_management/next/world-first-repl-persistent-pty/smoke/*`
   - `docs/project_management/next/sequencing.json`
3. Set `F0-exec-preflight` status to `in_progress` in `docs/project_management/next/world-first-repl-persistent-pty/tasks.json`; add START entry to `docs/project_management/next/world-first-repl-persistent-pty/session_log.md`; commit docs (`docs: start F0-exec-preflight`).

## Requirements (record results in `execution_preflight_report.md`)

Fill `docs/project_management/next/world-first-repl-persistent-pty/execution_preflight_report.md` with exactly one recommendation:
- `ACCEPT`: triads may begin.
- `REVISE`: do not start triads until the listed issues are fixed and the preflight is re-run.

Minimum checks (no ambiguity; each must have evidence in the report):

### 0) Slices are sized for reliable execution
- Verify each slice (`C0`–`C5`) is one behavior delta (no “grab bag” scope).
- If not, recommendation MUST be `REVISE` with the exact slice(s) that must be split.

### 1) Inputs are coherent
- Confirm the Planning Pack quality gate is `ACCEPT` and current (no drift).
- Confirm required artifacts exist and are internally consistent:
  - `plan.md`, `tasks.json`, `session_log.md`, all specs, kickoff prompts, `integration_map.md`, `manual_testing_playbook.md`, smoke scripts.

### 2) Cross-platform implications are explicitly covered
- Confirm `tasks.json` meta declares:
  - `meta.behavior_platforms_required` and `meta.ci_parity_platforms_required` (and WSL flags if applicable).
- Confirm platform-fix integration structure exists per slice for CI parity platforms:
  - `X-integ-core`, `X-integ-<platform>`, `X-integ`.

### 3) Smoke scripts are not “toy” checks
- Confirm smoke scripts mirror the manual playbook’s intent:
  - The scripts run real commands/workflows and assert exit codes and observable output (not just “command runs”).
  - Each required behavior platform has a smoke script and the manual playbook references it.

### 4) CI dispatch path is runnable
- Confirm the CI dispatch commands embedded in integration tasks are valid and match the feature branch ref:
  - `make ci-compile-parity CI_WORKFLOW_REF="feat/world-first-repl-persistent-pty" CI_REMOTE=origin CI_CLEANUP=1`
  - `make feature-smoke FEATURE_DIR="docs/project_management/next/world-first-repl-persistent-pty" PLATFORM=behavior SMOKE_SLICE_ID="<slice>" RUNNER_KIND=self-hosted WORKFLOW_REF="feat/world-first-repl-persistent-pty" REMOTE=origin CLEANUP=1 RUN_INTEG_CHECKS=1`
- If required self-hosted runners/labels are missing, recommendation MUST be `REVISE`.

## Commands (minimum; include exit codes in the report)
- `FEATURE_DIR="docs/project_management/next/world-first-repl-persistent-pty"; jq -e . "$FEATURE_DIR/tasks.json" >/dev/null` (expected exit `0`)
- `jq -e . docs/project_management/next/sequencing.json >/dev/null` (expected exit `0`)
- `make planning-lint FEATURE_DIR="$FEATURE_DIR"` (expected exit `0`)

## End Checklist
1. Update `docs/project_management/next/world-first-repl-persistent-pty/execution_preflight_report.md` with:
   - the recommendation (`ACCEPT` or `REVISE`),
   - commands run (verbatim) and exit codes,
   - any required fixes (explicit).
2. Set `F0-exec-preflight` status to `completed` in `docs/project_management/next/world-first-repl-persistent-pty/tasks.json`; add END entry to `docs/project_management/next/world-first-repl-persistent-pty/session_log.md` (include the recommendation and any required fixes).
3. Commit docs (`docs: finish F0-exec-preflight`).
