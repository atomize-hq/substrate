# Kickoff: C0-integ-core (integration core)

## Scope
- Merge code + tests, resolve drift to spec, and make the slice green on the primary dev platform.
- Spec: `docs/project_management/next/warn-config-global-show-workspace-overrides/C0-spec.md`
- Execution workflow standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/warn-config-global-show-workspace-overrides-c0-integ-core` on branch `warn-config-global-show-workspace-overrides-c0-integ-core` and that `.taskmeta.json` exists at the worktree root.
2. Read: `docs/project_management/next/warn-config-global-show-workspace-overrides/plan.md`, `docs/project_management/next/warn-config-global-show-workspace-overrides/tasks.json`, `docs/project_management/next/warn-config-global-show-workspace-overrides/session_log.md`, spec, this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start FEATURE_DIR="docs/project_management/next/warn-config-global-show-workspace-overrides" TASK_ID="C0-integ-core"`

## Requirements
- Reconcile code/tests to spec (spec wins).
- If the slice is too large to make green deterministically (multiple subsystems, many unrelated acceptance bullets), stop and ask the operator to split the slice before continuing.
- Merge code+test branches into this worktree, then run required integration gates (must be green before any CI smoke dispatch):
  - `cargo fmt`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - relevant tests
  - `make integ-checks`
- Optional (once per feature, when you want a clean-cache assurance): `make preflight`

### CI audit (recommended; reduces redundant multi-OS CI)

Before dispatching any CI from this worktree, run the advisory CI audit. This helps avoid re-running CI/smoke when:
- There are no code-affecting changes since the last green run that covered required platforms, or
- Changes are docs/planning-only (anything under `docs/`), in which case CI may be skipped entirely.

Commands:
- CI Testing audit (default required platforms: `linux,macos,windows`):
  - `scripts/ci-audit/ci_audit.sh --ledger-path "docs/project_management/next/warn-config-global-show-workspace-overrides/logs/<slice>/ci-audit/ledger.jsonl" --kind ci-testing --orch-branch "feat/warn-config-global-show-workspace-overrides"`
- Feature Smoke audit (required platforms from `docs/project_management/next/warn-config-global-show-workspace-overrides/tasks.json` meta `behavior_platforms_required`):
  - `scripts/ci-audit/ci_audit.sh --ledger-path "docs/project_management/next/warn-config-global-show-workspace-overrides/logs/<slice>/ci-audit/ledger.jsonl" --kind feature-smoke --orch-branch "feat/warn-config-global-show-workspace-overrides" --feature-dir "docs/project_management/next/warn-config-global-show-workspace-overrides"`

Interpretation:
- If `RECOMMEND=skip`, do not dispatch that CI gate; record the recommendation and the last-green run id/URL in your handoff.
- If `RECOMMEND=run`, dispatch normally.

After any CI dispatch completes, record evidence (recommended; improves correctness when dispatch uses throwaway branches):
- `scripts/ci-audit/ci_audit_record.sh --ledger-path "docs/project_management/next/warn-config-global-show-workspace-overrides/logs/<slice>/ci-audit/ledger.jsonl" --kind <ci-testing|feature-smoke> --orch-branch "feat/warn-config-global-show-workspace-overrides" --run-id "<run-id>" --tested-sha "<sha>" --feature-dir "docs/project_management/next/warn-config-global-show-workspace-overrides"`
  - Use `<run-id>` from `RUN_ID=...` and `<sha>` from `HEAD=...` (both are printed by the dispatchers).
  - The ledger lives under `docs/project_management/next/warn-config-global-show-workspace-overrides/logs/...` and is not intended to be committed.

### Local behavioral smoke preflight (required when possible; fast fail before CI dispatch)

If `docs/project_management/next/warn-config-global-show-workspace-overrides/smoke/` exists and this machine matches a behavior platform for this feature, run the matching smoke script **locally** before dispatching compile parity or Feature Smoke.

Determine behavior platforms:
- `jq -r '.meta.behavior_platforms_required // [] | join(\",\")' "docs/project_management/next/warn-config-global-show-workspace-overrides/tasks.json"`

Preflight steps (choose the block matching your current platform):

Linux:
```bash
set -euo pipefail
cargo build --bin substrate
export PATH="$PWD/target/debug:$PATH"
bash "docs/project_management/next/warn-config-global-show-workspace-overrides/smoke/linux-smoke.sh"
```

macOS:
```bash
set -euo pipefail
cargo build --bin substrate
export PATH="$PWD/target/debug:$PATH"
bash "docs/project_management/next/warn-config-global-show-workspace-overrides/smoke/macos-smoke.sh"
```

Windows (PowerShell):
```powershell
$ErrorActionPreference = "Stop"
cargo build --bin substrate
$env:Path = "$pwd\\target\\debug;$env:Path"
pwsh -File "docs/project_management/next/warn-config-global-show-workspace-overrides\\smoke\\windows-smoke.ps1"
```

Expected:
- Exit `0`.

### Cross-platform compile parity (CI dispatch; required before smoke)

Before dispatching Feature Smoke (especially when using `RUNNER_KIND=self-hosted`), run a fast cross-platform compile parity preflight on GitHub-hosted runners to catch macOS/Windows compilation breaks early:
- `make ci-compile-parity CI_WORKFLOW_REF="feat/warn-config-global-show-workspace-overrides" CI_REMOTE=origin CI_CLEANUP=1`

Notes:
- This dispatches CI Testing in `mode=compile-parity` (fmt --check, check --all-targets, clippy -D warnings) across Linux/macOS/Windows.
- If it fails, fix compile parity **in this integ-core worktree/branch** (cfg/platform guards), commit, and re-run until green; do not proceed to Feature Smoke until it is green.

