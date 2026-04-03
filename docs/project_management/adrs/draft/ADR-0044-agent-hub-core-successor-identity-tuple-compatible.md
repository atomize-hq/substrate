# ADR-0044 — Agent Hub Core Successor (Identity-Tuple Compatible, Backend-ID Safe)

## Status
- Status: Draft
- Date (UTC): 2026-04-03
- Owner(s): Spenser McConnell (Substrate)

## Scope
- Feature directory: `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/`
- This ADR is docs-only; no pack files are created by this change.
- Sequencing spine: `docs/project_management/packs/sequencing.json`
- Standards:
  - `docs/project_management/system/standards/adr/EXECUTIVE_SUMMARY_STANDARD.md`

## Related Docs (links only)

This ADR supersedes the older backend-id-centric Agent Hub framing in ADR-0025 and is meant to be read together with the identity and gateway foundations below.

- Superseded ADR:
  - `docs/project_management/adrs/draft/ADR-0025-agent-hub-core-role-swappable.md`
- Identity tuple and deployment posture:
  - `docs/project_management/adrs/draft/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md`
- Tuple-axis policy surface:
  - `docs/project_management/adrs/draft/ADR-0043-adr-0027-identity-tuple-policy-surface.md`
- Config/policy foundation:
  - `docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md`
- Event and trace foundations:
  - `docs/project_management/adrs/draft/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md`
  - `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md`
- Gateway ownership and adapter contracts:
  - `docs/project_management/adrs/draft/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md`
  - `docs/project_management/adrs/draft/ADR-0041-substrate-gateway-backend-adapter-contract.md`
- Follow-on orchestration surface:
  - `docs/project_management/adrs/draft/ADR-0026-orchestration-toolbox-mcp.md`

## Executive Summary (Operator)

ADR_BODY_SHA256: f903ebf4711628a2ab287c89184603c6cac2efd5a9fbcbf57e72a3f57d7f4843
### Changes (operator-facing)
- Replace the older “role-swappable backend” assumption with a UAA-compatible agent hub contract
  - Existing: ADR-0025 treats any backend id as a possible orchestrator/member backend and mixes role assignment, transport, and backend identity into one conceptual surface.
  - New: Agent Hub speaks to backends through capability-driven, session-handle semantics (compatible with the repo’s `crates/agent-api-*` model), while keeping `backend_id` as a pure adapter identifier.
  - Why: operators need to know whether they are approving a runtime adapter, a routing authority, or an upstream model provider, and those are not the same thing.
  - Links:
    - `docs/project_management/adrs/draft/ADR-0025-agent-hub-core-role-swappable.md`
    - `docs/project_management/adrs/draft/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md`
    - `docs/project_management/adrs/draft/ADR-0043-adr-0027-identity-tuple-policy-surface.md`

- Separate pure agent-run identity from nested gateway-backed LLM identity
  - Existing: backend ids and trace attribution can be read as if they imply provider, auth authority, and protocol all at once.
  - New: pure agent runs expose `client`, `router`, `protocol`, and `backend_id`; `provider` and `auth_authority` are absent unless the agent triggers a nested LLM request through `substrate-gateway`.
  - Why: this keeps operator approval and audit output readable when a host orchestrator dispatches a world-scoped member agent that later calls an LLM.
  - Links:
    - `docs/project_management/adrs/draft/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md`
    - `docs/project_management/adrs/draft/ADR-0041-substrate-gateway-backend-adapter-contract.md`
    - `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md`

## Problem / Context
- ADR-0025 captured the first version of Agent Hub, but that framing predated the identity tuple clarified in ADR-0042 and the policy split clarified in ADR-0043.
- The remaining ambiguity is operator-facing, not just implementation-facing:
  - `backend_id` should stay the adapter/backend id used for allowlisting and trace attribution.
  - `client`, `router`, `protocol`, `provider`, and `auth_authority` should be visible as distinct concepts.
  - `provider` and `auth_authority` should not appear on pure agent orchestration records just because an agent backend exists.
