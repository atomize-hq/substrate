# SOW: Generalize Runtime State to a Session Participant Record

## Objective

Replace the current host-orchestrator-only runtime manifest model with a canonical session participant record that can represent both:

- the host-scoped orchestrator session, and
- world-scoped or host-scoped member sessions that belong to the same `orchestration_session_id`.

The end state is a single live-runtime record model that status, toolbox, doctor, and event producers can consume without treating the orchestrator as a special one-off artifact.

## Problem Statement

Today the live runtime store is shaped around a single host orchestrator manifest:

- `crates/shell/src/execution/agent_runtime/session.rs`
- `crates/shell/src/execution/agent_runtime/state_store.rs`
- `crates/shell/src/repl/async_repl.rs`

That shape is sufficient for the first retained host orchestrator bootstrap, but it is not a good source of truth for the v1 orchestration model documented in:

- `AGENT_ORCHESTRATION_GAP_MATRIX.md`
- `docs/project_management/adrs/draft/ADR-0044-agent-hub-core-successor-identity-tuple-compatible.md`
- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/agent-hub-session-protocol-spec.md`
- `docs/project_management/adrs/draft/ADR-0045-orchestration-toolbox-internal-mcp-identity-trace-contract.md`

Those contracts assume one orchestration session can contain multiple participants with distinct roles, scopes, world bindings, and lifecycle state. The current runtime store cannot model that cleanly.

## Current Repo Seams

### State owner and on-disk store

- `crates/shell/src/execution/agent_runtime/session.rs`
  - Defines `AgentRuntimeSessionHandle`, `AgentRuntimeSessionInternal`, and `AgentRuntimeSessionManifest`.
- `crates/shell/src/execution/agent_runtime/state_store.rs`
  - Persists JSON files under `~/.substrate/run/agent-hub/handles/`.
  - Exposes `list_manifests()`, `list_live_manifests()`, and `find_live_orchestrator(agent_id)`.

### Current producer

- `crates/shell/src/repl/async_repl.rs`
  - Allocates a single shell-owned orchestrator manifest.
  - Transitions it through `allocating`, `ready`, `running`, `stopping`, `stopped`, `failed`, and `invalidated`.
  - Emits pure-agent runtime events from that manifest via `translate_wrapper_event()` and `build_runtime_message_event()`.

### Current consumers

- `crates/shell/src/execution/agents_cmd.rs`
  - `substrate agent status` prefers live manifests before trace-derived session rows.
  - `substrate agent toolbox status|env` authorizes endpoint publication only from an authoritative live orchestrator manifest.
  - `substrate agent doctor` is still largely pre-runtime and does not validate a generalized live participant registry.

### Event and contract surfaces

- `crates/common/src/agent_events.rs`
  - Canonical agent-event envelope used by trace/status consumers.
- `docs/USAGE.md`
  - Documents the live-manifest preference for `agent status` and toolbox surfaces.
- `docs/TRACE.md`
  - Documents toolbox tool-call audit families and correlation expectations.
- `crates/shell/tests/agent_successor_contract_ahcsitc0.rs`
  - Locks current toolbox/status behavior against live manifest semantics.
- `crates/common/tests/agent_hub_event_envelope_schema.rs`
  - Locks additive event-envelope behavior.

## Current Blockers

1. The persisted record is semantically orchestrator-specific.
   - `AgentRuntimeSessionManifest::new()` hard-codes `role=orchestrator`.
   - The state store exposes `find_live_orchestrator()` instead of general participant queries.

2. The store keying and lookup model are too narrow.
   - Files are stored by `session_handle_id`, but consumer logic collapses live state to one latest orchestrator per `agent_id`.
   - `substrate agent status` still keys historical pure-agent rows by `(orchestration_session_id, agent_id)`, which will collapse same-agent member siblings.

3. Lifecycle ownership is modeled only for the retained host control turn.
   - `ownership_mode` currently defaults to `attached_control`.
   - There is no canonical representation for member-session ownership, world-backed member invalidation, or orchestrator-driven teardown of children.

4. Event production does not carry enough participant identity.
   - Runtime event producers populate `agent_id`, `backend_id`, `role`, and pure-agent tuple identity.
   - They do not publish a participant identifier, parent participant link, or a generalized resume lineage for member sessions.

5. Toolbox authorization is coupled to the orchestrator-specific store helper.
   - `substrate agent toolbox status|env` currently depends on `find_live_orchestrator()`.
   - Member-session support needs a generalized store without weakening the rule that only a live orchestrator authorizes toolbox wiring.

## Proposed Scope

Introduce a canonical session participant record and migrate all live-runtime consumers to that abstraction.

This SOW does not require a full `/v1/agents` service. It does require one authoritative on-disk participant registry that can support:

- one live orchestrator,
- zero or more live members,
- restart/invalidated replacement lineage,
- world binding and shared-world generation tracking,
- status/toolbox correlation, and
- additive event publication.

## Proposed Schema and State Changes

### Canonical record

Replace or rename `AgentRuntimeSessionManifest` to a participant-centric record, for example:

- `AgentRuntimeSessionParticipantRecord`
- or `AgentRuntimeParticipantRecordV1`

Keep the existing split between public session metadata and internal ownership metadata, but rename it away from orchestrator-only language.

### Required top-level participant fields

The record must carry, at minimum:

- `participant_id`
  - New canonical id for one runtime participant record.
  - May reuse the current `session_handle_id` value if the implementation wants a non-breaking first step.
- `orchestration_session_id`
- `agent_id`
- `backend_id`
- `role`
  - `orchestrator` or `member`
- `protocol`
- `execution.scope`
  - `host` or `world`
- `state`
- `opened_at`
- `last_transition_at`
- `world_id` and `world_generation`
  - required for world-scoped members, absent for host-scoped participants
- `parent_participant_id`
  - required for forked members or explicitly derived child sessions
- `resumed_from_participant_id`
  - required for replacement records created after invalidation/restart
- `orchestrator_participant_id`
  - absent on the orchestrator record, required on member records so children can be attributed to the controlling orchestrator even when multiple agents share the same `orchestration_session_id`

### Required internal/runtime fields

The internal block should continue to own ephemeral authority details:

- `resolved_agent_kind`
- `resolved_binary_path`
- `shell_owner_pid`
- `lease_token`
- `uaa_session_id`
- `latest_run_id`
- `cancel_supported`
- `ownership_mode`
- `ownership_valid`
- `ownership_verified_at`
- `control_owner_retained`
- `event_stream_active`
- `completion_observer_retained`
- `last_heartbeat_at`
- `last_event_at`
- `terminal_observed_at`
- `termination_reason`
- `last_error_bucket`
- `last_error_message`

### Ownership mode generalization

`ownership_mode` must stop implying “host orchestrator only”. The first implementation-ready enum should be:

- `attached_control`
  - current shell-owned orchestrator path
- `member_runtime`
  - live member session owned by Substrate runtime logic
- `replaced`
  - old participant retained only for lineage after restart/invalidation

If implementation prefers different token names, the semantics above still need to exist.

### On-disk location and migration

Recommended target:

- write new records under `~/.substrate/run/agent-hub/participants/`

Migration rules:

1. New code reads both:
   - legacy `handles/*.json`
   - new `participants/*.json`
2. New code writes only the participant location.
3. Toolbox/status treat the participant store as authoritative when both exist.
4. Legacy orchestrator-only manifests can be upgraded in-memory by mapping:
   - `session_handle_id -> participant_id`
   - `parent_session_handle_id -> parent_participant_id`
   - `resumed_from_session_handle_id -> resumed_from_participant_id`

This avoids a flag day while other parallel runtime work lands.

## Lifecycle Semantics

### Orchestrator lifecycle

The current orchestrator state machine in `crates/shell/src/repl/async_repl.rs` remains the baseline:

- `allocating -> ready -> running -> stopping -> stopped`
- terminal error states: `failed`, `invalidated`

The only required semantic change is that the record type must no longer imply “this is the only runtime participant”.

### Member lifecycle

Member participants must use the same state enum in `crates/shell/src/execution/agent_runtime/session.rs`, with these additional rules:

- members may be created directly in `allocating`
- world-scoped members must publish `world_id` and `world_generation` before entering `ready`
- members never authorize toolbox publication
- member invalidation on shared-world restart must create a replacement participant record instead of mutating lineage in place

### Restart and invalidation rules

The lifecycle rules in `agent-hub-session-protocol-spec.md` should become the store truth:

- host-scoped orchestrators do not enter `restarting`
- world-scoped members may enter `restarting`
- a shared-world restart invalidates every live member bound to the prior `(world_id, world_generation)`
- replacement participants must:
  - get a new `participant_id`
  - preserve `orchestration_session_id`, `agent_id`, `backend_id`, `role`, and `protocol`
  - set `resumed_from_participant_id`
  - increment `world_generation`

### Parent/child teardown semantics

When the orchestrator becomes terminal:

- live member participants under the same `orchestrator_participant_id` must not continue to advertise authoritative ownership
- they must transition to `invalidated` or `failed`
- toolbox/env must fail closed immediately afterward

## Impacted Surfaces

### `substrate agent status`

Files:

- `crates/shell/src/execution/agents_cmd.rs`
- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/contract.md`
- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/agent-hub-session-protocol-spec.md`

Required changes:

- Prefer live participant records for both orchestrator and member rows, not just the retained orchestrator.
- Stop collapsing runtime truth to one row per `agent_id`.
- Add an additive participant identifier to the JSON status surface.
  - Recommended field name: `participant_id`
  - Transitional alternative: reuse and expose `session_handle_id`
- Preserve current trace fallback behavior only for sessions not represented in the live participant store.
- Keep nested gateway-backed rows separate from pure-agent participant rows.

### `substrate agent doctor`

Files:

- `crates/shell/src/execution/agents_cmd.rs`
- `docs/USAGE.md`

Required changes:

- Doctor should remain pre-runtime and fail closed on config/policy/world readiness exactly as it does now.
- Do not require live participant records for a healthy doctor result.
- Add passive validation only:
  - if participant records exist, malformed schema or impossible role/scope combinations should fail the relevant doctor check
  - absence of participant records remains non-fatal

### `substrate agent toolbox status|env`

Files:

- `crates/shell/src/execution/agents_cmd.rs`
- `docs/project_management/adrs/draft/ADR-0045-orchestration-toolbox-internal-mcp-identity-trace-contract.md`
- `docs/USAGE.md`

Required changes:

- Replace `find_live_orchestrator(agent_id)` with a participant query that filters:
  - `role=orchestrator`
  - `execution.scope=host`
  - authoritative live ownership
- Keep the existing fail-closed rule:
  - member records alone never authorize toolbox env publication
- Continue using `orchestration_session_id` to derive the UDS endpoint path unless ADR-0045 is amended

### Event production and trace correlation

Files:

- `crates/shell/src/repl/async_repl.rs`
- `crates/common/src/agent_events.rs`
- `docs/TRACE.md`
- `crates/common/tests/agent_hub_event_envelope_schema.rs`

Required changes:

- Add additive optional event fields for participant correlation:
  - `participant_id`
  - `parent_participant_id`
  - `resumed_from_participant_id`
- Ensure pure-agent runtime events can carry `world_id` and `world_generation` from member participant records.
- Preserve the current tuple split:
  - pure-agent participant records stay `router=agent_hub`, `protocol=uaa.agent.session`
  - nested LLM activity stays separate with `router=substrate_gateway`

## Implementation Sequence

1. Lock the participant-record contract in code-facing docs.
   - Update the session-protocol spec and any operator contract surfaces that need an additive `participant_id`.

2. Introduce the generalized record model in `crates/shell/src/execution/agent_runtime/session.rs`.
   - Rename or replace the current manifest type.
   - Add constructors for orchestrator and member participants.

3. Generalize the state store in `crates/shell/src/execution/agent_runtime/state_store.rs`.
   - Read legacy handles plus new participants.
   - Add queries for:
     - all live participants
     - live participants by orchestration session
     - active live orchestrator

4. Migrate the current orchestrator writer in `crates/shell/src/repl/async_repl.rs`.
   - No behavior change yet; write the new participant record shape.

5. Land member-session writers in the relevant world/member runtime seam.
   - This likely starts in shell-side orchestration runtime paths, not in gateway runtime code.
   - Keep scope bounded to actual agent-hub member sessions, not generic gateway lifecycles.

6. Update `substrate agent status` to consume participant records first.

7. Update toolbox status/env to authorize from live orchestrator participant records.

8. Add passive doctor validation for malformed participant records.

9. Add event-envelope fields and runtime event emission for participant correlation.

10. Remove legacy handle-only assumptions after compatibility coverage is green.

## Acceptance Criteria

The work is complete when all of the following are true:

1. There is one canonical runtime participant record type that can represent both orchestrator and member sessions.
2. Live runtime persistence supports more than one participant per `orchestration_session_id`.
3. Status no longer collapses same-agent siblings to a single live row by `agent_id`.
4. Toolbox status/env derive authorization from a live orchestrator participant query, not a host-manifest special case.
5. Doctor remains pre-runtime, but malformed participant records fail closed when present.
6. Pure-agent event production can publish participant lineage additively without breaking nested gateway-backed records.
7. Legacy orchestrator-only manifests are either migrated or read compatibly during rollout.

## Validation and Testing Suggestions

### Targeted automated coverage

- `cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture`
  - extend existing toolbox/status/live-manifest tests to live participant records and mixed orchestrator/member scenarios
- `cargo test -p substrate-common --test agent_hub_event_envelope_schema -- --nocapture`
  - cover additive participant-correlation fields and omission rules
- add unit tests alongside:
  - `crates/shell/src/execution/agent_runtime/state_store.rs`
  - `crates/shell/src/execution/agent_runtime/session.rs`
  - `crates/shell/src/execution/agents_cmd.rs`

### Manual/runtime validation

- `substrate agent status --json`
  - verify multiple participants under one `orchestration_session_id`
- `substrate agent toolbox status --json`
  - verify member-only live state does not authorize toolbox
- `substrate agent toolbox env --json`
  - verify only a live orchestrator participant yields an endpoint
- `substrate agent doctor --json`
  - verify no active participant store is required, but malformed participant files fail closed
- `substrate world doctor --json`
  - re-run when validating world-scoped member paths and restart invalidation behavior

### Regression floor

- `cargo test --workspace -- --nocapture`

## Risks and Open Questions

1. Status-surface widening.
   - If multiple member participants per `agent_id` are allowed, the current JSON surface needs an additive participant identifier or it will remain lossy.

2. Migration path.
   - Reading both legacy handles and new participants is the safest rollout, but it adds temporary complexity to `state_store.rs`.

3. Ownership model for members.
   - The current `attached_control` semantics are clear for the retained orchestrator. Member ownership semantics need one precise definition before runtime code lands.

4. World-restart authority.
   - The participant store should reflect world restart/invalidation, but the exact owner of replacement creation must stay in the shell-side orchestration runtime, not drift into generic gateway lifecycle code.

5. Event-schema appetite.
   - Adding `participant_id` fields is low risk, but status/toolbox implementations should not wait on a larger trace-schema redesign.

## Out of Scope

- Turning this into a full agent-hub service API
- Reworking nested gateway runtime/status contracts
- Mutating toolbox tools or changing ADR-0045’s introspection-only v1 posture
- Renaming the local `agent-api-*` crates in the same slice
