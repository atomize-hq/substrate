# Triad Unified Wrapper Prompt (Checkpoint-Aware)

Use this after `F0-exec-preflight` is completed for a feature when you want a single orchestration run that:
- runs `<SLICE>-code` + `<SLICE>-test` in parallel (with Codex enabled),
- runs the slice integration task wired via `tasks.json` (`<SLICE>-code.integration_task`) (with Codex enabled),
- and, if `<SLICE>` is a schema v4+ CI checkpoint boundary slice, completes the entire checkpointed flow:
  - run the planned `CPk-ci-checkpoint` ops task,
  - run only the necessary `<SLICE>-integ-<platform>` platform-fix tasks (with Codex enabled) until CI parity is green,
  - then run `<SLICE>-integ` final aggregation (with Codex enabled).

Non-negotiables:
- Run from the orchestration checkout (repo root on the orchestration branch), not from any task worktree.
- Do not edit planning docs inside any task worktree.
- Do not dispatch cross-platform CI from per-slice integration worktrees; dispatch only from the checkpoint ops task.

## Copy/Paste Prompt Template

Fill in only:
- `FEATURE_DIR` (planning pack dir)
- `SLICE_ID` (slice prefix)

```text
You are the “Triad unified wrapper” orchestration agent.

## Inputs
FEATURE_DIR="<SET_ME>"   # e.g. docs/project_management/next/world-fs-granular-allow-deny
SLICE_ID="<SET_ME>"      # e.g. WFGAD1

## Non-negotiables
- Run from the orchestration checkout (repo root), not from any task worktree.
- Dispatch with Codex enabled when starting tasks: use `LAUNCH_CODEX=1` for `make triad-task-start*` commands.
- Do not edit planning docs inside any task worktree.

## Step 0: Preconditions
- Ensure orchestration checkout is clean:
  - `make triad-orch-ensure FEATURE_DIR="$FEATURE_DIR"`
  - `git status --porcelain=v1` must be empty.

## Step 1: Run code + test + the wired integration task (single command)
- Run:
  - `make triad-task-start-complete FEATURE_DIR="$FEATURE_DIR" SLICE_ID="$SLICE_ID"`

- Parse stdout key/value lines and store:
  - `INTEG_TASK_ID`
  - `BOUNDARY_SLICE` (0|1)
  - `NEXT_CHECKPOINT_TASK_ID` (empty or a `CPk-ci-checkpoint` task id)
  - `SUMMARY_JSON_PATH`

## Step 2: If not a checkpoint-boundary slice, stop here
- If `BOUNDARY_SLICE=0`:
  - Stop. The slice flow is complete.

## Step 3: If this is a checkpoint-boundary slice, run the planned CI checkpoint
- If `BOUNDARY_SLICE=1`:
  - `NEXT_CHECKPOINT_TASK_ID` MUST be non-empty. If it is empty, stop and report an error (planning pack mismatch).
  - Determine the checkpoint kickoff prompt path:
    - `CP_KICKOFF="$(jq -r --arg id \"$NEXT_CHECKPOINT_TASK_ID\" '.tasks[] | select(.id==$id) | .kickoff_prompt' \"$FEATURE_DIR/tasks.json\")"`
  - Execute the checkpoint exactly as the kickoff prompt specifies (from the orchestration checkout):
    - `cat "$CP_KICKOFF"`
    - Run the listed dispatch commands and record run ids/URLs in `"$FEATURE_DIR/session_log.md"`.
    - Update `"$FEATURE_DIR/tasks.json"` for the checkpoint task:
      - status `in_progress` at START and `completed` at END (commit both).

## Step 4: Handle checkpoint failures deterministically
- If the checkpoint gates fail:
  - Identify which OS(es) failed using the dispatcher outputs and/or GitHub run logs.
  - If the failure is a compile parity failure (build does not compile on some OS):
    - Fix it on the checkpoint slice’s `<SLICE>-integ-core` worktree/branch (not a platform-fix task), commit, and re-run the checkpoint gates until green.
  - If the failure is an OS-specific CI testing failure (tests fail only on certain OSes):
    - Start only the required platform-fix task(s) for the failing OS(es), with Codex enabled:
      - `make triad-task-start FEATURE_DIR="$FEATURE_DIR" TASK_ID="${SLICE_ID}-integ-linux" LAUNCH_CODEX=1` (linux)
      - `make triad-task-start FEATURE_DIR="$FEATURE_DIR" TASK_ID="${SLICE_ID}-integ-macos" LAUNCH_CODEX=1` (macos)
      - `make triad-task-start FEATURE_DIR="$FEATURE_DIR" TASK_ID="${SLICE_ID}-integ-windows" LAUNCH_CODEX=1` (windows)
    - After each platform-fix task, ensure it is finished successfully from inside its worktree:
      - `make triad-task-finish TASK_ID="${SLICE_ID}-integ-<platform>"`
    - Re-run the checkpoint gates until green.

## Step 5: Unblock the final aggregator and run it
- Once the checkpoint gates are green:
  - If you did not run a given platform-fix task, mark it `completed` as a no-op so the final aggregator `depends_on` is satisfied:
    - `make triad-mark-noop-platform-fixes-completed FEATURE_DIR="$FEATURE_DIR" SLICE_ID="$SLICE_ID"`

  - Start the final aggregator (this enforces that all depends_on tasks are completed):
    - `make triad-task-start-integ-final FEATURE_DIR="$FEATURE_DIR" SLICE_ID="$SLICE_ID" LAUNCH_CODEX=1`

## Output to operator
Return a concise summary that includes:
- The `SUMMARY_JSON_PATH` produced by `triad-task-start-complete`
- Checkpoint run ids/URLs + pass/fail
- Platform-fix tasks started (if any): task ids + final message paths
- Final aggregator: `FINAL_TASK_ID` + final message path
```

