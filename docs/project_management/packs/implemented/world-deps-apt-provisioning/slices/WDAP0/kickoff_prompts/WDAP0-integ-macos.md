# Kickoff: WDAP0-integ-macos (integration platform-fix — macos)

## Scope
- Ensure the slice is green for macos in the way required by the Planning Pack.
- Spec: `docs/project_management/packs/draft/world-deps-apt-provisioning/slices/WDAP0/WDAP0-spec.md`
- Execution workflow standard: `docs/project_management/system/standards/triad/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Run this task on a macos machine.
2. Verify you are in the task worktree `wt/world-deps-apt-provisioning-wdap0-integ-macos` on branch `world-deps-apt-provisioning-wdap0-integ-macos` and that `.taskmeta.json` exists at the worktree root.
3. Read: `docs/project_management/packs/draft/world-deps-apt-provisioning/plan.md`, `docs/project_management/packs/draft/world-deps-apt-provisioning/tasks.json`, `docs/project_management/packs/draft/world-deps-apt-provisioning/session_log.md`, spec, this prompt.
4. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start FEATURE_DIR="docs/project_management/packs/draft/world-deps-apt-provisioning" TASK_ID="WDAP0-integ-macos" TASK_PLATFORM=macos`

## Requirements
- Before validating smoke or making fixes, merge the slice’s core integration branch into this worktree:
  - Core integration task id: `WDAP0-integ-core`
- Run platform-local Rust quality gates before finishing:
  - `cargo fmt`
  - `cargo clippy --workspace --all-targets -- -D warnings`
- Advisory CI audit (ledger not committed):
  - Ledger path: `docs/project_management/packs/draft/world-deps-apt-provisioning/logs/WDAP0/ci-audit/ledger.jsonl`
  - Feature Smoke audit:
    - `scripts/ci-audit/ci_audit.sh --ledger-path "docs/project_management/packs/draft/world-deps-apt-provisioning/logs/WDAP0/ci-audit/ledger.jsonl" --kind feature-smoke --orch-branch "feat/world-deps-apt-provisioning" --required-platforms macos`
- Local smoke preflight:
  - `cargo build --bin substrate && export PATH=\"$PWD/target/debug:$PATH\" && bash \"docs/project_management/packs/draft/world-deps-apt-provisioning/smoke/macos-smoke.sh\"`
- Feature Smoke (repeat after fixes):
  - `make feature-smoke FEATURE_DIR="docs/project_management/packs/draft/world-deps-apt-provisioning" PLATFORM=macos SMOKE_SLICE_ID="WDAP0" RUNNER_KIND=self-hosted WORKFLOW_REF="feat/world-deps-apt-provisioning" REMOTE=origin CLEANUP=1 RUN_INTEG_CHECKS=1`

## End Checklist
1. Ensure the required gate is green for macos and capture the run id/URL.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="WDAP0-integ-macos"`
3. Hand off run id/URL and any platform-specific notes to the operator (do not edit planning docs inside the worktree).
4. Do not delete the worktree (feature cleanup removes worktrees at feature end).

