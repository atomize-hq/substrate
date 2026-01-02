# Triad Integration Wrapper Prompt (Cross-Platform Integ Orchestration)

Use this when you want a single orchestration run to:
- start `<SLICE>-integ-core`, run it with Codex enabled, and capture its final message,
- dispatch cross-platform smoke via CI and infer failing platforms,
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

2) Dispatch cross-platform smoke (CI) for the current slice state:
   - Determine `RUN_WSL` by reading `$FEATURE_DIR/tasks.json` meta (`wsl_required` and `wsl_task_mode`).
   - Run smoke from inside the integ-core worktree (smoke validates current `HEAD` via a throwaway branch):
     `cd "$WORKTREE" && make feature-smoke FEATURE_DIR="$FEATURE_DIR" PLATFORM=all RUNNER_KIND=self-hosted WORKFLOW_REF="$ORCH_BRANCH" REMOTE=origin CLEANUP=1 [RUN_WSL=1 if required]`
   Parse stdout to capture `RUN_ID`, `RUN_URL` (if present), and `SMOKE_FAILED_PLATFORMS` (if present).

3) If smoke failed and `SMOKE_FAILED_PLATFORMS` is non-empty, start only failing platform-fix tasks with Codex enabled (parallel):
   `make triad-task-start-platform-fixes-from-smoke FEATURE_DIR="$FEATURE_DIR" SLICE_ID="$SLICE_ID" SMOKE_RUN_ID="$RUN_ID" LAUNCH_CODEX=1`
   For each selected platform block, collect:
   - `PLATFORM`, `TASK_ID`, `WORKTREE`, `TASK_BRANCH`, `CODEX_EXIT`
   - `CODEX_LAST_MESSAGE_PATH`, `CODEX_EVENTS_PATH`, `CODEX_STDERR_PATH`
   Read and include the full contents of each `CODEX_LAST_MESSAGE_PATH`.

4) If smoke succeeded (or after platform fixes are complete), start the final aggregator:
   - Confirm `${SLICE_ID}-integ-core` and all required `${SLICE_ID}-integ-<platform>` tasks are `status=completed` in `$FEATURE_DIR/tasks.json` (the starter enforces this).
   - Run:
     `make triad-task-start-integ-final FEATURE_DIR="$FEATURE_DIR" SLICE_ID="$SLICE_ID" LAUNCH_CODEX=1`
   Parse stdout key/value lines and collect:
   - `FINAL_TASK_ID`, `WORKTREE`, `TASK_BRANCH`, `ORCH_BRANCH`
   - `CODEX_EXIT`, `CODEX_LAST_MESSAGE_PATH`, `CODEX_EVENTS_PATH`, `CODEX_STDERR_PATH`
   Read and include the full contents of `CODEX_LAST_MESSAGE_PATH`.

## Output to operator
Return a concise summary that includes:
- integ-core: `CODEX_EXIT` + final message + artifact paths
- smoke: `RUN_ID` + `RUN_URL` + `SMOKE_FAILED_PLATFORMS`
- each platform-fix task started: `CODEX_EXIT` + final message + artifact paths
- final aggregator: `CODEX_EXIT` + final message + artifact paths

Do NOT inline events/stderr logs; only paths and short summaries.
If any expected file is missing, report that clearly (do not guess).
```

