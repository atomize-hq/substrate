# Kickoff: WDAP0-integ-windows (integration platform-fix — windows)

## Scope
- Ensure the slice is green for windows in the way required by the Planning Pack.
- Spec: `docs/project_management/packs/draft/world-deps-apt-provisioning/slices/WDAP0/WDAP0-spec.md`
- Execution workflow standard: `docs/project_management/system/standards/triad/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Run this task on a windows machine.
2. Verify you are in the task worktree `wt/world-deps-apt-provisioning-wdap0-integ-windows` on branch `world-deps-apt-provisioning-wdap0-integ-windows` and that `.taskmeta.json` exists at the worktree root.
3. Read: `docs/project_management/packs/draft/world-deps-apt-provisioning/plan.md`, `docs/project_management/packs/draft/world-deps-apt-provisioning/tasks.json`, `docs/project_management/packs/draft/world-deps-apt-provisioning/session_log.md`, spec, this prompt.
4. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start FEATURE_DIR="docs/project_management/packs/draft/world-deps-apt-provisioning" TASK_ID="WDAP0-integ-windows" TASK_PLATFORM=windows`

## Requirements
- Before validating smoke or making fixes, merge the slice’s core integration branch into this worktree:
  - Core integration task id: `WDAP0-integ-core`
- Run platform-local Rust quality gates before finishing:
  - `cargo fmt`
  - `cargo clippy --workspace --all-targets -- -D warnings`
- Advisory CI audit (ledger not committed):
  - Ledger path: `docs/project_management/packs/draft/world-deps-apt-provisioning/logs/WDAP0/ci-audit/ledger.jsonl`
  - Feature Smoke audit:
    - `scripts/ci-audit/ci_audit.sh --ledger-path "docs/project_management/packs/draft/world-deps-apt-provisioning/logs/WDAP0/ci-audit/ledger.jsonl" --kind feature-smoke --orch-branch "feat/world-deps-apt-provisioning" --required-platforms windows`
- Local smoke preflight (PowerShell):
  - `cargo build --bin substrate; $env:Path=\"$pwd\\target\\debug;$env:Path\"; pwsh -File \"docs/project_management/packs/draft/world-deps-apt-provisioning\\smoke\\windows-smoke.ps1\"`
- Feature Smoke (repeat after fixes):
  - `make feature-smoke FEATURE_DIR="docs/project_management/packs/draft/world-deps-apt-provisioning" PLATFORM=windows SMOKE_SLICE_ID="WDAP0" RUNNER_KIND=self-hosted WORKFLOW_REF="feat/world-deps-apt-provisioning" REMOTE=origin CLEANUP=1 RUN_INTEG_CHECKS=1`

## End Checklist
1. Ensure the required gate is green for windows and capture the run id/URL.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="WDAP0-integ-windows"`
3. Hand off run id/URL and any platform-specific notes to the operator (do not edit planning docs inside the worktree).
4. Do not delete the worktree (feature cleanup removes worktrees at feature end).

