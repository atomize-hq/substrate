# ADR-0047 — Host Orchestrator Durable Session and Parked-Resumable Ownership

## Status
- Status: Draft
- Date (UTC): 2026-05-09
- Owner(s): Spenser McConnell (Substrate)

## Scope
- Feature directory: `docs/project_management/packs/draft/host-orchestrator-durable-session-and-parked-resumable-ownership/`
- Sequencing spine: `docs/project_management/packs/sequencing.json`
- Standards:
  - `docs/project_management/system/standards/adr/EXECUTIVE_SUMMARY_STANDARD.md`

## Related Docs (links only)

This ADR is a lifecycle correction for host orchestration. It does not replace the existing public caller surfaces; it clarifies what those surfaces bind to and what runtime durability they require.

- Foundational runtime / ownership ADRs:
  - `docs/adr/implemented/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md`
  - `docs/adr/implemented/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md`
- Public caller-surface and execution-plan context:
  - `llm-last-mile/PLAN-22.md`
  - `llm-last-mile/ORCH_PLAN-22.md`
  - `llm-last-mile/22-broaden-caller-surfaces-from-repl-first-to-public-session-member-turns.md`
- Existing runtime hotspots and observed failure seam:
  - `crates/shell/src/execution/agents_cmd.rs`
  - `crates/shell/src/execution/agent_runtime/control.rs`
  - `crates/shell/src/execution/agent_runtime/session.rs`
  - `crates/shell/src/execution/agent_runtime/state_store.rs`
  - `crates/shell/src/repl/async_repl.rs`
- Future-compatible workflow/router correlation context:
  - `docs/project_management/adrs/draft/ADR-0029-host-event-bus-and-router-daemon.md`
  - `docs/project_management/adrs/draft/ADR-0021-substrate-workflow-engine.md`
  - `docs/project_management/packs/PHASE_8_CROSS_CUTTING_DECISION_REGISTRY.md`
- Existing proof surfaces:
  - `crates/shell/tests/agent_public_control_surface_v1.rs`
  - `crates/shell/tests/repl_world_first_routing_v1.rs`

## Executive Summary (Operator)

ADR_BODY_SHA256: ebae56fd591e1cffea1d3e8262ed660fcf9123ef5d1574b7a3a98783525154cc

### Changes (operator-facing)
- Clarify that a host orchestration session is durable even when the current Codex wrapper process is not
  - Existing: host orchestration readiness and liveness are treated as if a transient helper process must remain continuously attached for the orchestration session to stay valid.
  - New: the durable unit is the Substrate-owned orchestration session plus its inbox/task state. A Codex session is an attachable execution client, not the durable orchestration authority itself.
  - Why: the intended `codex exec`-style launch model is prompt-driven and lazy. Requiring a continuously running attached client reintroduces the old bootstrap-first assumption that the current public surface intentionally moved away from.
  - Links:
    - `crates/shell/src/execution/agent_runtime/session.rs`
    - `crates/shell/src/repl/async_repl.rs`
- Preserve the current public CLI surface while redefining the meaning of host session persistence
  - Existing: `substrate agent start` and `substrate agent turn` are already the canonical public prompt-taking surfaces, but the runtime still tends to interpret clean helper exit as owner loss or invalidation.
  - New: `start`, `turn`, and `reattach` remain the public surface, but they bind to durable orchestration state with explicit attached, parked, and attention-needed postures.
  - Why: this keeps the public contract stable while correcting the underlying ownership model.
  - Links:
    - `llm-last-mile/PLAN-22.md`
    - `crates/shell/src/execution/agent_runtime/control.rs`
- Tighten the prompt-stream delivery contract after `Accepted`
  - Existing: the bridge can emit `Accepted` and later surface EOF because the owner-side control stream ended before a terminal envelope was emitted.
  - New: once `Accepted` is emitted, the request must end with an explicit terminal envelope (`Completed` or `Failed`) on every path.
  - Why: `Accepted` without terminal-delivery guarantees is an unstable public prompt contract and makes orchestrator parking indistinguishable from owner death.
  - Links:
    - `crates/shell/src/execution/agent_runtime/control.rs`

