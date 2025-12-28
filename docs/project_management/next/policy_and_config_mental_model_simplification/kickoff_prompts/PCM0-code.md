# Kickoff: PCM0-code (Workspace + config inventory and CLI)

## Scope
- Implement workspace discovery/init and config schema/discovery/precedence and config CLI per `PCM0-spec.md`.
- Production code only; do not add or modify tests.

## Start Checklist
1. `git checkout feat/policy-and-config-mental-model-simplification && git pull --ff-only`
2. Read: `docs/project_management/next/policy_and_config_mental_model_simplification/plan.md`, `docs/project_management/next/policy_and_config_mental_model_simplification/tasks.json`, `docs/project_management/next/policy_and_config_mental_model_simplification/session_log.md`, `docs/project_management/next/policy_and_config_mental_model_simplification/PCM0-spec.md`, `docs/project_management/next/policy_and_config_mental_model_simplification/decision_register.md`, `docs/project_management/next/policy_and_config_mental_model_simplification/integration_map.md`, and this prompt.
3. Set `PCM0-code` status to `in_progress` in `docs/project_management/next/policy_and_config_mental_model_simplification/tasks.json`; add a START entry to `docs/project_management/next/policy_and_config_mental_model_simplification/session_log.md`; commit docs (`docs: start PCM0-code`).
4. Create branch and worktree:
   - `git checkout -b pcm-pcm0-config-code`
   - `git worktree add wt/pcm0-config-code pcm-pcm0-config-code`
5. Do not edit docs/tasks/session_log.md inside the worktree.

## Requirements
- Implement strict YAML parsing and legacy `.substrate/settings.yaml` rejection per `PCM0-spec.md`.
- Implement `substrate workspace init` and `substrate config` commands per `PCM0-spec.md`.

## Required Commands
- `cargo fmt`
- `cargo clippy --workspace --all-targets -- -D warnings`

## End Checklist
1. Run required commands; capture outputs for the END entry.
2. Commit worktree changes.
3. Merge back to `feat/policy-and-config-mental-model-simplification` (ff-only).
4. Update `docs/project_management/next/policy_and_config_mental_model_simplification/tasks.json` + `docs/project_management/next/policy_and_config_mental_model_simplification/session_log.md` (END entry), commit docs (`docs: finish PCM0-code`).
5. Remove worktree.

