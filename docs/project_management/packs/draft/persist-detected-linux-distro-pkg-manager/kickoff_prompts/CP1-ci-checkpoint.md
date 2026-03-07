# Kickoff: CP1-ci-checkpoint (CI checkpoint)

## Scope
- Run the cross-platform checkpoint gates for `PDLDPM2` from the orchestration checkout.
- This task runs on the orchestration branch and does not use a task worktree.
- Standards:
  - `docs/project_management/system/standards/triad/TASK_TRIADS_AND_FEATURE_SETUP.md`
  - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/ci_checkpoint_plan.md`

## Start Checklist

Do not edit planning docs inside the worktree.

1. Ensure you are on `feat/persist-detected-linux-distro-pkg-manager`.
2. Read `pre-planning/ci_checkpoint_plan.md`, `tasks.json`, `session_log.md`, `pre-planning/impact_map.md`, and this prompt.
3. Confirm this checkpoint validates `PDLDPM2`.
4. Compute the checkpoint checkout SHA from `PDLDPM2-integ-core`:
   - `CORE_BRANCH="$(jq -r --arg id "PDLDPM2-integ-core" '.tasks[] | select(.id==$id) | .git_branch' docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/tasks.json)"`
   - `CHECKOUT_SHA="$(git rev-parse "$CORE_BRANCH")"`

## Requirements
- Run the CI audit before dispatching:
  - `FEATURE_DIR="docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager"`
  - `scripts/ci-audit/ci_audit.sh --ledger-path "$FEATURE_DIR/logs/CP1/ci-audit/ledger.jsonl" --kind ci-testing --orch-branch "feat/persist-detected-linux-distro-pkg-manager"`
  - `scripts/ci-audit/ci_audit.sh --ledger-path "$FEATURE_DIR/logs/CP1/ci-audit/ledger.jsonl" --kind feature-smoke --orch-branch "feat/persist-detected-linux-distro-pkg-manager" --feature-dir "$FEATURE_DIR"`
- Dispatch compile parity for `CHECKOUT_SHA`:
  - `make ci-compile-parity CI_WORKFLOW_REF="feat/persist-detected-linux-distro-pkg-manager" CI_REMOTE=origin CI_CLEANUP=1 CI_CHECKOUT_REF="$CHECKOUT_SHA"`
- Dispatch Linux behavior smoke for `CHECKOUT_SHA`:
  - `make feature-smoke FEATURE_DIR="docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager" PLATFORM=behavior SMOKE_SLICE_ID="PDLDPM2" SMOKE_CHECKOUT_REF="$CHECKOUT_SHA" RUNNER_KIND=self-hosted WORKFLOW_REF="feat/persist-detected-linux-distro-pkg-manager" REMOTE=origin CLEANUP=1 RUN_INTEG_CHECKS=0`
- If smoke fails, start only the failing platform-fix tasks:
  - `make triad-task-start-platform-fixes-from-smoke FEATURE_DIR="docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager" SLICE_ID="PDLDPM2" SMOKE_RUN_ID="<run-id>" LAUNCH_CODEX=1`

## End Checklist
1. Record the audit output, checkpoint checkout SHA, and run ids in `session_log.md`.
2. Set `CP1-ci-checkpoint` to `completed` in `tasks.json`, add an END entry, and commit docs.
