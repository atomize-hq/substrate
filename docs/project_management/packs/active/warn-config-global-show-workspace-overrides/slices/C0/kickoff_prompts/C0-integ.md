# Kickoff: C0-integ (integration final — cross-platform merge)

## Scope
- Merge platform-fix branches (if any) and finalize the slice with a clean, auditable cross-platform green state.
- Spec: `docs/project_management/packs/active/warn-config-global-show-workspace-overrides/slices/C0/C0-spec.md`
- Execution workflow standard: `docs/project_management/system/standards/triad/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`
- This task is responsible for merging back to the orchestration branch after all platforms are green (fast-forward when possible; otherwise a merge commit, preserving the orchestration branch’s Planning Pack files under the feature dir).

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/warn-config-global-show-workspace-overrides-c0-integ` on branch `warn-config-global-show-workspace-overrides-c0-integ` and that `.taskmeta.json` exists at the worktree root.
2. Read: `docs/project_management/packs/active/warn-config-global-show-workspace-overrides/plan.md`, `docs/project_management/packs/active/warn-config-global-show-workspace-overrides/tasks.json`, `docs/project_management/packs/active/warn-config-global-show-workspace-overrides/session_log.md`, spec, this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start FEATURE_DIR="docs/project_management/packs/active/warn-config-global-show-workspace-overrides" TASK_ID="C0-integ"`

## Requirements
- Merge the relevant integration branches for this slice:
  - The core integration branch (e.g., `*-integ-core`) and any platform-fix integration branches (`*-integ-linux|macos|windows|wsl`) that produced commits.
- Do not merge the orchestration branch into this worktree to “pick up task status/docs updates”; the finisher merges back while preserving the orchestration branch’s Planning Pack files.
- If the integration state has grown too large/unstable (many conflicts, large refactors, multiple unrelated changes), stop and ask the operator to split follow-up triads rather than forcing everything through a single final merge.
- Run:
  - `cargo fmt`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - relevant tests
  - `make integ-checks`

### CI audit (recommended; reduces redundant multi-OS CI)

Before dispatching compile parity, Feature Smoke, or CI Testing, run the advisory CI audit:
- CI Testing audit (default required platforms: `linux,macos,windows`):
  - `scripts/ci-audit/ci_audit.sh --ledger-path "docs/project_management/packs/active/warn-config-global-show-workspace-overrides/logs/<slice>/ci-audit/ledger.jsonl" --kind ci-testing --orch-branch "feat/warn-config-global-show-workspace-overrides"`
- Feature Smoke audit (required platforms from `docs/project_management/packs/active/warn-config-global-show-workspace-overrides/tasks.json` meta `behavior_platforms_required`):
  - `scripts/ci-audit/ci_audit.sh --ledger-path "docs/project_management/packs/active/warn-config-global-show-workspace-overrides/logs/<slice>/ci-audit/ledger.jsonl" --kind feature-smoke --orch-branch "feat/warn-config-global-show-workspace-overrides" --feature-dir "docs/project_management/packs/active/warn-config-global-show-workspace-overrides"`

Policy:
- If `DIFF_CLASS=docs_only` (docs/planning-only changes), CI/smoke may be skipped entirely.
- Otherwise, if `RECOMMEND=run`, dispatch normally.

After any CI dispatch completes, record evidence (recommended):
- `scripts/ci-audit/ci_audit_record.sh --ledger-path "docs/project_management/packs/active/warn-config-global-show-workspace-overrides/logs/<slice>/ci-audit/ledger.jsonl" --kind <ci-testing|feature-smoke> --orch-branch "feat/warn-config-global-show-workspace-overrides" --run-id "<run-id>" --tested-sha "<sha>" --feature-dir "docs/project_management/packs/active/warn-config-global-show-workspace-overrides"`
  - Use `<run-id>` from `RUN_ID=...` and `<sha>` from `HEAD=...` (both are printed by the dispatchers).
  - The ledger lives under `docs/project_management/packs/active/warn-config-global-show-workspace-overrides/logs/...` and is not intended to be committed.
- Local behavioral smoke preflight (fast fail before CI dispatch; required when possible):
  - If `docs/project_management/packs/active/warn-config-global-show-workspace-overrides/smoke/` exists and this machine matches a behavior platform, build `substrate`, add `target/debug` to `PATH`, and run the matching smoke script locally.
  - Linux: `cargo build --bin substrate && export PATH="$PWD/target/debug:$PATH" && bash "docs/project_management/packs/active/warn-config-global-show-workspace-overrides/smoke/linux-smoke.sh"`
  - macOS: `cargo build --bin substrate && export PATH="$PWD/target/debug:$PATH" && bash "docs/project_management/packs/active/warn-config-global-show-workspace-overrides/smoke/macos-smoke.sh"`
  - Windows: `cargo build --bin substrate; $env:Path=\"$pwd\\target\\debug;$env:Path\"; pwsh -File \"docs/project_management/packs/active/warn-config-global-show-workspace-overrides\\smoke\\windows-smoke.ps1\"`
- Ensure cross-platform compile parity is green for this exact `HEAD` (fast fail), unless ci-audit recommends skip:
  - `make ci-compile-parity CI_WORKFLOW_REF="feat/warn-config-global-show-workspace-overrides" CI_REMOTE=origin CI_CLEANUP=1`
- Run behavioral smoke via CI to confirm the merged result is green (P3-008):
  - Run from this final integration worktree (smoke validates current `HEAD` via a throwaway remote branch).
  - Dispatch behavioral smoke in a single run (preferred):
    - `make feature-smoke FEATURE_DIR="docs/project_management/packs/active/warn-config-global-show-workspace-overrides" PLATFORM=behavior SMOKE_SLICE_ID="<slice>" RUNNER_KIND=self-hosted WORKFLOW_REF="feat/warn-config-global-show-workspace-overrides" REMOTE=origin CLEANUP=1 RUN_INTEG_CHECKS=1`
      - `SMOKE_SLICE_ID` is optional; when provided, the workflow exports `SUBSTRATE_SMOKE_SLICE_ID` for slice-scoped smoke scripts.
      - If WSL coverage is required for this feature, add `RUN_WSL=1`.
- Complete the slice closeout gate report:
  - `docs/project_management/packs/active/warn-config-global-show-workspace-overrides/slices/C0/C0-closeout_report.md`

## End Checklist
1. Ensure all required platforms are green (include run ids/URLs).
2. From inside the worktree, run: `make triad-task-finish TASK_ID="C0-integ"`
3. Run CI Testing on this final integration commit before merging to `testing`:
   - First, run `scripts/ci-audit/ci_audit.sh ... --kind ci-testing` (above).
   - If `RECOMMEND=run`: from inside this worktree run `scripts/ci/dispatch_ci_testing.sh --workflow-ref "feat/warn-config-global-show-workspace-overrides" --remote origin --cleanup --mode full`
   - If `RECOMMEND=skip` (including `DIFF_CLASS=docs_only`): you may skip dispatch; include the ci-audit output lines in the handoff.
4. Hand off run ids/URLs (smoke + CI Testing) and closeout report completion to the operator (do not edit planning docs inside the worktree).
5. Do not delete the worktree (feature cleanup removes worktrees at feature end).

Naming note:
- The task id for the final aggregator is `<slice>-integ` (this prompt’s `C0-integ`). The helper command to start it is named `triad-task-start-integ-final`.
