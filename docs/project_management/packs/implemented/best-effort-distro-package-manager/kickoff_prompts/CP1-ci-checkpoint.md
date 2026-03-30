# Kickoff: CP1-ci-checkpoint (CI checkpoint)

## Scope
- Run the cross-platform CI gates defined by `docs/project_management/packs/draft/best-effort-distro-package-manager/pre-planning/ci_checkpoint_plan.md`.
- This task runs on the orchestration checkout. No worktree is used.

## Start Checklist
Do not edit planning docs inside the worktree.

1. Ensure you are on `feat/best-effort-distro-package-manager`.
2. Read `pre-planning/ci_checkpoint_plan.md`, `tasks.json`, `session_log.md`, `pre-planning/impact_map.md`, and this prompt.
3. Compute the checkpoint SHA from `BEDPM3-integ-core`:
   - `CORE_BRANCH="$(jq -r --arg id "BEDPM3-integ-core" '.tasks[] | select(.id==$id) | .git_branch' "docs/project_management/packs/draft/best-effort-distro-package-manager/tasks.json")"`
   - `CHECKOUT_SHA="$(git rev-parse "$CORE_BRANCH")"`

## Requirements
- Run CI audit before dispatch:
  - `scripts/ci-audit/ci_audit.sh --ledger-path "docs/project_management/packs/draft/best-effort-distro-package-manager/logs/BEDPM3/ci-audit/ledger.jsonl" --kind ci-testing --orch-branch "feat/best-effort-distro-package-manager"`
  - `scripts/ci-audit/ci_audit.sh --ledger-path "docs/project_management/packs/draft/best-effort-distro-package-manager/logs/BEDPM3/ci-audit/ledger.jsonl" --kind feature-smoke --orch-branch "feat/best-effort-distro-package-manager" --feature-dir "docs/project_management/packs/draft/best-effort-distro-package-manager"`
- Dispatch the checkpoint gates against `CHECKOUT_SHA`:
  - `make ci-compile-parity CI_WORKFLOW_REF="feat/best-effort-distro-package-manager" CI_REMOTE=origin CI_CLEANUP=1 CI_CHECKOUT_REF="$CHECKOUT_SHA"`
  - `make ci-testing CI_WORKFLOW_REF="feat/best-effort-distro-package-manager" CI_REMOTE=origin CI_CLEANUP=1 CI_MODE=quick CI_CHECKOUT_REF="$CHECKOUT_SHA"`
  - `make feature-smoke FEATURE_DIR="docs/project_management/packs/draft/best-effort-distro-package-manager" PLATFORM=behavior SMOKE_SLICE_ID="BEDPM3" SMOKE_CHECKOUT_REF="$CHECKOUT_SHA" RUNNER_KIND=self-hosted WORKFLOW_REF="feat/best-effort-distro-package-manager" REMOTE=origin CLEANUP=1 RUN_INTEG_CHECKS=0`
- If smoke or CI parity fails, start only the matching BEDPM3 platform-fix task or tasks from the orchestration checkout.

## End Checklist
1. Record audit output plus run ids and URLs in `session_log.md`.
2. Mark `CP1-ci-checkpoint` completed in `tasks.json`.
3. Hand off the failing platform list when follow-up work is required.
