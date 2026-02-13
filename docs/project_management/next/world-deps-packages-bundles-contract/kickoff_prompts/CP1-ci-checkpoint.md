# Kickoff: CP1-ci-checkpoint (CI checkpoint)

## Scope
- Run the cross-platform CI gates defined by `docs/project_management/next/world-deps-packages-bundles-contract/ci_checkpoint_plan.md`.
- This task runs on the orchestration checkout (no worktree). Do not edit planning docs inside any worktree.

## Start Checklist
Do not edit planning docs inside the worktree.

1. Ensure you are on the orchestration branch `feat/world-deps-packages-bundles-contract` (or the orchestration worktree).
2. Read: `docs/project_management/next/world-deps-packages-bundles-contract/ci_checkpoint_plan.md`, `docs/project_management/next/world-deps-packages-bundles-contract/tasks.json`, `docs/project_management/next/world-deps-packages-bundles-contract/session_log.md`, `docs/project_management/next/world-deps-packages-bundles-contract/impact_map.md`.
3. Confirm which slice id this checkpoint is validating (per `ci_checkpoint_plan.md`). Use that slice id for:
   - `SMOKE_SLICE_ID="<slice>"`
   - platform-fix task ids and wrappers (e.g., `<slice>-integ-linux`)
   - Schema v4+ note: confirm `<slice>` is listed in `tasks.json` `meta.checkpoint_boundaries` (checkpoint-boundary slice).
4. Determine the exact commit that this checkpoint validates:
   - This checkpoint validates the **core integration branch** for the checkpoint slice (`<slice>-integ-core`).
   - Compute `CHECKOUT_SHA` from `tasks.json` without checking out the branch:
     - `CORE_BRANCH="$(jq -r --arg id "<slice>-integ-core" '.tasks[] | select(.id==$id) | .git_branch' "docs/project_management/next/world-deps-packages-bundles-contract/tasks.json")"`
     - `CHECKOUT_SHA="$(git rev-parse "$CORE_BRANCH")"`
   - Use `CHECKOUT_SHA` for:
     - `CI_CHECKOUT_REF="$CHECKOUT_SHA"` (CI Testing / compile parity)
     - `SMOKE_CHECKOUT_REF="$CHECKOUT_SHA"` (Feature Smoke)

## CI audit (recommended)

Run the advisory CI audit to avoid redundant dispatch:
- Ledger path (not committed): `docs/project_management/next/world-deps-packages-bundles-contract/logs/<slice>/ci-audit/ledger.jsonl`
- CI Testing audit:
- `scripts/ci-audit/ci_audit.sh --ledger-path "docs/project_management/next/world-deps-packages-bundles-contract/logs/<slice>/ci-audit/ledger.jsonl" --kind ci-testing --orch-branch "feat/world-deps-packages-bundles-contract"`
- Feature Smoke audit:
- `scripts/ci-audit/ci_audit.sh --ledger-path "docs/project_management/next/world-deps-packages-bundles-contract/logs/<slice>/ci-audit/ledger.jsonl" --kind feature-smoke --orch-branch "feat/world-deps-packages-bundles-contract" --feature-dir "docs/project_management/next/world-deps-packages-bundles-contract"`

Policy:
- If `RECOMMEND=skip`, do not dispatch that gate; record the audit output lines + last-green run evidence in your handoff.
- If `RECOMMEND=run`, dispatch normally and record run id/URL.

## Required gates (dispatch from orchestration checkout)

1) Cross-platform compile parity (fast fail; GitHub-hosted):
- `make ci-compile-parity CI_WORKFLOW_REF="feat/world-deps-packages-bundles-contract" CI_REMOTE=origin CI_CLEANUP=1 CI_CHECKOUT_REF="$CHECKOUT_SHA"`

2) Cross-platform behavioral smoke (self-hosted; behavior platforms only):
- `make feature-smoke FEATURE_DIR="docs/project_management/next/world-deps-packages-bundles-contract" PLATFORM=behavior SMOKE_SLICE_ID="<slice>" SMOKE_CHECKOUT_REF="$CHECKOUT_SHA" RUNNER_KIND=self-hosted WORKFLOW_REF="feat/world-deps-packages-bundles-contract" REMOTE=origin CLEANUP=1 RUN_INTEG_CHECKS=0`

## If smoke fails

Start only failing platform-fix tasks (from orchestration checkout):
- Single multi-platform smoke run (`PLATFORM=behavior`):
  - `make triad-task-start-platform-fixes-from-smoke FEATURE_DIR="docs/project_management/next/world-deps-packages-bundles-contract" SLICE_ID="<slice>" SMOKE_RUN_ID="<run-id>" LAUNCH_CODEX=1`
- Per-platform smoke runs:
  - `make triad-task-start-platform-fixes FEATURE_DIR="docs/project_management/next/world-deps-packages-bundles-contract" SLICE_ID="<slice>" PLATFORMS="<csv>" LAUNCH_CODEX=1`

## End Checklist

1. Record run ids/URLs (compile parity + smoke, and any CI Testing runs) in `session_log.md`.
2. Mark this task `completed` in `tasks.json` and add an END entry.
3. If this checkpoint is blocking the next slice group, do not proceed until the checkpoint is completed.

