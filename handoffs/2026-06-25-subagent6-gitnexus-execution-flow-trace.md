# 2026-06-25 subagent6: GitNexus execution-flow trace

## Short objective
Trace the live `run_world_task` / world-member launch flow from host/public toolbox ingress through shell translation, world-service member dispatch, retained-member runtime, and completion/failure surfaces; identify the tightest choke point for the active symptom family.

## GitNexus queries/context/resources used
- `gitnexus status` -> initially stale (`abdc3eb` indexed vs `856e4ce` current), then refreshed to current with `gitnexus analyze . --index-only`.
- Attempted GitNexus flow queries after refresh:
  - `gitnexus query -r /Users/spensermcconnell/__Active_Code/atomize-hq/substrate -l 5 'run_world_task'`
  - `gitnexus context -r substrate run_world_task`
  - `gitnexus cypher -r substrate 'MATCH (n) RETURN count(n) AS c'`
- All three CLI graph calls hung with no output in this environment (25s+ timeout), so direct process/resource bodies were unavailable.
- GitNexus metadata/resources still used:
  - repo-local `.gitnexus/meta.json` confirmed current commit `856e4ce`, 300 indexed processes, graph/FTS available
  - `~/.gitnexus/registry.json` confirmed multiple `substrate` entries, so later commands were pinned by full path when possible
- Source-confirmation files traced after GitNexus CLI hung:
  - `crates/shell/src/repl/async_repl.rs`
  - `crates/shell/src/execution/agent_runtime/tool_invocation_contract.rs`
  - `crates/shell/src/execution/orchestrator_world_dispatch.rs`
  - `crates/shell/src/execution/routing/dispatch/world_ops.rs`
  - `crates/world-service/src/service.rs`
  - `crates/world-service/src/member_runtime.rs`
  - `docs/USAGE.md`
  - `crates/world-service/tests/member_runtime_world_placement_v1.rs`

## Exact execution-flow chain in order
1. **Host/public or toolbox ingress**
   - `decode_internal_toolbox_dispatch_request(...)` deserializes a `HostToolInvocationRequestEnvelopeV1`.
   - `handle_internal_toolbox_dispatch_request(...)` requires a live orchestrator runtime, resolves authoritative world binding, builds runtime metadata, and translates the tool call into an internal `WorldDispatchRequestV1`.
2. **Request contract**
   - `translate_host_tool_invocation_request_to_internal_dispatch_request_v1(...)` freezes the tool vocabulary.
   - For `run_world_task`, `translate_run_world_task_to_internal_dispatch_request_v1(...)` emits **ephemeral** mode with typed task payload.
   - For retained follow-up, the same contract routes exact-handle calls into `continue_world_worker` / other follow-up verbs.
3. **Shell orchestrator world dispatch**
   - `dispatch_run_world_task_request_with_started_task_run_id_tx(...)` -> `run_world_task_with_started_task_run_id_tx(...)`.
   - Shell enforces steering policy, resolves runtime contract, materializes the runtime descriptor, and prepares an active-ephemeral task record.
4. **World-member launch transport request**
   - `build_run_world_task_transport_request(...)` synthesizes a fresh ephemeral participant id (`awm_<uuid>`) plus `initial_prompt`.
   - `build_agent_client_and_member_dispatch_request_impl(...)` wraps that payload as `member_dispatch` inside `/v1/execute/stream` against the world Unix socket.
   - Retained launch uses the sibling path `build_spawn_world_worker_transport_request(...)` (`ash_<uuid>`), while retained follow-up later becomes `build_continue_world_worker_submit_request(...)`.
5. **World-service member dispatch ingress**
   - `WorldService::execute_stream(...)` sees `req.member_dispatch`, ensures the session world, resolves authoritative placement, builds member launch env, then calls `member_runtime.launch(...)`.
6. **Placement / binding gate**
   - `validate_member_dispatch_binding(...)` fail-closes on orchestration-session, `world_id`, `world_generation`, or inactive-binding mismatch.
   - `resolve_authoritative_member_placement_context(...)` picks the overlay root when full isolation is active.
   - `resolve_member_runtime_effective_cwd(...)` maps host cwd into the authoritative overlay view and fail-closes if it falls outside or is missing.
7. **World-member runtime launch**
   - `MemberRuntimeManager::launch(...)` validates the runtime binary, prepares launcher/env, requires `initial_prompt`, starts the wrapper run-control, creates `ActiveMemberRuntime`, and calls `register_member(...)` before stream frames flow.
   - `register_member(...)` keys retained ownership by `(orchestration_session_id, world_generation, backend_id)` and rejects duplicates before launch succeeds.
8. **Bootstrap event / resume-handle surfacing**
   - While wrapper events stream, `surfaced_uaa_session_id_from_data(...)` harvests `/internal/uaa_session_id` or `/session/id`.
   - `remember_uaa_session_id(...)` stores that resumable identity on the retained member.
   - On clean completion with surfaced session id, `finish_bootstrap(..., preserve_retained_member=true)` keeps the retained slot and only closes bootstrap state.
