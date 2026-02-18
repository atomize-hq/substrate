# Kickoff: WDP5-integ-linux (integration platform-fix — linux)

## Scope
- Ensure WDP5 is green for Linux.
- Spec: `docs/project_management/packs/active/world-deps-packages-bundles-contract/WDP5-spec.md`
- This task must not merge back to orchestration; `WDP5-integ` performs the merge.

## Start Checklist
Do not edit planning docs inside the worktree.

1. Run on Linux.
2. Verify you are in the task worktree `wt/world-deps-packages-bundles-contract-wdp5-integ-linux` on branch `world-deps-packages-bundles-contract-wdp5-integ-linux` and that `.taskmeta.json` exists.
3. Read: plan, tasks, session_log, spec, this prompt.

## Requirements
- Merge the WDP5 core branch into this worktree:
  - `git merge world-deps-packages-bundles-contract-wdp5-integ-core`
- Run Linux smoke locally:
  - `cargo build --bin substrate && export PATH="$PWD/target/debug:$PATH" && bash docs/project_management/packs/active/world-deps-packages-bundles-contract/smoke/linux-smoke.sh`
- Because WSL coverage is required for this feature (bundled), dispatch Linux + WSL smoke (self-hosted) from this worktree when needed:
  - `make feature-smoke FEATURE_DIR="docs/project_management/packs/active/world-deps-packages-bundles-contract" PLATFORM=linux RUN_WSL=1 SMOKE_SLICE_ID="WDP5" RUNNER_KIND=self-hosted WORKFLOW_REF="feat/world-deps-packages-bundles-contract" REMOTE=origin CLEANUP=1 RUN_INTEG_CHECKS=0`
- Fix and re-run until green:
  - `cargo fmt`
  - `cargo clippy --workspace --all-targets -- -D warnings`

## End Checklist
1. From inside the worktree, run: `make triad-task-finish TASK_ID="WDP5-integ-linux"`
2. Hand off the smoke evidence (run id/URL or local transcript) to the operator.
