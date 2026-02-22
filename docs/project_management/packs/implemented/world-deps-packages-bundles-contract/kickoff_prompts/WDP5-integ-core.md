# Kickoff: WDP5-integ-core (integration core)

## Scope
- Merge code + tests, resolve drift to spec, and make the slice green on the primary dev platform.
- Spec: `docs/project_management/packs/active/world-deps-packages-bundles-contract/WDP5-spec.md`
- Execution workflow standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/world-deps-packages-bundles-contract-wdp5-integ-core` on branch `world-deps-packages-bundles-contract-wdp5-integ-core` and that `.taskmeta.json` exists.
2. Read: plan, tasks, session_log, spec, this prompt.

## Requirements
- Merge code+test branches into this worktree and make the spec green:
  - `cargo fmt`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - relevant tests
  - `make integ-checks`
- Do not dispatch cross-platform CI from this task. Finish this task, then run `CP2-ci-checkpoint` from orchestration.
- If this machine matches a behavior platform, run the local smoke script before finishing.

## End Checklist
1. From inside the worktree, run: `make triad-task-finish TASK_ID="WDP5-integ-core"`
2. Update tasks/session_log on orchestration branch; do not delete worktrees.

