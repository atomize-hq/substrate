# Kickoff: WFGADAXA2-integ-core (integration core)

## Scope
- Merge WFGADAXA2 code+tests and get primary-platform green before dispatching CP1.
- Spec: `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX-addon-v3-alignment/WFGADAXA2-spec.md`
- Execution standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start checklist
Do not edit planning docs inside the worktree.

1. Verify worktree: `wt/world-fs-granular-allow-deny-appendix-addon-v3-wfgadaxa2-integ-core` on branch `world-fs-granular-allow-deny-appendix-addon-v3-wfgadaxa2-integ-core` with `.taskmeta.json`.
2. Read: plan/tasks/session_log/spec + this prompt.

## Requirements
- Merge code+test branches for WFGADAXA2.
- Run:
  - `cargo fmt`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - relevant tests
  - `make integ-checks`
- Do not dispatch cross-platform CI from this task; that happens in CP1.

## End checklist
1. Ensure gates are green.
2. `make triad-task-finish TASK_ID="WFGADAXA2-integ-core"`

