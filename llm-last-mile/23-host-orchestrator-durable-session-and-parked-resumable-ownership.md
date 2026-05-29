# SOW: Host Orchestrator Durable Session And Parked-Resumable Ownership

Status: implementation-oriented follow-on draft. This SOW expands and operationalizes [ADR-0047 — Host Orchestrator Durable Session and Parked-Resumable Ownership](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md) after the public caller surfaces landed in [20-public-non-interactive-agent-caller-surface.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/20-public-non-interactive-agent-caller-surface.md) and the public turn path was hardened in [22-broaden-caller-surfaces-from-repl-first-to-public-session-member-turns.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/22-broaden-caller-surfaces-from-repl-first-to-public-session-member-turns.md).

The public verbs are already the right verbs: `substrate agent start`, `substrate agent turn`, `substrate agent reattach`, and `substrate agent stop`. The missing work is not new routing syntax. The missing work is durable lifecycle truth: Substrate must own the orchestration session as the durable authority, while a Codex or equivalent backend process is only an attachable execution client that may exit cleanly without invalidating the session.

This SOW deliberately covers more than posture naming. It pulls in the ADR's full runtime-state contract, deferred host-work durability requirements, resolved `reattach` and compaction decisions, greenfield state-adoption rules, and the operator/test obligations required to make the new lifecycle model real.

## Current Design Alignment

This SOW predates the newer durable-obligation pivot frozen in:

- [DESIGN-durable-orchestration-obligation-ledger.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/DESIGN-durable-orchestration-obligation-ledger.md)
- [DESIGN-durable-orchestration-notification-inbox-contract.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/DESIGN-durable-orchestration-notification-inbox-contract.md)
- [DESIGN-auto-attach-trigger-and-work-queue-contract.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/DESIGN-auto-attach-trigger-and-work-queue-contract.md)

Read this document accordingly:

- the host-session ownership and posture model here is still directionally relevant,
- references below to per-session durable `inbox/` artifacts and `pending_inbox_count` describe the narrower landed runtime shape from this planning phase,
- the preferred forward artifact model is now one canonical session-local obligation ledger, with inbox/review and auto-attach handled as projections over the same obligation record,
- and this document should no longer be treated as the source of truth for the long-term deferred-work artifact shape.

## Objective

Make host orchestration durable across clean prompt-driven backend exits by:

- preserving the public `start|turn|reattach|stop` contract exactly as already landed,
- making the Substrate-owned orchestration session, not the attached client process, the durable authority,
- introducing explicit host postures for `active_attached`, `parked_resumable`, `awaiting_attention`, and `terminal`,
- persisting world-originated approvals, completion notices, follow-up messages, and runtime alerts as session-local durable obligations, with inbox/review handled as a projection,
- making `turn` and `reattach` resume valid parked host sessions without fuzzy recovery,
- tightening the public prompt bridge so every request that emits `Accepted` also emits an explicit `Completed` or `Failed`,
- and standardizing the internal runtime-state schema and filesystem layout needed to make those guarantees authoritative from first write.

## Why This Is Needed

The public surface already looks durable, but the runtime still behaves too much like attached process retention is the orchestration authority itself.

- Public prompt-taking is already wired through `run_start(...)`, `run_turn(...)`, `run_reattach(...)`, and `run_stop(...)` in [agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:301).
- Exact public follow-up targeting is already enforced by `resolve_public_turn_target(...)` in [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:711), which resolves one authoritative retained slot from exact `(orchestration_session_id, backend_id)`.
- The public prompt bridge already exposes `Accepted`, `Completed`, and `Failed` envelopes and submits prompt work through `run_public_prompt_command(...)` in [control.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs:1063).
- `OrchestrationSessionRecord` still only models `Allocating | Active | Invalidated | Stopping | Stopped | Failed` in [orchestration_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs:7), which is insufficient to express “valid but detached and resumable.”
- Participant liveness still leans on attachment diagnostics such as `control_owner_retained`, `event_stream_active`, `completion_observer_retained`, and `terminal_observed_at` in [session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs:81).
- The existing REPL test `start_host_orchestrator_runtime_invalidates_when_attached_control_exits()` in [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:7335) documents the current lifecycle bias directly: clean attached-client exit still tends to collapse into invalidation rather than a valid parked session.
- World follow-up already depends on resumption plumbing such as `build_session_resume_extension(...)` in [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:35), but there is still no first-class Substrate-owned durable obligation model or parked-host posture separating “no client attached right now” from “session is dead.”

