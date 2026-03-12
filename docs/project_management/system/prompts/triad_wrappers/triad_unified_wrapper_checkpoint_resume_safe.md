# Triad Unified Wrapper Prompt (Checkpoint-Aware, Resume-Safe)

Use this after `F0-exec-preflight` is completed for a feature when you want a single orchestration run that can either start fresh or resume from an interrupted state and:
- runs `<SLICE>-code` + `<SLICE>-test` in parallel (with Codex enabled),
- runs the slice integration task wired via `tasks.json` (`<SLICE>-code.integration_task`) (with Codex enabled),
- and, if `<SLICE>` is a schema v4+ CI checkpoint boundary slice, resumes and completes the entire checkpointed flow:
  - run or resume the planned `CPk-ci-checkpoint` ops task,
  - run or resume only the necessary `<SLICE>-integ-<platform>` platform-fix tasks (with Codex enabled) until CI parity is green,
  - then run or resume `<SLICE>-integ` final aggregation (with Codex enabled).

Recovery model:
- The source of truth is current `tasks.json` task status plus existing artifacts in `session_log.md` and `logs/<slice>/...`.
- Never re-run a task whose `status` is already `completed`.
- Prefer reusing the existing worktree/logs for `in_progress` tasks instead of creating parallel worktrees.
- `make triad-task-start-complete ...` is already resume-safe for the `code`/`test`/wired integration portion; use that instead of rebuilding triad bookkeeping manually.
- `START_AT` is an optional override. It may move the resume pointer forward, but it must never force re-running an already-completed stage.

Non-negotiables:
- Run from the orchestration checkout (repo root on the orchestration branch), not from any task worktree.
- Do not edit planning docs inside any task worktree.
- Do not dispatch cross-platform CI from per-slice integration worktrees; dispatch only from the checkpoint ops task.

## Copy/Paste Prompt Template

Fill in only:
- `FEATURE_DIR` (planning pack dir)
- `SLICE_ID` (slice prefix)
- optional `START_AT` (`auto|triad|checkpoint|platform-fixes|final-aggregator`)