- The hub also needs to preserve the existing orchestration split:
  - a host-scoped orchestrator that controls control-plane decisions,
  - world-scoped member agents that can run inside a world boundary,
  - explicit, operator-verifiable world reuse and restart semantics.

## Goals
- Preserve the control-plane vs event-plane split.
- Preserve fail-closed routing and role-gating posture.
- Make the host-scoped orchestrator and world-scoped member model explicit.
- Align agent backends with UAA-style semantics:
  - capability discovery,
  - session handles,
  - resume/fork,
  - structured status and event emission.
- Keep `backend_id` strictly as the adapter/backend id for `agents.allowed_backends` and trace attribution.
- Make nested LLM usage visible without overloading the base agent-run identity tuple.

## Non-Goals
- Creating a new config file family.
- Replacing ADR-0027 or the gateway ownership ADRs.
- Adding a second permanent host gateway.
- Redefining the tuple semantics already locked by ADR-0042.
- Defining exact header names or env var names for nested request hints.

## Control plane vs event plane (authoritative)

This ADR keeps the same architectural split established by the earlier Agent Hub work:

- Control plane:
  - session start/stop/resume/fork,
  - orchestrator selection,
  - member dispatch,
  - world placement decisions,
  - policy-gated tool/control calls.
- Event plane:
  - append-only structured status/progress/alert records,
  - world reuse/restart notifications,
  - trace attribution for audit and operator visibility.

Non-negotiable rules:
- The hub MUST NOT take host/world actions merely because it observed event-plane records.
- Control-plane actions MUST be explicitly attributed and fail closed when policy, scope, or capability checks fail.
- If a required orchestrator, world boundary, session handle, or backend capability is missing, the hub MUST stop rather than guessing.

## User Contract (Authoritative)

### CLI
- This ADR introduces no new top-level commands.
- Existing agent-hub surfaces are extended additively:
  - `substrate agent list`
    - MUST show `backend_id`, `execution.scope`, `role`, capability summary, and the agent backend contract/protocol.
    - MUST not imply `provider` or `auth_authority` for a pure orchestration entry.
  - `substrate agent status`
    - MUST show the active orchestrator selection, active sessions, `world_id`/`world_generation` when applicable, and the backend adapter identity used for routing.
    - MUST show `provider` and `auth_authority` only for nested LLM-backed sub-operations, not as a default field on pure agent runs.
  - `substrate agent doctor`
    - MUST fail closed if the orchestrator is missing, denied, world-scoped when it must be host-scoped, or otherwise not eligible.
- Exit codes:
  - Exit code taxonomy: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
  - This ADR introduces no new exit codes.

### Identity model

For pure agent runs, the operator-visible record is:

- `client`
  - Meaning: the originating agent runtime or caller surface that initiated the run.
  - Examples: `codex`, `claude_code`, a future UAA-backed client.

- `router`
  - Meaning: the orchestration authority that accepted the run and selected placement/routing.
  - Example: `agent-hub` for agent orchestration records.

- `protocol`
  - Meaning: the agent backend contract being spoken.
  - For this ADR, that contract MUST align with capability-driven session semantics:
    - capability advertisement,
    - session handle acquisition,
    - resume/fork,
    - structured event/status exchange.

- `backend_id`
  - Meaning: the adapter/backend id in `<kind>:<name>` form.
  - Usage: `agents.allowed_backends` and trace attribution only.
  - Rule: it MUST NOT be read as provider, auth authority, or protocol identity.

- `provider`
  - Meaning: absent on pure agent runs.
  - Presence rule: only present on a nested LLM request record when the agent triggers gateway-backed model execution.

- `auth_authority`
  - Meaning: absent on pure agent runs.
  - Presence rule: only present on a nested LLM request record when the agent triggers gateway-backed model execution and a credential or billing authority is relevant.

For nested LLM calls made by an agent, the visible record changes:

