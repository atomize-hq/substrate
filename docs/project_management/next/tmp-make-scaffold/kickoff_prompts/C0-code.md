# Kickoff: C0-code (code)

## Scope
- Production code only; no tests.
- Spec: `docs/project_management/next/tmp-make-scaffold/C0-spec.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/tmp-make-scaffold-c0-code` on branch `tmp-make-scaffold-c0-code` and that `.taskmeta.json` exists at the worktree root.
2. Read: plan.md, tasks.json, session_log.md, spec, this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start-pair FEATURE_DIR="docs/project_management/next/tmp-make-scaffold" SLICE_ID="C0"` (preferred; starts code+test in parallel)
   - `make triad-task-start FEATURE_DIR="docs/project_management/next/tmp-make-scaffold" TASK_ID="C0-code"` (single task only)

## Requirements
- Implement exactly the behaviors and error handling in the spec.
- Run: `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`

## End Checklist
1. Run required commands; capture outputs.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="C0-code"`
3. On the orchestration branch, update tasks.json + session_log.md END entry; commit docs (`docs: finish C0-code`).
4. Do not delete the worktree (feature cleanup removes worktrees at feature end).
