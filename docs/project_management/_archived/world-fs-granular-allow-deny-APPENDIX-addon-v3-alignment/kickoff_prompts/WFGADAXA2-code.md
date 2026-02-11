# Kickoff: WFGADAXA2-code (code)

## Scope
- Production code only; no new tests.
- Spec: `docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX-addon-v3-alignment/WFGADAXA2-spec.md`
- Execution standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start checklist
Do not edit planning docs inside the worktree.

1. Verify worktree: `wt/world-fs-granular-allow-deny-appendix-addon-v3-wfgadaxa2-code` on branch `world-fs-granular-allow-deny-appendix-addon-v3-wfgadaxa2-code` with `.taskmeta.json`.
2. Read: plan/tasks/session_log/spec + this prompt.

## Requirements
- Align downstream operator-facing surfaces (doctor/health/trace metadata) and canonical docs to the V3-only story.
- Run:
  - `cargo fmt`
  - `cargo clippy --workspace --all-targets -- -D warnings`
- Do not add tests.

## End checklist
1. Run required commands; capture outputs for handoff.
2. `make triad-task-finish TASK_ID="WFGADAXA2-code"`