```text
You are the “Triad unified wrapper” orchestration agent, resume-safe variant.

## Inputs
FEATURE_DIR="<SET_ME>"   # e.g. docs/project_management/packs/active/world_process_exec_tracing_parity
SLICE_ID="<SET_ME>"      # e.g. WPEP0
START_AT="auto"          # optional: auto|triad|checkpoint|platform-fixes|final-aggregator

## Non-negotiables
- Run from the orchestration checkout (repo root), not from any task worktree.
- Dispatch with Codex enabled when starting or re-launching task agents: use `LAUNCH_CODEX=1`.
- Do not edit planning docs inside any task worktree.
- Never regress completed task status back to `pending` or `in_progress`.
- Never discard existing run evidence; inspect existing logs/session entries before re-dispatching.

## Step 0: Preconditions
- Ensure orchestration checkout is clean:
  - `make triad-orch-ensure FEATURE_DIR="$FEATURE_DIR"`
  - `git status --porcelain=v1` must be empty.

## Step 1: Discover current state before doing anything
- Compute canonical task ids:
  - `CODE_TASK_ID="${SLICE_ID}-code"`
  - `TEST_TASK_ID="${SLICE_ID}-test"`
  - `INTEG_TASK_ID="$(jq -r --arg id "$CODE_TASK_ID" '.tasks[] | select(.id==$id) | .integration_task // empty' "$FEATURE_DIR/tasks.json")"`
  - If `INTEG_TASK_ID` is empty, stop and report a planning pack mismatch.
- Discover boundary metadata:
  - `BOUNDARY_SLICE=1` only if `tasks.json` `meta.checkpoint_boundaries` contains `SLICE_ID`; otherwise `0`.
  - If `BOUNDARY_SLICE=1`, determine `NEXT_CHECKPOINT_TASK_ID` from `pre-planning/ci_checkpoint_plan.md` (legacy fallback: root `ci_checkpoint_plan.md`) by selecting the checkpoint whose final `slices[]` entry is `SLICE_ID`.
  - `FINAL_TASK_ID="${SLICE_ID}-integ"`
- Read task statuses from `tasks.json`:
  - `CODE_STATUS`, `TEST_STATUS`, `INTEG_STATUS`
  - if boundary: `CHECKPOINT_STATUS`, `FINAL_STATUS`, and every `${SLICE_ID}-integ-<platform>` task status that exists in `tasks.json`
- Discover existing artifacts:
  - `LATEST_SUMMARY_JSON="$(ls -1t "$FEATURE_DIR/logs/$SLICE_ID/wrapper/"*.summary.json 2>/dev/null | head -n 1 || true)"`
  - `SESSION_LOG="$FEATURE_DIR/session_log.md"`
  - Task last-message paths:
    - code: `"$FEATURE_DIR/logs/$SLICE_ID/code/last_message.md"`
    - test: `"$FEATURE_DIR/logs/$SLICE_ID/test/last_message.md"`
    - wired integration kind: `"$FEATURE_DIR/logs/$SLICE_ID/<integ-kind>/last_message.md"`
    - platform-fix/final task kinds follow the same `logs/$SLICE_ID/<task-kind>/last_message.md` rule.
- Derive the initial candidate validation SHA:
  - If `LATEST_SUMMARY_JSON` exists: `CHECKOUT_SHA="$(jq -r '.heads.integration // empty' "$LATEST_SUMMARY_JSON")"`
  - Else: `CHECKOUT_SHA="$(git rev-parse "$(jq -r --arg id "$INTEG_TASK_ID" '.tasks[] | select(.id==$id) | .git_branch' "$FEATURE_DIR/tasks.json")")"`
  - If one or more `${SLICE_ID}-integ-<platform>` tasks are already `completed`, inspect the most recent matching `## END` entry in `session_log.md` and/or the relevant task branch HEADs, then update `CHECKOUT_SHA` to the latest completed platform-fix branch HEAD before re-dispatching checkpoint gates.

## Step 2: Resolve the resume stage
- If `START_AT` is not `auto`, treat it as a lower bound only. Completed stages still stay skipped.
- Otherwise resolve `RESUME_STAGE` as follows:
  - If `CODE_STATUS`, `TEST_STATUS`, or `INTEG_STATUS` is not `completed`: `RESUME_STAGE="triad"`.
  - Else if `BOUNDARY_SLICE=0`: `RESUME_STAGE="done"`.
  - Else if `FINAL_STATUS="completed"`: `RESUME_STAGE="done"`.
  - Else if `CHECKPOINT_STATUS` is not `completed`: `RESUME_STAGE="checkpoint"`.
  - Else if any started `${SLICE_ID}-integ-<platform>` task is still `in_progress`: `RESUME_STAGE="platform-fixes"`.
  - Else: `RESUME_STAGE="final-aggregator"`.
- Report the detected state before continuing:
  - current task statuses,
  - `LATEST_SUMMARY_JSON` (or “missing”),
  - chosen `RESUME_STAGE`,
  - current `CHECKOUT_SHA`.

## Step 3: If `RESUME_STAGE=triad`, run or resume code + test + wired integration
- Run:
  - `make triad-task-start-complete FEATURE_DIR="$FEATURE_DIR" SLICE_ID="$SLICE_ID"`
- Parse stdout key/value lines and store:
  - `INTEG_TASK_ID`
  - `BOUNDARY_SLICE`
  - `NEXT_CHECKPOINT_TASK_ID`
  - `SUMMARY_JSON_PATH`
- Re-read `tasks.json` and refresh:
  - `CODE_STATUS`, `TEST_STATUS`, `INTEG_STATUS`
  - `CHECKOUT_SHA="$(jq -r '.heads.integration // empty' "$SUMMARY_JSON_PATH")"`
