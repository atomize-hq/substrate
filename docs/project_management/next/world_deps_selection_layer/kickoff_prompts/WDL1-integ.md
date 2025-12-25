# Kickoff: WDL1-integ (Install classes integration)

## Scope
- Merge WDL1 code + tests; reconcile to `S1-spec-install-classes.md`.

## Start Checklist
1. Confirm `WDL1-code` and `WDL1-test` are completed.
2. `git checkout feat/world-sync && git pull --ff-only`
3. Read: `docs/project_management/next/world_deps_selection_layer/plan.md`, `docs/project_management/next/world_deps_selection_layer/tasks.json`, `docs/project_management/next/world_deps_selection_layer/session_log.md`, `docs/project_management/next/world_deps_selection_layer/S1-spec-install-classes.md`, and this prompt.
4. Set `WDL1-integ` status to `in_progress` in `docs/project_management/next/world_deps_selection_layer/tasks.json`; add a START entry to `docs/project_management/next/world_deps_selection_layer/session_log.md`; commit docs (`docs: start WDL1-integ`).
5. Create branch and worktree:
   - `git checkout -b ws-wdl1-install-classes-integ`
   - `git worktree add wt/wdl1-install-classes-integ ws-wdl1-install-classes-integ`
6. Do not edit docs/tasks/session_log.md inside the worktree.

## Requirements
- Merge WDL1 code+tests; resolve drift so behavior matches `S1` exactly.
- Run `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, relevant tests, then `make preflight`.
- Run the feature-local smoke script for your platform and record results in `docs/project_management/next/world_deps_selection_layer/session_log.md`:
  - Linux: `bash docs/project_management/next/world_deps_selection_layer/smoke/linux-smoke.sh`
  - macOS: `bash docs/project_management/next/world_deps_selection_layer/smoke/macos-smoke.sh`
  - Windows: `pwsh -File docs/project_management/next/world_deps_selection_layer/smoke/windows-smoke.ps1`

## End Checklist
1. Ensure fmt/clippy/tests pass; ensure `make preflight` completes; capture outputs.
2. Run the feature-local smoke script for your platform; capture output in the END entry.
3. Commit integration worktree changes.
4. Merge back to `feat/world-sync` (ff-only).
5. Update `docs/project_management/next/world_deps_selection_layer/tasks.json` + `docs/project_management/next/world_deps_selection_layer/session_log.md` (END entry), commit docs (`docs: finish WDL1-integ`).
6. Remove worktree.
