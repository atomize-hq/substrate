# 2026-06-25 subagent1: run_world_task transport and dispatch

## Short objective
Trace the live `run_world_task` path from the injected toolbox Unix socket through shell translation, world-service member dispatch, and world-member launch; explain (a) how a run can return `completed` yet produce no host-visible file, and (b) where the immediate `failed to launch run_world_task over world member dispatch` retry most likely comes from.

## Exact files / symbols / commands inspected
- Context files:
  - `handoffs/2026-06-21-135249-world-agent-run-world-task-auth-seeding.md`
  - `handoffs/2026-06-21-154318-world-agent-run-world-task-model-config.md`
  - `handoffs/2026-06-22-211455-spec-62-world-worker-retained-debug.md`
  - `handoffs/2026-06-23-retained-world-worker-lifecycle-debug.md`
  - `llm-last-mile/SPEC-63-retained-world-worker-parked-resume-session-handle-contract.md`
  - `llm-last-mile/PLAN-63-retained-world-worker-parked-resume-session-handle-contract.md`
  - `llm-last-mile/TASKS-63.md`
  - `llm-last-mile/DESIGN-world-worker-lifecycle-model.md`
- Source files / symbols:
  - `crates/shell/src/repl/async_repl.rs`
    - `handle_internal_toolbox_dispatch_request`
    - `decode_internal_toolbox_dispatch_request`
  - `crates/shell/src/execution/agent_runtime/tool_invocation_contract.rs`
    - `HostToolInvocationRequestEnvelopeV1`
    - `translate_host_tool_invocation_request_to_internal_dispatch_request_v1`
    - `translate_run_world_task_to_internal_dispatch_request_v1`
    - `normalize_run_world_task_receipt_v1`
  - `crates/shell/src/execution/orchestrator_world_dispatch.rs`
    - `dispatch_run_world_task_request_with_started_task_run_id_tx`
    - `run_world_task_with_started_task_run_id_tx`
    - `resolve_world_dispatch_contract`
    - `build_run_world_task_transport_request`
    - `execute_run_world_task_stream`
    - `summarize_run_world_task_result`
    - `ActiveEphemeralTerminalTruthGuard`
  - `crates/shell/src/execution/routing/dispatch/world_ops.rs`
    - `build_agent_client_and_member_dispatch_request_impl`
    - `build_member_dispatch_payload`
  - `crates/world-service/src/service.rs`
    - `execute_stream`
    - `resolve_authoritative_member_placement_context`
    - `validate_member_dispatch_binding`
    - `resolve_member_runtime_effective_cwd`
  - `crates/world-service/src/member_runtime.rs`
    - `MemberRuntimeManager::launch`
    - `MemberRuntimeManager::submit_turn`
    - `finish_bootstrap`
    - `register_member`
    - `find_submit_target`
    - `remember_uaa_session_id`
    - `surfaced_uaa_session_id_from_data`
    - `build_submitted_turn_run_request`
    - `duplicate_retained_member_error`
    - `prepare_codex_runtime_env`
  - `crates/world-service/tests/member_runtime_retained_lifecycle_v1.rs`
  - `crates/world-service/tests/member_runtime_world_placement_v1.rs`
  - `docs/USAGE.md`
- GitNexus / shell commands used:
  - `npx gitnexus status`
  - `npx gitnexus query -r substrate "run_world_task world member dispatch"`
  - `npx gitnexus context -r substrate -f crates/shell/src/execution/routing/dispatch/world_ops.rs build_agent_client_and_member_dispatch_request_impl`
  - `rg -n ...` across `crates/shell`, `crates/world-service`, `handoffs`, `docs`
  - targeted `sed -n` reads for the symbols above

## Concrete evidence
- Toolbox ingress is a typed JSON envelope, not an ad hoc path:
  - `async_repl.rs` decodes `HostToolInvocationRequestEnvelopeV1 { version, tool_name, tool_call_id?, arguments }`, resolves authoritative world binding, then translates the request with `translate_host_tool_invocation_request_to_internal_dispatch_request_v1`.
- Linux `run_world_task` takes a special async path that streams a `task_run_id` early:
  - `handle_internal_toolbox_dispatch_request` spawns `dispatch_run_world_task_request_with_started_task_run_id_tx(...)` and forwards the first streamed member `Start { span_id }` frame as the toolbox `task_run_id` before terminal outcome normalization.
- Shell-side dispatch is explicitly ephemeral, but world-service launch still uses member-runtime registration machinery:
  - `build_run_world_task_transport_request` synthesizes a fresh `participant_id` (`awm_<uuid>`) and `initial_prompt`, then `build_agent_client_and_member_dispatch_request_impl` wraps it as `member_dispatch` inside `/v1/execute/stream` against the world Unix socket.
- World-service validates the shared world and places the member inside the authoritative overlay/world view:
  - `validate_member_dispatch_binding` requires matching `orchestration_session_id`, `world_id`, `world_generation`, and active shared binding.
  - `resolve_authoritative_member_placement_context` uses the overlay root when full isolation is active, and `resolve_member_runtime_effective_cwd` maps repo cwd into that overlay root.