### Cross-platform smoke (CI dispatch; validation-only)

Run CI smoke from this **integration-core worktree**, because the smoke dispatcher tests the current `HEAD` by creating a throwaway remote branch at that commit.

Important (P3-008):
- Smoke is required only for the feature’s **behavior platforms** (`tasks.json` meta: `behavior_platforms_required`).
- CI parity may be required for a broader set of platforms (`tasks.json` meta: `ci_parity_platforms_required` / legacy `platforms_required`).

Recommended dispatch (explicit params; leave `CLEANUP=1` on unless debugging temp branches):

1) Dispatch behavioral smoke in a single run (preferred):
   - `make feature-smoke FEATURE_DIR="docs/project_management/next/warn-config-global-show-workspace-overrides" PLATFORM=behavior SMOKE_SLICE_ID="<slice>" RUNNER_KIND=self-hosted WORKFLOW_REF="feat/warn-config-global-show-workspace-overrides" REMOTE=origin CLEANUP=1 RUN_INTEG_CHECKS=1`
     - `SMOKE_SLICE_ID` is optional; when provided, the workflow exports `SUBSTRATE_SMOKE_SLICE_ID` for slice-scoped smoke scripts.
   - If WSL coverage is required for this feature, add `RUN_WSL=1`.

What the dispatcher does (`scripts/ci/dispatch_feature_smoke.sh` via `make feature-smoke`):
- Creates/pushes a throwaway branch like `tmp/feature-smoke/<feature>/<platform>/<ts>` at current `HEAD`.
- Dispatches the workflow from `WORKFLOW_REF` while checking out that throwaway branch.
- Prints `DISPATCH_OK=0|1`, `RUN_ID=<id>`, `RUN_URL=<url>`, `SMOKE_PASSED_PLATFORMS=<csv>`, and `SMOKE_FAILED_PLATFORMS=<csv>` (plus `ERROR_KIND`/`ERROR_MESSAGE` on failures).
- Deletes the throwaway remote branch when `CLEANUP=1`.

Note:
- If smoke fails, `make feature-smoke` will still print `DISPATCH_OK=1` + `RUN_ID`/`RUN_URL`, but will exit non-zero (GNU make typically reports this as exit code 2). Do not re-run just to “get a run id”; use `RUN_URL` to inspect failures and start platform-fix tasks as needed.

If any platform smoke fails:
- Do not attempt platform-specific fixes in integ-core.
- Ask the operator to start only the failing platform-fix tasks **from the orchestration checkout** (not from a task worktree):
  - If you have a single smoke run that covers multiple platforms (typical `PLATFORM=behavior` case):
    - `make triad-task-start-platform-fixes-from-smoke FEATURE_DIR="docs/project_management/next/warn-config-global-show-workspace-overrides" SLICE_ID="<slice>" SMOKE_RUN_ID="<run-id>" LAUNCH_CODEX=1`
  - If you dispatched per-platform smoke (multiple run ids):
    - `make triad-task-start-platform-fixes FEATURE_DIR="docs/project_management/next/warn-config-global-show-workspace-overrides" SLICE_ID="<slice>" PLATFORMS="<csv>" LAUNCH_CODEX=1`
  - `<slice>` is the triad id prefix (e.g., `C0`, `C3`).
  - `<run-id>` is the `RUN_ID` from the failing smoke run (only used for `triad-task-start-platform-fixes-from-smoke`).
  - `<csv>` is the comma-separated list of platforms you need platform-fix tasks for (typically the failing behavior platforms, plus `wsl` if `wsl_task_mode="separate"`).

If behavioral smoke is green for all behavior platforms:
- Platform-fix tasks (if present in the pack) may still be required for CI-only failures (e.g., clippy warnings on macOS/Windows), so do not mark them no-op yet.
- The wrapper/final gate MUST run CI Testing; if CI Testing is green, then mark platform-fix tasks `completed` as no-ops to unblock the final aggregator’s `depends_on`:
  - `scripts/triad/mark_noop_platform_fixes_completed.sh --feature-dir "docs/project_management/next/warn-config-global-show-workspace-overrides" --slice-id "<slice>"` (optional: add `--from-smoke-run "<run-id>"` for logging)

Once all required platform-fix tasks are completed, ask the operator to start the final aggregator from the orchestration checkout:
- `make triad-task-start-integ-final FEATURE_DIR="docs/project_management/next/warn-config-global-show-workspace-overrides" SLICE_ID="<slice>" LAUNCH_CODEX=1`
  - Note: the final aggregator task id is `<slice>-integ` (the command name contains `integ-final`).

## End Checklist
1. Ensure your merged state is committed and local integration gates are green:
   - From inside the worktree, run: `make triad-task-finish TASK_ID="C0-integ-core"`
2. Checkpoint gate (`CP1-ci-checkpoint`):
   - Do not dispatch cross-platform smoke from this worktree.
   - From the orchestration checkout, start and complete `CP1-ci-checkpoint` using `docs/project_management/next/warn-config-global-show-workspace-overrides/kickoff_prompts/CP1-ci-checkpoint.md`.
   - If `ci_audit` recommends `RECOMMEND=skip` (including `DIFF_CLASS=docs_only`), do not dispatch; include the audit output lines in the handoff.
   - If dispatch runs, include the smoke run evidence in the handoff:
     - `RUN_ID=<id>`
     - `RUN_URL=<url>`
     - `SMOKE_PASSED_PLATFORMS=<csv>`
     - `SMOKE_FAILED_PLATFORMS=<csv>`
3. Hand off run ids/URLs and next-step instructions to the operator (do not edit planning docs inside the worktree).
4. Do not delete the worktree (feature cleanup removes worktrees at feature end).
