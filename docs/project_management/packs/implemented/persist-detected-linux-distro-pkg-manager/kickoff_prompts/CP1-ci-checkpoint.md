# Kickoff: CP1-ci-checkpoint (CI checkpoint)

## Scope
- Run the cross-platform CI gates defined by `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/ci_checkpoint_plan.md`.
- This task runs on the orchestration checkout (no worktree). Do not edit planning docs inside any worktree.

## Start Checklist
Do not edit planning docs inside the worktree.

1. Ensure you are on the orchestration branch `feat/persist-detected-linux-distro-pkg-manager`.
2. Read: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/ci_checkpoint_plan.md`, `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/tasks.json`, `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/session_log.md`, `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/impact_map.md`.
3. This checkpoint validates slice `PDLDPM2` and the core integration task `PDLDPM2-integ-core`.
4. Compute `CHECKOUT_SHA` for the core integration branch without checking it out:
   - `CORE_BRANCH="$(jq -r --arg id "PDLDPM2-integ-core" '.tasks[] | select(.id==$id) | .git_branch' "docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/tasks.json")"`
   - `CHECKOUT_SHA="$(git rev-parse "$CORE_BRANCH")"`

## CI audit

Ledger path (not committed):
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/logs/PDLDPM2/ci-audit/ledger.jsonl`

Run audits:
- CI Testing:
  - `scripts/ci-audit/ci_audit.sh --ledger-path "docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/logs/PDLDPM2/ci-audit/ledger.jsonl" --kind ci-testing --orch-branch "feat/persist-detected-linux-distro-pkg-manager"`
- Feature Smoke:
  - `scripts/ci-audit/ci_audit.sh --ledger-path "docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/logs/PDLDPM2/ci-audit/ledger.jsonl" --kind feature-smoke --orch-branch "feat/persist-detected-linux-distro-pkg-manager" --feature-dir "docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager"`

If `ci_audit.sh` outputs `RECOMMEND=skip` for a gate:
- Do not dispatch that gate.
- Record the audit output lines and last-green evidence in `session_log.md`.

## Required gates (dispatch from orchestration checkout)

1) Cross-platform compile parity:
- `make ci-compile-parity CI_WORKFLOW_REF="feat/persist-detected-linux-distro-pkg-manager" CI_REMOTE=origin CI_CLEANUP=1 CI_CHECKOUT_REF="$CHECKOUT_SHA"`

2) CI Testing (quick):
- `make ci-testing CI_WORKFLOW_REF="feat/persist-detected-linux-distro-pkg-manager" CI_REMOTE=origin CI_CLEANUP=1 CI_MODE=quick CI_CHECKOUT_REF="$CHECKOUT_SHA"`

3) Linux behavior smoke:
- `make feature-smoke FEATURE_DIR="docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager" PLATFORM=behavior SMOKE_SLICE_ID="PDLDPM2" SMOKE_CHECKOUT_REF="$CHECKOUT_SHA" RUNNER_KIND=self-hosted WORKFLOW_REF="feat/persist-detected-linux-distro-pkg-manager" REMOTE=origin CLEANUP=1 RUN_INTEG_CHECKS=0`

## If smoke fails

Start only failing platform-fix tasks:
- `make triad-task-start-platform-fixes-from-smoke FEATURE_DIR="docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager" SLICE_ID="PDLDPM2" SMOKE_RUN_ID="<run-id>" LAUNCH_CODEX=1`

## End Checklist

1. Record run ids or URLs for compile parity, CI Testing, and smoke in `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/session_log.md`.
2. Mark task `CP1-ci-checkpoint` as `completed` in `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/tasks.json`.