## Problem / Context
- The current host orchestrator path still carries an assumption from an older bootstrap model:
  - if the helper process becomes authoritative-live and later its control stream ends, the orchestration session is at risk of being invalidated as though durable ownership was lost.
- That assumption conflicts with the intended launch posture:
  - REPL launch should not bootstrap a fake backend session with a dummy prompt.
  - a host backend session should be created only when the user actually runs a prompt-taking action such as `::cli:codex ...` or `substrate agent start --backend ... --prompt ...`.
  - the resulting backend process may behave like `codex exec`, meaning it is naturally prompt-driven and not guaranteed to remain alive indefinitely after the prompt completes.
- The deeper requirement is not "keep the Codex process alive forever." The real requirement is:
  - the host orchestration session must remain durable,
  - world-launched work must still be able to report end messages, approval needs, and follow-up state back into Substrate,
  - and Substrate must be able to resume or reattach an execution client later without treating the orchestration session itself as lost.
- This implies a missing architectural boundary:
  - Substrate needs a durable orchestration session and delivery surface that outlives any one Codex wrapper process.
  - world-originated messages cannot depend on a particular attached client process still being alive.

## Goals
- Make the Substrate-owned orchestration session the durable authority for host orchestration.
- Treat Codex or equivalent backend sessions as attachable execution clients rather than the durable orchestrator identity.
- Define an explicit parked/resumable host posture that is valid and routable, not an implicit invalidation state.
- Preserve the existing public CLI contract:
  - `substrate agent start` remains the canonical public root prompt-taking surface.
  - `substrate agent turn` remains the canonical public follow-up surface.
  - `substrate agent reattach` remains the canonical explicit reattachment surface.
- Require post-`Accepted` terminal-delivery guarantees for the public prompt bridge.
- Preserve strict world posture rules:
  - root `start` remains host-only in v1,
  - world-only root start stays fail-closed,
  - detached-world follow-up remains fail-closed until a valid host owner is re-established through the sanctioned path.
- Create an architecture shape that will remain compatible with a future Substrate-native context/prompt builder.

## Non-Goals
- Changing the public selector contract for `turn`; exact `(orchestration_session_id, backend_id)` remains authoritative.
- Introducing fuzzy routing, default routing, or new public selector types.
- Changing Linux world-member follow-up away from `MemberTurnSubmitRequestV1` and `/v1/member_turn/stream`.
- Replacing the current world follow-up ownership rules with direct world-to-world public routing.
- Designing the full native prompt builder in this ADR.
- Defining a remote multi-tenant orchestration control plane.

## User Contract (Authoritative)

### CLI
- Commands:
  - `substrate agent start --backend <backend_id> --prompt ...` remains the canonical public root prompt-taking surface.
  - `substrate agent turn --session <orchestration_session_id> --backend <backend_id> --prompt ...` remains the canonical public follow-up surface.
  - `substrate agent reattach --session <orchestration_session_id>` remains the canonical explicit host reattachment surface.
  - `substrate agent stop --session <orchestration_session_id>` remains the canonical explicit shutdown surface.
- Root prompt rules:
  - root `start` remains host-only in v1.
  - world-only backends must fail closed as root-start targets.
- Follow-up rules:
  - public `turn` requires exact `--session <orchestration_session_id>` and exact `--backend <backend_id>`.
  - no fuzzy session routing, no fuzzy backend routing, and no latest-session fallback are introduced here.
- Delivery guarantee:
  - once a public prompt request emits `Accepted`, it MUST terminate with an explicit terminal envelope from the Substrate-owned bridge.
  - EOF or stream disappearance without a terminal envelope is a runtime bug, not an operator-facing steady-state contract.

### Session posture contract
- `active_attached`
  - A host execution client is currently attached to the orchestration session and can actively receive prompt traffic and interactive work.
- `parked_resumable`
  - The orchestration session remains valid.
  - No host execution client is currently attached.
  - Substrate retains durable routing state, inbox state, and enough session identity to resume host execution later.
  - This is the normal state after a clean prompt-driven host client exit when orchestration remains live.
