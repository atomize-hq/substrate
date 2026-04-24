# Kickoff: CP2-ci-checkpoint (CI checkpoint)

## Scope
- Run the checkpoint gate for the group ending at `AHCSITC3`.
- This task runs on the orchestration checkout. It does not use a task worktree.
- Source of truth: `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/pre-planning/ci_checkpoint_plan.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Ensure you are on `feat/agent-hub-core-successor-identity-tuple-compatible`.
2. Read `pre-planning/ci_checkpoint_plan.md`, `tasks.json`, `session_log.md`, and this prompt.
3. Confirm the checkpoint slice is `AHCSITC3` and the checkpoint task depends on `AHCSITC3-integ-core`.
4. Compute the checkpoint checkout SHA from the `AHCSITC3-integ-core` branch before dispatching CI.

## Requirements
- Run the advisory CI audit if available, then dispatch compile parity for the checkpoint checkout SHA.
- Record run ids, URLs, and any required Linux/macOS/Windows parity follow-ups in `session_log.md`.
- Keep the task scoped to compile parity. `pre-planning/ci_checkpoint_plan.md` currently leaves feature smoke disabled for this checkpoint.

## End Checklist
1. Confirm compile parity is green or record the blocking failures precisely.
2. Mark `CP2-ci-checkpoint` completed in `tasks.json` and add the END entry to `session_log.md`.
3. Do not start parity-fix tasks without recording the checkpoint evidence that justified them.
