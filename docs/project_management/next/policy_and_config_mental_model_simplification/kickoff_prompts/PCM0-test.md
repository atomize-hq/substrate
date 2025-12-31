# Kickoff: PCM0-test (Workspace + config inventory and CLI)

## Scope
- Add and update tests that lock in workspace discovery/init and config precedence and strict parsing per `PCM0-spec.md`.
- Tests only; do not edit production code.

## Start Checklist

Do not edit planning docs inside the worktree.

1. `git checkout feat/policy_and_config && git pull --ff-only`
2. Read: `docs/project_management/next/policy_and_config_mental_model_simplification/plan.md`, `docs/project_management/next/policy_and_config_mental_model_simplification/tasks.json`, `docs/project_management/next/policy_and_config_mental_model_simplification/session_log.md`, `docs/project_management/next/policy_and_config_mental_model_simplification/PCM0-spec.md`, and this prompt.
3. Set `PCM0-test` status to `in_progress` in `docs/project_management/next/policy_and_config_mental_model_simplification/tasks.json`; add a START entry to `docs/project_management/next/policy_and_config_mental_model_simplification/session_log.md`; commit docs (`docs: start PCM0-test`).
4. Create branch and worktree:
   - `git checkout -b pcm-pcm0-config-test`
   - `git worktree add wt/pcm0-config-test pcm-pcm0-config-test`
5. Do not edit docs/tasks/session_log.md inside the worktree.

## Requirements
- Tests must cover:
  - walk-up discovery and no-workspace behavior,
  - nested init refusal (exit 2 and no writes),
  - config precedence CLI > env > workspace > global > defaults for all keys,
  - protected excludes always present and non-removable.

## Required Commands
- `cargo fmt`
- Targeted `cargo test ...` for tests added/modified

## End Checklist
1. Run required commands; capture outputs for the END entry.
2. Commit worktree changes.
3. Merge back to `feat/policy_and_config` (ff-only).
4. Update `docs/project_management/next/policy_and_config_mental_model_simplification/tasks.json` + `docs/project_management/next/policy_and_config_mental_model_simplification/session_log.md` (END entry), commit docs (`docs: finish PCM0-test`).
5. Remove worktree.
