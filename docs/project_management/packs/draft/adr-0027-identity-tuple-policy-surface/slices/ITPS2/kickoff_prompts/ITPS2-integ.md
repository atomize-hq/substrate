# Kickoff: ITPS2-integ (integration)

## Scope
- Merge `ITPS2-code` and `ITPS2-test`, reconcile to spec, and make the slice green.
- Spec: `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/slices/ITPS2/ITPS2-spec.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify the worktree is `wt/adr-0027-identity-tuple-policy-surface-itps2-integ` on branch `adr-0027-identity-tuple-policy-surface-itps2-integ`.
2. Read `plan.md`, `tasks.json`, `session_log.md`, the slice spec, and this prompt.
3. Stop if the worktree is missing `.taskmeta.json`.

## Requirements
- Merge the code and test branches for ITPS2.
- Ensure the integrated state satisfies `AC-ITPS2-01` through `AC-ITPS2-07`.
- Run `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, relevant tests, and `make integ-checks`.
- Complete `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/slices/ITPS2/ITPS2-closeout_report.md`.

## End Checklist
1. Run `make triad-task-finish TASK_ID="ITPS2-integ"` inside the worktree.
2. Hand off the green commands, outcomes, and closeout report status.
3. Do not edit planning docs inside the worktree.
