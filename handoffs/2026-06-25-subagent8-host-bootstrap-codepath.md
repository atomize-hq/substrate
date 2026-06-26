# Host orchestrator -> first world-binding bootstrap investigation

Date: 2026-06-25
Branch: feat/internal-host-orchestrator-world-dispatch-bootstrap
Mode: read-only investigation

## Question
Where is host-orchestrator -> first world-binding bootstrap prevented, and is that a temporary slice gate or the wrong architecture hard-coded?

## Short answer
Current code intentionally prevents fresh toolbox `run_world_task` / `spawn_world_worker` from a host-only session before any authoritative session `world_id` + `world_generation` exist.

That prevention happens in the host-toolbox adapter before dispatch translation, and again in the dispatch validator.

The only shipped first-binding bootstrap is public `substrate agent start --scope world`, which is host-first: it pre-opens a shared world, captures authoritative binding proof, persists that binding onto the host orchestration session, then launches the host owner-helper. It does **not** eagerly allocate a world member.

So the thread has merit as a real missing bridge between host durable authority and world-dispatch actionability. It does **not** look like an accidental one-off hard-code. The docs, prompt contract, and start path all agree on this gate.

## Exact host-only toolbox -> `missing_world_binding` path
1. The toolbox server is the per-session UDS registered by `register_internal_toolbox_transport_for_session(...)` in `crates/shell/src/repl/async_repl.rs:5031-5076`.
2. Incoming JSON is decoded into `HostToolInvocationRequestEnvelopeV1` by `decode_internal_toolbox_dispatch_request(...)` in `crates/shell/src/repl/async_repl.rs:5672-5681`.
3. `handle_internal_toolbox_dispatch_request(...)` immediately snapshots the live orchestration session and asks for an authoritative session world binding via `authoritative_world_binding_for_session_v1(...)` in `crates/shell/src/repl/async_repl.rs:5531-5561`.
4. That resolves to `authoritative_world_binding(...)` in `crates/shell/src/execution/agent_runtime/tool_invocation_contract.rs:1263-1281`, which hard-fails if `session.world_id` or `session.world_generation` is missing, returning:
   - `missing_world_binding: orchestration session <id> has no authoritative world binding`
5. If binding exists, the adapter translates the host tool call into `WorldDispatchRequestV1` in `translate_host_tool_invocation_request_to_internal_dispatch_request_v1(...)` at `crates/shell/src/execution/agent_runtime/tool_invocation_contract.rs:1063-1105`, injecting `world_id` / `world_generation` into the request via `build_dispatch_request_v1(...)` at `:1014-1055`.
6. The resulting `WorldDispatchRequestV1` itself requires `world_id` and `world_generation` in `crates/shell/src/execution/agent_runtime/dispatch_contract.rs:427-470`.
7. Dispatch then resolves the authoritative caller in `prepare_orchestrator_world_dispatch(...)` at `crates/shell/src/execution/orchestrator_world_dispatch.rs:429-455`, and the world-dispatch layer independently re-validates the same binding in `validate_authoritative_world_binding(...)` at `:2387-2418`.

## Exact public world-start binding attach path
1. `substrate agent start --scope world ...` enters `run_start(...)` in `crates/shell/src/execution/agents_cmd.rs:311-452`.
2. The start plan is built by `build_world_start_session_birth_plan(...)` in `crates/shell/src/execution/agents_cmd.rs:1349-1433`.
   - Important: this builds a **host** owner-helper launch plan. The helper session starts with `world_id: None` and `world_generation: None`.
3. Before launching the helper, `run_start(...)` calls `establish_public_world_start_binding(...)` in `crates/shell/src/execution/agents_cmd.rs:324-341`.
4. `establish_public_world_start_binding_async(...)` opens a shared world using `SharedWorldOwnerSpec { action: AttachOrCreate }` in `crates/shell/src/execution/agents_cmd.rs:1451-1491` and extracts authoritative binding proof from `ready.shared_world`.
5. That proof is stuffed back into the helper plan at `crates/shell/src/execution/agents_cmd.rs:339-340`.
6. When the hidden owner-helper starts, `run_hidden_owner_helper(...)` passes that binding as `initial_world_binding` in `crates/shell/src/repl/async_repl.rs:3298-3341`.
7. Host runtime startup persists the binding onto the orchestration session through `persist_world_binding_authority(...)` in `crates/shell/src/repl/async_repl.rs:3466-3481`, implemented by `set_orchestration_session_world_binding(...)` in `crates/shell/src/execution/agent_runtime/control.rs:1640-1664` and `crates/shell/src/execution/agent_runtime/state_store.rs:3588-3609`.

