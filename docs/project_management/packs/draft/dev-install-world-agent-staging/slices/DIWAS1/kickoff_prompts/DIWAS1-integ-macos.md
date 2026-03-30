# Kickoff: DIWAS1-integ-macos (integration macOS follow-up)

## Scope
- macOS compile parity follow-up work after `CP1-ci-checkpoint` if macOS reports a regression.
- This task is a conditional follow-up; it may be a no-op if the checkpoint is green.

## Start Checklist
Do not edit planning docs inside the worktree.

1. Start only after `CP1-ci-checkpoint` completes and reports macOS follow-up.
2. Verify you are in the task worktree `wt/dev-install-world-agent-staging-diwas1-integ-macos` on branch `dev-install-world-agent-staging-diwas1-integ-macos` and that `.taskmeta.json` exists at the worktree root.
3. Read: `platform-parity-spec.md`, `tasks.json`, `session_log.md`, and the slice spec.

## Validation (minimum)
- Run the macOS parity validation required by the checkpoint (compile parity and any targeted unit tests that failed in CI).

## End Checklist
1. Record macOS evidence and any fix commits in `session_log.md`.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="DIWAS1-integ-macos"`.

