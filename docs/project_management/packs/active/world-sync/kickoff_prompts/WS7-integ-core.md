# Kickoff: WS7-integ-core (integration core)

## Scope
- Merge code + tests, resolve drift to spec, and make the slice green on the primary dev platform.
- Spec: `docs/project_management/packs/active/world-sync/WS7-spec.md`
- Execution workflow standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/world-sync-ws7-integ-core` on branch `world-sync-ws7-integ-core` and that `.taskmeta.json` exists at the worktree root.
2. Read: `docs/project_management/packs/active/world-sync/plan.md`, `docs/project_management/packs/active/world-sync/tasks.json`, `docs/project_management/packs/active/world-sync/session_log.md`, spec, this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start FEATURE_DIR="docs/project_management/packs/active/world-sync" TASK_ID="WS7-integ-core"`

## Requirements
- Reconcile code/tests to spec (spec wins).
- If the slice is too large to make green deterministically (multiple subsystems, many unrelated acceptance bullets), stop and ask the operator to split the slice before continuing.
- Merge code+test branches into this worktree, then run required integration gates (must be green before finishing this task):
  - `cargo fmt`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - relevant tests
  - `make integ-checks`
- Optional (once per feature, when you want a clean-cache assurance): `make preflight`

### CI checkpoints (required; cross-platform CI is not a per-slice step)

For cross-platform automation packs, cross-platform CI gates (compile parity + Feature Smoke) run only at the checkpoint boundaries defined in:
- `docs/project_management/packs/active/world-sync/ci_checkpoint_plan.md`

Rules:
- Do not dispatch cross-platform CI from this integration-core task.
- If this slice is the end of a checkpoint group, finish this task, then run the matching checkpoint task (e.g., `CP1-ci-checkpoint`) from the orchestration checkout.
- If checkpoint CI fails, start only the failing platform-fix tasks from the orchestration checkout using the checkpoint’s run id(s).

### Local behavioral smoke preflight (required when possible)

If `docs/project_management/packs/active/world-sync/smoke/` exists and this machine matches a behavior platform for this feature, run the matching smoke script locally before finishing this task.

Determine behavior platforms:
- `jq -r '.meta.behavior_platforms_required // [] | join(\",\")' "docs/project_management/packs/active/world-sync/tasks.json"`

Preflight steps (choose the block matching your current platform):

Linux:
```bash
set -euo pipefail
cargo build --bin substrate
export PATH="$PWD/target/debug:$PATH"
bash "docs/project_management/packs/active/world-sync/smoke/linux-smoke.sh"
```

macOS:
```bash
set -euo pipefail
cargo build --bin substrate
export PATH="$PWD/target/debug:$PATH"
bash "docs/project_management/packs/active/world-sync/smoke/macos-smoke.sh"
```

Expected:
- Exit `0`.

## End Checklist
1. Ensure your merged state is committed and local integration gates are green:
   - From inside the worktree, run: `make triad-task-finish TASK_ID="WS7-integ-core"`
2. Update tasks/session_log on orchestration branch and hand off next-step instructions to the operator (do not edit planning docs inside the worktree).
3. Do not delete the worktree (feature cleanup removes worktrees at feature end).
