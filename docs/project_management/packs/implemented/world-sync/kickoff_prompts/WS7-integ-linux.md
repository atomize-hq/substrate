# Kickoff: WS7-integ-linux (integration platform-fix — linux)

## Scope

- Ensure the slice is green for **linux** in the way required by the Planning Pack:
  - If `linux` is in `tasks.json` meta `behavior_platforms_required`: this is a **behavioral** platform-fix task (smoke required).
  - Otherwise: this is a **CI parity** platform-fix task (compile/test/lint parity required; smoke not required).
- This task is allowed to make production-code and/or test changes as needed to achieve the required platform green state, but must not edit planning docs inside the worktree.
- Spec: `docs/project_management/packs/active/world-sync/WS7-spec.md`
- Execution workflow standard: `docs/project_management/system/standards/triad/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`
- This task must not merge back to the orchestration branch; the final aggregator integration task performs the merge once all platforms are green.

## Start Checklist

Do not edit planning docs inside the worktree.

1. Run this task on a machine that matches the required platform: **linux**.
2. Verify you are in the task worktree `wt/world-sync-ws7-integ-linux` on branch `world-sync-ws7-integ-linux` and that `.taskmeta.json` exists at the worktree root.
   - Do all work (edits, builds/tests, commits, and `make triad-task-finish`) from inside this worktree.
3. Read: `docs/project_management/packs/active/world-sync/plan.md`, `docs/project_management/packs/active/world-sync/tasks.json`, `docs/project_management/packs/active/world-sync/session_log.md`, spec, this prompt.
4. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start FEATURE_DIR="docs/project_management/packs/active/world-sync" TASK_ID="WS7-integ-linux" TASK_PLATFORM=linux`

## Requirements

- Before validating smoke or making fixes, merge the slice’s core integration branch into this worktree:
  - Find the core integration task for this slice in `tasks.json` (e.g., `WS7-integ-core`) and merge its task branch into your current branch.
  - This ensures your platform-fix work starts from the merged code+test state that passed local integration gates.
- Keep fixes narrowly scoped to this platform’s failures. If you discover a broader cross-platform refactor is required, stop and ask the operator to split out an enabling slice instead of piling it into this platform-fix task.
- Run the platform-local Rust quality gates before finishing (CI Testing parity on this OS):
  - `cargo fmt`
  - `cargo clippy --workspace --all-targets -- -D warnings`
- Use the advisory CI audit to avoid redundant dispatch:
  - Ledger path (recommended; not committed): `docs/project_management/packs/active/world-sync/logs/WS7/ci-audit/ledger.jsonl`
  - Feature Smoke audit (for a single platform):
    - `scripts/ci-audit/ci_audit.sh --ledger-path "docs/project_management/packs/active/world-sync/logs/WS7/ci-audit/ledger.jsonl" --kind feature-smoke --orch-branch "feat/world-sync" --required-platforms linux`
  - CI Testing audit (if you are about to dispatch CI Testing):
    - `scripts/ci-audit/ci_audit.sh --ledger-path "docs/project_management/packs/active/world-sync/logs/WS7/ci-audit/ledger.jsonl" --kind ci-testing --orch-branch "feat/world-sync"`
- If this is a behavioral platform (P3-008), run a local behavioral smoke preflight before dispatching CI smoke:
  - Build `substrate`, add `target/debug` to `PATH`, then run the matching smoke script locally.
  - Linux: `cargo build --bin substrate && export PATH="$PWD/target/debug:$PATH" && bash "docs/project_management/packs/active/world-sync/smoke/linux-smoke.sh"`
  - macOS: `cargo build --bin substrate && export PATH="$PWD/target/debug:$PATH" && bash "docs/project_management/packs/active/world-sync/smoke/macos-smoke.sh"`
  - Decide whether smoke is required for this platform (P3-008):
  - `jq -r '.meta.behavior_platforms_required // [] | join(",")' "docs/project_management/packs/active/world-sync/tasks.json"`
  - If `linux` is in `behavior_platforms_required`, run Feature Smoke for this platform (repeat until green if you make fixes):
    - `make feature-smoke FEATURE_DIR="docs/project_management/packs/active/world-sync" PLATFORM=linux SMOKE_SLICE_ID="WS7" SMOKE_CHECKOUT_REF="$(git rev-parse HEAD)" RUNNER_KIND=self-hosted WORKFLOW_REF="feat/world-sync" REMOTE=origin CLEANUP=1 RUN_INTEG_CHECKS=1`
      - `SMOKE_SLICE_ID` is optional; when provided, the workflow exports `SUBSTRATE_SMOKE_SLICE_ID` for slice-scoped smoke scripts.
    - After dispatch completes, record evidence (recommended):
      - Set `RUN_ID` to the numeric id in the Actions run URL (example: `.../actions/runs/123456789`).
      - `scripts/ci-audit/ci_audit_record.sh --ledger-path "docs/project_management/packs/active/world-sync/logs/WS7/ci-audit/ledger.jsonl" --kind feature-smoke --orch-branch "feat/world-sync" --run-id "$RUN_ID" --tested-sha "$(git rev-parse HEAD)" --required-platforms linux`
  - Otherwise, do **not** run Feature Smoke for this platform. Instead, treat this task as CI parity-only:
    - Prefer local parity fixes first (fmt/clippy/tests on this OS), then ask the operator to run the next CI checkpoint task (e.g., `CP1-ci-checkpoint`) to validate cross-platform parity on the orchestration branch.
    - If you need an immediate cross-platform signal, run:
      - `make ci-compile-parity CI_WORKFLOW_REF="feat/world-sync" CI_REMOTE=origin CI_CLEANUP=1`
      - `scripts/ci/dispatch_ci_testing.sh --workflow-ref "feat/world-sync" --remote origin --cleanup --mode quick`
    - After dispatch completes, record evidence (recommended):
      - Set `RUN_ID` to the numeric id in the Actions run URL (example: `.../actions/runs/123456789`).
      - `scripts/ci-audit/ci_audit_record.sh --ledger-path "docs/project_management/packs/active/world-sync/logs/WS7/ci-audit/ledger.jsonl" --kind ci-testing --orch-branch "feat/world-sync" --run-id "$RUN_ID" --tested-sha "$(git rev-parse HEAD)"`
    - Fix compile/test/lint parity failures for this platform and re-run until green.

## End Checklist

1. Ensure the required gate is green for linux (smoke for behavior platforms; CI parity gates otherwise) and capture the run id/URL.
   - If you intentionally skipped dispatch due to ci-audit (`RECOMMEND=skip`), include the ci-audit output lines + last-green run evidence in your handoff.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="WS7-integ-linux"`
3. Hand off run id/URL and any platform-specific notes to the operator (do not edit planning docs inside the worktree).
4. Do not delete the worktree (feature cleanup removes worktrees at feature end).
