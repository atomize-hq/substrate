# Kickoff: PCM1-code (Policy inventory and CLI)

## Scope
- Implement policy schema/discovery/strict parsing/invariants and policy CLI per `PCM1-spec.md`.
- Production code only; do not add or modify tests.

## Start Checklist

Do not edit planning docs inside the worktree.

1. `git checkout feat/policy_and_config && git pull --ff-only`
2. Read: `docs/project_management/next/policy_and_config_mental_model_simplification/plan.md`, `docs/project_management/next/policy_and_config_mental_model_simplification/tasks.json`, `docs/project_management/next/policy_and_config_mental_model_simplification/session_log.md`, `docs/project_management/next/policy_and_config_mental_model_simplification/PCM1-spec.md`, `docs/project_management/next/policy_and_config_mental_model_simplification/decision_register.md`, `docs/project_management/next/policy_and_config_mental_model_simplification/integration_map.md`, and this prompt.
3. Set `PCM1-code` status to `in_progress` in `docs/project_management/next/policy_and_config_mental_model_simplification/tasks.json`; add a START entry to `docs/project_management/next/policy_and_config_mental_model_simplification/session_log.md`; commit docs (`docs: start PCM1-code`).
4. Create branch and worktree:
   - `git checkout -b pcm-pcm1-policy-code`
   - `git worktree add wt/pcm1-policy-code pcm-pcm1-policy-code`
5. Do not edit docs/tasks/session_log.md inside the worktree.

## Requirements
- Implement strict YAML parsing and invariants per `PCM1-spec.md`.
- Implement `substrate policy` commands per `PCM1-spec.md`.

## Required Commands
- `cargo fmt`
- `cargo clippy --workspace --all-targets -- -D warnings`

## End Checklist
1. Run required commands; capture outputs for the END entry.
2. Commit worktree changes.
3. Merge back to `feat/policy_and_config` (ff-only).
4. Update `docs/project_management/next/policy_and_config_mental_model_simplification/tasks.json` + `docs/project_management/next/policy_and_config_mental_model_simplification/session_log.md` (END entry), commit docs (`docs: finish PCM1-code`).
5. Remove worktree.