The result is a contract mismatch:

- `start` and `turn` already look like durable orchestration commands,
- but clean attached-client exit can still be interpreted as lost ownership,
- and world-originated follow-up pressure still lacks a durable retained surface when no host client is currently attached.

## Relationship To Existing Slices

This SOW consumes and narrows follow-on work after existing `llm-last-mile` slices.

- [19-public-agent-control-surfaces.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/19-public-agent-control-surfaces.md) made the public lifecycle namespace explicit.
- [20-public-non-interactive-agent-caller-surface.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/20-public-non-interactive-agent-caller-surface.md) made `start`, `turn`, and `reattach` the public prompt-taking and lifecycle surface.
- [22-broaden-caller-surfaces-from-repl-first-to-public-session-member-turns.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/22-broaden-caller-surfaces-from-repl-first-to-public-session-member-turns.md) hardened exact public turn targeting and the world-member follow-up path.

This slice must not reopen:

- exact `(orchestration_session_id, backend_id)` public turn targeting,
- host-only root `start` in v1,
- detached-world fail-closed posture,
- or the Linux-first `MemberTurnSubmitRequestV1` world follow-up contract.

It only clarifies what those commands bind to, how host continuity survives clean detach, and what persisted runtime state must exist to make that durable model true.

## Scope

In scope:

- define a durable host orchestration session model that survives clean attached-client exit,
- persist explicit host posture independently from raw lifecycle state,
- add a canonical session-local durable deferred-work model under the session root,
- standardize additive session and participant metadata required for parked/resumable ownership,
- make `turn` and `reattach` resume valid parked host sessions without fuzzy routing,
- preserve detached-world fail-closed behavior while broadening host durability,
- make resolved deferred-work retention and later compaction explicit rather than implicit,
- require greenfield runtime-state writers to adopt the new session-root contract from first write,
- and update tests and operator-facing docs so the new lifecycle truth is externally provable.

## Out Of Scope

This slice does not include:

- changing the public selector contract for `turn`,
- introducing fuzzy routing, default routing, or new public selector types,
- redesigning Linux world-member follow-up away from `MemberTurnSubmitRequestV1`,
- making detached-world follow-up self-sustaining without a valid host orchestration session,
- redesigning REPL grammar,
- designing the full Substrate-native prompt/context builder,
- defining a remote multi-tenant orchestration control plane,
- generalizing the new attachment schema to hypothetical non-host attached clients,
- or turning `reattach` into a prompt-taking shortcut.

## Required Runtime Contract

### 1. Public verbs stay stable

The public contract remains:

- `substrate agent start --backend <backend_id> --prompt ...`
- `substrate agent turn --session <orchestration_session_id> --backend <backend_id> --prompt ...`
- `substrate agent reattach --session <orchestration_session_id>`
- `substrate agent stop --session <orchestration_session_id>`

This slice must not rename, broaden, or overload those verbs.

### 2. The durable authority is the orchestration session, not the attached client

After a session is created successfully, clean exit of the prompt-driven host client must not automatically mean orchestration loss. The durable authority is the Substrate-owned session record, participant state, and durable deferred-work state. In the current forward design, that deferred-work state is the session-local obligation ledger, not a standalone inbox artifact.

An attached Codex or equivalent backend process is:

- a valid execution client while attached,
- resumable if recovery metadata remains intact,
- and not the durable authority for whether the orchestration session itself remains valid.

### 3. Host posture must be explicit

The runtime must expose semantics equivalent to:

- `active_attached`
  - a host execution client is attached and can receive prompt traffic immediately
- `parked_resumable`
  - the orchestration session remains valid, but no host execution client is currently attached
- `awaiting_attention`
  - the orchestration session remains valid and has pending approvals, completion messages, follow-up messages, or runtime alerts that require host-side review or resumption
- `terminal`
  - the orchestration session is no longer routable

