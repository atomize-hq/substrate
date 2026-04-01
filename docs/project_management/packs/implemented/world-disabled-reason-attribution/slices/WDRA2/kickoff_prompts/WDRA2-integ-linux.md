# Kickoff: WDRA2-integ-linux (integration platform-fix — Linux)

## Scope
- Ensure WDRA2 is green for Linux.
- Spec: `docs/project_management/packs/draft/world-disabled-reason-attribution/slices/WDRA2/WDRA2-spec.md`
- Execution workflow standard: `docs/project_management/system/standards/triad/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Run this task on a machine that matches Linux.
2. Verify you are in the task worktree `wt/world-disabled-reason-attribution-wdra2-integ-linux` on branch `world-disabled-reason-attribution-wdra2-integ-linux` and that `.taskmeta.json` exists at the worktree root.
3. Read: `docs/project_management/packs/draft/world-disabled-reason-attribution/plan.md`, `docs/project_management/packs/draft/world-disabled-reason-attribution/tasks.json`, `docs/project_management/packs/draft/world-disabled-reason-attribution/session_log.md`, spec, this prompt.
4. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start FEATURE_DIR="docs/project_management/packs/draft/world-disabled-reason-attribution" TASK_ID="WDRA2-integ-linux" TASK_PLATFORM=linux`

## Requirements
- Merge `WDRA2-integ-core` into this task branch before validating.
- Run:
  - `cargo fmt`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - relevant replay tests
  - bash "docs/project_management/packs/draft/world-disabled-reason-attribution/smoke/linux-smoke.sh"
- Use the advisory CI audit ledger path if dispatch is needed:
  - `docs/project_management/packs/draft/world-disabled-reason-attribution/logs/WDRA2/ci-audit/ledger.jsonl`

## End Checklist
1. Ensure the required gate is green for Linux and capture the run id or local evidence.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="WDRA2-integ-linux"`
3. Hand off platform notes to the operator.
4. Do not delete the worktree.