9. **Shell-side one-shot outcome handling**
   - `execute_run_world_task_stream(...)` returns the first streamed `Start { span_id }` as the active `task_run_id`, tracks `Registered` events, and then exits on terminal `Exit`.
   - `summarize_run_world_task_result(...)` appends: `backend surfaced continuity metadata but the dispatch returned terminally without retained shell state` when the backend emitted `Registered` but shell kept no retained participant record.
10. **Retained follow-up path**
   - `continue_world_worker(...)` resolves the exact retained target and builds `MemberTurnSubmitRequestV1`.
   - `MemberRuntimeManager::submit_turn(...)` requires the retained participant to still exist, validates exact retained-slot ownership, and resumes via `build_submitted_turn_run_request(...)`, which injects `agent_api.session.resume.v1` using the surfaced `uaa_session_id`.
11. **Completion / failure surfaces**
   - One-shot success: shell returns terminal `RunWorldTaskOutcomeV1` plus `task_run_id`.
   - Fresh-launch failure before any stream body: shell surfaces `failed to launch run_world_task over world member dispatch` from the `client.execute_stream(...).await` call.
   - Retained follow-up failure: world-service fails via `find_submit_target(...)`, `validate_submit_target_slot(...)`, or missing `uaa_session_id` if the retained runtime is no longer resumable.

## Top 3 choke points where the active symptom family could arise
1. **Duplicate retained-slot collision on a second fresh `run_world_task` launch**
   - `register_member(...)` rejects any existing retained slot for the same `(orchestration_session_id, world_generation, backend_id)`.
   - `duplicate_retained_member_error(...)` is exactly the kind of pre-stream failure that shell would collapse into `failed to launch run_world_task over world member dispatch`.
2. **Overlay-vs-host visibility mismatch for file side effects**
   - `docs/USAGE.md` says full-isolation internal dispatch writes land in the world view first and become host-visible only after `substrate workspace sync --direction from_world`.
   - `member_runtime_world_placement_v1.rs` explicitly asserts that a relative write must not appear on host before reconciliation.
3. **Exact binding / placement validation failure before runtime launch or follow-up resume**
   - `validate_member_dispatch_binding(...)` and `resolve_member_runtime_effective_cwd(...)` can reject stale/misaligned session/world/cwd state before launch.
   - On retained follow-up, `find_submit_target(...)` / `validate_submit_target_slot(...)` can fail if shell targets a participant that world-service no longer recognizes.

## Which choke point looks most likely after source confirmation
**Most likely: choke point #1, a split-brain duplicate retained-slot collision.**

Why:
- The first symptom (`completed` + `task_run_id` but no host-visible file) is already explainable without failure by the overlay/pending-diff contract.
- The launch path for `run_world_task` is explicitly **ephemeral on the shell side** but can still preserve retained continuity inside `world-service` when bootstrap exits `0` after surfacing `uaa_session_id`.
- A later fresh `run_world_task` for the same orchestration session/world/backend generates a new `participant_id`, but **not** a new retained-key tuple, so `register_member(...)` is the tightest pre-stream rejection point.
- That lines up with the exact shell surface: `failed to launch run_world_task over world member dispatch` occurs around `client.execute_stream(...).await`, i.e. before streamed frames are processed.

## Stale-index or graph blind spots
- GitNexus was stale at mission start and had to be refreshed first.
- Even after refresh, `gitnexus query`, `context`, and `cypher` all hung without output, so I could not fetch the intended process/resource traces directly from the graph.
- `~/.gitnexus/registry.json` shows multiple repos sharing the `substrate` alias; that ambiguity is avoidable with full repo paths, but the hang persisted even when path-pinning was used.
- `.gitnexus/meta.json` shows the graph is present and current, but `incrementalInProgress` was also populated, so this session may have caught GitNexus in a partially busy/blocked local state.

## Final take
The active symptom family does **not** currently read like a generic toolbox ingress failure or a proof that world work never happened. The most coherent end-to-end explanation is:
- toolbox/public ingress translated correctly into typed internal dispatch,
- shell launched `run_world_task` as an ephemeral member-dispatch bootstrap,
- world-service placed that member into the authoritative overlay world and likely surfaced resumable session identity,
- shell still returned a terminal one-shot receipt with no retained shell participant,
- and then a later fresh `run_world_task` launch for the same session/world/backend most likely collided with the retained slot world-service had already preserved.

So the two reported symptoms likely come from **different layers of the same seam**:
- **missing host-visible file** -> normal full-isolation reconciliation behavior unless sync occurs,
- **later failed to launch run_world_task over world member dispatch** -> most likely duplicate retained-member slot rejection before stream start.
