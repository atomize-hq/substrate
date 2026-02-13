# Kickoff: WDP5-integ-macos (integration platform-fix — macos)

## Scope
- Ensure WDP5 is green for macOS.
- Spec: `docs/project_management/next/world-deps-packages-bundles-contract/WDP5-spec.md`
- This task must not merge back to orchestration; `WDP5-integ` performs the merge.

## Start Checklist
Do not edit planning docs inside the worktree.

1. Run on macOS.
2. Verify you are in the task worktree `wt/world-deps-packages-bundles-contract-wdp5-integ-macos` on branch `world-deps-packages-bundles-contract-wdp5-integ-macos` and that `.taskmeta.json` exists.
3. Read: plan, tasks, session_log, spec, this prompt.

## Requirements
- Merge the WDP5 core branch into this worktree:
  - `git merge world-deps-packages-bundles-contract-wdp5-integ-core`
- Run macOS smoke locally:
  - `cargo build --bin substrate && export PATH="$PWD/target/debug:$PATH" && bash docs/project_management/next/world-deps-packages-bundles-contract/smoke/macos-smoke.sh`
- Fix and re-run until green:
  - `cargo fmt`
  - `cargo clippy --workspace --all-targets -- -D warnings`

## End Checklist
1. From inside the worktree, run: `make triad-task-finish TASK_ID="WDP5-integ-macos"`
2. Hand off the smoke evidence (run id/URL or local transcript) to the operator.

