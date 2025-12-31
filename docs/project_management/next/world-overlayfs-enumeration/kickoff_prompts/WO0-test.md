# Kickoff: WO0-test (Tests: enumeration probe + fallback + metadata)

## Scope
- Tests only (plus minimal test-only helpers if required for deterministic testing); no production code.
- ADR: `docs/project_management/next/ADR-0004-world-overlayfs-directory-enumeration-reliability.md`
- Spec: `docs/project_management/next/world-overlayfs-enumeration/WO0-spec.md`

## Start Checklist

Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/woe-wo0-test` on branch `woe-wo0-test` and that `.taskmeta.json` exists at the worktree root.
2. Read: `docs/project_management/next/world-overlayfs-enumeration/plan.md`, `docs/project_management/next/world-overlayfs-enumeration/tasks.json`, `docs/project_management/next/world-overlayfs-enumeration/session_log.md`, ADR, spec, and this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start-pair FEATURE_DIR="docs/project_management/next/world-overlayfs-enumeration" SLICE_ID="WO0"`

## Requirements
- Add Linux coverage for:
  - Enumeration correctness (created file appears via directory enumeration in world view).
  - Forced primary-strategy enumeration failure driving fuse-overlayfs fallback selection.
  - Trace metadata fields and allowed fallback-reason enums from ADR-0004.
  - Doctor JSON keys required by ADR-0004.

## Commands (required)
- `cargo fmt`
- Run targeted `cargo test ...` commands for the tests you add/modify.

## End Checklist
1. Run required commands and capture outputs.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="WO0-test"`.
3. On the orchestration branch, update `tasks.json` and add the END entry to `session_log.md`; commit docs (`docs: finish WO0-test`).
4. Do not delete the worktree (feature cleanup removes worktrees at feature end).
