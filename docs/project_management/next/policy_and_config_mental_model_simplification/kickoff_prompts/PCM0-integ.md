# Kickoff: PCM0-integ (Workspace + config inventory and CLI)

## Scope
- Merge PCM0 code+tests, reconcile to `PCM0-spec.md`, and run integration requirements.
- Integration owns the final green state for PCM0.

## Start Checklist
1. `git checkout feat/policy-and-config-mental-model-simplification && git pull --ff-only`
2. Read: `docs/project_management/next/policy_and_config_mental_model_simplification/plan.md`, `docs/project_management/next/policy_and_config_mental_model_simplification/tasks.json`, `docs/project_management/next/policy_and_config_mental_model_simplification/session_log.md`, `docs/project_management/next/policy_and_config_mental_model_simplification/PCM0-spec.md`, and this prompt.
3. Set `PCM0-integ` status to `in_progress` in `docs/project_management/next/policy_and_config_mental_model_simplification/tasks.json`; add a START entry to `docs/project_management/next/policy_and_config_mental_model_simplification/session_log.md`; commit docs (`docs: start PCM0-integ`).
4. Create branch and worktree:
   - `git checkout -b pcm-pcm0-config-integ`
   - `git worktree add wt/pcm0-config-integ pcm-pcm0-config-integ`
5. Do not edit docs/tasks/session_log.md inside the worktree.

## Requirements
- Merge `pcm-pcm0-config-code` and `pcm-pcm0-config-test` branches.
- Reconcile all behavior to `PCM0-spec.md`.

## Required Commands
- `cargo fmt`
- `cargo clippy --workspace --all-targets -- -D warnings`
- Relevant `cargo test ...`
- `make preflight`

## End Checklist
1. Run required commands; capture outputs for the END entry.
2. Run feature smoke scripts:
   - `bash docs/project_management/next/policy_and_config_mental_model_simplification/smoke/linux-smoke.sh`
   - `bash docs/project_management/next/policy_and_config_mental_model_simplification/smoke/macos-smoke.sh`
   - `pwsh -File docs/project_management/next/policy_and_config_mental_model_simplification/smoke/windows-smoke.ps1`
3. Commit integration changes.
4. Merge back to `feat/policy-and-config-mental-model-simplification` (ff-only).
5. Update `docs/project_management/next/policy_and_config_mental_model_simplification/tasks.json` + `docs/project_management/next/policy_and_config_mental_model_simplification/session_log.md` (END entry), commit docs (`docs: finish PCM0-integ`).
6. Remove worktree.

