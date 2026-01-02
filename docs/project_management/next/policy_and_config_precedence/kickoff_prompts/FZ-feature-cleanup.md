# FZ-feature-cleanup Kickoff — Policy + Config Precedence (feature cleanup)

You are the cleanup agent for `FZ-feature-cleanup`.

Scope:
- Remove retained task worktrees for this feature and optionally prune local/remote task branches.

Non-negotiable rule:
- Do not edit planning docs inside the worktree.

Start checklist:
1. Ensure the orchestration branch is checked out:
   - `make triad-orch-ensure FEATURE_DIR="docs/project_management/next/policy_and_config_precedence"`
2. Confirm all feature tasks are completed and merged as intended.
3. Update `docs/project_management/next/policy_and_config_precedence/tasks.json`:
   - set `FZ-feature-cleanup.status` to `in_progress`
4. Append a START entry to `docs/project_management/next/policy_and_config_precedence/session_log.md`; commit docs (`docs: start FZ-feature-cleanup`)

Execution:
1. Dry-run cleanup:
   - `make triad-feature-cleanup FEATURE_DIR="docs/project_management/next/policy_and_config_precedence" DRY_RUN=1 REMOVE_WORKTREES=1 PRUNE_LOCAL=1`
2. Apply cleanup:
   - `make triad-feature-cleanup FEATURE_DIR="docs/project_management/next/policy_and_config_precedence" REMOVE_WORKTREES=1 PRUNE_LOCAL=1`
3. Paste the stdout summary block into the END entry.

End checklist:
1. Update `tasks.json` to `completed` and append an END entry with the cleanup summary; commit docs (`docs: finish FZ-feature-cleanup`).