- `awaiting_attention`
  - The orchestration session remains valid but has pending world-originated work, completion messages, or approval requirements that require host-side review or resumption.
- `terminal`
  - The orchestration session is closed, invalidated, or fully completed and can no longer accept prompt or orchestration traffic.

### Detached-host versus detached-world rules
- Host:
  - a parked host session is valid and resumable.
  - clean host client exit after valid session establishment must not be treated as equivalent to fatal owner loss.
- World:
  - detached-world follow-up remains fail-closed.
  - world follow-up must not broaden into self-sustaining public continuity without a valid host orchestration session that remains routable through Substrate.

### Event delivery contract
- World-originated approvals, completion notices, and other orchestration-relevant events must land in a durable Substrate-owned inbox, queue, or task ledger that outlives any attached Codex client process.
- A live attached client may consume from that inbox in real time.
- If no client is attached, Substrate must retain the event and expose the need for resume/reattach rather than losing or invalidating the orchestration session.

## Internal Runtime-State Schema (Authoritative, internal)

This section defines the minimum retained runtime-state schema required to implement the durable host-orchestration contract in this ADR.

This is an internal persisted runtime-state contract. It is not a config/policy surface, does not add operator-settable keys, and does not change the file-family ownership defined by ADR-0027.

### Canonical filesystem layout
- Canonical live-state authority remains under:
  - `~/.substrate/run/agent-hub/sessions/<orchestration_session_id>/session.json`
  - `~/.substrate/run/agent-hub/sessions/<orchestration_session_id>/participants/<participant_id>.json`
  - `~/.substrate/run/agent-hub/sessions/<orchestration_session_id>/leases/<participant_id>.lease`
- This ADR adds a canonical durable inbox path under the same session root:
  - `~/.substrate/run/agent-hub/sessions/<orchestration_session_id>/inbox/<item_id>.json`
- The session-root files above are the sole live-state authority from first write.

### Orchestration session record
- Existing `session.json` fields remain authoritative:
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
- Required additive fields:
  - `posture: active_attached|parked_resumable|awaiting_attention|terminal`
  - `posture_changed_at: <timestamp>`
  - `attached_participant_id: <participant_id>|null`
  - `pending_inbox_count: <u64>`
  - `last_parked_at: <timestamp>|null`
  - `last_attention_at: <timestamp>|null`
  - `parked_reason: <string>|null`
- Required semantics:
  - `state` remains the lifecycle state machine for allocation, active execution, stopping, failure, invalidation, and terminal completion.
  - `posture` is the attachability and attention summary. It must not be inferred solely from `state`.
  - `active_session_handle_id` retains its existing compatibility meaning: the authoritative orchestrator participant for the orchestration session. It is not proof that a host client is currently attached.
  - `attached_participant_id` is the authoritative pointer to the currently attached host execution client. It must be `null` whenever `posture` is `parked_resumable`, `awaiting_attention`, or `terminal`.
  - `pending_inbox_count` counts unresolved inbox items for the orchestration session.
  - `posture=awaiting_attention` is required when the session remains non-terminal, `attached_participant_id=null`, and `pending_inbox_count>0`.
  - `posture=parked_resumable` is required when the session remains non-terminal, `attached_participant_id=null`, `pending_inbox_count=0`, and at least one authoritative host participant remains `resume_eligible=true`.
  - `posture=terminal` must align with a non-routable session state such as `Invalidated`, `Stopped`, or `Failed`.
  - `posture` is explicit persisted truth and must not be reconstructed heuristically from attachment diagnostics or legacy attachment flags.

### Participant record
- Existing participant fields remain authoritative:
  - handle fields such as `participant_id`, `orchestration_session_id`, `backend_id`, `role`, `protocol`, `state`, lineage links, `world_id`, and `world_generation`
  - internal fields such as `uaa_session_id`, `latest_run_id`, `cancel_supported`, `ownership_mode`, `ownership_valid`, `last_heartbeat_at`, `last_event_at`, `terminal_observed_at`, `termination_reason`, `last_error_bucket`, and `last_error_message`
