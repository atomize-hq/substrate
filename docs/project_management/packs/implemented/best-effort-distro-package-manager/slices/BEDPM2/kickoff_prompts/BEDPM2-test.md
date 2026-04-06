# Kickoff: BEDPM2-test (test)

## Scope
- Tests only; no production-code edits.
- Spec: `docs/project_management/packs/implemented/best-effort-distro-package-manager/slices/BEDPM2/BEDPM2-spec.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in `wt/best-effort-distro-package-manager-bedpm2-test` on branch `best-effort-distro-package-manager-bedpm2-test` and that `.taskmeta.json` exists.
2. Read `plan.md`, `tasks.json`, `session_log.md`, the BEDPM2 spec, and this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run `make triad-task-start-pair FEATURE_DIR="docs/project_management/packs/implemented/best-effort-distro-package-manager" SLICE_ID="BEDPM2"`.

## Requirements
- Add or update tests that enforce the BEDPM2 AC IDs.
- Keep coverage deterministic and scoped to wrapper and operator-contract behavior.
- Run `cargo fmt` and the targeted tests you add or touch.

## End Checklist
1. Capture the targeted test command and results.
2. From inside the worktree, run `make triad-task-finish TASK_ID="BEDPM2-test"`.
3. Hand off results to the operator. Do not edit planning docs inside the worktree.
