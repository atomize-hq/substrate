# Kickoff: C0-integ (integration final — cross-platform merge)

## Scope
- Merge platform-fix branches (if any) and finalize the slice with a clean, auditable cross-platform green state.
- Spec: `docs/project_management/next/tmp-make-scaffold/C0-spec.md`
- This task is responsible for the fast-forward merge back to the orchestration branch after all platforms are green.

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/tmp-make-scaffold-c0-integ` on branch `tmp-make-scaffold-c0-integ` and that `.taskmeta.json` exists at the worktree root.
2. Read: plan.md, tasks.json, session_log.md, spec, this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start FEATURE_DIR="docs/project_management/next/tmp-make-scaffold" TASK_ID="C0-integ"`

## Requirements
- Merge the relevant integration branches for this slice:
  - `tmp-make-scaffold-c0-integ-core` and any platform-fix branches (`tmp-make-scaffold-c0-integ-linux|macos|windows`) that produced commits.
- Run:
  - `cargo fmt`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - relevant tests
  - `make integ-checks`
- Run cross-platform smoke via CI to confirm the merged result is green:
  - Run from this worktree (smoke validates current `HEAD` via a throwaway remote branch):
  - `make feature-smoke FEATURE_DIR="docs/project_management/next/tmp-make-scaffold" PLATFORM=all RUN_WSL=1 RUNNER_KIND=self-hosted WORKFLOW_REF="feat/tmp-make-scaffold" REMOTE=origin CLEANUP=1`
- Complete the slice closeout gate report:
  - `docs/project_management/next/tmp-make-scaffold/C0-closeout_report.md`

## End Checklist
1. Ensure all required platforms are green (include run ids/URLs).
2. From inside the worktree, run: `make triad-task-finish TASK_ID="C0-integ"`
3. Run CI Testing on this final integration commit before merging to `testing` (even if CI Testing was run earlier on integ-core):
   - From inside this worktree: `scripts/ci/dispatch_ci_testing.sh --workflow-ref feat/tmp-make-scaffold --remote origin --cleanup`
   - You may skip this only if the operator already has a CI Testing run for this exact `HEAD` commit SHA.
4. Hand off run ids/URLs (smoke + CI Testing) and closeout report completion to the operator (do not edit planning docs inside the worktree).
5. Do not delete the worktree (feature cleanup removes worktrees at feature end).
