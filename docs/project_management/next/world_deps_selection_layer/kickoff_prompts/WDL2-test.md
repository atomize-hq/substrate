# Kickoff: WDL2-test (System packages provisioning tests)

## Scope
- Add tests for provisioning package list computation, gating, and exit codes per `S2-spec-system-packages-provisioning.md`.
- Tests only; do not modify production code (except minimal test-only helpers if required).

## Start Checklist
1. `git checkout feat/world-sync && git pull --ff-only`
2. Read: `docs/project_management/next/world_deps_selection_layer/plan.md`, `docs/project_management/next/world_deps_selection_layer/tasks.json`, `docs/project_management/next/world_deps_selection_layer/session_log.md`, `docs/project_management/next/world_deps_selection_layer/S2-spec-system-packages-provisioning.md`, and this prompt.
3. Set `WDL2-test` status to `in_progress` in `docs/project_management/next/world_deps_selection_layer/tasks.json`; add a START entry to `docs/project_management/next/world_deps_selection_layer/session_log.md`; commit docs (`docs: start WDL2-test`).
4. Create branch and worktree:
   - `git checkout -b ws-wdl2-provision-test`
   - `git worktree add wt/wdl2-provision-test ws-wdl2-provision-test`
5. Do not edit docs/tasks/session_log.md inside the worktree.

## Requirements
- Tests must cover:
  - Deterministic package list computation (stable ordering and de-duplication).
  - Platform gating behavior and exit codes (Linux host unsupported → exit 4).
  - Exit code mapping for backend unavailable vs unsupported vs config errors.

## Required Commands
- `cargo fmt`
- Targeted `cargo test ...` commands for the tests added/changed.

## End Checklist
1. Run required commands; capture outputs for the END entry.
2. Commit worktree changes.
3. Merge back to `feat/world-sync` (ff-only).
4. Update `docs/project_management/next/world_deps_selection_layer/tasks.json` + `docs/project_management/next/world_deps_selection_layer/session_log.md` (END entry), commit docs (`docs: finish WDL2-test`).
5. Remove worktree.
