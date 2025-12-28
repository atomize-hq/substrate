# Kickoff: WDL0-code (Selection config + UX)

## Scope
- Implement YAML selection config loading, precedence, `--all` semantics, CLI UX, and exit codes per `S0-spec-selection-config-and-ux.md`.
- Production code only; do not add or modify tests.

## Non-Goals
- Do not implement install classes (`WDL1`).
- Do not implement system packages provisioning (`WDL2`).

## Start Checklist
1. `git checkout feat/world-sync && git pull --ff-only`
2. Read: `docs/project_management/next/world_deps_selection_layer/plan.md`, `docs/project_management/next/world_deps_selection_layer/tasks.json`, `docs/project_management/next/world_deps_selection_layer/session_log.md`, `docs/project_management/next/world_deps_selection_layer/S0-spec-selection-config-and-ux.md`, `docs/project_management/next/world_deps_selection_layer/decision_register.md`, `docs/project_management/next/world_deps_selection_layer/integration_map.md`, and this prompt.
3. Set `WDL0-code` status to `in_progress` in `docs/project_management/next/world_deps_selection_layer/tasks.json`; add a START entry to `docs/project_management/next/world_deps_selection_layer/session_log.md`; commit docs (`docs: start WDL0-code`).
4. Create branch and worktree:
   - `git checkout -b ws-wdl0-selection-code`
   - `git worktree add wt/wdl0-selection-code ws-wdl0-selection-code`
5. Do not edit docs/tasks/session_log.md inside the worktree.

## Requirements
- Primary implementation targets:
  - `crates/shell/src/builtins/world_deps/*`
  - `crates/common/src/paths.rs` (YAML-only naming must align with `Y0-spec.md`)
- Behavior must match `S0` exactly, including the defined exit codes and “configured but empty selection” semantics.

## Required Commands
- `cargo fmt`
- `cargo clippy --workspace --all-targets -- -D warnings`

## End Checklist
1. Run required commands; capture outputs for the END entry.
2. Commit worktree changes.
3. Merge back to `feat/world-sync` (ff-only).
4. Update `docs/project_management/next/world_deps_selection_layer/tasks.json` + `docs/project_management/next/world_deps_selection_layer/session_log.md` (END entry), commit docs (`docs: finish WDL0-code`).
5. Remove worktree.
