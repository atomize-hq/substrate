# agent-hub-core-successor-identity-tuple-compatible — policy spec

Owner standard:
- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Scope

- This spec is authoritative for fail-closed control-plane evaluation for ADR-0044.
- This spec owns the ordered deny flow for:
  - orchestrator selection
  - member-dispatch eligibility
  - required world-boundary availability
  - the boundary between agent-hub control-plane approval and gateway-side nested LLM approval
- This spec does not redefine tuple semantics, gateway tuple-axis constraints, config-file families, or exit-code taxonomy.

Canonical references:
- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/contract.md`
- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/agent-hub-session-protocol-spec.md`
- `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/contract.md`
- `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/SCHEMA.md`
- `docs/contracts/substrate-gateway-policy-evaluation.md`
- `docs/project_management/adrs/draft/ADR-0043-adr-0027-identity-tuple-policy-surface.md`
- `docs/project_management/adrs/draft/ADR-0044-agent-hub-core-successor-identity-tuple-compatible.md`

## Governing inputs

The ordered evaluation in this spec uses these effective inputs and no others:

- control-plane config and policy inputs:
  - `agents.hub.orchestrator_agent_id`
  - `agents.allowed_backends`
  - `agents.fail_closed.routing`
  - `agents.hub.world_restart.on_drift`
- effective inventory and protocol inputs:
  - the resolved agent inventory item keyed by `agent_id`
  - derived `backend_id = "<kind>:<agent_id>"`
  - `execution.scope`
  - `protocol`
  - capability booleans from `AgentBackendCapabilityDescriptorV1`
- dispatch-time runtime inputs:
  - requested target `agent_id`
  - requested `role`
  - current world-boundary availability for the orchestration session
  - current shared-world drift status
- nested gateway inputs that remain external:
  - `llm.allowed_backends`
  - `llm.constraints.routers`
  - `llm.constraints.providers`
  - `llm.constraints.protocols`
  - `llm.constraints.auth_authorities`

Explicit non-inputs:

- event-plane records by themselves
- trace-only observations by themselves
- gateway-local mutable config
- gateway-local token persistence
- `provider`
- `auth_authority`
- any heuristic derived from human-readable status output

## Control-plane trust boundary

- Event-plane observation never authorizes a control-plane action.
- The hub evaluates control-plane authorization from the effective config, effective policy, effective inventory, protocol capabilities, and world-boundary state only.
- A prior successful nested gateway request does not authorize orchestrator selection, member dispatch, resume, fork, stop, or world restart.
- Gateway tuple-axis approval remains external to this control plane and is reused only when an agent explicitly triggers a nested LLM request.

## Ordered evaluation flow

Agent-hub control-plane policy evaluation follows this fixed order:

1. Resolve the selected orchestrator id from `agents.hub.orchestrator_agent_id`.
2. Resolve the effective orchestrator inventory item by that `agent_id`.
3. Derive `backend_id = "<kind>:<agent_id>"` for the orchestrator candidate.
4. Apply the agent-side backend allowlist:
   - `agents.allowed_backends` gates the derived `backend_id`
   - no tuple field participates in this gate
5. Validate orchestrator eligibility in this exact order:
   - the inventory item is enabled
   - `execution.scope = host`
   - `protocol = substrate.agent.session`
   - `session_start = true`
   - `session_resume = true`
   - `session_fork = true`
   - `session_stop = true`
   - `status_snapshot = true`
   - `event_stream = true`
6. Resolve the requested member inventory item, if the control-plane call dispatches or resumes a member session.
7. Derive the member `backend_id = "<kind>:<agent_id>"`.
8. Apply `agents.allowed_backends` to the member `backend_id`.
9. Validate member protocol eligibility:
   - `protocol = substrate.agent.session`
   - the required session capability for the requested action is `true`
10. Apply execution-scope gating:
   - `execution.scope = host` permits host-scoped member dispatch without world attachment
   - `execution.scope = world` requires an available shared world or an allowed world allocation path
11. Apply world-drift posture:
   - when drift is absent, reuse the current shared world
   - when drift is present and `agents.hub.world_restart.on_drift = auto_restart`, restart is permitted and replacement handles must be allocated before more work is dispatched
   - when drift is present and `agents.hub.world_restart.on_drift = fail_closed`, the control plane denies further world-scoped work until an explicit restart path succeeds
12. If an agent explicitly triggers nested LLM work, stop agent-hub control-plane evaluation and hand the nested request to the gateway policy surface governed by ADR-0043 and `docs/contracts/substrate-gateway-policy-evaluation.md`.

## Deny taxonomy

### Orchestrator selection denial

These conditions deny before any member dispatch is considered:

- `agents.hub.orchestrator_agent_id` is absent
- the referenced `agent_id` is missing from the effective inventory
- the referenced inventory item is disabled
- `execution.scope = world`
- `protocol != substrate.agent.session`
- any required orchestrator capability is `false`

