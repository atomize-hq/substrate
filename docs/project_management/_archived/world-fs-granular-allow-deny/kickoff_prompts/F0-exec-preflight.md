# Kickoff: F0-exec-preflight (ops)

Standard:
- `docs/project_management/standards/EXECUTION_PREFLIGHT_GATE_STANDARD.md`
- `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Scope
- Execution-time preflight only (no production code).
- Validate the Planning Pack is runnable and the smoke/manual validation surfaces are present and coherent before starting WFGAD0.

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are on the orchestration branch `feat/world-fs-granular-allow-deny`.
2. Read: `plan.md`, `tasks.json`, `session_log.md`, `quality_gate_report.md`, `manual_testing_playbook.md`, this prompt.

## Requirements (runnable)
- Run:
  - `make planning-lint FEATURE_DIR="docs/project_management/_archived/world-fs-granular-allow-deny"`
  - `bash -n docs/project_management/_archived/world-fs-granular-allow-deny/smoke/linux-smoke.sh`
  - `bash -n docs/project_management/_archived/world-fs-granular-allow-deny/smoke/macos-smoke.sh`
- Fill: `docs/project_management/_archived/world-fs-granular-allow-deny/execution_preflight_report.md`

## End Checklist
1. Commit the filled preflight report on the orchestration branch.
2. Proceed only if the report recommendation is `ACCEPT`.
