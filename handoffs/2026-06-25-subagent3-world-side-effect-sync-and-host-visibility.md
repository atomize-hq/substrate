# World side-effect sync and host visibility — paper trail

## Short objective
Trace the current one-shot `run_world_task` path and answer a narrow question: if the world worker really creates `from_the_world_worker.md`, what must happen for that change to become host-visible at the repo root, and what current-code paths allow a completed run to finish without that host-visible file.

## Relevant code/docs/data flow traced
- Prior handoffs:
  - `handoffs/2026-06-21-135249-world-agent-run-world-task-auth-seeding.md`
  - `handoffs/2026-06-21-154318-world-agent-run-world-task-model-config.md`
  - `handoffs/2026-06-22-211455-spec-62-world-worker-retained-debug.md`
- Current authority/docs:
  - `llm-last-mile/SPEC-63-retained-world-worker-parked-resume-session-handle-contract.md`
  - `docs/WORLD.md`
  - `docs/TRACE.md`
  - `docs/USAGE.md`
  - `docs/internals/world/workspace_sync_filesystem_model.md`
- Current runtime/code path:
  - `crates/shell/src/execution/orchestrator_world_dispatch.rs`
  - `crates/shell/src/execution/routing/dispatch/world_ops.rs`
  - `crates/world-service/src/service.rs`
  - `crates/world-service/src/member_runtime.rs`
  - `crates/world-service/tests/member_runtime_world_placement_v1.rs`
  - `crates/world/src/session.rs`
  - `crates/world/src/overlayfs/mod.rs`
  - `crates/world/src/overlayfs/layering.rs`

## Answer
Yes: with current code, a completed `run_world_task` can legitimately finish without host-visible file materialization.

The host-visible path is **not** `run_world_task -> exit 0 -> host file appears`. The actual carry-back path is:

1. shell `run_world_task` builds a typed `member_dispatch` request against the current shared world;
2. world-service resolves an authoritative placement root;
   - in full isolation this is the world overlay root, not the host repo root;
   - for a repo-root launch, effective cwd becomes that overlay root;
3. the member runtime runs with that overlay-root cwd, so a relative write like `from_the_world_worker.md` lands in the world overlay view first;
4. `run_world_task` completes by streaming only start/event/exit frames; the member-runtime completion frames carry `fs_diff: None`;
5. host visibility happens only later through the normal pending-diff reconciliation path (`pending_diff` -> `workspace sync --direction from_world` or equivalent auto-sync/apply path), which copies the world-side relative path back onto the host workspace root and then clears the pending diff.

So if the worker wrote `from_the_world_worker.md` relative to its default cwd, the world-visible file is expected to be at the overlay merged root for that world (typically `/var/lib/substrate/overlay/<world_id>/merged/from_the_world_worker.md` on a root-owned Linux service) and only becomes host-visible at `<workspace_root>/from_the_world_worker.md` after the sync/reconciliation path runs.

## What would have to happen for a completed task to not surface a host-visible file
1. **Most direct current-code explanation:** the worker wrote the file into the world overlay, but no sync/reconciliation step ran afterward.
2. The worker wrote successfully, but wrote to a different world path than the operator expected (different relative subpath, `/project/...`, `/tmp/...`, etc.).
3. The worker attempted a host-absolute repo path inside full isolation; current placement tests say that should fail, so completion would require the agent to swallow that failure and still exit 0.
4. The worker never actually executed the write even though the wrapper process exited 0; current `run_world_task` success is based on terminal exit state, not side-effect verification.

## Ranked hypotheses for this symptom

### 1. World-only write succeeded; host sync never happened
**Why this is #1**
- `docs/USAGE.md` explicitly says full-isolation world-dispatch writes become host-visible only after `substrate workspace sync --direction from_world` (or equivalent auto-sync).
- `member_runtime_world_placement_v1.rs` explicitly proves: relative write lands in overlay; same relative file is **not** on host before reconciliation.
- `member_runtime.rs` completion frames always emit `fs_diff: None`.
- `execute_run_world_task_stream()` records `task_run_id`, watches `registered`, and waits for `Exit`; it does not inspect or apply filesystem diffs.
- I do not see any auto-sync hook in the `run_world_task` path itself.

**For**
- Exactly matches “completed with a real task_run_id, no host file”.

**Against**
- None from current code/docs. This is the expected behavior unless a later sync step ran.

### 2. The prompt/runtime wrote, but to the wrong world path
**Why this is #2**
- `run_world_task` does not validate that the requested pathname was created.
- Completion only reflects wrapper exit, not artifact verification.
- A worker can legally write inside other writable world locations and still complete.

**For**
- Explains “completed” plus “missing repo-root file” without requiring sync failure.

