# Kickoff: WFGADAXA1-integ (integration)

## Scope
- Merge WFGADAXA1 code+tests, make the slice green, and complete the slice closeout gate.
- Spec: `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX-addon-v3-alignment/WFGADAXA1-spec.md`
- Closeout: `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX-addon-v3-alignment/WFGADAXA1-closeout_report.md`
- Execution standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start checklist
Do not edit planning docs inside the worktree.

1. Verify worktree: `wt/world-fs-granular-allow-deny-appendix-addon-v3-wfgadaxa1-integ` on branch `world-fs-granular-allow-deny-appendix-addon-v3-wfgadaxa1-integ` with `.taskmeta.json`.
2. Read: plan/tasks/session_log/spec + this prompt.

## Requirements
- Merge code+test branches for WFGADAXA1.
- Ensure shell↔world-agent snapshot protocol is V3-only and consistent with Appendix PROTOCOL.
- Run:
  - `cargo fmt`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - relevant tests
  - `make integ-checks`
- Fill out `WFGADAXA1-closeout_report.md`.

## End checklist
1. Ensure gates are green and closeout report is updated.
2. `make triad-task-finish TASK_ID="WFGADAXA1-integ"`

