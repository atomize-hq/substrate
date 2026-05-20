# Kickoff: ITPS3-code (code)

## Scope
- Production code only; no new tests.
- Spec: `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/slices/ITPS3/ITPS3-spec.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify the worktree is `wt/adr-0027-identity-tuple-policy-surface-itps3-code` on branch `adr-0027-identity-tuple-policy-surface-itps3-code`.
2. Read `plan.md`, `tasks.json`, `session_log.md`, the slice spec, and this prompt.
3. Confirm `ITPS2-integ` is complete before proceeding.

## Requirements
- Implement only the production behaviors required by `AC-ITPS3-01` through `AC-ITPS3-07`.
- Run targeted baseline tests before and after the change.
- Run `cargo fmt` and `cargo clippy --workspace --all-targets -- -D warnings`.

## End Checklist
1. Run `make triad-task-finish TASK_ID="ITPS3-code"` inside the worktree.
2. Hand off the baseline test commands and results.
3. Do not edit planning docs inside the worktree.
