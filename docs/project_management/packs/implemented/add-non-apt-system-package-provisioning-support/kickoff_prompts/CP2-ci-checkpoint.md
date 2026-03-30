# Kickoff: CP2-ci-checkpoint (CI checkpoint)

## Scope
- Run the cross-platform CI gates for the `NASP3`-`NASP4` checkpoint group.
- Plan: `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/pre-planning/ci_checkpoint_plan.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Ensure you are on the orchestration branch `feat/add-non-apt-system-package-provisioning-support`.
2. Read `pre-planning/ci_checkpoint_plan.md`, `tasks.json`, `session_log.md`, and this prompt.
3. Confirm `NASP4-integ-core` is completed and pushed.

## Required commands
- `scripts/ci-audit/ci_audit.sh --kind ci-testing --orch-branch "feat/add-non-apt-system-package-provisioning-support"`
- `scripts/ci-audit/ci_audit.sh --kind feature-smoke --orch-branch "feat/add-non-apt-system-package-provisioning-support" --feature-dir "docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support"`
- `make ci-compile-parity CI_WORKFLOW_REF="feat/add-non-apt-system-package-provisioning-support" CI_REMOTE=origin CI_CLEANUP=1`
- `make feature-smoke FEATURE_DIR="docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support" PLATFORM=behavior SMOKE_SLICE_ID="NASP4" RUNNER_KIND=self-hosted WORKFLOW_REF="feat/add-non-apt-system-package-provisioning-support" REMOTE=origin CLEANUP=1 RUN_INTEG_CHECKS=0`

## End Checklist
1. Record run ids and URLs in `session_log.md`.
2. Mark the task completed on the orchestration branch.
