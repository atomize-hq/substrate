# Kickoff: C0-code (code)

## Scope
- Production code only; no new tests.
- Spec: `docs/project_management/next/policy-patch-only-broker-effective-resolution/C0-spec.md`
- Execution workflow standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/policy-patch-only-broker-effective-resolution-c0-code` on branch `policy-patch-only-broker-effective-resolution-c0-code` and that `.taskmeta.json` exists at the worktree root.
2. Read: `docs/project_management/next/policy-patch-only-broker-effective-resolution/plan.md`, `docs/project_management/next/policy-patch-only-broker-effective-resolution/tasks.json`, `docs/project_management/next/policy-patch-only-broker-effective-resolution/session_log.md`, spec, this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start-pair FEATURE_DIR="docs/project_management/next/policy-patch-only-broker-effective-resolution" SLICE_ID="C0"`

## Requirements
- Implement exactly the behaviors and error handling in the spec.
- Run: `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`
- Do not add new tests or new test files.
- Baseline testing is required (before and after) and is recorded in the task handoff.

## End Checklist
1. From inside the worktree, run: `make triad-task-finish TASK_ID="C0-code"`.
2. Hand off baseline test commands and outcomes to the operator.

