# Kickoff: WFGADAXA0-integ (integration)

## Scope
- Merge WFGADAXA0 code+tests, make the slice green, and complete the slice closeout gate.
- Spec: `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX-addon-v3-alignment/WFGADAXA0-spec.md`
- Closeout: `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX-addon-v3-alignment/WFGADAXA0-closeout_report.md`
- Execution standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start checklist
Do not edit planning docs inside the worktree.

1. Verify worktree: `wt/world-fs-granular-allow-deny-appendix-addon-v3-wfgadaxa0-integ` on branch `world-fs-granular-allow-deny-appendix-addon-v3-wfgadaxa0-integ` with `.taskmeta.json`.
2. Read: plan/tasks/session_log/spec + this prompt.

## Requirements
- Merge code+test branches for WFGADAXA0.
- Reconcile implementation and tests to the authoritative Appendix A.6 contract.
- Run:
  - `cargo fmt`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo test -p shell --tests -- --nocapture` (note: crate is `shell`, not `substrate-shell`)
  - `make integ-checks`
- Fill out `WFGADAXA0-closeout_report.md` per `docs/project_management/standards/SLICE_CLOSEOUT_GATE_STANDARD.md`.

## End checklist
1. Ensure gates are green and closeout report is updated.
2. `make triad-task-finish TASK_ID="WFGADAXA0-integ"`
