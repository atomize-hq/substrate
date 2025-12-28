# Kickoff: PCM2-test (Policy mode + routing semantics)

## Scope
- Add and update tests that lock in policy.mode semantics, requires-world behavior, and save-to-policy targeting per `PCM2-spec.md`.
- Tests only; do not edit production code.

## Start Checklist
1. `git checkout feat/policy-and-config-mental-model-simplification && git pull --ff-only`
2. Read: `docs/project_management/next/policy_and_config_mental_model_simplification/plan.md`, `docs/project_management/next/policy_and_config_mental_model_simplification/tasks.json`, `docs/project_management/next/policy_and_config_mental_model_simplification/session_log.md`, `docs/project_management/next/policy_and_config_mental_model_simplification/PCM2-spec.md`, and this prompt.
3. Set `PCM2-test` status to `in_progress` in `docs/project_management/next/policy_and_config_mental_model_simplification/tasks.json`; add a START entry to `docs/project_management/next/policy_and_config_mental_model_simplification/session_log.md`; commit docs (`docs: start PCM2-test`).
4. Create branch and worktree:
   - `git checkout -b pcm-pcm2-routing-test`
   - `git worktree add wt/pcm2-routing-test pcm-pcm2-routing-test`
5. Do not edit docs/tasks/session_log.md inside the worktree.

## Requirements
- Tests must cover:
  - disabled does not evaluate policy decisions,
  - observe allows execution and records would-deny,
  - enforce denies and fails closed when world is required and unavailable,
  - save-to-policy writes target selection.

## Required Commands
- `cargo fmt`
- Targeted `cargo test ...` for tests added/modified

## End Checklist
1. Run required commands; capture outputs for the END entry.
2. Commit worktree changes.
3. Merge back to `feat/policy-and-config-mental-model-simplification` (ff-only).
4. Update `docs/project_management/next/policy_and_config_mental_model_simplification/tasks.json` + `docs/project_management/next/policy_and_config_mental_model_simplification/session_log.md` (END entry), commit docs (`docs: finish PCM2-test`).
5. Remove worktree.

