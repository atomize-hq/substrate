# Kickoff: CP2-ci-checkpoint (CI checkpoint)

## Scope
- Run the cross-platform checkpoint gates defined by `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/pre-planning/ci_checkpoint_plan.md`.
- This task runs on the orchestration checkout. No worktree is used.

## Start Checklist
Do not edit planning docs inside the worktree.

1. Ensure the orchestration checkout is on `feat/llm-and-agent-identity-tuple-and-deployment-posture`.
2. Read: `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/pre-planning/ci_checkpoint_plan.md`, `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/tasks.json`, `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/session_log.md`, and `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/pre-planning/impact_map.md`.
3. Confirm `LAITDP2` is listed in `tasks.json` `meta.checkpoint_boundaries`.
4. Compute the checkpoint checkout SHA from `LAITDP2-integ-core`:
   - `CORE_BRANCH="$(jq -r --arg id "LAITDP2-integ-core" '.tasks[] | select(.id==$id) | .git_branch' docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/tasks.json)"`
   - `CHECKOUT_SHA="$(git rev-parse "$CORE_BRANCH")"`

## Required Gates
1. Run compile parity:
   - `make ci-compile-parity CI_WORKFLOW_REF="feat/llm-and-agent-identity-tuple-and-deployment-posture" CI_REMOTE=origin CI_CLEANUP=1 CI_CHECKOUT_REF="$CHECKOUT_SHA"`
2. Run feature smoke:
   - `make feature-smoke FEATURE_DIR="docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture" PLATFORM=behavior SMOKE_SLICE_ID="LAITDP2" SMOKE_CHECKOUT_REF="$CHECKOUT_SHA" RUNNER_KIND=self-hosted WORKFLOW_REF="feat/llm-and-agent-identity-tuple-and-deployment-posture" REMOTE=origin CLEANUP=1 RUN_INTEG_CHECKS=0`
3. If smoke fails, start only the failing platform-fix tasks for `LAITDP2`.

## End Checklist
1. Record run ids and URLs in `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/session_log.md`.
2. Mark `CP2-ci-checkpoint` completed in `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/tasks.json`.
3. Do not begin feature cleanup until this task is green.
