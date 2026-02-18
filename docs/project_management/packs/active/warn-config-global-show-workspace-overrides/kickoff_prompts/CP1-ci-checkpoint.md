# Kickoff: CP1-ci-checkpoint (CI checkpoint)

## Scope
- Run the cross-platform CI gates defined by `docs/project_management/packs/active/warn-config-global-show-workspace-overrides/ci_checkpoint_plan.md`.
- This task runs on the orchestration checkout (no worktree). Do not edit planning docs inside any worktree.

## Start Checklist
Do not edit planning docs inside the worktree.

1. Ensure you are on the orchestration branch `feat/warn-config-global-show-workspace-overrides`.
2. Read: `docs/project_management/packs/active/warn-config-global-show-workspace-overrides/ci_checkpoint_plan.md`, `docs/project_management/packs/active/warn-config-global-show-workspace-overrides/tasks.json`, `docs/project_management/packs/active/warn-config-global-show-workspace-overrides/session_log.md`, `docs/project_management/packs/active/warn-config-global-show-workspace-overrides/impact_map.md`.
3. This checkpoint validates slice `C0` and the core integration task `C0-integ-core`.
4. Compute `CHECKOUT_SHA` for the core integration branch without checking it out:
   - `CORE_BRANCH="$(jq -r --arg id "C0-integ-core" '.tasks[] | select(.id==$id) | .git_branch' "docs/project_management/packs/active/warn-config-global-show-workspace-overrides/tasks.json")"`
   - `CHECKOUT_SHA="$(git rev-parse "$CORE_BRANCH")"`

## CI audit (recommended)

Ledger path (not committed):
- `docs/project_management/packs/active/warn-config-global-show-workspace-overrides/logs/C0/ci-audit/ledger.jsonl`

Run audits:
- CI Testing:
  - `scripts/ci-audit/ci_audit.sh --ledger-path "docs/project_management/packs/active/warn-config-global-show-workspace-overrides/logs/C0/ci-audit/ledger.jsonl" --kind ci-testing --orch-branch "feat/warn-config-global-show-workspace-overrides"`
- Feature Smoke:
  - `scripts/ci-audit/ci_audit.sh --ledger-path "docs/project_management/packs/active/warn-config-global-show-workspace-overrides/logs/C0/ci-audit/ledger.jsonl" --kind feature-smoke --orch-branch "feat/warn-config-global-show-workspace-overrides" --feature-dir "docs/project_management/packs/active/warn-config-global-show-workspace-overrides"`

## Required gates (dispatch from orchestration checkout)

1) Cross-platform compile parity:
- `make ci-compile-parity CI_WORKFLOW_REF="feat/warn-config-global-show-workspace-overrides" CI_REMOTE=origin CI_CLEANUP=1 CI_CHECKOUT_REF="$CHECKOUT_SHA"`

2) Cross-platform behavior smoke (behavior platforms only):
- `make feature-smoke FEATURE_DIR="docs/project_management/packs/active/warn-config-global-show-workspace-overrides" PLATFORM=behavior SMOKE_SLICE_ID="C0" SMOKE_CHECKOUT_REF="$CHECKOUT_SHA" RUNNER_KIND=self-hosted WORKFLOW_REF="feat/warn-config-global-show-workspace-overrides" REMOTE=origin CLEANUP=1 RUN_INTEG_CHECKS=0`

## If smoke fails

Start only failing platform-fix tasks:
- `make triad-task-start-platform-fixes-from-smoke FEATURE_DIR="docs/project_management/packs/active/warn-config-global-show-workspace-overrides" SLICE_ID="C0" SMOKE_RUN_ID="<run-id>" LAUNCH_CODEX=1`

## End Checklist

1. Record run ids/URLs for compile parity and smoke in `docs/project_management/packs/active/warn-config-global-show-workspace-overrides/session_log.md`.
2. Mark task `CP1-ci-checkpoint` as `completed` in `docs/project_management/packs/active/warn-config-global-show-workspace-overrides/tasks.json`.

