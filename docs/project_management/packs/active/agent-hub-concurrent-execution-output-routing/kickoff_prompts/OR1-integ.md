# Kickoff: OR1-integ (integration final — cross-platform merge)

## Scope
- Merge platform-fix branches (if any) and finalize OR1 with a clean, auditable merged state.
- Spec: `docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing/OR1-spec.md`
- Closeout report: `docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing/OR1-closeout_report.md` (completed on orchestration branch)
- Execution workflow standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`
- This task merges back to the orchestration branch after all platforms are green.

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/agent-hub-concurrent-execution-output-routing-or1-integ` on branch `agent-hub-concurrent-execution-output-routing-or1-integ` and that `.taskmeta.json` exists at the worktree root.
2. Read: `plan.md`, `tasks.json`, `session_log.md`, `OR1-spec.md`, and this prompt.
3. Verify `CP1-ci-checkpoint` is completed and run ids/URLs are recorded in `session_log.md`.

## Requirements
- Merge the relevant branches for OR1:
  - `agent-hub-concurrent-execution-output-routing-or1-integ-core`
  - `agent-hub-concurrent-execution-output-routing-or1-integ-linux`
  - `agent-hub-concurrent-execution-output-routing-or1-integ-macos`
  - `agent-hub-concurrent-execution-output-routing-or1-integ-windows`
- Run:
  - `cargo fmt`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - relevant tests
  - `make integ-checks`
- Closeout report is completed on the orchestration branch:
  - `docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing/OR1-closeout_report.md`

## End Checklist
1. From inside the worktree, run: `make triad-task-finish TASK_ID="OR1-integ"`
2. Hand off closeout report completion requirements and key outputs to the operator (do not edit planning docs inside the worktree).

