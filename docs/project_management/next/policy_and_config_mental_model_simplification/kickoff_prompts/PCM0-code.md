# Kickoff: PCM0-code (code) — Workspace + config inventory and CLI

## Scope
- Production code only; no new tests.
- Spec: `docs/project_management/next/policy_and_config_mental_model_simplification/PCM0-spec.md`
- Execution workflow standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. If `.taskmeta.json` exists at the worktree root, you were started via triad automation and are already on the correct task branch; proceed.
2. If `.taskmeta.json` is missing, stop and ask the operator to start the task in a dedicated worktree (preferred: concurrent code+test) and re-run you from inside that worktree.
3. Read (end-to-end): `plan.md`, `tasks.json`, `session_log.md`, `PCM0-spec.md`, `decision_register.md`, `integration_map.md`, and this prompt.

## Requirements
- Implement strict YAML parsing and legacy `.substrate/settings.yaml` rejection per `PCM0-spec.md`.
- Implement `substrate workspace init` and `substrate config` commands per `PCM0-spec.md`.

## Required Commands
- `cargo fmt`
- `cargo clippy --workspace --all-targets -- -D warnings`

Baseline testing (required):
- Before changes: run a targeted baseline test set relevant to your change.
- After changes: re-run the same tests and ensure results are unchanged (or improved).

## End Checklist
1. Run required commands and baseline tests; capture outcomes.
2. Commit changes to the task branch in this worktree.
3. If triad automation is configured for this feature, run: `make triad-task-finish TASK_ID="PCM0-code"`.
4. Hand off results to the operator (do not edit planning docs inside the worktree; do not delete the worktree).
