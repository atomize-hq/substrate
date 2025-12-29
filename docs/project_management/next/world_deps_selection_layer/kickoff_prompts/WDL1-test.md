# Kickoff: WDL1-test (Install classes tests)

## Scope
- Add tests for install-class behavior and manifest validation per `S1-spec-install-classes.md`.
- Tests only; do not modify production code (except minimal test-only helpers if required).

## Start Checklist

Do not edit planning docs inside the worktree.

1. `git checkout feat/world-sync && git pull --ff-only`
2. Read: `docs/project_management/next/world_deps_selection_layer/plan.md`, `docs/project_management/next/world_deps_selection_layer/tasks.json`, `docs/project_management/next/world_deps_selection_layer/session_log.md`, `docs/project_management/next/world_deps_selection_layer/S1-spec-install-classes.md`, and this prompt.
3. Set `WDL1-test` status to `in_progress` in `docs/project_management/next/world_deps_selection_layer/tasks.json`; add a START entry to `docs/project_management/next/world_deps_selection_layer/session_log.md`; commit docs (`docs: start WDL1-test`).
4. Create branch and worktree:
   - `git checkout -b ws-wdl1-install-classes-test`
   - `git worktree add wt/wdl1-install-classes-test ws-wdl1-install-classes-test`
5. Do not edit docs/tasks/session_log.md inside the worktree.

## Requirements
- Tests must cover:
  - Manifest validation for install classes (schema + mutually exclusive fields).
  - Routing behavior: `system_packages` and `manual` are blocked with the defined exit codes/messages.
  - Runtime path refuses OS package managers (no `apt` execution) per S1.

## Required Commands
- `cargo fmt`
- Targeted `cargo test ...` commands for the tests added/changed.

## End Checklist
1. Run required commands; capture outputs for the END entry.
2. Commit worktree changes.
3. Merge back to `feat/world-sync` (ff-only).
4. Update `docs/project_management/next/world_deps_selection_layer/tasks.json` + `docs/project_management/next/world_deps_selection_layer/session_log.md` (END entry), commit docs (`docs: finish WDL1-test`).
5. Remove worktree.
