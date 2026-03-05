# Kickoff: CP2-ci-checkpoint (CI checkpoint)

## Scope
- Run the cross-platform CI gates defined by `docs/project_management/packs/draft/world-deps-apt-provisioning/pre-planning/ci_checkpoint_plan.md`.
- This task runs on the orchestration checkout (no worktree). Do not edit planning docs inside any worktree.

## Start Checklist
Do not edit planning docs inside the worktree.

1. Ensure you are on the orchestration branch `feat/world-deps-apt-provisioning` (or the orchestration worktree).
2. Read:
   - `docs/project_management/packs/draft/world-deps-apt-provisioning/pre-planning/ci_checkpoint_plan.md`
   - `docs/project_management/packs/draft/world-deps-apt-provisioning/tasks.json`
   - `docs/project_management/packs/draft/world-deps-apt-provisioning/session_log.md`
   - `docs/project_management/packs/draft/world-deps-apt-provisioning/pre-planning/impact_map.md`
3. Confirm which slice id this checkpoint is validating (per `ci_checkpoint_plan.md`).
   - For this pack, CP2 validates the checkpoint-boundary slice `WDAP1`.
   - Schema v4+ note: confirm `WDAP1` is listed in `tasks.json` `meta.checkpoint_boundaries`.
4. Determine the exact commit that this checkpoint validates:
   - This checkpoint validates the core integration branch for the checkpoint slice (`WDAP1-integ-core`).
   - Compute `CHECKOUT_SHA` from `tasks.json` without checking out the branch:
     - `CORE_BRANCH="$(jq -r --arg id \"WDAP1-integ-core\" '.tasks[] | select(.id==$id) | .git_branch' \"docs/project_management/packs/draft/world-deps-apt-provisioning/tasks.json\")"`
     - `CHECKOUT_SHA="$(git rev-parse \"$CORE_BRANCH\")"`
   - Use `CHECKOUT_SHA` for:
     - `CI_CHECKOUT_REF="$CHECKOUT_SHA"` (CI Testing / compile parity)
     - `SMOKE_CHECKOUT_REF="$CHECKOUT_SHA"` (Feature Smoke)

## CI audit (recommended)

Run the advisory CI audit to avoid redundant dispatch:
- Ledger path (not committed): `docs/project_management/packs/draft/world-deps-apt-provisioning/logs/WDAP1/ci-audit/ledger.jsonl`
- CI Testing audit:
  - `scripts/ci-audit/ci_audit.sh --ledger-path "docs/project_management/packs/draft/world-deps-apt-provisioning/logs/WDAP1/ci-audit/ledger.jsonl" --kind ci-testing --orch-branch "feat/world-deps-apt-provisioning"`
- Feature Smoke audit:
  - `scripts/ci-audit/ci_audit.sh --ledger-path "docs/project_management/packs/draft/world-deps-apt-provisioning/logs/WDAP1/ci-audit/ledger.jsonl" --kind feature-smoke --orch-branch "feat/world-deps-apt-provisioning" --feature-dir "docs/project_management/packs/draft/world-deps-apt-provisioning"`

Policy:
- If `RECOMMEND=skip`, do not dispatch that gate; record the audit output lines and last-green run evidence in your handoff.
- If `RECOMMEND=run`, dispatch normally and record run id/URL.

## Required gates (dispatch from orchestration checkout)

1) Cross-platform compile parity (fast fail; GitHub-hosted):
- `make ci-compile-parity CI_WORKFLOW_REF="feat/world-deps-apt-provisioning" CI_REMOTE=origin CI_CLEANUP=1 CI_CHECKOUT_REF="$CHECKOUT_SHA"`

2) Cross-platform behavioral smoke (self-hosted; behavior platforms only):
- `make feature-smoke FEATURE_DIR="docs/project_management/packs/draft/world-deps-apt-provisioning" PLATFORM=behavior SMOKE_SLICE_ID="WDAP1" SMOKE_CHECKOUT_REF="$CHECKOUT_SHA" RUNNER_KIND=self-hosted WORKFLOW_REF="feat/world-deps-apt-provisioning" REMOTE=origin CLEANUP=1 RUN_INTEG_CHECKS=0`

Notes:
- `SMOKE_SLICE_ID` is optional but recommended; the workflow exports `SUBSTRATE_SMOKE_SLICE_ID` for slice-scoped smoke scripts.

## If smoke fails

Start only failing platform-fix tasks (from orchestration checkout):
- Single multi-platform smoke run (`PLATFORM=behavior`):
  - `make triad-task-start-platform-fixes-from-smoke FEATURE_DIR="docs/project_management/packs/draft/world-deps-apt-provisioning" SLICE_ID="WDAP1" SMOKE_RUN_ID="<run-id>" LAUNCH_CODEX=1`
- Per-platform smoke runs:
  - `make triad-task-start-platform-fixes FEATURE_DIR="docs/project_management/packs/draft/world-deps-apt-provisioning" SLICE_ID="WDAP1" PLATFORMS="<csv>" LAUNCH_CODEX=1`

## End Checklist

1. Record run ids/URLs (compile parity + smoke, and any CI Testing runs) in `docs/project_management/packs/draft/world-deps-apt-provisioning/session_log.md`.
2. Mark this task `completed` in `tasks.json` and add an END entry.