**Against**
- Default cwd for repo-root `run_world_task` is the overlay root for the workspace, so if the prompt truly said “create `from_the_world_worker.md` here” and the agent obeyed literally, the default path should have been correct.

### 3. The one-shot run surfaced continuity metadata, preserved a hidden retained slot, and later retries failed earlier for lifecycle reasons unrelated to file visibility
**Why this matters**
- `member_runtime.finish_bootstrap()` preserves the retained slot on clean exit when a session handle (`uaa_session_id`) was surfaced.
- `run_world_task` still reports a terminal one-shot outcome and does **not** persist retained shell state.
- A later launch on the same `(orchestration_session_id, world_generation, backend_id)` can then fail earlier on duplicate retained-slot ownership.

**For**
- This fits the user’s note that later retries failed earlier at world-member launch.
- It does not explain the missing host file by itself, but it strongly suggests the original run may have been a real world execution, not a fake completion.

**Against**
- This is a retry/lifecycle explanation, not the direct host-visibility explanation.

### 4. Prompt noop / swallowed write failure / agent chose not to write
**Why this is still possible**
- `run_world_task` success does not prove the requested side effect happened.
- If the agent ignored the file instruction, or attempted an invalid absolute path and then still exited 0, shell-side result handling would still call the run “completed”.

**For**
- Compatible with current success semantics.

**Against**
- Current code/docs already provide a stronger systemic explanation (#1), so this is a fallback hypothesis rather than the leading one.

## Evidence for or against each hypothesis
- **For overlay-only/no-sync:**
  - `crates/world-service/src/service.rs`: member dispatch goes down the dedicated member-runtime branch instead of the normal non-PTY `exec_result.fs_diff` branch.
  - `crates/world-service/src/member_runtime.rs`: completion emits `ExecuteStreamFrame::Exit { fs_diff: None, ... }`.
  - `crates/shell/src/execution/orchestrator_world_dispatch.rs`: `execute_run_world_task_stream()` uses only `Start`, `Event`, `Exit`, and `Error`; no sync/apply step follows completion.
  - `crates/world-service/tests/member_runtime_world_placement_v1.rs`: relative write lands in overlay and is not host-visible before reconciliation.
  - `docs/USAGE.md` / `docs/internals/world/workspace_sync_filesystem_model.md`: current contract is pending diff + later sync.
- **For wrong-path/noop possibilities:**
  - no current postcondition checks exist for the requested file path.
- **For hidden retained-slot retry failure:**
  - `crates/world-service/src/member_runtime.rs`: `finish_bootstrap_preserves_retained_slot_when_session_handle_exists` test and `register_member` uniqueness by `(orchestration_session_id, world_generation, backend_id)`.

## Best discriminating logs/artifacts
To separate **prompt-noop** vs **world-only-write** vs **sync failure**, the best artifacts are:

1. **Pending diff snapshot immediately after the completed run**
   - best discriminator for “world-only write happened”.
   - if `from_the_world_worker.md` appears in pending diff, the worker wrote it and the missing host file is a sync/visibility issue, not a prompt noop.

2. **World-side file read against the active world before any cleanup/replacement**
   - read `from_the_world_worker.md` from the world view (same world/session binding).
   - present-in-world + absent-on-host = clear sync/visibility issue.

3. **`workspace sync --direction from_world --dry-run --verbose` against the same world**
   - if it plans `from_the_world_worker.md`, the write exists and is syncable.
   - if it refuses or finds nothing, that narrows toward wrong path / no write / wrong world binding.

4. **Streamed/raw agent wrapper events for that `task_run_id`**
   - best discriminator for prompt-noop vs actual attempted write when no pending diff exists.
   - current public surface is weak here; `run_world_task` does not expose a first-class dedicated trace stream, so raw wrapper/world-service logs are more valuable than shell summary text.

5. **Duplicate retained-slot / early-launch failure message on later retries**
   - if later retries fail on an already-retained backend/world slot, that supports the idea that the first run genuinely launched and surfaced continuity metadata.

## Final take
Current code/docs strongly favor this interpretation:

- the original completed `run_world_task` very plausibly did execute in the world;
- if it wrote `from_the_world_worker.md` by relative path, the file should have landed in the world overlay view first, not the host repo root;
- current one-shot `run_world_task` does **not** itself prove or apply filesystem side effects to the host;
- therefore “completed + real task_run_id + no host file” is fully consistent with the current pending-diff model and is **not** by itself evidence that the prompt was ignored;
- the highest-value next discriminator is a same-world pending-diff / world-file inspection for `from_the_world_worker.md`;
- the later “failed earlier at launch” retries are plausibly explained by the separate hidden-retained-slot lifecycle seam, not by the host-visibility seam.
