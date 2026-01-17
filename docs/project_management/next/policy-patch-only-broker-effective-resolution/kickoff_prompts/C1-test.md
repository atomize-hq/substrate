# Kickoff: C1-test (test)

## Scope
- Tests only (plus minimal test-only helpers/fixtures if required); no production code.
- Spec: `docs/project_management/next/policy-patch-only-broker-effective-resolution/C1-spec.md`
- Execution workflow standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/policy-patch-only-broker-effective-resolution-c1-test` on branch `policy-patch-only-broker-effective-resolution-c1-test` and that `.taskmeta.json` exists at the worktree root.
2. Read: plan, tasks, session log, spec, this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start-pair FEATURE_DIR="docs/project_management/next/policy-patch-only-broker-effective-resolution" SLICE_ID="C1"`

## Requirements
- Add tests that enforce the C1 acceptance criteria.
- Run: `cargo fmt` and the targeted tests you add/touch.
- Tests that are red prior to code landing must fail deterministically for spec-defined reasons.

## End Checklist
1. From inside the worktree, run: `make triad-task-finish TASK_ID="C1-test"`.
2. Hand off the test commands and outcomes to the operator.

