# Triad Integration Wrapper Prompt (Cross-Platform Integ Orchestration)

Use this when you want a single orchestration run to:
- start `<SLICE>-integ-core`, run it with Codex enabled, and capture its final message,
- run cross-platform compile parity and Feature Smoke for the integ-core commit **only when `<SLICE>` is a CI checkpoint boundary** (see `ci_checkpoint_plan.md`),
- start only the failing `<SLICE>-integ-<platform>` tasks (optionally with Codex enabled) and capture their final messages,
- start `<SLICE>-integ` (final aggregator) and capture its final message,
- report exit codes + artifact paths for all runs.

Notes:
- Run this from the **orchestration checkout** (repo root), not inside any task worktree.
- This wrapper assumes an automation-enabled Planning Pack (`tasks.json` meta: `schema_version >= 3` and `meta.automation.enabled=true`).
- This wrapper is for **checkpoint boundary slices only**. If `<SLICE>` is not the last slice in its checkpoint group, run integ-core and stop (do not dispatch cross-platform CI from this wrapper).
- If you want a single prompt that covers both normal slices and checkpoint-boundary slices, use:
  - `docs/project_management/system/prompts/triad_wrappers/triad_unified_wrapper_checkpoint_aware.md`
- Schema v4+ note (boundary-only platform-fix): only slices listed in `tasks.json` `meta.checkpoint_boundaries` will have `*-integ-core` / `*-integ-<platform>` tasks. If a slice does not have `${SLICE_ID}-integ-core` in `tasks.json`, this wrapper is not applicable.
- This wrapper does not assume you are editing `tasks.json`/`session_log.md`, but **`make triad-task-start-integ-final` requires `depends_on` tasks are `status=completed`**. If CI smoke is green and platform-fix tasks were no-ops, the operator must still mark them `completed` on the orchestration branch to unblock the final aggregator.
- Naming note: the final aggregator task id is `${SLICE_ID}-integ` (started via `make triad-task-start-integ-final ...`), not `${SLICE_ID}-integ-final`.
- `CODEX_LAST_MESSAGE_PATH` is the final Codex message for that task run only; if additional manual work happens after Codex exits, record it separately (closeout report/session log), not by trusting the stale Codex summary.
- Dispatch runs from the orchestration/task ref (not `main`/`testing`). GitHub only allows `workflow_dispatch` for a workflow file if that workflow is registered on the default branch; when workflows change, land workflow-file-only changes on `main` to register them before relying on dispatch.

## Copy/Paste Prompt Template

Fill in only:
- `FEATURE_DIR` (planning pack dir)
- `SLICE_ID` (slice prefix; e.g. `PCP0`)

```text
You are the “Triad integration wrapper” orchestration agent.

## Inputs
FEATURE_DIR="<SET_ME>"   # e.g. docs/project_management/_archived/policy_and_config_precedence
SLICE_ID="<SET_ME>"      # e.g. PCP0

## CI Audit (recommended)
LEDGER_PATH="$FEATURE_DIR/logs/$SLICE_ID/ci-audit/ledger.jsonl"

## Non-negotiables
- Run from the orchestration checkout (repo root), not from a task worktree.
- Prefer triad automation scripts (`make triad-task-start*`) and capture Codex artifacts from `<feature_dir>/logs/<slice>/<task-kind>/...` (survives `cargo clean`).
- Do not edit planning docs inside any task worktree.

## What to do
1) Start and run integ-core with Codex enabled:
   `make triad-task-start FEATURE_DIR="$FEATURE_DIR" TASK_ID="${SLICE_ID}-integ-core" LAUNCH_CODEX=1`
   Parse stdout key/value lines and collect:
   - `WORKTREE`, `TASK_BRANCH`, `ORCH_BRANCH`
   - `CODEX_EXIT`, `CODEX_LAST_MESSAGE_PATH`, `CODEX_EVENTS_PATH`, `CODEX_STDERR_PATH`
   Read and include the full contents of `CODEX_LAST_MESSAGE_PATH`.
   Store `WORKTREE` as `INTEG_CORE_WORKTREE`.
   Also capture the commit SHA after the run:
   - `INTEG_CORE_HEAD_SHA=$(git -C "$INTEG_CORE_WORKTREE" rev-parse HEAD)`
   - Set the initial checkpoint validation SHA:
     - `CHECKOUT_SHA="$INTEG_CORE_HEAD_SHA"`
   - If you land any fixes on integ-core or a platform-fix branch, update `CHECKOUT_SHA` to the commit you are validating before re-dispatching checkpoint gates.

1.5) Confirm local behavioral smoke preflight was run (fast fail; when possible):
   - The integ-core kickoff prompt requires a local behavioral smoke preflight when `"$FEATURE_DIR/smoke/"` exists and the platform matches.
   - Confirm the integ-core agent explicitly reported the local smoke preflight as:
     - executed, and
     - green (exit `0`).
   - Only if the preflight is missing or ambiguous, re-run it locally from the integ-core worktree before any CI dispatch.
   - Determine behavior platforms from the pack:
     - `jq -r '.meta.behavior_platforms_required // [] | join(",")' "$FEATURE_DIR/tasks.json"`
   - Local smoke preflight (choose the block matching your current platform):
     - Linux:
       - `cd "$INTEG_CORE_WORKTREE" && cargo build --bin substrate && export PATH="$PWD/target/debug:$PATH" && bash "$FEATURE_DIR/smoke/linux-smoke.sh"`
     - macOS:
       - `cd "$INTEG_CORE_WORKTREE" && cargo build --bin substrate && export PATH="$PWD/target/debug:$PATH" && bash "$FEATURE_DIR/smoke/macos-smoke.sh"`
     - Windows (PowerShell):
       - `cd "$INTEG_CORE_WORKTREE"; cargo build --bin substrate; $env:Path=\"$pwd\\target\\debug;$env:Path\"; pwsh -File \"$FEATURE_DIR\\smoke\\windows-smoke.ps1\"`
   - If local smoke preflight is re-run and fails:
     - Treat this as **blocking**.
     - Fix behavior drift in the **integ-core** branch/worktree, commit, and re-run local smoke preflight until it is green.
     - Only then proceed to cross-platform CI dispatch (compile parity and Feature Smoke).

