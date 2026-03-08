# Kickoff: BEDPM3-integ-linux (integration platform-fix — linux)

## Scope
- Resolve Linux behavior-smoke failures for BEDPM3 after `CP1-ci-checkpoint`.
- Spec: `docs/project_management/packs/draft/best-effort-distro-package-manager/slices/BEDPM3/BEDPM3-spec.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Run this task on a Linux host.
2. Verify you are in `wt/best-effort-distro-package-manager-bedpm3-integ-linux` on branch `best-effort-distro-package-manager-bedpm3-integ-linux` and that `.taskmeta.json` exists.
3. Read `plan.md`, `tasks.json`, `session_log.md`, the BEDPM3 spec, and this prompt.
4. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run `make triad-task-start FEATURE_DIR="docs/project_management/packs/draft/best-effort-distro-package-manager" TASK_ID="BEDPM3-integ-linux" TASK_PLATFORM=linux`.

## Requirements
- Merge `BEDPM3-integ-core` into this worktree before fixing anything.
- Run `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, and `bash "docs/project_management/packs/draft/best-effort-distro-package-manager/smoke/linux-smoke.sh"`.
- Use CI audit before dispatch:
  - `scripts/ci-audit/ci_audit.sh --ledger-path "docs/project_management/packs/draft/best-effort-distro-package-manager/logs/BEDPM3/ci-audit/ledger.jsonl" --kind feature-smoke --orch-branch "feat/best-effort-distro-package-manager" --required-platforms linux`
- When dispatch is required, run:
  - `make feature-smoke FEATURE_DIR="docs/project_management/packs/draft/best-effort-distro-package-manager" PLATFORM=linux SMOKE_SLICE_ID="BEDPM3" RUNNER_KIND=self-hosted WORKFLOW_REF="feat/best-effort-distro-package-manager" REMOTE=origin CLEANUP=1 RUN_INTEG_CHECKS=1`

## End Checklist
1. Capture the commands, the smoke result, and the run id or ci_audit skip evidence.
2. From inside the worktree, run `make triad-task-finish TASK_ID="BEDPM3-integ-linux"`.
3. Hand off results to the operator. Do not edit planning docs inside the worktree.
