# Kickoff: PDLDPM2-integ-windows (integration windows)

## Scope
- Resolve CP1 parity drift for PDLDPM2 on Windows.
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/ci_checkpoint_plan.md` keeps behavior smoke on linux only, so this task uses CI parity evidence.

## Start Checklist
Do not edit planning docs inside the worktree.

1. Run this task on a Windows machine.
2. Confirm the worktree is `wt/persist-detected-linux-distro-pkg-manager-pdldpm2-integ-windows` on branch `persist-detected-linux-distro-pkg-manager-pdldpm2-integ-windows` and `.taskmeta.json` exists at the worktree root.
3. Read `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/tasks.json`, `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/session_log.md`, the PDLDPM2 spec, the CP1 results, and this prompt.
4. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run `make triad-task-start FEATURE_DIR="docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager" TASK_ID="PDLDPM2-integ-windows"`.

## Requirements
- Merge `PDLDPM2-integ-core` into the worktree before platform fixes.
- Keep changes limited to the Windows failure set reported by CP1.
- Run `cargo fmt`.
- Run `cargo clippy --workspace --all-targets -- -D warnings`.
- Rerun the failing Windows command set until it passes.
- Do not dispatch feature smoke from this task.

## End Checklist
1. Capture the fixed Windows command set and final results for the operator.
2. Run `make triad-task-finish TASK_ID="PDLDPM2-integ-windows"` from inside the worktree.
3. Leave the worktree in place.
