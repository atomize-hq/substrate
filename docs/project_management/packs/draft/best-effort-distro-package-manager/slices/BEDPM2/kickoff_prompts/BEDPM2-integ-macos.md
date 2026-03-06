# Kickoff: BEDPM2-integ-macos (integration macOS)

## Scope
- Resolve CP1 parity drift for BEDPM2 on macOS.
- `docs/project_management/packs/draft/best-effort-distro-package-manager/pre-planning/ci_checkpoint_plan.md` keeps `feature_smoke=false` for CP1, so this task uses CI parity evidence.

## Start Checklist
Do not edit planning docs inside the worktree.

1. Run this task on a macOS machine.
2. Confirm the worktree is `wt/best-effort-distro-package-manager-bedpm2-integ-macos` on branch `best-effort-distro-package-manager-bedpm2-integ-macos` and `.taskmeta.json` exists at the worktree root.
3. Read `docs/project_management/packs/draft/best-effort-distro-package-manager/tasks.json`, `docs/project_management/packs/draft/best-effort-distro-package-manager/session_log.md`, the BEDPM2 spec, the CP1 results, and this prompt.
4. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run `make triad-task-start FEATURE_DIR="docs/project_management/packs/draft/best-effort-distro-package-manager" TASK_ID="BEDPM2-integ-macos"`.

## Requirements
- Merge `BEDPM2-integ-core` into the worktree before platform fixes.
- Keep changes limited to the macOS failure set reported by CP1.
- Run `cargo fmt`.
- Run `cargo clippy --workspace --all-targets -- -D warnings`.
- Rerun the failing macOS command set until it passes.
- Do not dispatch feature smoke from this task.

## End Checklist
1. Capture the fixed macOS command set and final results for the operator.
2. Run `make triad-task-finish TASK_ID="BEDPM2-integ-macos"` from inside the worktree.
3. Leave the worktree in place.