`awaiting_attention` may be persisted directly or derived from unresolved attention-driving obligations. The currently landed runtime may continue to surface that through `pending_inbox_count`, but that field should now be read as a narrow projection rather than the preferred long-term durable artifact.

### 4. Detached-host and detached-world rules stay distinct

- A parked host session is valid and resumable.
- Detached-world follow-up remains fail-closed.
- World follow-up must continue to route through a valid host orchestration session owned by Substrate.

This slice must not use parked-host durability as a reason to loosen detached-world fail-closed posture.

### 5. World-originated delivery must be durable without an attached client

Approvals, completion notices, follow-up messages, and runtime alerts from world-side work must materialize as Substrate-owned session-local obligations. A runtime may project those obligations into inbox/review rows or attach-processing state, but the absence of an attached client must not:

- silently drop them,
- silently consume them,
- or invalidate the session just because no client is currently attached.

### 6. Accepted implies explicit terminal delivery

Once a public prompt request emits `Accepted`, the prompt bridge must always terminate with an explicit `Completed` or `Failed` envelope. Stream disappearance, EOF, or helper exit without a terminal envelope is a runtime bug and must not be treated as the operator-facing steady state for parking.

### 7. `reattach` semantics are owner recovery only

`substrate agent reattach --session <orchestration_session_id>` is the explicit attached-owner recovery verb.

Required rules:

- `reattach` restores a live attached owner loop when recovery metadata is intact,
- `reattach` does not submit a prompt,
- `reattach` does not implicitly consume deferred review work,
- `reattach` does not act as a one-shot follow-up-turn shortcut,
- one-shot prompt-taking resume remains on `substrate agent turn --session ... --backend ... --prompt ...`.

## Internal Runtime-State Contract

This section is implementation-binding for live-state authority. It is internal runtime state, not a new operator config or policy surface.

### Canonical filesystem layout

Canonical live-state authority remains under:

- `~/.substrate/run/agent-hub/sessions/<orchestration_session_id>/session.json`
- `~/.substrate/run/agent-hub/sessions/<orchestration_session_id>/participants/<participant_id>.json`
- `~/.substrate/run/agent-hub/sessions/<orchestration_session_id>/leases/<participant_id>.lease`

This SOW originally assumed a canonical per-session `inbox/` artifact under the same session root. The forward design direction supersedes that artifact-level choice: the canonical deferred-work authority is now the obligation ledger, while any `inbox/` directory should be treated as an implementation-era projection or compatibility surface rather than the preferred long-term source of truth.

These session-root files remain the sole live-state authority from first write.

### Orchestration session record

Existing `session.json` identity and lifecycle fields remain authoritative, including:

- `orchestration_session_id`
- `shell_trace_session_id`
- `workspace_root`
- `shell_owner_pid`
- `state`
- `opened_at`
- `last_active_at`
- `orchestrator_agent_id`
- `orchestrator_backend_id`
- `orchestrator_protocol`
- `active_session_handle_id`
- `latest_run_id`
- `world_id`
- `world_generation`
- `invalidation_reason`
- `closed_at`

Required additive fields:

- `posture: active_attached|parked_resumable|awaiting_attention|terminal`
- `posture_changed_at: <timestamp>`
- `attached_participant_id: <participant_id>|null`
- `pending_inbox_count: <u64>`
- `last_parked_at: <timestamp>|null`
- `last_attention_at: <timestamp>|null`
- `parked_reason: <string>|null`

Required semantics:

- `state` remains the lifecycle state machine for allocation, active execution, stopping, failure, invalidation, and terminal completion.
- `posture` is the attachability and attention summary. It must not be inferred solely from `state`.
- `active_session_handle_id` retains its existing compatibility meaning as the authoritative orchestrator participant for the session. It is not proof that a host client is currently attached.
- `attached_participant_id` is the authoritative pointer to the currently attached host execution client. It must be `null` whenever `posture` is `parked_resumable`, `awaiting_attention`, or `terminal`.
- `pending_inbox_count` reflects the currently landed runtime's unresolved review projection. The forward design should generalize the same posture input to unresolved attention-driving obligations rather than a canonical inbox artifact.
- `posture=awaiting_attention` is required when the session remains non-terminal, `attached_participant_id=null`, and unresolved attention-driving work exists. In the current landed runtime, that may still surface as `pending_inbox_count>0`.
- `posture=parked_resumable` is required when the session remains non-terminal, `attached_participant_id=null`, no unresolved attention-driving work exists, and at least one authoritative host participant remains `resume_eligible=true`.
- `posture=terminal` must align with a non-routable session state such as `Invalidated`, `Stopped`, or `Failed`.
- `posture` is explicit persisted truth and must not be reconstructed heuristically from attachment diagnostics or legacy attachment flags.

