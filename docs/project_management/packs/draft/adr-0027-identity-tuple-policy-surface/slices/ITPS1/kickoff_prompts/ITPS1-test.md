# Kickoff: ITPS1-test (test)

## Scope
- Tests only; no production code edits.
- Spec: `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/slices/ITPS1/ITPS1-spec.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify the worktree is `wt/adr-0027-identity-tuple-policy-surface-itps1-test` on branch `adr-0027-identity-tuple-policy-surface-itps1-test`.
2. Read `plan.md`, `tasks.json`, `session_log.md`, the slice spec, and this prompt.
3. Confirm `ITPS0-integ` is complete before proceeding.

## Requirements
- Add or update tests that enforce `AC-ITPS1-01` through `AC-ITPS1-08`.
- Keep the task scoped to test-only changes.
- Run `cargo fmt` plus the targeted tests you add or modify.

## End Checklist
1. Run `make triad-task-finish TASK_ID="ITPS1-test"` inside the worktree.
2. Hand off the targeted test commands and outcomes.
3. Do not edit planning docs inside the worktree.