- If `BOUNDARY_SLICE=0` and `INTEG_STATUS=completed`:
  - Stop. The slice flow is complete.
- If boundary:
  - Continue with the checkpoint stage. Do not re-run triad work again in this session.

## Step 4: If this is a boundary slice, run or resume the checkpoint stage
- Preconditions for boundary slices:
  - `NEXT_CHECKPOINT_TASK_ID` must be non-empty.
  - `INTEG_TASK_ID` must be `${SLICE_ID}-integ-core`; otherwise stop and report a planning pack mismatch for schema v4+ boundary slices.
- If `CHECKPOINT_STATUS=pending`:
  - Start the checkpoint bookkeeping exactly once:
    - mark the checkpoint task `in_progress`,
    - append the `START` entry to `session_log.md`,
    - commit the orchestration-branch bookkeeping change.
- If `CHECKPOINT_STATUS=in_progress`:
  - Resume from the current evidence; do not create duplicate START entries.
  - Read the latest checkpoint run ids/URLs already recorded in `session_log.md` before dispatching anything new.
- Before any checkpoint dispatch, run the local behavioral smoke preflight on your current host platform when `$FEATURE_DIR/smoke/` exists and you do not already have a clear green preflight for the current candidate `CHECKOUT_SHA`:
  - `INTEG_CORE_WT="$(jq -r --arg id "${SLICE_ID}-integ-core" '.tasks[] | select(.id==$id) | .worktree' "$FEATURE_DIR/tasks.json")"`
  - Linux: `cd "$INTEG_CORE_WT" && cargo build --bin substrate && export PATH="$PWD/target/debug:$PATH" && bash "$FEATURE_DIR/smoke/linux-smoke.sh"`
  - macOS: `cd "$INTEG_CORE_WT" && cargo build --bin substrate && export PATH="$PWD/target/debug:$PATH" && bash "$FEATURE_DIR/smoke/macos-smoke.sh"`
  - If the smoke scripts do not exist for the current OS, skip preflight.
- Determine the checkpoint kickoff prompt path:
  - `CP_KICKOFF="$(jq -r --arg id "$NEXT_CHECKPOINT_TASK_ID" '.tasks[] | select(.id==$id) | .kickoff_prompt' "$FEATURE_DIR/tasks.json")"`
- Execute the checkpoint exactly as the kickoff prompt specifies, but resume-safely:
  - `cat "$CP_KICKOFF"`
  - Reuse existing run evidence when it already proves green for the current `CHECKOUT_SHA`.
  - If existing evidence is missing, stale, failed, cancelled, or validates an older SHA, re-dispatch the required checkpoint gates from the orchestration checkout using the current `CHECKOUT_SHA`.
  - Record every newly dispatched run id/URL in `"$FEATURE_DIR/session_log.md"`.
- If the checkpoint gates are green:
  - If `CHECKPOINT_STATUS` is not yet `completed`, mark it `completed` in `tasks.json`, append the `END` entry to `session_log.md`, and commit.
  - Continue to final aggregation.
- If the checkpoint gates are not green:
  - Continue to platform-fixes.

## Step 5: Handle checkpoint failures deterministically, including resume
- Identify which OS(es) still need fixes using the latest dispatcher outputs and/or GitHub run logs.
  - Treat `cancelled` jobs as unknown; after the first fix lands, re-run gates to discover the remaining failures.
- If you cannot start a required `${SLICE_ID}-integ-<platform>` task because it is blocked by `depends_on`:
  - This is a planning pack wiring mismatch. For schema v4+ boundary-only platform-fix, platform-fix tasks must depend on `${SLICE_ID}-integ-core` (not `CPk-ci-checkpoint`).
  - Fix `tasks.json`, commit on the orchestration branch, then continue.