### Participant record

Existing participant fields remain authoritative, including:

- handle and lineage fields such as `participant_id`, `orchestration_session_id`, `backend_id`, `role`, `protocol`, `state`, `world_id`, and `world_generation`
- internal fields such as `uaa_session_id`, `latest_run_id`, `cancel_supported`, `ownership_mode`, `ownership_valid`, `last_heartbeat_at`, `last_event_at`, `terminal_observed_at`, `termination_reason`, `last_error_bucket`, and `last_error_message`

Required additive internal fields for host-orchestrator participants:

- `attached_client_present: <bool>`
- `last_attached_at: <timestamp>|null`
- `last_detached_at: <timestamp>|null`
- `detach_reason: <string>|null`
- `resume_eligible: <bool>`

Required semantics:

- `uaa_session_id` remaining populated after clean client exit is valid and expected for a parked-resumable host session.
- `resume_eligible=true` with `attached_client_present=false` and a non-terminal participant `state` is a valid parked host posture, not a failure condition.
- `control_owner_retained`, `event_stream_active`, and `completion_observer_retained` remain attachment diagnostics. They must no longer be treated as the sole proof that the orchestration session itself is valid.
- These new fields are required for host-orchestrator participants. Member-runtime participants may leave them unset or at safe defaults when the semantics do not apply.
- `uaa_session_id` is an identifier and correlation field, not proof of attachment, liveness, or resumability on its own.

### Historical inbox artifact note

This section captures the narrower inbox-shaped artifact assumed by the landed runtime during this planning phase.

For future architecture and new design work, the canonical artifact is now `OrchestrationObligationV1`, and inbox/read surfaces should be treated as review projections over that ledger rather than as a second durable authority.

### Historical durable inbox item

Each unresolved or retained orchestration event must be persisted as one file under:

- `sessions/<orchestration_session_id>/inbox/<item_id>.json`

Minimum durable inbox schema:

- `schema_version: <u32>`
- `item_id: <string>`
- `orchestration_session_id: <string>`
- `kind: approval_required|completion_notice|follow_up_message|runtime_alert`
- `state: pending|acknowledged|dismissed`
- `created_at: <timestamp>`
- `resolved_at: <timestamp>|null`
- `correlation: <object>`
- `payload_schema: <string>`
- `payload: <object>`

Minimum correlation envelope:

- `source_event_type: <string>|null`
- `source_span_id: <string>|null`
- `source_cmd_id: <string>|null`
- `source_trace_session_id: <string>|null`
- `origin_participant_id: <participant_id>|null`
- `origin_backend_id: <backend_id>|null`
- `origin_run_id: <string>|null`
- `caused_by_turn_id: <string>|null`
- `workflow_id: <string>|null`
- `workflow_run_id: <string>|null`
- `workflow_node_id: <string>|null`
- `request_id: <string>|null`
- `idempotency_key: <string>|null`

Required semantics:

- `state=pending` items contribute to `pending_inbox_count`.
- Lack of an attached host client must never delete, skip, or silently consume a pending item.
- A live attached host client may observe and acknowledge items in real time, but the persisted inbox item remains the durable source of truth until it is resolved.
- Resolved items may be compacted later, but only after they have transitioned out of `pending`.
- `item_id` is the session-local durable inbox record identifier. It is not a router `request_id`, workflow run identifier, or trace span identifier.
- The correlation envelope is additive and nullable by design; fields may be unset when the source family does not define them.
- Correlation fields are join keys for trace, router, workflow, and orchestration analysis. They do not themselves grant delivery, liveness, or resumability semantics.
- The canonical naming and required/optional classification of cross-cutting correlation fields remain owned by ADR-0028 and the Phase 8 registry. This slice only requires that inbox items carry an explicit compatible correlation envelope and must not rely on heuristic joins.

