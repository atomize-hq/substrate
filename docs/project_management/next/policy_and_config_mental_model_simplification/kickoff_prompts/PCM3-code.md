# Kickoff: PCM3-code (Env scripts + world enable home + legacy removals)

## Scope
- Implement env.sh/manager_env.sh behavior, world enable --home semantics, and legacy removals per `PCM3-spec.md`.
- Production code only; do not add or modify tests.

## Start Checklist
1. `git checkout feat/policy-and-config-mental-model-simplification && git pull --ff-only`
2. Read: `docs/project_management/next/policy_and_config_mental_model_simplification/plan.md`, `docs/project_management/next/policy_and_config_mental_model_simplification/tasks.json`, `docs/project_management/next/policy_and_config_mental_model_simplification/session_log.md`, `docs/project_management/next/policy_and_config_mental_model_simplification/PCM3-spec.md`, and this prompt.
3. Set `PCM3-code` status to `in_progress` in `docs/project_management/next/policy_and_config_mental_model_simplification/tasks.json`; add a START entry to `docs/project_management/next/policy_and_config_mental_model_simplification/session_log.md`; commit docs (`docs: start PCM3-code`).
4. Create branch and worktree:
   - `git checkout -b pcm-pcm3-env-code`
   - `git worktree add wt/pcm3-env-code pcm-pcm3-env-code`
5. Do not edit docs/tasks/session_log.md inside the worktree.

## Requirements
- Implement env.sh ownership and exports and manager_env.sh runtime wiring per `PCM3-spec.md`.
- Implement `substrate world enable --home` semantics and reject `--prefix` per `PCM3-spec.md`.
- Remove legacy env vars and flags listed in `PCM3-spec.md`.

## Required Commands
- `cargo fmt`
- `cargo clippy --workspace --all-targets -- -D warnings`

## End Checklist
1. Run required commands; capture outputs for the END entry.
2. Commit worktree changes.
3. Merge back to `feat/policy-and-config-mental-model-simplification` (ff-only).
4. Update `docs/project_management/next/policy_and_config_mental_model_simplification/tasks.json` + `docs/project_management/next/policy_and_config_mental_model_simplification/session_log.md` (END entry), commit docs (`docs: finish PCM3-code`).
5. Remove worktree.