- `client`
  - stays the originating agent runtime or agent session.
- `router`
  - becomes `substrate-gateway`.
- `provider`
  - identifies the upstream model provider that actually fulfilled the request.
- `auth_authority`
  - identifies the credential/billing authority used for that nested request.
- `protocol`
  - identifies the gateway protocol surface, such as a Responses-style or Messages-style request.

Operator rule:
- Do not collapse the nested LLM identity back into the base agent record.
- Do not infer `provider` or `auth_authority` from `backend_id`.

Concrete example:
- Host orchestrator dispatches a world-scoped member agent
  - Agent Hub selects `backend_id=cli:claude_code` as the host-scoped orchestrator (role assignment is carried separately; do not encode role in the id).
  - Agent Hub records the pure orchestration run as:
    - `client=claude_code`
    - `router=agent-hub`
    - `protocol=uaa.agent.session`
    - `backend_id=cli:claude_code`
    - `provider`: absent
    - `auth_authority`: absent
  - Agent Hub then dispatches a world-scoped member backend:
    - `backend_id=cli:codex`
    - `client=codex`
    - `router=agent-hub`
    - `protocol=uaa.agent.session`
    - `world_id=world-17`
    - `world_generation=3`
    - `provider`: absent
    - `auth_authority`: absent
- The world-scoped member later calls LLM via `substrate-gateway`
  - The nested LLM request record is separate from the orchestration record:
    - `client=codex`
    - `router=substrate-gateway`
    - `protocol=openai.responses`
    - `backend_id=api:openai`
    - `provider=openai`
    - `auth_authority=codex_subscription`
  - Operator reading rule:
    - the agent run stays `agent-hub` + UAA protocol + `backend_id`
    - the nested LLM record carries `provider` and `auth_authority`
    - the `world_id` remains on the world-scoped agent record so boundary sharing stays verifiable

### Config
- This ADR introduces no new config files.
- Source of truth for config/policy storage remains ADR-0027, with tuple-axis policy clarifications from ADR-0043.
- Key config expectations:
  - `agents.hub.orchestrator_agent_id` selects the orchestrator by agent inventory id.
  - The selected orchestrator MUST be host-scoped.
  - If the selected orchestrator is world-scoped, the hub MUST fail closed.
  - World-scoped member agents MAY be dispatched into a shared world boundary for the orchestration session.
  - `agents.allowed_backends` remains the backend allowlist keyed by `backend_id`.
  - `llm.allowed_backends` and the tuple-axis allowlists from ADR-0043 remain the gateway-side policy gates for nested LLM requests.
- Backend id mapping:
  - `backend_id = "<kind>:<agent_id>"`
  - This derived id is the only id used for allowlisting and trace attribution on the agent side.

### Platform guarantees
- Linux:
  - Host-scoped orchestration is allowed.
  - World-scoped member agents must fail closed if the world boundary is unavailable and routing is fail-closed.
  - World reuse and drift restarts must remain visible via `world_id`, `world_generation`, and alert events.
- macOS:
  - Same operator-visible semantics as Linux.
- Windows:
  - Same operator-visible semantics as Linux.

## Architecture Shape
- Components:
  - `crates/agent-hub` (new or successor module): registry, session router, and world-placement coordinator.
  - `crates/shell`: operator-facing list/status/doctor presentation.
  - `crates/trace`: canonical event and trace attribution sink.
  - `substrate-gateway`: nested LLM fulfillment boundary for agent-triggered model calls.

- End-to-end flow:
  - Inputs:
    - agent inventory entry and derived `backend_id`
    - capability descriptor from the backend contract
    - orchestration role assignment
    - world placement policy
  - Derived state:
    - agent session handle
    - active `world_id`
    - `world_generation`
    - event-plane attribution fields
  - Actions:
    - choose the host-scoped orchestrator
    - dispatch member agents using UAA-style session semantics
    - record pure agent events without provider/auth fields
    - route nested LLM calls through `substrate-gateway` when an agent explicitly asks for model help
  - Outputs:
    - status/doctor output
    - structured event stream
    - trace records that remain joinable without heuristic inference

