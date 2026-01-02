# Triad Integration Wrapper Prompt (Cross-Platform Integ Orchestration)

Use this when you want a single orchestration run to:
- start `<SLICE>-integ-core`, run it with Codex enabled, and capture its final message,
- use integ-core’s reported cross-platform smoke outcome to infer failing platforms,
- start only the failing `<SLICE>-integ-<platform>` tasks (optionally with Codex enabled) and capture their final messages,
- start `<SLICE>-integ` (final aggregator) and capture its final message,
- report exit codes + artifact paths for all runs.

Notes:
- Run this from the **orchestration checkout** (repo root), not inside any task worktree.
- This wrapper does not assume you are editing `tasks.json`/`session_log.md`, but **`make triad-task-start-integ-final` requires `depends_on` tasks are `status=completed`**. If CI smoke is green and platform-fix tasks were no-ops, the operator must still mark them `completed` on the orchestration branch to unblock the final aggregator.

## Copy/Paste Prompt Template

Fill in only:
- `FEATURE_DIR` (planning pack dir)
- `SLICE_ID` (slice prefix; e.g. `PCP0`)

```text
You are the “Triad integration wrapper” orchestration agent.

## Inputs
FEATURE_DIR="<SET_ME>"   # e.g. docs/project_management/next/policy_and_config_precedence
SLICE_ID="<SET_ME>"      # e.g. PCP0

## Non-negotiables
- Run from the orchestration checkout (repo root), not from a task worktree.
- Prefer triad automation scripts (`make triad-task-start*`) and capture Codex artifacts from `target/triad/...`.
- Do not edit planning docs inside any task worktree.

## What to do
1) Start and run integ-core with Codex enabled:
   `make triad-task-start FEATURE_DIR="$FEATURE_DIR" TASK_ID="${SLICE_ID}-integ-core" LAUNCH_CODEX=1`
   Parse stdout key/value lines and collect:
   - `WORKTREE`, `TASK_BRANCH`, `ORCH_BRANCH`
   - `CODEX_EXIT`, `CODEX_LAST_MESSAGE_PATH`, `CODEX_EVENTS_PATH`, `CODEX_STDERR_PATH`
   Read and include the full contents of `CODEX_LAST_MESSAGE_PATH`.
   Also capture the commit SHA after the run:
   - `INTEG_CORE_HEAD_SHA=$(git -C "$WORKTREE" rev-parse HEAD)`

2) Parse the integ-core agent’s smoke outcome from its final message:
   - Extract and report these exact keys if present (one per line):
     - `RUN_ID=<id>`
     - `RUN_URL=<url>`
     - `SMOKE_PASSED_PLATFORMS=<csv>`
     - `SMOKE_FAILED_PLATFORMS=<csv>`
   - If `SMOKE_FAILED_PLATFORMS` is missing, treat this as a failure (integ-core must run and report cross-platform smoke).

3) If smoke failed and `SMOKE_FAILED_PLATFORMS` is non-empty, start only failing platform-fix tasks with Codex enabled (parallel):
   `make triad-task-start-platform-fixes-from-smoke FEATURE_DIR="$FEATURE_DIR" SLICE_ID="$SLICE_ID" SMOKE_RUN_ID="$RUN_ID" LAUNCH_CODEX=1`
   For each selected platform block, collect:
   - `PLATFORM`, `TASK_ID`, `WORKTREE`, `TASK_BRANCH`, `CODEX_EXIT`
   - `CODEX_LAST_MESSAGE_PATH`, `CODEX_EVENTS_PATH`, `CODEX_STDERR_PATH`
   Read and include the full contents of each `CODEX_LAST_MESSAGE_PATH`.
   Also capture each platform-fix commit SHA after the run:
   - `HEAD_SHA=$(git -C "$WORKTREE" rev-parse HEAD)`

4) If smoke succeeded (no platform fixes needed):
   - Do not start any platform-fix tasks; no platform-fix worktrees/branches are expected.
   - Mark the platform-fix tasks as `completed` no-ops on the orchestration branch so the final aggregator can start (its `depends_on` enforces completion).
   - Prefer using the helper script:
     - `scripts/triad/mark_noop_platform_fixes_completed.sh --feature-dir "$FEATURE_DIR" --slice-id "$SLICE_ID" --from-smoke-run "$RUN_ID"`

5) After platform fixes are complete (or marked no-op), start the final aggregator:
   - Confirm `${SLICE_ID}-integ-core` and all required `${SLICE_ID}-integ-<platform>` tasks are `status=completed` in `$FEATURE_DIR/tasks.json` (the starter enforces this).
   - Run:
     `make triad-task-start-integ-final FEATURE_DIR="$FEATURE_DIR" SLICE_ID="$SLICE_ID" LAUNCH_CODEX=1`
   Parse stdout key/value lines and collect:
   - `FINAL_TASK_ID`, `WORKTREE`, `TASK_BRANCH`, `ORCH_BRANCH`
   - `CODEX_EXIT`, `CODEX_LAST_MESSAGE_PATH`, `CODEX_EVENTS_PATH`, `CODEX_STDERR_PATH`
   Read and include the full contents of `CODEX_LAST_MESSAGE_PATH`.
   Also capture the commit SHA after the run:
   - `FINAL_HEAD_SHA=$(git -C "$WORKTREE" rev-parse HEAD)`

## Output to operator
Return a concise summary that includes:
- integ-core: `CODEX_EXIT` + `INTEG_CORE_HEAD_SHA` + final message + artifact paths
- smoke (as reported by integ-core): `RUN_ID` + `RUN_URL` + `SMOKE_PASSED_PLATFORMS` + `SMOKE_FAILED_PLATFORMS`
- each platform-fix task started: `CODEX_EXIT` + final message + artifact paths
- final aggregator: `CODEX_EXIT` + `FINAL_HEAD_SHA` + final message + artifact paths

Do NOT inline events/stderr logs; only paths and short summaries.
If any expected file is missing, report that clearly (do not guess).
```