The deny explanation must identify the exact failing condition and, when applicable, the exact policy key:

- `"effective config agents.hub.orchestrator_agent_id is unset"`
- `"orchestrator agent '<agent_id>' is missing from effective inventory"`
- `"orchestrator agent '<agent_id>' is disabled in effective inventory"`
- `"orchestrator agent '<agent_id>' is world-scoped and host-scoped orchestration is required"`
- `"orchestrator agent '<agent_id>' does not advertise protocol 'substrate.agent.session'"`
- `"orchestrator agent '<agent_id>' is missing required capability '<capability>'"`

### Policy allowlist denial

These conditions deny after orchestrator selection succeeds and before any world-boundary check or member dispatch is considered:

- the selected orchestrator derived `backend_id` is denied by `agents.allowed_backends`
- any required world-scoped member derived `backend_id` is denied by `agents.allowed_backends`

The deny explanation must identify the exact failing condition and the exact policy key:

- `"selected orchestrator backend '<backend_id>' is not allowlisted by effective policy agents.allowed_backends"`
- `"required world-scoped member backend '<backend_id>' is not allowlisted by effective policy agents.allowed_backends"`

### Member dispatch denial

These conditions deny after orchestrator eligibility succeeds:

- the requested member `agent_id` is missing from effective inventory
- the requested member inventory item is disabled
- the derived member `backend_id` is denied by `agents.allowed_backends`
- `protocol != substrate.agent.session`
- the requested action needs a capability that is `false`

The deny explanation must identify the exact failing condition and, when applicable, the exact policy key:

- `"member agent '<agent_id>' is missing from effective inventory"`
- `"member agent '<agent_id>' is disabled in effective inventory"`
- `"member backend '<backend_id>' is not allowlisted by effective policy agents.allowed_backends"`
- `"member agent '<agent_id>' does not advertise protocol 'substrate.agent.session'"`
- `"member agent '<agent_id>' is missing required capability '<capability>' for action '<action>'"`

### Required world-boundary denial

These conditions deny only for `execution.scope = world`:

- no shared world is available and allocation cannot succeed
- the world boundary is unavailable while `agents.fail_closed.routing = true`
- drift is present and `agents.hub.world_restart.on_drift = fail_closed`
- the previous world-scoped handle is invalidated and no replacement handle is ready

The deny explanation must identify the exact failing condition:

- `"world-scoped member '<agent_id>' requires a shared world, but no world boundary is available"`
- `"world-scoped member '<agent_id>' is denied because agents.fail_closed.routing requires a world boundary"`
- `"world-scoped member '<agent_id>' is denied because shared-world drift requires explicit restart under agents.hub.world_restart.on_drift=fail_closed"`
- `"world-scoped member '<agent_id>' is denied because the prior handle is invalidated and no replacement handle is ready"`

## Nested gateway reuse boundary

- Agent-hub approval stops at the agent boundary.
- Nested LLM requests reuse gateway policy gates from ADR-0043 in this exact order:
  - `llm.allowed_backends`
  - `llm.constraints.routers`
  - `llm.constraints.protocols`
  - `llm.constraints.providers`
  - `llm.constraints.auth_authorities`
- Agent-hub approval does not imply gateway approval.
- Gateway approval does not back-authorize orchestrator or member control-plane actions.
- `backend_id` remains the only agent-side allowlist key. Agent-hub control-plane evaluation never substitutes `provider`, `auth_authority`, `router`, or `protocol` for `backend_id`.

## Observation-only boundary

- `substrate agent list`, `substrate agent status`, structured events, and trace records are observation surfaces.
- Observation surfaces may explain why a prior action succeeded or failed.
- Observation surfaces do not open a permission path.
- A control-plane entrypoint that depends only on prior observation is invalid and must be rejected as a design error before implementation.

## Failure buckets

- Invalid integration:
  - malformed or structurally invalid inventory data
  - contradictory capability descriptors
  - impossible restart state transitions
- Policy denial:
  - backend allowlist denial
  - host-scoped orchestrator requirement failure
  - missing required capability
  - fail-closed world-boundary denial
- Component unavailable:
  - world backend unavailable before dispatch can start
  - shared world allocation path unavailable
- Transient runtime failure:
  - restart attempt times out
  - connection reset while allocating a replacement world

These buckets remain distinct in docs, code, and tests.

## Acceptance criteria

- Control-plane evaluation uses only the governing inputs named by this spec and rejects event-plane records as authorization inputs.
- `substrate agent doctor` evaluates orchestrator host/protocol/capability selection before the `policy_allowlist` phase that checks derived `backend_id` values.
- A world-scoped orchestrator is always denied.
- Member dispatch always reapplies `agents.allowed_backends` to the member `backend_id`.
- World-scoped member dispatch denies when the world boundary is unavailable under fail-closed posture.
- Nested LLM requests always hand off to gateway policy evaluation and never inherit approval by implication from agent-hub control-plane success.
