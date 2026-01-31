# Triad Wrapper Prompt (Post-Preflight Automation)

Use this **after** `F0-exec-preflight` is completed for a feature, to:
- start **code + test in parallel** via triad automation **with Codex enabled**, then
- start the slice’s **integration merge** task (one Codex session), then
- report **exit codes + final messages** plus artifact paths.

Fast path (preferred when available):
- Run the fully automated wrapper command instead of doing the manual bookkeeping steps in this prompt:
  - `make triad-task-start-complete FEATURE_DIR="docs/project_management/next/<feature>" SLICE_ID="<slice>"`
  - It updates `tasks.json` + `session_log.md`, runs code+test in parallel, runs the slice’s integration task as wired in `tasks.json` (`<slice>-code.integration_task`), and writes a wrapper summary under `FEATURE_DIR/logs/<slice>/wrapper/`.

Notes:
- Run this from the **orchestration checkout** (repo root on the orchestration branch), **not** inside a task worktree.
- Code and test are separate concerns:
  - Code agent = production code only.
  - Test agent = tests only.
  - Integration (per-slice merge task) owns green + merge/reconcile for the slice’s spec.
- This wrapper must invoke triad automation with Codex enabled: `LAUNCH_CODEX=1`.
- This wrapper is responsible for orchestration-branch bookkeeping:
  - Mark `tasks.json` START/END status updates.
  - Append START/END entries to `session_log.md`.
  - Never edit planning docs inside any task worktree.
- If a slice feels “big” (many unrelated acceptance bullets, multiple subsystems, or a broad refactor + new behavior), stop and ask the operator to split it into smaller triads before dispatching.

## Copy/Paste Prompt Template

Fill in only:
- `FEATURE_DIR` (planning pack dir)
- `SLICE_ID` (slice prefix; e.g. `PCP0`)