- For each failing platform:
  - If task status is `completed`:
    - Reuse its existing branch/worktree/message and do not restart it.
  - If task status is `in_progress`:
    - Reuse the existing worktree from `tasks.json`.
    - If `logs/$SLICE_ID/integ-<platform>/codex.pid` points to a live process, wait for it to exit before deciding.
    - If `logs/$SLICE_ID/integ-<platform>/last_message.md` is missing or stale, re-launch Codex into the existing worktree:
      - `make triad-task-start FEATURE_DIR="$FEATURE_DIR" TASK_ID="${SLICE_ID}-integ-<platform>" LAUNCH_CODEX=1`
    - Finish the task from inside its worktree:
      - `make triad-task-finish TASK_ID="${SLICE_ID}-integ-<platform>"`
  - If task status is `pending`:
    - Start only the missing failing platform tasks with Codex enabled:
      - If you have a single failing smoke run id that identifies the failing platforms, prefer:
        - `make triad-task-start-platform-fixes-from-smoke FEATURE_DIR="$FEATURE_DIR" SLICE_ID="$SLICE_ID" SMOKE_RUN_ID="<run-id>" LAUNCH_CODEX=1`
      - Otherwise use:
        - `make triad-task-start-platform-fixes FEATURE_DIR="$FEATURE_DIR" SLICE_ID="$SLICE_ID" PLATFORMS="<csv failing platforms>" LAUNCH_CODEX=1`
    - For each newly started platform-fix task, follow its start/end checklist in `tasks.json` and finish it from inside its worktree:
      - `make triad-task-finish TASK_ID="${SLICE_ID}-integ-<platform>"`
- After any platform-fix branch lands:
  - Update `CHECKOUT_SHA` to the HEAD you are now validating.
  - Re-run the checkpoint dispatch commands until green.
- Repeat until the checkpoint stage is green, then ensure the checkpoint task itself is marked `completed`.

## Step 6: Run or resume the final aggregator
- Once the checkpoint gates are green:
  - If you did not run a given platform-fix task, mark it `completed` as a no-op so the final aggregator `depends_on` is satisfied:
    - `make triad-mark-noop-platform-fixes-completed FEATURE_DIR="$FEATURE_DIR" SLICE_ID="$SLICE_ID"`
- Resolve final aggregation behavior by status:
  - If `FINAL_STATUS=completed`:
    - Stop. The full slice workflow is already complete.
  - If `FINAL_STATUS=pending`:
    - Start the final aggregator:
      - `make triad-task-start-integ-final FEATURE_DIR="$FEATURE_DIR" SLICE_ID="$SLICE_ID" LAUNCH_CODEX=1`
  - If `FINAL_STATUS=in_progress`:
    - Reuse the existing final aggregator worktree from `tasks.json`.
    - If `logs/$SLICE_ID/integ/codex.pid` points to a live process, wait for it to exit.
    - If `logs/$SLICE_ID/integ/last_message.md` is missing, re-launch Codex into the existing final-aggregator worktree:
      - `make triad-task-start FEATURE_DIR="$FEATURE_DIR" TASK_ID="$FINAL_TASK_ID" LAUNCH_CODEX=1`
    - Finish from inside the final aggregator worktree:
      - `make triad-task-finish TASK_ID="$FINAL_TASK_ID"`

## Output to operator
Return a concise summary that includes:
- Detected `RESUME_STAGE`
- The wrapper summary path used or created (`SUMMARY_JSON_PATH` or `LATEST_SUMMARY_JSON`)
- Current/final task statuses for code, test, wired integration, checkpoint, platform fixes, and final aggregator
- Checkpoint run ids/URLs + pass/fail for the runs you reused or dispatched in this session
- Platform-fix tasks started or resumed (if any): task ids + final message paths
- Final aggregator: `FINAL_TASK_ID` + final message path
- The final `CHECKOUT_SHA` that was validated
```
