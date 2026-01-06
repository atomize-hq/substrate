# Triad Integration Wrapper Prompt (Cross-Platform Integ Orchestration)

Use this when you want a single orchestration run to:
- start `<SLICE>-integ-core`, run it with Codex enabled, and capture its final message,
- use integ-core’s reported cross-platform smoke outcome to infer failing platforms,
- start only the failing `<SLICE>-integ-<platform>` tasks (optionally with Codex enabled) and capture their final messages,
- start `<SLICE>-integ` (final aggregator) and capture its final message,
- dispatch CI Testing for the final integration commit (gate before merging to `testing`),
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
FEATURE_DIR="<SET_ME>"   # e.g. docs/project_management/_archived/policy_and_config_precedence
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
   Store `WORKTREE` as `INTEG_CORE_WORKTREE`.
   Also capture the commit SHA after the run:
   - `INTEG_CORE_HEAD_SHA=$(git -C "$INTEG_CORE_WORKTREE" rev-parse HEAD)`

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

4) If smoke succeeded (no platform fixes needed for smoke):
   - Do not start any platform-fix tasks yet; they may still be needed for CI-only failures on macOS/Windows.

5) Dispatch CI Testing (gate before deciding “no-op platform fixes” when smoke is green):
   - Run from the integ-core worktree (validates `INTEG_CORE_HEAD_SHA` via a throwaway branch):
     - `cd "$INTEG_CORE_WORKTREE" && scripts/ci/dispatch_ci_testing.sh --workflow-ref "$ORCH_BRANCH" --remote origin --cleanup`
   - Parse and report the CI Testing stdout contract: `RUN_ID`, `RUN_URL`, `CONCLUSION`, `CI_FAILED_OSES`, `CI_FAILED_JOBS`.
   - The CI Testing dispatcher enforces a **2 hour** max wait by default; override only if needed via `CI_TESTING_WATCH_TIMEOUT_SECS`.
   - If CI Testing is green: continue to step (6).
   - If CI Testing fails:
     - Derive platforms to fix from `CI_FAILED_OSES`:
       - `macos-*` -> `macos`
       - `windows-*` -> `windows`
       - `ubuntu-*` -> `linux`
     - Start only those platform-fix tasks:
       - `make triad-task-start-platform-fixes FEATURE_DIR="$FEATURE_DIR" SLICE_ID="$SLICE_ID" PLATFORMS="<csv>" LAUNCH_CODEX=1`
     - Do not mark no-ops; proceed to the final aggregator after fixes are complete (the final aggregator run + final CI Testing gate must be green).

6) Decide the platform-fix path:
   - If CI Testing is green and `SMOKE_FAILED_PLATFORMS` is empty, then no platform-fix worktrees/branches are expected for this slice.
   - In that case, mark platform-fix tasks as `completed` no-ops to unblock the final aggregator’s `depends_on`:
   - `scripts/triad/mark_noop_platform_fixes_completed.sh --feature-dir "$FEATURE_DIR" --slice-id "$SLICE_ID" --from-smoke-run "$RUN_ID"`
   - Otherwise (smoke failed or CI Testing failed), do not mark no-ops; platform-fix tasks must run and reach green.

7) After platform fixes are complete (or marked no-op), start the final aggregator:
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

8) Dispatch CI Testing for the final integration commit (gate before merging to `testing`):
   - Run from the final aggregator worktree (validates `FINAL_HEAD_SHA` via a throwaway branch):
     - `cd "$FINAL_WORKTREE" && scripts/ci/dispatch_ci_testing.sh --workflow-ref "$ORCH_BRANCH" --remote origin --cleanup`
   - If CI Testing fails here:
     - Treat this as blocking (even if smoke is green).
     - Derive platforms to fix from `CI_FAILED_OSES`, start platform-fix tasks, then re-run the final aggregator and this final CI Testing gate.

9) After the final CI Testing gate is green, merge to `testing`.

## Output to operator
Return a concise summary that includes:
- integ-core: `CODEX_EXIT` + `INTEG_CORE_HEAD_SHA` + final message + artifact paths
- smoke (as reported by integ-core): `RUN_ID` + `RUN_URL` + `SMOKE_PASSED_PLATFORMS` + `SMOKE_FAILED_PLATFORMS`
- each platform-fix task started: `CODEX_EXIT` + final message + artifact paths
- final aggregator: `CODEX_EXIT` + `FINAL_HEAD_SHA` + final message + artifact paths
- CI Testing (pre-final, used for platform selection when smoke is green): `RUN_ID` + `RUN_URL` + `CONCLUSION` + `CI_FAILED_OSES` + `CI_FAILED_JOBS`
- CI Testing (final gate on `FINAL_HEAD_SHA`): `RUN_ID` + `RUN_URL` + `CONCLUSION` + `CI_FAILED_OSES` + `CI_FAILED_JOBS`

Do NOT inline events/stderr logs; only paths and short summaries.
If any expected file is missing, report that clearly (do not guess).
```
