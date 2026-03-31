# Kickoff: DIWAS1-integ-linux (integration linux follow-up)

## Scope
- Linux follow-up work after `CP1-ci-checkpoint` if Linux feature smoke reports a regression.
- This task is a conditional follow-up; it may be a no-op if the checkpoint is green.

## Start Checklist
Do not edit planning docs inside the worktree.

1. Start only after `CP1-ci-checkpoint` completes and reports Linux follow-up.
2. Verify you are in the task worktree `wt/dev-install-world-agent-staging-diwas1-integ-linux` on branch `dev-install-world-agent-staging-diwas1-integ-linux` and that `.taskmeta.json` exists at the worktree root.
3. Read: `platform-parity-spec.md`, `tasks.json`, `session_log.md`, and the slice spec.

## Validation (minimum)
- Re-run Linux validation for the boundary slice:
  - `bash docs/project_management/packs/draft/dev-install-world-agent-staging/smoke/linux-smoke.sh`
  - `bash tests/installers/install_smoke.sh`

## End Checklist
1. Record Linux evidence and any fix commits in `session_log.md`.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="DIWAS1-integ-linux"`.
