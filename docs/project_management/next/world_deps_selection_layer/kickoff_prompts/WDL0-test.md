# Kickoff: WDL0-test (Selection config + UX tests)

## Scope
- Add tests for selection parsing/precedence, scoping, and exit-code behavior per `S0-spec-selection-config-and-ux.md`.
- Tests only; do not modify production code (except minimal test-only helpers if required).

## Start Checklist
1. `git checkout feat/world-sync && git pull --ff-only`
2. Read: `docs/project_management/next/world_deps_selection_layer/plan.md`, `docs/project_management/next/world_deps_selection_layer/tasks.json`, `docs/project_management/next/world_deps_selection_layer/session_log.md`, `docs/project_management/next/world_deps_selection_layer/S0-spec-selection-config-and-ux.md`, and this prompt.
3. Set `WDL0-test` status to `in_progress` in `docs/project_management/next/world_deps_selection_layer/tasks.json`; add a START entry to `docs/project_management/next/world_deps_selection_layer/session_log.md`; commit docs (`docs: start WDL0-test`).
4. Create branch and worktree:
   - `git checkout -b ws-wdl0-selection-test`
   - `git worktree add wt/wdl0-selection-test ws-wdl0-selection-test`
5. Do not edit docs/tasks/session_log.md inside the worktree.

## Requirements
- Tests must cover:
  - Selection file parsing + precedence (workspace overrides global).
  - Configured-but-empty selection semantics (no world-agent calls; exit 0).
  - `--all` semantics (inventory scope; ignores selection files).
  - Unknown tools in selection → exit 2.

## Required Commands
- `cargo fmt`
- Targeted `cargo test ...` commands for the tests added/changed.

## End Checklist
1. Run required commands; capture outputs for the END entry.
2. Commit worktree changes.
3. Merge back to `feat/world-sync` (ff-only).
4. Update `docs/project_management/next/world_deps_selection_layer/tasks.json` + `docs/project_management/next/world_deps_selection_layer/session_log.md` (END entry), commit docs (`docs: finish WDL0-test`).
5. Remove worktree.
