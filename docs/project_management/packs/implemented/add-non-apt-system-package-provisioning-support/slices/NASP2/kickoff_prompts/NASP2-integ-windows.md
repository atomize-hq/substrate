# Kickoff: NASP2-integ-windows (integration platform-fix — windows)

## Scope
- Make the NASP2 checkpoint slice green for **windows**.
- Spec: `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/NASP2/NASP2-spec.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Run this task on a Windows machine if possible.
2. Verify worktree `wt/add-non-apt-system-package-provisioning-support-nasp2-integ-windows` on branch `add-non-apt-system-package-provisioning-support-nasp2-integ-windows`.
3. Read `plan.md`, `tasks.json`, `session_log.md`, the slice spec, and this prompt.

## Requirements
- Merge the `NASP2-integ-core` branch before making fixes.
- Run `cargo fmt` and `cargo clippy --workspace --all-targets -- -D warnings`.
- Run a local smoke preflight when possible, then dispatch:
  - `make feature-smoke FEATURE_DIR="docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support" PLATFORM=windows SMOKE_SLICE_ID="NASP2" RUNNER_KIND=self-hosted WORKFLOW_REF="feat/add-non-apt-system-package-provisioning-support" REMOTE=origin CLEANUP=1 RUN_INTEG_CHECKS=1`

## End Checklist
1. Run `make triad-task-finish TASK_ID="NASP2-integ-windows"` inside the worktree.
2. Hand off the smoke run id or URL to the operator.
