# Kickoff: WDD2-integ-windows (integration platform-fix — windows)

## Scope
- Ensure the slice is green for **windows** in the way required by the Planning Pack:
  - If `windows` is in `tasks.json` meta `behavior_platforms_required`: this is a **behavioral** platform-fix task (smoke required).
  - Otherwise: this is a **CI parity** platform-fix task (compile/test/lint parity required; smoke not required).
- This task is allowed to make production-code and/or test changes as needed to achieve the required platform green state, but must not edit planning docs inside the worktree.
- Spec: `docs/project_management/packs/draft/world-disabled-diagnostics/slices/WDD2/WDD2-spec.md`
- Execution workflow standard: `docs/project_management/system/standards/triad/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`
- This task must not merge back to the orchestration branch; the final aggregator integration task performs the merge once all platforms are green.

## Start Checklist
Do not edit planning docs inside the worktree.

1. Run this task on a machine that matches the required platform: **windows**.
2. Verify you are in the task worktree `wt/world-disabled-diagnostics-wdd2-integ-windows` on branch `world-disabled-diagnostics-wdd2-integ-windows` and that `.taskmeta.json` exists at the worktree root.
   - Do all work (edits, builds/tests, commits, and `make triad-task-finish`) from inside this worktree.
3. Read: `docs/project_management/packs/draft/world-disabled-diagnostics/plan.md`, `docs/project_management/packs/draft/world-disabled-diagnostics/tasks.json`, `docs/project_management/packs/draft/world-disabled-diagnostics/session_log.md`, spec, this prompt.
4. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start FEATURE_DIR="docs/project_management/packs/draft/world-disabled-diagnostics" TASK_ID="WDD2-integ-windows" TASK_PLATFORM=windows`

## Requirements
- Before validating smoke or making fixes, merge the slice’s core integration branch into this worktree:
  - Core integration task id: `WDD2-integ-core`
- Keep fixes narrowly scoped to this platform’s failures. If you discover a broader cross-platform refactor is required, stop and ask the operator to split out an enabling slice instead of piling it into this platform-fix task.
- Run the platform-local Rust quality gates before finishing (CI Testing parity on this OS):
  - `cargo fmt`
  - `cargo clippy --workspace --all-targets -- -D warnings`
- Use the advisory CI audit to avoid redundant dispatch:
  - Ledger path (recommended; not committed): `docs/project_management/packs/draft/world-disabled-diagnostics/logs/WDD2/ci-audit/ledger.jsonl`
  - Feature Smoke audit (for a single platform):
    - `scripts/ci-audit/ci_audit.sh --ledger-path "docs/project_management/packs/draft/world-disabled-diagnostics/logs/WDD2/ci-audit/ledger.jsonl" --kind feature-smoke --orch-branch "feat/world-disabled-diagnostics" --required-platforms windows`
  - CI Testing audit (if you are about to dispatch CI Testing):
    - `scripts/ci-audit/ci_audit.sh --ledger-path "docs/project_management/packs/draft/world-disabled-diagnostics/logs/WDD2/ci-audit/ledger.jsonl" --kind ci-testing --orch-branch "feat/world-disabled-diagnostics"`
- If this is a behavioral platform, run a local behavioral smoke preflight before dispatching CI smoke:
  - `cargo build --bin substrate; $env:Path=\"$pwd\\target\\debug;$env:Path\"; pwsh -File \"docs/project_management/packs/draft/world-disabled-diagnostics\\smoke\\windows-smoke.ps1\"`
- If `windows` is in `behavior_platforms_required`, run Feature Smoke for this platform (repeat until green if you make fixes):
  - `make feature-smoke FEATURE_DIR="docs/project_management/packs/draft/world-disabled-diagnostics" PLATFORM=windows SMOKE_SLICE_ID="WDD2" RUNNER_KIND=self-hosted WORKFLOW_REF="feat/world-disabled-diagnostics" REMOTE=origin CLEANUP=1 RUN_INTEG_CHECKS=1`

## End Checklist
1. Ensure the required gate is green for windows and capture the run id/URL.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="WDD2-integ-windows"`
3. Hand off run id/URL and any platform-specific notes to the operator (do not edit planning docs inside the worktree).
4. Do not delete the worktree (feature cleanup removes worktrees at feature end).
