# Kickoff: CP1-ci-checkpoint (CI checkpoint)

## Scope
- Run the cross-platform CI gates defined by `docs/project_management/packs/draft/world-disabled-reason-attribution/pre-planning/ci_checkpoint_plan.md`.
- This task runs on the orchestration checkout.

## Start Checklist
Do not edit planning docs inside the worktree.

1. Ensure you are on the orchestration branch `feat/world-disabled-reason-attribution`.
2. Read:
   - `docs/project_management/packs/draft/world-disabled-reason-attribution/pre-planning/ci_checkpoint_plan.md`
   - `docs/project_management/packs/draft/world-disabled-reason-attribution/tasks.json`
   - `docs/project_management/packs/draft/world-disabled-reason-attribution/session_log.md`
   - `docs/project_management/packs/draft/world-disabled-reason-attribution/pre-planning/impact_map.md`
3. Confirm that this checkpoint validates the boundary slice `WDRA2`.
4. Determine the exact commit that this checkpoint validates:
   - `CORE_BRANCH="$(jq -r --arg id "WDRA2-integ-core" '.tasks[] | select(.id==$id) | .git_branch' "docs/project_management/packs/draft/world-disabled-reason-attribution/tasks.json")"`
   - `CHECKOUT_SHA="$(git rev-parse "$CORE_BRANCH")"`

## Required gates
- Compile parity:
  - `make ci-compile-parity CI_WORKFLOW_REF="feat/world-disabled-reason-attribution" CI_REMOTE=origin CI_CLEANUP=1 CI_CHECKOUT_REF="$CHECKOUT_SHA"`
- Feature smoke:
  - `make feature-smoke FEATURE_DIR="docs/project_management/packs/draft/world-disabled-reason-attribution" PLATFORM=behavior SMOKE_SLICE_ID="WDRA2" SMOKE_CHECKOUT_REF="$CHECKOUT_SHA" RUNNER_KIND=self-hosted WORKFLOW_REF="feat/world-disabled-reason-attribution" REMOTE=origin CLEANUP=1 RUN_INTEG_CHECKS=0`

## If smoke fails
- `make triad-task-start-platform-fixes-from-smoke FEATURE_DIR="docs/project_management/packs/draft/world-disabled-reason-attribution" SLICE_ID="WDRA2" SMOKE_RUN_ID="<run-id>" LAUNCH_CODEX=1`

## End Checklist
1. Record run ids and URLs in `session_log.md`.
2. Mark this task completed in `tasks.json` and add an END entry.
