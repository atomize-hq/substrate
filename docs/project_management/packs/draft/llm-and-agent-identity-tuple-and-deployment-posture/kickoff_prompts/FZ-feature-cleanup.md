# Kickoff: FZ-feature-cleanup (feature cleanup)

## Scope
- Remove retained task worktrees and finish the feature-level cleanup.
- This task runs on the orchestration branch.

Do not edit planning docs inside the worktree.

## Preconditions
- All feature tasks are completed.
- The orchestration checkout is clean.

## How To Run
- Dry run:
  - `make triad-feature-cleanup FEATURE_DIR="docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture" DRY_RUN=1 REMOVE_WORKTREES=1 PRUNE_LOCAL=1`
- Real run:
  - `make triad-feature-cleanup FEATURE_DIR="docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture" REMOVE_WORKTREES=1 PRUNE_LOCAL=1`
- Remote prune when needed:
  - `make triad-feature-cleanup FEATURE_DIR="docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture" PRUNE_REMOTE=origin PRUNE_LOCAL=1 REMOVE_WORKTREES=1`

## Output Requirement
- Paste the cleanup summary block into the `FZ-feature-cleanup` END entry in `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/session_log.md`.