- A `run_world_task` launch can preserve retained runtime state even though the shell only returns a terminal one-shot outcome:
  - `MemberRuntimeManager::launch` registers the member before streaming.
  - It records `uaa_session_id` from `/internal/uaa_session_id` or `/session/id`.
  - If completion exit is `0` **and** a session id was surfaced, `preserve_retained_member` becomes true and `finish_bootstrap()` only closes bootstrap; it does **not** unregister the member.
  - Shell-side `execute_run_world_task_stream` merely sets `saw_registered_event = true`; it never persists retained participant state for this path.
  - `summarize_run_world_task_result` therefore appends: `backend surfaced continuity metadata but the dispatch returned terminally without retained shell state`.
- The repo already encodes that host-visible file absence is normal under full isolation unless reconciliation occurs:
  - `docs/USAGE.md`: relative internal world-dispatch writes land in the world view first and become host-visible only after `substrate workspace sync --direction from_world` (or equivalent).
  - `member_runtime_world_placement_v1.rs`: `relative write must not appear on host before workspace sync reconciliation`.
- The immediate retry failure string is consistent with a pre-stream launch rejection inside world-service, not a later streamed member error:
  - shell wraps `client.execute_stream(execute_request).await` with `context("failed to launch run_world_task over world member dispatch")`.
  - if `member_runtime.launch` rejects before returning a stream response, transport returns non-success HTTP and shell emits exactly that message.
- The most plausible pre-stream rejection after the first “completed + continuity metadata” run is retained-slot collision inside `register_member(...)`:
  - `register_member` keys retained members by `(orchestration_session_id, world_generation, backend_id)`.
  - `duplicate_retained_member_error` says: `a retained world member is already active for orchestration_session_id ... world_generation ... backend_id ...`.
  - `build_run_world_task_transport_request` generates a new `participant_id` each retry, but the retained-key tuple stays the same for the same session/world/backend.
  - So a first run that preserved a retained member can cause the second fresh `run_world_task` launch to fail immediately before any stream frame is emitted.

## Most likely failure seam(s), ranked
1. **Ephemeral `run_world_task` leaking into preserved retained slot state in world-service**
   - First run likely surfaced a registered/session-handle event, exited `0`, and therefore stayed retained in `world-service` only.
   - Shell intentionally did not retain that state, so the path ended “terminally without retained shell state”.
   - Second run then likely hit `register_member` duplicate retained-slot rejection for the same session/world/backend.
2. **Full-isolation overlay/reconciliation mismatch, not task non-execution, explains the missing host file**
   - Current repo truth says relative writes land in overlay and require explicit sync to appear host-side.
   - So `from_the_world_worker.md` missing at host repo root does not by itself mean the first run failed.
3. **Less likely: another pre-stream launch failure inside `member_runtime.launch` / direct wrapper bootstrap**
   - e.g. binary/config/env prep failure.
   - This is weaker because the first run already reached terminal success with continuity metadata, which strongly suggests the core bootstrap path worked at least once in this session.

## What this seems to rule out
- **Not** a missing toolbox transport or envelope decode problem: the first run returned a valid `task_run_id` and normalized terminal outcome.
- **Not** a missing world binding / backend allowlisting problem on the failing retry path: current symptom notes say the host session still showed valid world binding / `cli:codex-world` allowance and parked cleanly.
- **Not** evidence that “world writes are broken” in general: repo docs/tests and the June 22 handoff already separate overlay persistence from host reconciliation.
- **Not** primarily a `submit_turn` routing bug for this specific retry symptom: the retry error happens on fresh `run_world_task` launch before the retained follow-up seam is even used.

## Next 3 highest-value checks
1. **Capture the exact `/v1/execute/stream` error body for the second retry**
   - Goal: confirm world-service returned the duplicate retained-slot message from `duplicate_retained_member_error(...)`.
2. **After the first “completed + continuity metadata” run, inspect whether a retained world member is still live in world-service while shell/session state lacks a matching retained participant record**
   - Goal: prove the shell/world-service state split directly.
3. **Re-run the file-write smoke with a relative target plus explicit `from_world` reconciliation (or pending-diff inspection) instead of checking host root immediately**
   - Goal: separate “task did no work” from “work landed only in overlay”.

## Final take
Current repo truth points to a split-brain seam, not a generic transport outage. The toolbox UDS request is getting through, shell is translating it into a typed ephemeral `member_dispatch`, and world-service is launching through the direct member-runtime path. On at least one successful first run, that member likely surfaced a resumable/session-handle event and exited cleanly, which makes `world-service` preserve a retained slot keyed by session/world/backend. But the shell’s `run_world_task` path deliberately remains a terminal one-shot path and does not retain that worker in shell state, so the result can honestly say “completed” while also noting “continuity metadata ... without retained shell state.” That same preserved world-service slot then makes the next fresh `run_world_task` launch for the same session/world/backend the most likely duplicate-retained-member collision, which shell surfaces only as `failed to launch run_world_task over world member dispatch`. Separately, the missing host-side file is already explainable by the repo’s full-isolation overlay contract: no reconciliation, no host-visible relative file yet.