- Required additive internal fields for host-orchestrator participants:
  - `attached_client_present: <bool>`
  - `last_attached_at: <timestamp>|null`
  - `last_detached_at: <timestamp>|null`
  - `detach_reason: <string>|null`
  - `resume_eligible: <bool>`
- Required semantics:
  - `uaa_session_id` remaining populated after clean client exit is valid and expected for a parked-resumable host session.
  - `resume_eligible=true` with `attached_client_present=false` and a non-terminal participant `state` is a valid parked host posture, not a failure condition.
  - `control_owner_retained`, `event_stream_active`, and `completion_observer_retained` remain attachment diagnostics. They must no longer be treated as the sole proof that the orchestration session itself is valid.
  - These new attachment fields are required for host-orchestrator participants. Member-runtime participants may leave them unset or at safe defaults when the semantics do not apply.
  - `uaa_session_id` is an identifier and correlation field, not proof of attachment, liveness, or resumability on its own.

### Durable inbox item
- Each unresolved or retained orchestration event must be persisted as one file under:
  - `sessions/<orchestration_session_id>/inbox/<item_id>.json`
- Minimum item schema:
  - `orchestration_session_id: <string>`
  - `item_id: <string>`
  - `kind: approval_required|completion_notice|follow_up_message|runtime_alert`
  - `state: pending|acknowledged|dismissed`
  - `created_at: <timestamp>`
  - `updated_at: <timestamp>`
  - `resolved_at: <timestamp>|null`
  - `message: <string>|null`
- Required semantics:
  - `state=pending` items contribute to `pending_inbox_count`.
  - Lack of an attached host client must never delete, skip, or silently consume a pending item.
  - A live attached host client may observe and acknowledge items in real time, but the persisted inbox item remains the durable source of truth until it is resolved.
  - Resolved items may be compacted later, but only after they have transitioned out of `pending`.
  - `item_id` is the session-local durable inbox record identifier. It is not a router `request_id`, workflow run identifier, or trace span identifier.
  - The currently landed schema is intentionally minimal. Richer correlation or payload envelopes remain additive follow-on work rather than implied current-state truth.

### Lease file
- Existing lease payloads may remain minimal and additive.
- If additive attachment fields are mirrored into lease files for fast-path checks, `session.json` and `participants/<participant_id>.json` remain authoritative.

### Greenfield runtime-state adoption rules
- This runtime-state schema is greenfield.
- Older flat session, participant, or lease layouts are not part of the contract.
- New runtime-state writers MUST write the session-root layout and required posture/attachment fields from first write.
- Readers MUST treat the persisted fields in this section as authoritative rather than reconstructing posture from attachment heuristics.
- No config schema change and no policy schema change is required by this runtime-state schema.

## Resolved Design Decisions

These decisions are no longer open in this ADR. They are pinned here so the durable-session contract stays canonical and retained.

### `reattach` semantics are attached-owner recovery only
- `substrate agent reattach --session <orchestration_session_id>` is the explicit attached-owner recovery verb.
- `reattach` MUST restore a live attached owner loop for the selected host orchestration session when recovery metadata is intact.
- `reattach` MUST NOT submit a prompt, consume inbox work implicitly, or act as a one-shot follow-up-turn shortcut.
- One-shot prompt-taking resume stays on:
  - `substrate agent turn --session <orchestration_session_id> --backend <backend_id> --prompt ...`
- Rationale:
  - this matches the already-landed public caller-surface contract,
  - preserves the distinction between operational ownership recovery and foreground turn submission,
  - and avoids reintroducing ambiguous `resume` semantics into the public lifecycle surface.

### `fork` semantics are successor allocation only
- `substrate agent fork --session <orchestration_session_id>` is the explicit successor-allocation verb.
- `fork` MUST allocate a new durable orchestration session before any later follow-up work is considered.
- `fork` MUST copy the source session's persisted host attach-contract shape but MUST clear inherited `continuity_uaa_session_id` on the successor.
- `fork` MUST return the successor honestly as `parked_resumable` with `attached_participant_id = null`.
- `fork` MUST preserve source lineage truth through the successor participant/session records.
- `fork` MUST NOT submit a prompt, attach a live owner loop immediately, or reuse the parent's continuity token as if the successor were already attached.
- Rationale:
  - successor allocation is durable-state work, not prompt execution,
  - keeping the successor unattached avoids ownership theater and false `active_attached` truth,
  - and clearing inherited continuity preserves the distinction between "resume this exact backend session" and "allocate a new durable successor session".

