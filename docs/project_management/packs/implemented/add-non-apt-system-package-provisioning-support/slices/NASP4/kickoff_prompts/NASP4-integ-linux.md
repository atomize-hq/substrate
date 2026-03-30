# Kickoff: NASP4-integ-linux (integration platform-fix — linux)

## Scope
- Make the NASP4 checkpoint slice green for **linux**.
- Spec: `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/NASP4/NASP4-spec.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Run this task on a linux machine if possible.
2. Verify worktree `wt/add-non-apt-system-package-provisioning-support-nasp4-integ-linux` on branch `add-non-apt-system-package-provisioning-support-nasp4-integ-linux`.
3. Read `plan.md`, `tasks.json`, `session_log.md`, the slice spec, and this prompt.

## Requirements
- Merge the `NASP4-integ-core` branch before making fixes.
- Run `cargo fmt` and `cargo clippy --workspace --all-targets -- -D warnings`.
- Run a local smoke preflight when possible, then dispatch:
  - `make feature-smoke FEATURE_DIR="docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support" PLATFORM=linux SMOKE_SLICE_ID="NASP4" RUNNER_KIND=self-hosted WORKFLOW_REF="feat/add-non-apt-system-package-provisioning-support" REMOTE=origin CLEANUP=1 RUN_INTEG_CHECKS=1`

## End Checklist
1. Run `make triad-task-finish TASK_ID="NASP4-integ-linux"` inside the worktree.
2. Hand off the smoke run id or URL to the operator.
