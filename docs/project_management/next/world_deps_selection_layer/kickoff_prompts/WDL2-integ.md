# Kickoff: WDL2-integ (System packages provisioning integration)

## Scope
- Merge WDL2 code + tests; reconcile to `S2-spec-system-packages-provisioning.md`.

## Start Checklist
1. Confirm `WDL2-code` and `WDL2-test` are completed.
2. `git checkout feat/world-sync && git pull --ff-only`
3. Read: `docs/project_management/next/world_deps_selection_layer/plan.md`, `docs/project_management/next/world_deps_selection_layer/tasks.json`, `docs/project_management/next/world_deps_selection_layer/session_log.md`, `docs/project_management/next/world_deps_selection_layer/S2-spec-system-packages-provisioning.md`, and this prompt.
4. Set `WDL2-integ` status to `in_progress` in `docs/project_management/next/world_deps_selection_layer/tasks.json`; add a START entry to `docs/project_management/next/world_deps_selection_layer/session_log.md`; commit docs (`docs: start WDL2-integ`).
5. Create branch and worktree:
   - `git checkout -b ws-wdl2-provision-integ`
   - `git worktree add wt/wdl2-provision-integ ws-wdl2-provision-integ`
6. Do not edit docs/tasks/session_log.md inside the worktree.

## Requirements
- Merge WDL2 code+tests; resolve drift so behavior matches `S2` exactly.
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
5. Update `docs/project_management/next/world_deps_selection_layer/tasks.json` + `docs/project_management/next/world_deps_selection_layer/session_log.md` (END entry), commit docs (`docs: finish WDL2-integ`).
6. Remove worktree.