### Resolved inbox items remain retained for audit, then compacted later
- Resolving an inbox item MUST remove it from authoritative pending/live counts immediately.
- Resolving an inbox item MUST NOT immediately delete the persisted inbox artifact.
- Resolved inbox items SHOULD be retained until:
  - the orchestration session has reached terminal state,
  - the item is no longer `pending`,
  - the item has aged past a bounded retention floor, and
  - no unresolved item still depends on it for causation/debug correlation.
- Compaction is a maintenance step, not the acknowledgement primitive.
- This ADR intentionally fixes compaction eligibility rules but does not fix a numeric retention duration.
- The exact numeric retention floor is an implementation/operations follow-on and should be aligned later with adjacent router/workflow retention policy rather than guessed here.
- Rationale:
  - the broader event-plane and trace contracts in this repo are append-only and audit-oriented,
  - live-state authority and historical retention are intentionally distinct,
  - and immediate deletion would make detached-host review and postmortem analysis weaker without providing a meaningful contract benefit.

### Future inbox kinds remain additive within one canonical envelope
- The durable inbox outer envelope pinned by this ADR is canonical:
  - `schema_version`
  - `item_id`
  - `orchestration_session_id`
  - `kind`
  - `state`
  - `created_at`
  - `resolved_at`
  - `correlation`
  - `payload_schema`
  - `payload`
- Future inbox item kinds are allowed only as additive expansions of the `kind` enum.
- Future kinds MUST NOT introduce new ad hoc top-level envelope fields as their primary shape.
- Kind-specific structure MUST live under the versioned `payload_schema` plus `payload` contract.
- Any future cross-cutting join/correlation additions MUST remain additive and defer to ADR-0028 / Phase 8 for canonical naming and required/optional classification rather than creating inbox-only correlation dialects.
- Rationale:
  - this preserves one durable session-local inbox envelope across approvals, completions, follow-up work, and later workflow-aware attention items,
  - keeps inbox growth compatible with ADR-0029 router records and ADR-0021 workflow correlation,
  - and avoids top-level envelope drift as future kinds are introduced.

### Non-host attached-client generalization is out of scope for this ADR
- The attached-client metadata pinned by this ADR is intentionally host-oriented.
- No additional generalized attachment metadata is required in this ADR for hypothetical future non-host attached clients.
- If Substrate later introduces a new attached-client class that is not host-rooted, that work MUST land as an additive follow-on contract that:
  - states the new attached-client type explicitly,
  - defines any new persisted metadata fields explicitly,
  - and proves that the broader attachment model does not weaken the current host-rooted fail-closed posture.
- Rationale:
  - the current orchestration model is explicitly host-rooted,
  - public root start remains host-only in v1,
  - and broadening the attachment taxonomy now would speculate beyond the runtime model this ADR is actually standardizing.

## Architecture Shape
- Components:
  - `crates/shell` runtime/session/state layers:
    - own the durable orchestration session identity, posture state, and routing gates.
  - Substrate-owned durable inbox / task ledger:
    - owns world-to-host message durability when no client is attached.
  - attachable execution client:
    - a Codex session or equivalent backend client that may attach, run, exit, and later resume against the same orchestration session.
  - world-service and world-member submit path:
    - continue to own world-member turn execution and typed request submission for Linux world follow-up.