## Sequencing / Dependencies
- Sequencing entry: `docs/project_management/packs/sequencing.json` → `agent-hub-core-successor`
- Prerequisite integration task IDs:
  - ADR-0042 must exist before this successor is considered complete.
  - ADR-0043 must exist before policy/config interpretation is considered complete.
  - ADR-0027 remains the source of truth for config file families and fail-closed posture.
  - ADR-0017 and ADR-0028 remain the event-plane and trace prerequisites.
  - ADR-0040 and ADR-0041 remain the gateway ownership and adapter prerequisites.

## Work Lift (discovery estimate)

<!-- PM_LIFT_VECTOR:BEGIN -->
```json
{
  "model_version": 1,
  "touch": {
    "create_files": 1,
    "edit_files": 1,
    "delete_files": 0,
    "deprecate_files": 0,
    "crates_touched": 0,
    "boundary_crossings": 1
  },
  "contract": {
    "cli_flags": 0,
    "config_keys": 0,
    "exit_codes": 0,
    "file_formats": 0,
    "behavior_deltas": 1
  },
  "qa": { "new_test_files": 0, "new_test_cases": 0 },
  "docs": { "new_docs_files": 1 },
  "ops": { "new_smoke_steps": 0, "ci_changes": 0 },
  "risk": {
    "cross_platform": true,
    "security_sensitive": true,
    "concurrency_or_ordering": true,
    "migration_or_backfill": false,
    "unknowns_high": false
  },
  "notes": "Docs-only successor ADR; no implementation or pack-file work is created by this change."
}
```
<!-- PM_LIFT_VECTOR:END -->

## Security / Safety Posture
- Fail-closed rules:
  - Missing or invalid orchestrator selection MUST fail closed.
  - World-scoped orchestration MUST fail closed when the world boundary is unavailable and routing is fail-closed.
  - `backend_id` MUST remain the only backend allowlist key on the agent side.
  - Pure agent records MUST NOT synthesize `provider` or `auth_authority`.
  - Nested LLM requests MUST be gated separately by gateway policy and must not inherit permissions from agent orchestration by implication.
- Protected invariants:
  - `client` is not a substitute for `backend_id`.
  - `router` is not a substitute for `provider`.
  - `protocol` does not convey approval authority.
  - `provider` and `auth_authority` are absent unless an agent actually triggers gateway-backed LLM execution.

## Validation Plan (Authoritative)

### Tests
- Unit tests: not required for this ADR.
- Integration tests: not required for this ADR.

### Manual validation
- Confirm the host orchestrator is selected from `agents.hub.orchestrator_agent_id` and is host-scoped.
- Confirm a world-scoped member agent is shown with `world_id` and `world_generation` in operator-visible status output.
- Confirm a pure agent run does not report `provider` or `auth_authority`.
- Confirm a nested LLM-backed sub-operation does report `provider` and `auth_authority` on the gateway record, not on the pure agent record.

## Rollout / Backwards Compatibility
- This ADR is additive in implementation terms but supersedes ADR-0025 in semantic terms.
- Existing backend ids remain valid allowlist and trace identifiers.
- Operators should stop reading backend ids as if they encode provider, auth authority, or protocol meaning.
- Any future implementation work must preserve the distinction between agent orchestration identity and nested LLM identity.

## Decision Summary
- Options (required; at least two):
  - A) Keep the old backend-id-centric agent hub model and let operator context infer provider/auth/protocol meaning.
  - B) Adopt a UAA-compatible agent hub model with explicit `client`, `router`, `protocol`, and nested LLM tuple handling, while keeping `backend_id` adapter-only.
- Selection:
  - Chosen: B
  - Rationale: this is the smallest model that keeps orchestration understandable, preserves fail-closed control-plane behavior, and prevents `backend_id` from becoming an overloaded catch-all.
