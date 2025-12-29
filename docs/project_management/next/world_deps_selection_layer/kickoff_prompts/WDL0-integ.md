# Kickoff: WDL0-integ (Selection config + UX integration)

## Scope
- Merge WDL0 code + tests; reconcile to `S0-spec-selection-config-and-ux.md`.

## Start Checklist

Do not edit planning docs inside the worktree.

1. Confirm `WDL0-code` and `WDL0-test` are completed.
2. `git checkout feat/world-sync && git pull --ff-only`
3. Read: `docs/project_management/next/world_deps_selection_layer/plan.md`, `docs/project_management/next/world_deps_selection_layer/tasks.json`, `docs/project_management/next/world_deps_selection_layer/session_log.md`, `docs/project_management/next/world_deps_selection_layer/S0-spec-selection-config-and-ux.md`, and this prompt.
4. Set `WDL0-integ` status to `in_progress` in `docs/project_management/next/world_deps_selection_layer/tasks.json`; add a START entry to `docs/project_management/next/world_deps_selection_layer/session_log.md`; commit docs (`docs: start WDL0-integ`).
5. Create branch and worktree:
   - `git checkout -b ws-wdl0-selection-integ`
   - `git worktree add wt/wdl0-selection-integ ws-wdl0-selection-integ`
6. Do not edit docs/tasks/session_log.md inside the worktree.

## Requirements
- Merge WDL0 code+tests; resolve drift so behavior matches `S0` exactly.
- Run `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, relevant tests, then `make integ-checks`.
- Run the feature-local smoke script for your platform and record results in `docs/project_management/next/world_deps_selection_layer/session_log.md`:
  - Linux: `bash docs/project_management/next/world_deps_selection_layer/smoke/linux-smoke.sh`
  - macOS: `bash docs/project_management/next/world_deps_selection_layer/smoke/macos-smoke.sh`
  - Windows: `pwsh -File docs/project_management/next/world_deps_selection_layer/smoke/windows-smoke.ps1`

## End Checklist
1. Ensure fmt/clippy/tests pass; ensure `make integ-checks` completes; capture outputs.
2. Run the feature-local smoke script for your platform; capture output in the END entry.
3. Commit integration worktree changes.
4. Merge back to `feat/world-sync` (ff-only).
5. Update `docs/project_management/next/world_deps_selection_layer/tasks.json` + `docs/project_management/next/world_deps_selection_layer/session_log.md` (END entry), commit docs (`docs: finish WDL0-integ`).
6. Remove worktree.
