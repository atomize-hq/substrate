# Kickoff: BEDPM0-integ-windows (integration windows)

## Scope
- Resolve CP1 parity drift for BEDPM0 on Windows.
- `docs/project_management/packs/draft/best-effort-distro-package-manager/pre-planning/ci_checkpoint_plan.md` keeps `feature_smoke=false` for CP1, so this task uses CI parity evidence.

## Start Checklist
Do not edit planning docs inside the worktree.

1. Run this task on a Windows machine.
2. Confirm the worktree is `wt/best-effort-distro-package-manager-bedpm0-integ-windows` on branch `best-effort-distro-package-manager-bedpm0-integ-windows` and `.taskmeta.json` exists at the worktree root.
3. Read `docs/project_management/packs/draft/best-effort-distro-package-manager/tasks.json`, `docs/project_management/packs/draft/best-effort-distro-package-manager/session_log.md`, the BEDPM0 spec, the CP1 results, and this prompt.
4. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run `make triad-task-start FEATURE_DIR="docs/project_management/packs/draft/best-effort-distro-package-manager" TASK_ID="BEDPM0-integ-windows"`.

## Requirements
- Merge `BEDPM0-integ-core` into the worktree before platform fixes.
- Keep changes limited to the Windows failure set reported by CP1.
- Run `cargo fmt`.
- Run `cargo clippy --workspace --all-targets -- -D warnings`.
- Rerun the failing Windows command set until it passes.
- Do not dispatch feature smoke from this task.

## End Checklist
1. Capture the fixed Windows command set and final results for the operator.
2. Run `make triad-task-finish TASK_ID="BEDPM0-integ-windows"` from inside the worktree.
3. Leave the worktree in place.
