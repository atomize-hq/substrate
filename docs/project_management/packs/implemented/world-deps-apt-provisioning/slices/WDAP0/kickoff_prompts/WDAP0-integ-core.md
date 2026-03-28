# Kickoff: WDAP0-integ-core (integration core)

## Scope
- Merge code + tests, resolve drift to spec, and make the slice green on the primary dev platform.
- Spec: `docs/project_management/packs/draft/world-deps-apt-provisioning/slices/WDAP0/WDAP0-spec.md`
- Execution workflow standard: `docs/project_management/system/standards/triad/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/world-deps-apt-provisioning-wdap0-integ-core` on branch `world-deps-apt-provisioning-wdap0-integ-core` and that `.taskmeta.json` exists at the worktree root.
2. Read: `docs/project_management/packs/draft/world-deps-apt-provisioning/plan.md`, `docs/project_management/packs/draft/world-deps-apt-provisioning/tasks.json`, `docs/project_management/packs/draft/world-deps-apt-provisioning/session_log.md`, spec, this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start FEATURE_DIR="docs/project_management/packs/draft/world-deps-apt-provisioning" TASK_ID="WDAP0-integ-core"`

## Requirements
- Reconcile code/tests to spec (spec wins).
- If the slice is too large to make green deterministically, stop and ask the operator to split the slice before continuing.
- Merge code+test branches into this worktree, then run integration gates (must be green before finishing this task):
  - `cargo fmt`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - relevant tests
  - `make integ-checks`

### CI checkpoints (required)

For cross-platform automation packs, cross-platform CI gates (compile parity + Feature Smoke) run only at checkpoint boundaries defined in:
- `docs/project_management/packs/draft/world-deps-apt-provisioning/pre-planning/ci_checkpoint_plan.md`

Rules:
- Do not dispatch cross-platform CI from this integration-core task.
- This slice is the end of the checkpoint group for CP1; finish this task, then run `CP1-ci-checkpoint` from the orchestration checkout.
- If checkpoint CI fails, start only the failing platform-fix tasks from the orchestration checkout using the checkpoint run id(s).

### Local behavioral smoke preflight (run when possible)

If `docs/project_management/packs/draft/world-deps-apt-provisioning/smoke/` exists and this machine matches a behavior platform for this feature, run the matching smoke script locally before finishing this task.

Determine behavior platforms:
- `jq -r '.meta.behavior_platforms_required // [] | join(\",\")' \"docs/project_management/packs/draft/world-deps-apt-provisioning/tasks.json\"`

Linux:
```bash
set -euo pipefail
cargo build --bin substrate
export PATH="$PWD/target/debug:$PATH"
bash "docs/project_management/packs/draft/world-deps-apt-provisioning/smoke/linux-smoke.sh"
```

macOS:
```bash
set -euo pipefail
cargo build --bin substrate
export PATH="$PWD/target/debug:$PATH"
bash "docs/project_management/packs/draft/world-deps-apt-provisioning/smoke/macos-smoke.sh"
```

Windows (PowerShell):
```powershell
$ErrorActionPreference = "Stop"
cargo build --bin substrate
$env:Path = "$pwd\\target\\debug;$env:Path"
pwsh -File "docs/project_management/packs/draft/world-deps-apt-provisioning\\smoke\\windows-smoke.ps1"
```

Expected:
- Exit `0`.

## End Checklist
1. Ensure merged state is committed and local integration gates are green:
   - From inside the worktree, run: `make triad-task-finish TASK_ID="WDAP0-integ-core"`
2. Update tasks/session_log on orchestration branch and hand off next-step instructions to the operator (do not edit planning docs inside the worktree).
3. Do not delete the worktree (feature cleanup removes worktrees at feature end).

