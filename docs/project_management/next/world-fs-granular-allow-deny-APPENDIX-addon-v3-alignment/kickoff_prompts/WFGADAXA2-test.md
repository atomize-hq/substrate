# Kickoff: WFGADAXA2-test (test)

## Scope
- Tests only; no production code.
- Spec: `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX-addon-v3-alignment/WFGADAXA2-spec.md`
- Execution standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start checklist
Do not edit planning docs inside the worktree.

1. Verify worktree: `wt/world-fs-granular-allow-deny-appendix-addon-v3-wfgadaxa2-test` on branch `world-fs-granular-allow-deny-appendix-addon-v3-wfgadaxa2-test` with `.taskmeta.json`.
2. Read: plan/tasks/session_log/spec + this prompt.

## Requirements
- Update/add tests so downstream operator surfaces do not regress back to V2 naming (doctor/health/trace metadata).
- Run:
  - `cargo fmt`
  - targeted tests you add/touch

## End checklist
1. Run required commands; capture outputs for handoff.
2. `make triad-task-finish TASK_ID="WFGADAXA2-test"`

