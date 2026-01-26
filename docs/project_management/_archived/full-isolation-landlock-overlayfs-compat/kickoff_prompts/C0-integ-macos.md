# Kickoff: C0-integ-macos (integration platform-fix — macos)

## Scope
- Ensure the slice is green for **macos** in the way required by the Planning Pack:
  - If `macos` is in `tasks.json` meta `behavior_platforms_required`: this is a **behavioral** platform-fix task (smoke required).
  - Otherwise: this is a **CI parity** platform-fix task (compile/test/lint parity required; smoke not required).
- This task is allowed to make production-code and/or test changes as needed to achieve the required platform green state, but must not edit planning docs inside the worktree.
- Spec: `docs/project_management/_archived/full-isolation-landlock-overlayfs-compat/C0-spec.md`
- Execution workflow standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`
- This task must not merge back to the orchestration branch; the final aggregator integration task performs the merge once all platforms are green.

## Start Checklist
Do not edit planning docs inside the worktree.

1. Run this task on a machine that matches the required platform: **macos**.
2. Verify you are in the task worktree `wt/full-isolation-landlock-overlayfs-compat-c0-integ-macos` on branch `full-isolation-landlock-overlayfs-compat-c0-integ-macos` and that `.taskmeta.json` exists at the worktree root.
   - Do all work (edits, builds/tests, commits, and `make triad-task-finish`) from inside this worktree.
3. Read: `docs/project_management/_archived/full-isolation-landlock-overlayfs-compat/plan.md`, `docs/project_management/_archived/full-isolation-landlock-overlayfs-compat/tasks.json`, `docs/project_management/_archived/full-isolation-landlock-overlayfs-compat/session_log.md`, spec, this prompt.
4. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start FEATURE_DIR="docs/project_management/_archived/full-isolation-landlock-overlayfs-compat" TASK_ID="C0-integ-macos" TASK_PLATFORM=macos`

## Requirements
- Before validating smoke or making fixes, merge the slice’s core integration branch into this worktree:
  - Find the core integration task for this slice in `tasks.json` (e.g., `C0-integ-core`) and merge its task branch into your current branch.
  - This ensures your platform-fix work starts from the merged code+test state that passed local integration gates.
- Keep fixes narrowly scoped to this platform’s failures. If you discover a broader cross-platform refactor is required, stop and ask the operator to split out an enabling slice instead of piling it into this platform-fix task.
- Run the platform-local Rust quality gates before finishing (CI Testing parity on this OS):
  - `cargo fmt`
  - `cargo clippy --workspace --all-targets -- -D warnings`
- If this is a behavioral platform (P3-008), run a local behavioral smoke preflight before dispatching CI smoke:
  - Build `substrate`, add `target/debug` to `PATH`, then run the matching smoke script locally.
  - Linux: `cargo build --bin substrate && export PATH="$PWD/target/debug:$PATH" && bash "docs/project_management/_archived/full-isolation-landlock-overlayfs-compat/smoke/linux-smoke.sh"`
  - macOS: `cargo build --bin substrate && export PATH="$PWD/target/debug:$PATH" && bash "docs/project_management/_archived/full-isolation-landlock-overlayfs-compat/smoke/macos-smoke.sh"`
  - Windows: `cargo build --bin substrate; $env:Path=\"$pwd\\target\\debug;$env:Path\"; pwsh -File \"docs/project_management/_archived/full-isolation-landlock-overlayfs-compat\\smoke\\windows-smoke.ps1\"`
  - Decide whether smoke is required for this platform (P3-008):
  - `jq -r '.meta.behavior_platforms_required // [] | join(",")' "docs/project_management/_archived/full-isolation-landlock-overlayfs-compat/tasks.json"`
  - If `macos` is in `behavior_platforms_required`, run Feature Smoke for this platform (repeat until green if you make fixes):
    - `make feature-smoke FEATURE_DIR="docs/project_management/_archived/full-isolation-landlock-overlayfs-compat" PLATFORM=macos SMOKE_SLICE_ID="<slice>" RUNNER_KIND=self-hosted WORKFLOW_REF="feat/full-isolation-landlock-overlayfs-compat" REMOTE=origin CLEANUP=1 RUN_INTEG_CHECKS=1`
      - `SMOKE_SLICE_ID` is optional; when provided, the workflow exports `SUBSTRATE_SMOKE_SLICE_ID` for slice-scoped smoke scripts.
    - If this is the Linux task and WSL coverage is required (see `tasks.json` meta: `wsl_required` + `wsl_task_mode`):
      - Bundled (default): dispatch with `RUN_WSL=1`:
        - `make feature-smoke FEATURE_DIR="docs/project_management/_archived/full-isolation-landlock-overlayfs-compat" PLATFORM=linux RUN_WSL=1 SMOKE_SLICE_ID="<slice>" RUNNER_KIND=self-hosted WORKFLOW_REF="feat/full-isolation-landlock-overlayfs-compat" REMOTE=origin CLEANUP=1 RUN_INTEG_CHECKS=1`
  - Otherwise, do **not** run Feature Smoke for this platform. Instead, treat this task as CI parity-only:
    - Prefer local parity fixes first (fmt/clippy/tests on this OS), then ask the operator to re-run CI parity gates from integ-core/final.
    - If you need an immediate cross-platform signal, run:
      - `make ci-compile-parity CI_WORKFLOW_REF="feat/full-isolation-landlock-overlayfs-compat" CI_REMOTE=origin CI_CLEANUP=1`
      - `scripts/ci/dispatch_ci_testing.sh --workflow-ref "feat/full-isolation-landlock-overlayfs-compat" --remote origin --cleanup --mode quick`
    - Fix compile/test/lint parity failures for this platform and re-run until green.

## End Checklist
1. Ensure the required gate is green for macos (smoke for behavior platforms; CI parity gates otherwise) and capture the run id/URL.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="C0-integ-macos"`
3. Hand off run id/URL and any platform-specific notes to the operator (do not edit planning docs inside the worktree).
4. Do not delete the worktree (feature cleanup removes worktrees at feature end).