### Resolved deferred-work retention and compaction contract

Resolved deferred-work artifacts or projections remain retained for audit and later compaction.

Required rules:

- resolving an inbox item removes it from authoritative pending/live counts immediately,
- resolving an inbox item must not immediately delete the persisted inbox artifact,
- resolved items should be retained until the session is terminal, the item is no longer pending, the item has aged past a bounded retention floor, and no unresolved item still depends on it for causation or debug correlation,
- compaction is a maintenance step, not the acknowledgement primitive,
- this slice fixes compaction eligibility rules but does not choose a numeric retention duration.

### Future obligation-kind growth stays additive

The forward durable outer envelope is now the obligation ledger; the inbox-shaped envelope above remains useful only as a historical implementation reference.

Required rules:

- future inbox item kinds may expand the `kind` enum only additively,
- future kinds must not introduce new ad hoc top-level envelope fields as their primary shape,
- kind-specific structure must live under `payload_schema` plus `payload`,
- future cross-cutting join/correlation additions must remain additive and defer to ADR-0028 and the Phase 8 registry instead of creating inbox-only dialects.

### Lease-file authority

Existing lease payloads may remain minimal and additive. If fast-path attachment fields are mirrored into lease files, `session.json` and `participants/<participant_id>.json` remain authoritative.

### Greenfield runtime-state adoption rules

This runtime-state schema is greenfield.

Required rules:

- new runtime-state writers must write the session-root layout and required posture/attachment fields from first write,
- readers must treat the persisted fields in this SOW as authoritative rather than reconstructing posture from heuristics,
- no config schema change and no policy schema change is required by this runtime-state contract,
- older flat session, participant, or lease layouts are not part of this contract.

## Architecture Shape

Components:

- `crates/shell` runtime/session/state layers own durable orchestration-session identity, posture state, and routing gates.
- A Substrate-owned durable obligation ledger owns world-to-host deferred-work durability when no client is attached.
- An attachable execution client, such as a Codex session, may attach, run, exit, and later resume against the same orchestration session.
- `world-agent` and the world-member submit path continue to own world-member execution and typed request submission for Linux world follow-up.

End-to-end shape:

- public `start`, `turn`, `reattach`, and `stop` operate on durable orchestration-session state,
- world-originated events are translated into durable obligations whether or not a host client is attached,
- posture is derived from explicit persisted session and participant truth, not transient attachment heuristics,
- `turn` and `reattach` resume a valid parked host session through sanctioned host paths only,
- and prompt-bridge `Accepted` responses are emitted only under a runtime invariant that guarantees explicit terminal delivery.

## Concrete Work Breakdown

### 1. Add explicit session-posture state to the live-state authority

Update the orchestration session persistence layer in [orchestration_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs) and the state-store/session writers in [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs) and [session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs) so the persisted session record gains:

- explicit `posture`,
- `posture_changed_at`,
- `attached_participant_id`,
- `pending_inbox_count`,
- `last_parked_at`,
- `last_attention_at`,
- `parked_reason`.

This work must make the persisted session record authoritative for operator-visible posture and must not keep posture as an inferred view over legacy attachment booleans.

### 2. Add host attachment metadata to participant records

Update host-orchestrator participant persistence so participant records gain:

- `attached_client_present`,
- `last_attached_at`,
- `last_detached_at`,
- `detach_reason`,
- `resume_eligible`.

This is the bridge between raw process-level attachment changes and durable parked/resume semantics. It must make clean detach a first-class valid condition rather than a failure-shaped absence.

### 3. Stop treating clean attached-client exit as automatic invalidation

Adjust the runtime and REPL lifecycle handling in [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs) so that:

- clean prompt-driven host client exit transitions a valid host session into `parked_resumable`,
- a detached parked session with unresolved attention-driving work transitions into `awaiting_attention`,
- explicit `stop`, fatal invalidation, or terminal failure still transition to `terminal`,
- retained world/member linkage is preserved when the session remains valid,
- and legacy attachment diagnostics remain diagnostics rather than sole validity authority.

