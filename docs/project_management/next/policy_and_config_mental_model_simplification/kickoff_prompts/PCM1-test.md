# Kickoff: PCM1-test (Policy inventory and CLI)

## Scope
- Add and update tests that lock in policy strict parsing, invariants, discovery, and CLI behavior per `PCM1-spec.md`.
- Tests only; do not edit production code.

## Start Checklist
1. `git checkout feat/policy_and_config && git pull --ff-only`
2. Read: `docs/project_management/next/policy_and_config_mental_model_simplification/plan.md`, `docs/project_management/next/policy_and_config_mental_model_simplification/tasks.json`, `docs/project_management/next/policy_and_config_mental_model_simplification/session_log.md`, `docs/project_management/next/policy_and_config_mental_model_simplification/PCM1-spec.md`, and this prompt.
3. Set `PCM1-test` status to `in_progress` in `docs/project_management/next/policy_and_config_mental_model_simplification/tasks.json`; add a START entry to `docs/project_management/next/policy_and_config_mental_model_simplification/session_log.md`; commit docs (`docs: start PCM1-test`).
4. Create branch and worktree:
   - `git checkout -b pcm-pcm1-policy-test`
   - `git worktree add wt/pcm1-policy-test pcm-pcm1-policy-test`
5. Do not edit docs/tasks/session_log.md inside the worktree.

## Requirements
- Tests must cover:
  - policy discovery selection order,
  - invariants failing load,
  - strict parsing rejecting unknown keys and type mismatches.

## Required Commands
- `cargo fmt`
- Targeted `cargo test ...` for tests added/modified

## End Checklist
1. Run required commands; capture outputs for the END entry.
2. Commit worktree changes.
3. Merge back to `feat/policy_and_config` (ff-only).
4. Update `docs/project_management/next/policy_and_config_mental_model_simplification/tasks.json` + `docs/project_management/next/policy_and_config_mental_model_simplification/session_log.md` (END entry), commit docs (`docs: finish PCM1-test`).
5. Remove worktree.