2) Confirm `<SLICE_ID>` is a CI checkpoint boundary for this feature:
   - Read: `$FEATURE_DIR/ci_checkpoint_plan.md`
   - Find the checkpoint that contains `$SLICE_ID`.
   - If `$SLICE_ID` is not the **last** slice listed in that checkpoint’s `slices[]`, stop after step (1.5) and return a summary. Do not dispatch cross-platform CI from this wrapper.

3) If the checkpoint plan requires compile parity, run cross-platform compile parity for the current candidate commit (fast fail; do this before relying on smoke):
   - Run from the orchestration checkout (validates `CHECKOUT_SHA` via checkout_ref):
     - `make ci-compile-parity CI_WORKFLOW_REF="$ORCH_BRANCH" CI_REMOTE=origin CI_CLEANUP=1 CI_CHECKOUT_REF="$CHECKOUT_SHA"`
   - Parse and report the dispatcher stdout contract:
     - `RUN_ID`, `RUN_URL`, `CONCLUSION`, `CI_FAILED_OSES`, `CI_FAILED_JOBS`
   - Before dispatching, run advisory audit (skip allowed for docs/planning-only changes):
     - `scripts/ci-audit/ci_audit.sh --ledger-path "$LEDGER_PATH" --kind ci-testing --orch-branch "$ORCH_BRANCH"`
     - If `RECOMMEND=skip`, do not dispatch compile parity; treat this gate as satisfied and report the ci-audit output in the wrapper summary (include `LAST_GREEN_RUN_ID/LAST_GREEN_RUN_URL` when available).
   - If you dispatch and it succeeds, record evidence:
     - `scripts/ci-audit/ci_audit_record.sh --ledger-path "$LEDGER_PATH" --kind ci-testing --orch-branch "$ORCH_BRANCH" --run-id "$RUN_ID" --tested-sha "$CHECKOUT_SHA"`
   - If compile parity is green: continue to step (4).
   - If compile parity fails:
     - Treat this as **blocking**: do not proceed to Feature Smoke dispatch or platform-fix selection yet.
     - Use the run logs (`RUN_URL` / `gh run view "$RUN_ID" --log-failed`) to identify the compile errors.
     - Fix compile parity on the **integ-core branch/worktree** (these are typically missing `#[cfg]` guards / platform gates, or accidental platform-specific APIs leaking into cross-platform crates).
     - Commit the fix(es) on the integ-core task branch, update:
       - `CHECKOUT_SHA=$(git -C "$INTEG_CORE_WORKTREE" rev-parse HEAD)`
     - Re-run this compile parity step until green, then continue to step (4).

4) If the checkpoint plan requires Feature Smoke, dispatch it for the current candidate commit (only after compile parity is green):
   - Smoke is required only for the feature’s **behavior platforms** (P3-008; see `tasks.json` meta: `behavior_platforms_required`).
   - Dispatch exactly one run using `PLATFORM=behavior`:
     - `make feature-smoke FEATURE_DIR="$FEATURE_DIR" PLATFORM=behavior SMOKE_SLICE_ID="$SLICE_ID" SMOKE_CHECKOUT_REF="$CHECKOUT_SHA" RUNNER_KIND=self-hosted WORKFLOW_REF="$ORCH_BRANCH" REMOTE=origin CLEANUP=1 RUN_INTEG_CHECKS=0`
   - Before requiring smoke keys, run advisory audit for Feature Smoke:
     - `scripts/ci-audit/ci_audit.sh --ledger-path "$LEDGER_PATH" --kind feature-smoke --orch-branch "$ORCH_BRANCH" --feature-dir "$FEATURE_DIR"`
     - If `RECOMMEND=skip` (including `DIFF_CLASS=docs_only`), do not dispatch smoke; record the ci-audit output in the wrapper summary and skip steps (5)–(6).
   - Parse and report the smoke dispatcher stdout contract:
     - `RUN_ID`, `RUN_URL`, `CONCLUSION`, `SMOKE_PASSED_PLATFORMS`, `SMOKE_FAILED_PLATFORMS`
   - If you dispatch and it succeeds, record evidence:
     - `scripts/ci-audit/ci_audit_record.sh --ledger-path "$LEDGER_PATH" --kind feature-smoke --orch-branch "$ORCH_BRANCH" --run-id "$RUN_ID" --tested-sha "$CHECKOUT_SHA" --feature-dir "$FEATURE_DIR"`

