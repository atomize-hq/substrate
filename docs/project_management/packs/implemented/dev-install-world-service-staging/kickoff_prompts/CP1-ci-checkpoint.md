# Kickoff: CP1-ci-checkpoint (CI checkpoint)

## Scope
- Run the cross-platform CI gates defined by `docs/project_management/packs/draft/dev-install-world-service-staging/pre-planning/ci_checkpoint_plan.md`.
- This task runs on the orchestration checkout (no worktree).

## Start Checklist
Do not edit planning docs inside the worktree.

1. Ensure you are on the orchestration branch `feat/dev-install-world-service-staging`.
2. Read:
   - `docs/project_management/packs/draft/dev-install-world-service-staging/pre-planning/ci_checkpoint_plan.md`
   - `docs/project_management/packs/draft/dev-install-world-service-staging/tasks.json`
   - `docs/project_management/packs/draft/dev-install-world-service-staging/session_log.md`
   - `docs/project_management/packs/draft/dev-install-world-service-staging/platform-parity-spec.md`
3. Determine the checkpoint SHA:
   - This checkpoint validates the `DIWAS1-integ-core` branch tip.
   - `CORE_BRANCH="$(jq -r --arg id \"DIWAS1-integ-core\" '.tasks[] | select(.id==$id) | .git_branch' \"docs/project_management/packs/draft/dev-install-world-service-staging/tasks.json\")"`
   - `CHECKOUT_SHA="$(git rev-parse \"$CORE_BRANCH\")"`

## Required gates (dispatch from orchestration checkout)

1) Cross-platform compile parity:
- `make ci-compile-parity CI_WORKFLOW_REF="feat/dev-install-world-service-staging" CI_REMOTE=origin CI_CLEANUP=1 CI_CHECKOUT_REF="$CHECKOUT_SHA"`

2) Linux feature smoke:
- `make feature-smoke FEATURE_DIR="docs/project_management/packs/draft/dev-install-world-service-staging" PLATFORM=behavior SMOKE_SLICE_ID="DIWAS1" SMOKE_CHECKOUT_REF="$CHECKOUT_SHA" RUNNER_KIND=self-hosted WORKFLOW_REF="feat/dev-install-world-service-staging" REMOTE=origin CLEANUP=1 RUN_INTEG_CHECKS=0`

## If a gate fails
- Start only the required follow-up platform-fix task(s) for the boundary slice (`DIWAS1`): `DIWAS1-integ-linux`, `DIWAS1-integ-macos`, `DIWAS1-integ-windows`.
- Record run ids/URLs and the failing evidence in `session_log.md`.

## End Checklist
1. Record run ids/URLs (compile parity + smoke) in `docs/project_management/packs/draft/dev-install-world-service-staging/session_log.md`.
2. Mark `CP1-ci-checkpoint` complete via `make triad-task-finish TASK_ID="CP1-ci-checkpoint"` (or the pack’s standard closeout path).