- End-to-end flow:
  - Inputs:
    - public `start`, `turn`, `reattach`, and `stop`
    - world-originated completion, approval, or update events
    - exact `(orchestration_session_id, backend_id)` routing inputs
  - Derived state:
    - orchestration session posture
    - attached-client presence or absence
    - pending durable inbox items
    - whether follow-up is host-routable, world-routable, or fail-closed
  - Actions:
    - `start` creates or binds the durable orchestration session and runs the initial prompt
    - if the prompt-driven host client exits cleanly, Substrate transitions to `parked_resumable` instead of invalidating
    - world-originated messages land in the durable inbox regardless of client attachment
    - `turn` or `reattach` resumes a valid host orchestration session as needed
    - `Accepted` is emitted only under a bridge contract that guarantees terminal completion signaling
  - Outputs:
    - stable orchestration session state
    - explicit attached/parked/attention-needed posture
    - explicit terminal envelopes for accepted prompt requests

## Sequencing / Dependencies
- Sequencing entry: `docs/project_management/packs/sequencing.json` → `host-orchestrator-durable-session-and-parked-resumable-ownership` or the next available orchestration-runtime slot
- Prerequisite integration task IDs:
  - the public caller-surface hardening from `PLAN-22` remains prerequisite context
  - the exact host/world posture and identity-tuple rules from ADR-0042 remain prerequisite context
  - the detailed inbox/ledger runtime contract is expected as a follow-on contract/spec, not fully defined here

## Work Lift (discovery estimate)

<!-- PM_LIFT_VECTOR:BEGIN -->
```json
{
  "model_version": 1,
  "touch": {
    "create_files": 1,
    "edit_files": 8,
    "delete_files": 0,
    "deprecate_files": 0,
    "crates_touched": 2,
    "boundary_crossings": 2
  },
  "contract": {
    "cli_flags": 0,
    "config_keys": 0,
    "exit_codes": 0,
    "file_formats": 0,
    "behavior_deltas": 1
  },
  "qa": { "new_test_files": 0, "new_test_cases": 10 },
  "docs": { "new_docs_files": 2 },
  "ops": { "new_smoke_steps": 1, "ci_changes": 0 },
  "risk": {
    "cross_platform": true,
    "security_sensitive": false,
    "concurrency_or_ordering": true,
    "migration_or_backfill": true,
    "unknowns_high": true
  },
  "notes": "Primary lift is runtime ownership/state semantics, retained internal session/participant/inbox schema updates, and accepted-to-terminal delivery guarantees. Config/policy schema is unchanged."
}
```
<!-- PM_LIFT_VECTOR:END -->

## Security / Safety Posture
- Fail-closed rules:
  - world-only root start remains rejected.
  - detached-world public follow-up remains rejected until a sanctioned host path re-establishes routable ownership.
  - public `turn` remains exact-selector only.
  - world-originated messages must not bypass Substrate-owned routing and durability surfaces.
- Protected paths/invariants:
  - a clean prompt-driven host client exit after session establishment must not silently degrade into session invalidation.
  - accepted prompt requests must not end in silent stream loss.
  - durable orchestration state must remain authoritative over any one attached client process.

## Validation Plan (Authoritative)

### Tests
- Unit tests:
  - host session posture transitions:
    - `active_attached` -> `parked_resumable`
    - `parked_resumable` -> `active_attached`
    - `parked_resumable` -> `awaiting_attention`
    - legal transition to `terminal`
  - accepted prompt bridge:
    - once `Accepted` is emitted, the bridge always emits a terminal envelope on completion, cancellation, owner loss, or runtime invalidation
- Integration tests:
  - host `start` creates a valid orchestration session and allows clean client exit without invalidation
  - host `turn` succeeds against a valid parked host session
  - `reattach` restores attached host posture from a valid parked host session
  - world-originated completion or approval work is retained when no host client is attached
  - detached-world follow-up remains fail-closed
  - `repl_world_first_routing_v1.rs` remains green as a non-regression surface

### Manual validation
- Re-run host public manual flow outside the integration harness:
  - `substrate agent start --backend <host_backend_id> --prompt "hello" --json`
  - `substrate agent turn --session <orchestration_session_id> --backend <host_backend_id> --prompt "next" --json`
  - `substrate agent reattach --session <orchestration_session_id> --json`
- Confirm the host session parks cleanly when the prompt-driven client exits and can later resume.
- Confirm no post-`Accepted` stream ends without `Completed` or `Failed`.