5) If smoke failed, start only failing platform-fix tasks with Codex enabled (parallel):
   - If you have a single smoke run id that includes all failing platforms (typical `PLATFORM=behavior` case), use:
      `make triad-task-start-platform-fixes-from-smoke FEATURE_DIR="$FEATURE_DIR" SLICE_ID="$SLICE_ID" SMOKE_RUN_ID="$RUN_ID" LAUNCH_CODEX=1`
   For each selected platform block, collect:
   - `PLATFORM`, `TASK_ID`, `WORKTREE`, `TASK_BRANCH`, `CODEX_EXIT`
   - `CODEX_LAST_MESSAGE_PATH`, `CODEX_EVENTS_PATH`, `CODEX_STDERR_PATH`
   Read and include the full contents of each `CODEX_LAST_MESSAGE_PATH`.
   Also capture each platform-fix commit SHA after the run:
   - `HEAD_SHA=$(git -C "$WORKTREE" rev-parse HEAD)`

6) If smoke succeeded (no platform fixes needed for smoke):
   - Do not start any platform-fix tasks yet; they may still be needed for CI-only failures on macOS/Windows.

7) Decide the platform-fix path:
   - If compile parity is green and `SMOKE_FAILED_PLATFORMS` is empty, then no platform-fix worktrees/branches are expected for this slice.
   - In that case, mark platform-fix tasks as `completed` no-ops to unblock the final aggregator’s `depends_on`:
   - `scripts/triad/mark_noop_platform_fixes_completed.sh --feature-dir "$FEATURE_DIR" --slice-id "$SLICE_ID"`
     - If smoke was dispatched and you have a run id, you may add: `--from-smoke-run "<run-id>"`
   - Otherwise (smoke failed or compile parity failed), do not mark no-ops; platform-fix tasks must run and reach green.

8) After platform fixes are complete (or marked no-op), start the final aggregator:
   - Confirm `${SLICE_ID}-integ-core` and all required `${SLICE_ID}-integ-<platform>` tasks are `status=completed` in `$FEATURE_DIR/tasks.json` (the starter enforces this).
   - Run:
     `make triad-task-start-integ-final FEATURE_DIR="$FEATURE_DIR" SLICE_ID="$SLICE_ID" LAUNCH_CODEX=1`
   Parse stdout key/value lines and collect:
   - `FINAL_TASK_ID`, `WORKTREE`, `TASK_BRANCH`, `ORCH_BRANCH`
   - `CODEX_EXIT`, `CODEX_LAST_MESSAGE_PATH`, `CODEX_EVENTS_PATH`, `CODEX_STDERR_PATH`
   Read and include the full contents of `CODEX_LAST_MESSAGE_PATH`.
   Store `WORKTREE` as `FINAL_WORKTREE`.
   Also capture the commit SHA after the run:
   - `FINAL_HEAD_SHA=$(git -C "$FINAL_WORKTREE" rev-parse HEAD)`

9) If the checkpoint plan requires CI Testing, dispatch it from the orchestration checkout using checkout_ref (mode `quick` or `full` per the plan):
   - `scripts/ci/dispatch_ci_testing.sh --workflow-ref "$ORCH_BRANCH" --remote origin --cleanup --mode <quick|full> --checkout-ref "$CHECKOUT_SHA"`

## Output to operator
Return a concise summary that includes:
- integ-core: `CODEX_EXIT` + `INTEG_CORE_HEAD_SHA` + final message + artifact paths
- compile parity: `RUN_ID` + `RUN_URL` + `CONCLUSION` + `CI_FAILED_OSES` + `CI_FAILED_JOBS`
- smoke: `RUN_ID` + `RUN_URL` + `SMOKE_PASSED_PLATFORMS` + `SMOKE_FAILED_PLATFORMS`
- each platform-fix task started: `CODEX_EXIT` + final message + artifact paths
- final aggregator: `CODEX_EXIT` + `FINAL_HEAD_SHA` + final message + artifact paths

Do NOT inline events/stderr logs; only paths and short summaries.
If any expected file is missing, report that clearly (do not guess).
```