The existing invalidation-on-clean-exit behavior must be replaced, not just hidden behind docs.

### 4. Add the canonical durable obligation model under the session root

Introduce the canonical deferred-work artifact under the session root. The current forward design is the session-local obligation ledger; any `inbox/` directory that exists in the landed runtime should be treated as a review projection or compatibility artifact rather than the long-term canonical model.

Minimum implementation responsibilities:

- define the canonical deferred-work envelope,
- write pending obligations durably for world-originated approvals, completions, follow-up messages, and runtime alerts,
- maintain authoritative attention-driving work counts or equivalent live-state projections,
- preserve correlation metadata needed for trace/router/workflow joins,
- and ensure the absence of an attached client never causes event loss.

### 5. Preserve resolved-item audit retention and explicit compaction rules

The deferred-work implementation must distinguish:

- acknowledgement or dismissal of an item,
- removal of the item from authoritative pending/live counts,
- and later physical compaction of historical artifacts.

Resolved items must remain inspectable until compaction eligibility rules are met. Immediate deletion is not acceptable because it weakens detached-host review and postmortem analysis.

### 6. Make `turn` and `reattach` resume valid parked host sessions

Update the public control path across [agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs), [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs), [control.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs), and [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs) so that:

- `turn` can resume prompt-taking against a valid parked host session,
- `reattach` can explicitly restore attached host ownership without submitting a prompt,
- exact `(orchestration_session_id, backend_id)` routing remains authoritative,
- and no fuzzy session or backend recovery is introduced.

### 7. Harden the public prompt bridge terminal contract

`run_public_prompt_command(...)` already models `Accepted`, `Completed`, and `Failed`. This slice must convert that into a pinned invariant:

- every path after `Accepted` ends in `Completed` or `Failed`,
- parked-host transitions do not masquerade as owner disappearance,
- helper/client exit is surfaced through explicit terminal delivery rather than silent stream loss,
- and EOF after `Accepted` remains a failure path, not a valid parked steady state.

### 8. Preserve detached-world fail-closed posture while broadening host durability

Nothing in this slice may weaken:

- root host-only `start` in v1,
- exact public `turn` targeting,
- detached-world follow-up rejection,
- or the existing Linux-first world-member contract.

The only broadened continuity is parked host continuity.

### 9. Adopt the new runtime-state schema as greenfield live-state truth

Writers must move directly to the session-root layout and additive fields from first write. Readers must trust the persisted fields rather than reconstructing posture or resumability heuristically. This work should not introduce configuration toggles or policy schema changes to stage correctness.

### 10. Update focused docs and operator surfaces

Update only the docs needed to tell the truth about:

- durable host orchestration ownership,
- parked versus attached versus attention-needed posture,
- the durable obligation ledger plus its review and attach projections,
- and the accepted-to-terminal delivery guarantee.

This SOW is itself one of those truth surfaces and should remain aligned with the ADR.

## Sequencing / Dependencies

Prerequisite context that remains binding:

- the public caller-surface hardening from `PLAN-22`,
- the host/world posture and identity-tuple rules from ADR-0042,
- the workflow/router correlation posture from ADR-0021, ADR-0028, ADR-0029, and the Phase 8 registry.

This slice should assume those decisions are already pinned and should only add the durable host-session semantics that sit on top of them.

## Security / Safety Posture

Fail-closed rules that must remain true:

- world-only root `start` remains rejected,
- detached-world public follow-up remains rejected until a sanctioned host path restores routable ownership,
- public `turn` remains exact-selector only,
- world-originated messages must not bypass Substrate-owned routing and durability surfaces.

Protected invariants:

- clean prompt-driven host client exit after session establishment must not silently degrade into session invalidation,
- accepted prompt requests must not end in silent stream loss,
- durable orchestration state must remain authoritative over any one attached client process,
- and persisted deferred-work correlation data must not be mistaken for new routing authority.

## Acceptance Criteria

This slice is done when all of the following are true:

