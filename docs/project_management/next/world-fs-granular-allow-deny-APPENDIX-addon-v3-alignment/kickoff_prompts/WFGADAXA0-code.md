# Kickoff: WFGADAXA0-code (code)

## Scope
- Production code only; no new tests.
- Spec: `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX-addon-v3-alignment/WFGADAXA0-spec.md`
- Authoritative output contract: `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/contract.md` (§1.3 / Appendix A.6)
- Execution standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start checklist
Do not edit planning docs inside the worktree.

1. Verify worktree: `wt/world-fs-granular-allow-deny-appendix-addon-v3-wfgadaxa0-code` on branch `world-fs-granular-allow-deny-appendix-addon-v3-wfgadaxa0-code` with `.taskmeta.json`.
2. Read: plan/tasks/session_log/spec + this prompt.

## Requirements
- Implement Appendix A.6-compliant effective policy display (V3-shaped; explicit deny_list `[]`).
- Run:
  - `cargo fmt`
  - `cargo clippy --workspace --all-targets -- -D warnings`
- Baseline testing: choose a targeted baseline test set, run it before and after changes, and hand off results.
- Do not add new tests or new test files.

## End checklist
1. Run required commands; capture outputs for handoff.
2. `make triad-task-finish TASK_ID="WFGADAXA0-code"`