```text
You are the “Triad wrapper” orchestration agent.

## Inputs
FEATURE_DIR="<SET_ME>"   # e.g. docs/project_management/_archived/policy_and_config_precedence
SLICE_ID="<SET_ME>"      # e.g. PCP0

## Non-negotiables
- Run from the orchestration checkout (repo root), not from a task worktree.
- Dispatch with Codex enabled: run `make triad-task-start-pair ... LAUNCH_CODEX=1`.
- Prefer triad automation + concurrent execution (`triad-task-start-pair`) unless impossible.
- Do not edit planning docs inside any task worktree.

## What to do
0) Preconditions (orchestration checkout only)
   - Ensure you are on the orchestration branch and clean:
     - `make triad-orch-ensure FEATURE_DIR="$FEATURE_DIR"`
     - `git status --porcelain=v1` must be empty (if not, stop; do not start triads from a dirty orchestration checkout).
   - Ensure no other headless Codex run is still active (avoid overlapping runs mutating state):
     - `find "$FEATURE_DIR/logs" -name codex.pid -print -exec cat {} \\;` should print nothing (or only stale PIDs you’ve verified are not running).

1) START bookkeeping (orchestration branch)
   - Compute task ids:
     - `CODE_TASK_ID="${SLICE_ID}-code"`
     - `TEST_TASK_ID="${SLICE_ID}-test"`
   - Update `"$FEATURE_DIR/tasks.json"`:
     - Set `CODE_TASK_ID` and `TEST_TASK_ID` status to `in_progress`.
   - Append START entries to `"$FEATURE_DIR/session_log.md"` for both tasks (UTC timestamps):
     - Use `date -u +%Y-%m-%dT%H:%M:%SZ` for the timestamp.
     - Include: task id, expected branch/worktree (from tasks.json), and the exact dispatch command you’re about to run.
     - Standard START entry format (copy/paste; fill placeholders):
       - Code:
         ```
         ## START — <NOW_UTC> — code — <CODE_TASK_ID>
         - Worktree: `<CODE_WORKTREE_EXPECTED>`
         - Branch: `<CODE_TASK_BRANCH_EXPECTED>`
         - Orchestration branch: `<ORCH_BRANCH>`
         - Dispatch:
           - `make triad-task-start-pair FEATURE_DIR="$FEATURE_DIR" SLICE_ID="$SLICE_ID" LAUNCH_CODEX=1`
         ```
       - Test:
         ```
         ## START — <NOW_UTC> — test — <TEST_TASK_ID>
         - Worktree: `<TEST_WORKTREE_EXPECTED>`
         - Branch: `<TEST_TASK_BRANCH_EXPECTED>`
         - Orchestration branch: `<ORCH_BRANCH>`
         - Dispatch:
           - `make triad-task-start-pair FEATURE_DIR="$FEATURE_DIR" SLICE_ID="$SLICE_ID" LAUNCH_CODEX=1`
         ```
   - Commit the planning-pack START updates on the orchestration branch:
     - `git add "$FEATURE_DIR/tasks.json" "$FEATURE_DIR/session_log.md"`
     - Preferred: commit separately per task:
       - `git commit -m "docs: start ${CODE_TASK_ID}"`
       - `git commit -m "docs: start ${TEST_TASK_ID}"`
     - If you choose a single combined commit, make it obvious in the message (e.g., `docs: start ${SLICE_ID} code+test`).

2) Run:
   `make triad-task-start-pair FEATURE_DIR="$FEATURE_DIR" SLICE_ID="$SLICE_ID" LAUNCH_CODEX=1`

3) Parse the script stdout contract (key=value lines). Collect at minimum:
   - `CODEX_CODE_EXIT`, `CODEX_TEST_EXIT`
   - `CODEX_CODE_LAST_MESSAGE_PATH`, `CODEX_TEST_LAST_MESSAGE_PATH`
   - (if present) `CODEX_CODE_EVENTS_PATH`, `CODEX_TEST_EVENTS_PATH`
   - (if present) `CODEX_CODE_STDERR_PATH`, `CODEX_TEST_STDERR_PATH`
   - `CODE_WORKTREE`, `TEST_WORKTREE` (for navigation)

4) Capture commit SHAs (for the produced task branches):
   - `CODE_HEAD_SHA=$(git -C "$CODE_WORKTREE" rev-parse HEAD)`
   - `TEST_HEAD_SHA=$(git -C "$TEST_WORKTREE" rev-parse HEAD)`

5) Read and include the full contents of:
   - `CODEX_CODE_LAST_MESSAGE_PATH`
   - `CODEX_TEST_LAST_MESSAGE_PATH`
   If a file is missing, report that clearly (do not guess).

6) END bookkeeping (orchestration branch)
   - Ensure each task worktree is actually finished (this is how we avoid “agent forgot to run finish” drift):
     - From inside each worktree, run:
       - `cd "$CODE_WORKTREE" && make triad-task-finish TASK_ID="$CODE_TASK_ID" VERIFY_ONLY=1`
       - `cd "$TEST_WORKTREE" && make triad-task-finish TASK_ID="$TEST_TASK_ID" VERIFY_ONLY=1`
     - If `VERIFY_ONLY=1` fails because the task was not properly finished, re-run without `VERIFY_ONLY=1` to close it out:
       - `cd "$CODE_WORKTREE" && make triad-task-finish TASK_ID="$CODE_TASK_ID"`
       - `cd "$TEST_WORKTREE" && make triad-task-finish TASK_ID="$TEST_TASK_ID"`
     - Capture the `triad-task-finish` stdout contract (key=value lines) for each task and include it in the END entry (do not paste large logs).
   - Update `"$FEATURE_DIR/tasks.json"`:
     - If the task is finished successfully: set status to `completed`.
     - If Codex exited non-zero or the task cannot be finished cleanly: set status to `blocked` (do not mark as completed).
   - Append END entries to `"$FEATURE_DIR/session_log.md"` for both tasks (UTC timestamps):
     - Include: worktree, branch, HEAD sha, commands run (or `triad-task-finish` summary), and note any blockers.
     - Include links/paths to the Codex artifacts (last_message/events/stderr) but do not paste large logs.
     - Standard END entry format (copy/paste; fill placeholders):
       - Code:
         ```
         ## END — <NOW_UTC> — code — <CODE_TASK_ID>
         - Worktree: `<CODE_WORKTREE>`
         - Branch: `<CODE_TASK_BRANCH>`
         - HEAD: `<CODE_HEAD_SHA>`
         - Codex: `CODEX_CODE_EXIT=<exit>`
         - Finisher summary:
           - `TASK_BRANCH=<...>`
           - `WORKTREE=<...>`
           - `HEAD=<...>`
           - `COMMITS=<...>`
           - `CHECKS=<...>`
           - `SMOKE_RUN=<...>`
           - `MERGED_TO_ORCH=<...>`
         - Artifacts:
           - `CODEX_CODE_LAST_MESSAGE_PATH=<path>`
           - `CODEX_CODE_EVENTS_PATH=<path>` (if present)
           - `CODEX_CODE_STDERR_PATH=<path>` (if present)
         - Blockers: `NONE` (or describe)
         ```
       - Test:
         ```
         ## END — <NOW_UTC> — test — <TEST_TASK_ID>
         - Worktree: `<TEST_WORKTREE>`
         - Branch: `<TEST_TASK_BRANCH>`
         - HEAD: `<TEST_HEAD_SHA>`
         - Codex: `CODEX_TEST_EXIT=<exit>`
         - Finisher summary:
           - `TASK_BRANCH=<...>`
           - `WORKTREE=<...>`
           - `HEAD=<...>`
           - `COMMITS=<...>`
           - `CHECKS=<...>`
           - `SMOKE_RUN=<...>`
           - `MERGED_TO_ORCH=<...>`
         - Artifacts:
           - `CODEX_TEST_LAST_MESSAGE_PATH=<path>`
           - `CODEX_TEST_EVENTS_PATH=<path>` (if present)
           - `CODEX_TEST_STDERR_PATH=<path>` (if present)
         - Blockers: `NONE` (or describe)
         ```
   - Commit the planning-pack END updates on the orchestration branch:
     - `git add "$FEATURE_DIR/tasks.json" "$FEATURE_DIR/session_log.md"`
     - Preferred: commit separately per task:
       - `git commit -m "docs: finish ${CODE_TASK_ID}"`
       - `git commit -m "docs: finish ${TEST_TASK_ID}"`

7) Determine the per-slice integration task id (orchestration checkout)
   - Cross-platform automation packs use `*-integ-core` as the per-slice merge task.
   - Non-cross-platform packs use `*-integ`.

   Determine which exists in `tasks.json`:
   - Prefer integ-core when present:
     - `INTEG_TASK_ID="$(jq -r --arg s "$SLICE_ID" '.tasks[] | select(.id==($s+\"-integ-core\")) | .id' \"$FEATURE_DIR/tasks.json\" | head -n 1)"`
   - If empty, fall back to:
     - `INTEG_TASK_ID="${SLICE_ID}-integ"`

8) START bookkeeping for integration (orchestration branch)
   - Update `"$FEATURE_DIR/tasks.json"`:
     - Set `INTEG_TASK_ID` status to `in_progress`.
   - Append a START entry to `"$FEATURE_DIR/session_log.md"` (UTC timestamp) with:
     - task id, expected branch/worktree (from tasks.json), and dispatch command:
       - `make triad-task-start FEATURE_DIR="$FEATURE_DIR" TASK_ID="$INTEG_TASK_ID" LAUNCH_CODEX=1`
   - Commit the START update:
     - `git add "$FEATURE_DIR/tasks.json" "$FEATURE_DIR/session_log.md"`
     - `git commit -m "docs: start ${INTEG_TASK_ID}"`

9) Run integration (one Codex session)
   `make triad-task-start FEATURE_DIR="$FEATURE_DIR" TASK_ID="$INTEG_TASK_ID" LAUNCH_CODEX=1`

10) Parse the script stdout contract (key=value lines). Collect at minimum:
   - `WORKTREE` (store as `INTEG_WORKTREE`)
   - `CODEX_EXIT`
   - `CODEX_LAST_MESSAGE_PATH`
   - (if present) `CODEX_EVENTS_PATH`, `CODEX_STDERR_PATH`

11) Capture the integration commit SHA:
   - `INTEG_HEAD_SHA=$(git -C "$INTEG_WORKTREE" rev-parse HEAD)`

12) Read and include the full contents of:
   - `CODEX_LAST_MESSAGE_PATH`
   If the file is missing, report that clearly (do not guess).

13) END bookkeeping for integration (orchestration branch)
   - Ensure the integration worktree is actually finished:
     - `cd "$INTEG_WORKTREE" && make triad-task-finish TASK_ID="$INTEG_TASK_ID" VERIFY_ONLY=1`
     - If `VERIFY_ONLY=1` fails, run:
       - `cd "$INTEG_WORKTREE" && make triad-task-finish TASK_ID="$INTEG_TASK_ID"`
   - Update `"$FEATURE_DIR/tasks.json"`:
     - If the task is finished successfully: set status to `completed`.
     - If Codex exited non-zero or the task cannot be finished cleanly: set status to `blocked`.
   - Append an END entry to `"$FEATURE_DIR/session_log.md"` (UTC timestamp) with:
     - worktree, branch, HEAD sha, finisher summary, and Codex artifacts paths.
   - Commit the END update:
     - `git add "$FEATURE_DIR/tasks.json" "$FEATURE_DIR/session_log.md"`
     - `git commit -m "docs: finish ${INTEG_TASK_ID}"`

Next steps note (checkpointed cross-platform packs):
- Cross-platform CI dispatch does not run from the per-slice merge task. If this slice is the end of a checkpoint group, run the checkpoint task (for example `CP1-ci-checkpoint`) using the checkpoint kickoff prompt, or use `docs/project_management/standards/TRIAD_INTEGRATION_WRAPPER_PROMPT.md` for checkpoint-boundary slices.

## Output to operator
Return a concise summary that includes:
- Code task: exit code + `CODE_HEAD_SHA` + final message + artifact paths
- Test task: exit code + `TEST_HEAD_SHA` + final message + artifact paths
- Integration task: exit code + `INTEG_HEAD_SHA` + final message + artifact paths
- Links/paths to the full artifacts (last_message/events/stderr) for both tasks

Do NOT inline huge logs (events/stderr). Paths + short summary only.
```
