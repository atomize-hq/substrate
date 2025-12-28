# Kickoff: PCM2-code (Policy mode + routing semantics)

## Scope
- Implement policy.mode and routing semantics per `PCM2-spec.md`.
- Production code only; do not add or modify tests.

## Start Checklist
1. `git checkout feat/policy-and-config-mental-model-simplification && git pull --ff-only`
2. Read: `docs/project_management/next/policy_and_config_mental_model_simplification/plan.md`, `docs/project_management/next/policy_and_config_mental_model_simplification/tasks.json`, `docs/project_management/next/policy_and_config_mental_model_simplification/session_log.md`, `docs/project_management/next/policy_and_config_mental_model_simplification/PCM2-spec.md`, and this prompt.
3. Set `PCM2-code` status to `in_progress` in `docs/project_management/next/policy_and_config_mental_model_simplification/tasks.json`; add a START entry to `docs/project_management/next/policy_and_config_mental_model_simplification/session_log.md`; commit docs (`docs: start PCM2-code`).
4. Create branch and worktree:
   - `git checkout -b pcm-pcm2-routing-code`
   - `git worktree add wt/pcm2-routing-code pcm-pcm2-routing-code`
5. Do not edit docs/tasks/session_log.md inside the worktree.

## Requirements
- Implement disabled|observe|enforce semantics and “requires world” constraints per `PCM2-spec.md`.
- Implement save-to-policy write target selection per `PCM2-spec.md`.

## Required Commands
- `cargo fmt`
- `cargo clippy --workspace --all-targets -- -D warnings`

## End Checklist
1. Run required commands; capture outputs for the END entry.
2. Commit worktree changes.
3. Merge back to `feat/policy-and-config-mental-model-simplification` (ff-only).
4. Update `docs/project_management/next/policy_and_config_mental_model_simplification/tasks.json` + `docs/project_management/next/policy_and_config_mental_model_simplification/session_log.md` (END entry), commit docs (`docs: finish PCM2-code`).
5. Remove worktree.

