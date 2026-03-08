# Kickoff: BEDPM3-test (test)

## Scope
- Tests only; no production-code edits.
- Spec: `docs/project_management/packs/draft/best-effort-distro-package-manager/slices/BEDPM3/BEDPM3-spec.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in `wt/best-effort-distro-package-manager-bedpm3-test` on branch `best-effort-distro-package-manager-bedpm3-test` and that `.taskmeta.json` exists.
2. Read `plan.md`, `tasks.json`, `session_log.md`, the BEDPM3 spec, and this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run `make triad-task-start-pair FEATURE_DIR="docs/project_management/packs/draft/best-effort-distro-package-manager" SLICE_ID="BEDPM3"`.

## Requirements
- Add or update tests that enforce the BEDPM3 AC IDs.
- Keep coverage deterministic and Linux-scoped for behavior smoke.
- Run `cargo fmt` and the targeted tests you add or touch.

## End Checklist
1. Capture the targeted test command and results.
2. From inside the worktree, run `make triad-task-finish TASK_ID="BEDPM3-test"`.
3. Hand off results to the operator. Do not edit planning docs inside the worktree.