## Why this is host-first, not eager member allocation
- The world-scoped public start plan launches a host orchestrator, not a world member, in `crates/shell/src/execution/agents_cmd.rs:1374-1428`.
- No member runtime is allocated during `run_start(...)`.
- World members are only launched later through internal world-dispatch or retained-member paths, for example `spawn_world_worker` in `crates/shell/src/repl/async_repl.rs:5754-5834` or `ensure_member_runtime_ready_for_descriptor(...)` in `crates/shell/src/repl/async_repl.rs:7134-7223`.

## Missing bridge, exactly
There is no path that says:
- "I have a durable host orchestration session"
- "please allocate or attach the first shared world for this session"
- "persist that binding"
- "then continue with `run_world_task` or `spawn_world_worker`"

Instead, current code requires binding *before* tool translation.

That missing bridge is visible in the layering:
- toolbox tool translation requires session binding first: `tool_invocation_contract.rs:1263-1281`, `:1063-1105`
- shell dispatch request validation requires `world_id` / `world_generation`: `dispatch_contract.rs:427-470`
- world-service member dispatch still requires exact world identity and active binding: `crates/world-service/src/service.rs:2640-2691`

## Important nuance in world-service
`world-service` is already shared-world aware:
- `requested_shared_world_owner_spec(...)` in `crates/world-service/src/service.rs:2590-2598` synthesizes `SharedWorldOwnerSpec::AttachOrCreate` from a member-dispatch request.

But that does **not** solve first bootstrap, because the shell-side `MemberDispatchRequestV1` still has to arrive with exact `world_id` and `world_generation`, and world-service validates those exact fields in `crates/world-service/src/service.rs:2640-2691`.

So the bridge is not missing in the world backend. It is missing in the host control-plane step that would obtain and persist the first authoritative binding before forming dispatch.

## Stale-binding clue
There is also a real drift hazard.

Shell-side world-dispatch actionability uses the persisted session binding as authority first, then world-service validates against the live shared world later. There is no pre-dispatch refresh step.

Evidence:
- toolbox gating reads the session snapshot directly: `async_repl.rs:5553-5561`
- member launch requires session binding to match the active world session in `authoritative_member_world_binding(...)` at `crates/shell/src/repl/async_repl.rs:5983-6022`
- world-service independently re-validates against live binding in `crates/world-service/src/service.rs:2640-2691`

Binding is refreshed on explicit world restart flows, for example `restart_world_session(...)` persists the replacement binding in `crates/shell/src/repl/async_repl.rs:8851-8898`, but ordinary toolbox dispatch does not perform such a refresh.

## "Exactly one live orchestrator parent" clue
Later member-launch paths really do require a single live parent orchestrator.

`resolve_live_member_parent(...)` in `crates/shell/src/repl/async_repl.rs:5950-5979` requires exactly one live orchestrator parent in the current orchestration session, and `prepare_member_runtime_startup_for_descriptor(...)` calls it before member launch in `:6110-6180`.

Status/env surfaces also fail closed on ambiguity via `resolve_single_live_session_for_agent(...)` and `resolve_live_orchestrator_participant(...)` in `crates/shell/src/execution/agent_runtime/state_store.rs:1184-1281` and `:2525-2535`.

## Verdict
This is a real gap, but the current behavior looks like an intentional slice gate, not a random hard-coded architectural mistake.

Why:
- the prompt contract says host-only sessions cannot bootstrap first binding through tools: `crates/shell/src/execution/prompt_fulfillment.rs:95-106`
- the usage docs say the same thing: `docs/USAGE.md:92-95`
- the public bootstrap path is explicitly `agent start --scope world`, and it persists binding before the host helper starts: `agents_cmd.rs:324-341`, `1451-1491`, `async_repl.rs:3466-3481`

## Most likely concrete misunderstanding
The implementation currently treats:
- durable host authority, and
- authoritative world targeting

as two separate prerequisites, but only ships a public bootstrap for the second one.

That is not the host orchestrator being "the wrong surface". The more precise problem is that the internal host-toolbox control plane has no first-binding acquisition step, even though the product question naturally expects one.

## Recommendation
If the desired product is "host orchestrator can originate the first world dispatch", add one explicit internal bootstrap seam before `run_world_task` / `spawn_world_worker`.

Best shape:
- an internal `ensure_world_binding` / `attach_or_create_world_binding` control-plane step that:
  1. uses orchestration session durable authority,
  2. acquires shared-world proof,
  3. persists `world_id` / `world_generation`,
  4. only then allows existing world-dispatch verbs.

That keeps the fail-closed dispatch contract intact and avoids overloading `run_world_task` with hidden session-mutation side effects.