1. `substrate agent start`, `turn`, `reattach`, and `stop` keep the same public verb and selector contract.
2. A successfully established host orchestration session can survive clean attached-client exit without being invalidated automatically.
3. The runtime persists an explicit valid parked-resumable host posture instead of forcing detached host into terminal or invalidation semantics.
4. The runtime persists an explicit `awaiting_attention` posture directly or derives it from authoritative persisted attention-driving work state.
5. `session.json` persists the additive posture fields required by this SOW.
6. Host-orchestrator participant records persist the additive attachment and resumability fields required by this SOW.
7. The canonical durable deferred-work artifact exists under the session root, and any inbox/read surface is explicitly a projection or compatibility layer rather than a second durable authority.
8. World-originated approvals, completion notices, follow-up messages, and runtime alerts are retained durably as unresolved session-local obligations when no host client is attached.
9. A live attached host client may observe or acknowledge review projections in real time, but unresolved obligations remain the durable source of truth until resolved.
10. Resolving review state removes the underlying work from pending/live attention counts immediately but does not require immediate physical deletion of historical artifacts or projections.
11. Future obligation-kind growth stays additive under the canonical envelope rather than creating ad hoc top-level variants.
12. `turn` can resume prompt-taking against a valid parked host session using exact `(orchestration_session_id, backend_id)` targeting.
13. `reattach` can restore attached host ownership for a valid parked host session without submitting a prompt.
14. Detached-world follow-up remains fail-closed until a sanctioned host path re-establishes routable ownership.
15. Host-only root `start` remains fail-closed for world-only backends.
16. Once a public prompt request emits `Accepted`, it always ends with explicit `Completed` or `Failed`.
17. Prompt-stream EOF or silent helper exit after `Accepted` is treated as a bug or failure path, not an operator-facing steady state.
18. Greenfield runtime-state writers write the new session-root layout and additive posture/attachment fields from first write.
19. Readers treat persisted posture and attachment fields as authoritative rather than reconstructing them heuristically from legacy diagnostics.
20. Existing public world-member follow-up coverage from the `PLAN-22` slice remains green as a non-regression constraint.

## Testing Expectations

Primary coverage areas:

- orchestration session posture and state transitions in [orchestration_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs)
- participant attachment and resumability state in [session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs)
- state-store persistence and exact public turn resolution in [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
- prompt-bridge terminal-delivery invariants in [control.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs)
- REPL/runtime lifecycle handling in [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
- public caller-surface behavior in [agent_public_control_surface_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs)
- world-first routing non-regression in [repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs)

Required assertions:

- `active_attached -> parked_resumable` is covered explicitly,
- `parked_resumable -> active_attached` via `reattach` is covered explicitly,
- `parked_resumable -> awaiting_attention` is covered explicitly or proven through authoritative attention-driving work derivation,
- legal transitions to `terminal` are covered explicitly,
- host `start` creates a valid orchestration session and clean host client exit does not invalidate it,
- host `turn` succeeds against a valid parked session,
- `reattach` succeeds against a valid parked session and does not submit a prompt,
- world-originated retained work survives detached host periods,
- pending/live counts update correctly when review projections are acknowledged or dismissed,
- resolved items remain retained until compaction eligibility is met,
- detached-world follow-up still fails closed,
- existing exact public turn routing remains authoritative,
- and every accepted prompt request ends in `Completed` or `Failed`.

Recommended commands:

```bash
cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture
cargo test -p shell agent_runtime::control -- --nocapture
cargo test -p shell agent_runtime::state_store -- --nocapture
cargo test -p shell async_repl -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
```

## Manual Validation

Re-run the public host flow outside the integration harness:

```bash
substrate agent start --backend <host_backend_id> --prompt "hello" --json
substrate agent turn --session <orchestration_session_id> --backend <host_backend_id> --prompt "next" --json
substrate agent reattach --session <orchestration_session_id> --json
```

Confirm all of the following manually:

- the host session parks cleanly when the prompt-driven client exits,
- the parked session remains resumable rather than invalidated,
- world-originated messages create durable obligations and corresponding review projections even while no client is attached,
- `reattach` restores attached ownership without consuming review work implicitly,
- and no post-`Accepted` stream ends without `Completed` or `Failed`.
