# Kickoff: WDP5-integ-windows (integration platform-fix — windows)

## Scope
- Ensure WDP5 is green for Windows.
- Spec: `docs/project_management/next/world-deps-packages-bundles-contract/WDP5-spec.md`
- This task must not merge back to orchestration; `WDP5-integ` performs the merge.

## Start Checklist
Do not edit planning docs inside the worktree.

1. Run on Windows.
2. Verify you are in the task worktree `wt/world-deps-packages-bundles-contract-wdp5-integ-windows` on branch `world-deps-packages-bundles-contract-wdp5-integ-windows` and that `.taskmeta.json` exists.
3. Read: plan, tasks, session_log, spec, this prompt.

## Requirements
- Merge the WDP5 core branch into this worktree:
  - `git merge world-deps-packages-bundles-contract-wdp5-integ-core`
- Run Windows smoke locally:
  - `cargo build --bin substrate; $env:Path=\"$pwd\\target\\debug;$env:Path\"; pwsh -File docs\\project_management\\next\\world-deps-packages-bundles-contract\\smoke\\windows-smoke.ps1`
- Fix and re-run until green:
  - `cargo fmt`
  - `cargo clippy --workspace --all-targets -- -D warnings`

## End Checklist
1. From inside the worktree, run: `make triad-task-finish TASK_ID="WDP5-integ-windows"`
2. Hand off the smoke evidence (run id/URL or local transcript) to the operator.

