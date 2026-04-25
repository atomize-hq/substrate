# ADR-0045 — Orchestration Toolbox (Internal MCP; Identity- and Trace-Explicit)

## Status
- Status: Draft
- Date (UTC): 2026-04-03
- Owner(s): Spenser McConnell (Substrate)

## Scope
- Feature directory: `docs/project_management/packs/draft/orchestration-toolbox-mcp/`
- This ADR is docs-only; no pack files are created by this change.
- Sequencing spine: `docs/project_management/packs/sequencing.json`
- Standards:
  - `docs/project_management/system/standards/adr/EXECUTIVE_SUMMARY_STANDARD.md`

## Related Docs (links only)

This ADR supersedes the older orchestration-toolbox framing in ADR-0026 and should be read together with the tuple, gateway, and Agent Hub successor ADRs.

- Superseded ADR:
  - `docs/project_management/adrs/draft/ADR-0026-orchestration-toolbox-mcp.md`
- Identity tuple and deployment posture:
  - `docs/project_management/adrs/draft/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md`
- Tuple-axis policy surface:
  - `docs/project_management/adrs/draft/ADR-0043-adr-0027-identity-tuple-policy-surface.md`
- Agent Hub successor:
  - `docs/project_management/adrs/draft/ADR-0044-agent-hub-core-successor-identity-tuple-compatible.md`
- Config/policy foundation:
  - `docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md`
- Event and trace foundations:
  - `docs/project_management/adrs/draft/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md`
  - `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md`
- Gateway ownership and adapter contracts:
  - `docs/project_management/adrs/draft/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md`
  - `docs/project_management/adrs/draft/ADR-0041-substrate-gateway-backend-adapter-contract.md`

## Executive Summary (Operator)

ADR_BODY_SHA256: 41a4e1478057cd1a67695c503531f4fbcab3f4d08e2b3d896b7bfd8327f0e7d6
### Changes (operator-facing)
- Keep the orchestration toolbox as an internal MCP server, but make v1 explicitly introspection-only
  - Existing: ADR-0026 already intends an internal toolbox, but the operator-visible identity and trace story predates the tuple model.
  - New: toolbox calls are treated as control-plane reads with explicit `client`, `router`, `protocol`, and `backend_id` fields; v1 exposes no mutating tools and no nested LLM semantics.
  - Why: operators need a read-only toolbox that is easy to audit, fail closed, and impossible to confuse with a second execution plane.
  - Links:
    - `docs/project_management/adrs/draft/ADR-0026-orchestration-toolbox-mcp.md`
    - `docs/project_management/adrs/draft/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md`
    - `docs/project_management/adrs/draft/ADR-0044-agent-hub-core-successor-identity-tuple-compatible.md`

## Problem / Context
- Substrate needs a host-scoped orchestration toolbox so the orchestrator agent can inspect policy, sessions, traces, and graph views without bespoke SDK coupling.
- The older ADR-0026 intent remains correct on the main points:
  - internal MCP server
  - orchestrator-only access
  - introspection-only v1
  - UDS-first bind with optional loopback TCP
- What is still missing is the operator-facing contract now that the tuple model is explicit:
  - the toolbox must not overload `backend_id` with provider, auth, or protocol meaning
  - the toolbox must not be read as a second execution plane
  - the toolbox must not imply a host gateway
  - trace records must be joinable without heuristics and without leaking tool payloads by default

## Goals
- Keep the toolbox as an internal MCP server reachable only by the orchestrator.
- Keep v1 introspection-only.
- Align toolbox control-plane calls with the identity tuple model from ADR-0042.
- Align backend-id handling with the Agent Hub successor in ADR-0044.
- Keep toolbox placement host-scoped and UDS-first, with optional loopback TCP only when explicitly selected.
- Preserve trace joinability and correlation rules from ADR-0028.

## Non-Goals
- Adding mutating toolbox tools in v1.
- Introducing a new config file family or policy file family.
- Introducing a second permanent host gateway or a second execution plane.
- Defining a new nested LLM path for the toolbox itself in v1.
- Replacing the Agent Hub or gateway ownership ADRs.

## Control plane vs event plane (authoritative)

This ADR keeps the existing separation explicit:

- Control plane:
  - toolbox tool dispatch
  - policy introspection
  - agent/session inventory queries
  - trace and graph queries
  - operator explanation of allow/deny decisions
- Event plane:
  - append-only structured trace and status events
  - correlation records that describe what already happened

Non-negotiable rules:
- The toolbox MUST NOT become a second execution plane.
- The toolbox MUST NOT launch members, mutate worlds, or bypass Agent Hub/world boundaries.
- Any world-aware data the toolbox reads MUST come from existing Substrate internals and MUST still respect policy and redaction rules.
- If a future version adds nested LLM behavior, that behavior MUST be modeled as a separate gateway request and not conflated with the toolbox control-plane record.

## User Contract (Authoritative)

### CLI
- Commands:
  - `substrate agent toolbox status [--json]`
    - Behavior: report whether the internal orchestration toolbox is enabled, how it is bound, and which orchestrator identity is eligible to use it.
    - Output must include:
      - `toolbox_enabled`
      - `toolbox_version`
      - bind transport and endpoint location
      - orchestrator identity and scope
    - Exit codes:
      - Exit code taxonomy: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
      - `0`: success, including disabled
      - `2`: config/schema error
      - `3`: required dependency unavailable
      - `4`: unsupported or missing prerequisites for required posture
  - `substrate agent toolbox env [--json]`
    - Behavior: emit shell-compatible environment hints or structured JSON for connecting the orchestrator to the toolbox endpoint.
    - Output must include:
      - `SUBSTRATE_AGENT_TOOLBOX_ENDPOINT`
      - `SUBSTRATE_AGENT_TOOLBOX_VERSION`
    - Exit codes:
      - Exit code taxonomy: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
      - `0`: success
      - `2`: config/schema error
      - `3`: required dependency unavailable
      - `4`: unsupported or missing prerequisites for required posture

### Identity model

For toolbox control-plane tool calls, the operator-visible identity fields are:

- `client`
  - Meaning: the orchestrator agent runtime or caller surface that initiated the toolbox call.
  - Example: `claude_code`.

- `router`
  - Meaning: the control-plane authority that accepted the call and selected toolbox handling.
  - Example: `agent_toolbox`.

- `protocol`
  - Meaning: the toolbox wire contract.
  - Example: `mcp.toolbox.v1`.
  - This value carries the MCP protocol plus toolbox versioning.

- `backend_id`
  - Meaning: the adapter id for the orchestrator backend issuing the call.
  - Example: `cli:claude_code`.
  - Rule: `backend_id` is adapter-only. It is not provider, auth authority, or protocol identity.

- `provider`
  - Meaning: absent in toolbox v1.
  - Presence rule: only present if a nested LLM operation occurs, which toolbox v1 does not do.

- `auth_authority`
  - Meaning: absent in toolbox v1.
  - Presence rule: only present if a nested LLM operation occurs, which toolbox v1 does not do.

Operator rule:
- Do not infer `provider` or `auth_authority` from `backend_id`.
- Do not infer a second execution plane from the presence of `router=agent_toolbox`.

### Config
- This ADR does not add new config or policy file families.
- Source of truth for config and policy storage remains ADR-0027.
- Required enablement gates:
  - `agents.enabled`
  - `agents.toolbox.enabled`
  - `agents.toolbox.bind.transport`
- Policy gates:
  - `agents.allowed_backends` must allow the orchestrator backend id.
  - Any existing session/toolbox auth token gate must also pass.
- Transport rules:
  - `agents.toolbox.bind.transport=uds` is the default posture.
  - `agents.toolbox.bind.transport=tcp` is optional and remains loopback-only.
- Failure posture:
  - If the toolbox is disabled by config, policy, or token gating, the request MUST fail closed.

### Platform guarantees
- Host-scoped only:
  - The toolbox runs on the host and serves the orchestrator session context.
  - Any interaction with world state occurs through existing Substrate internals and does not imply a host gateway or a second router.
- UDS-first:
  - The preferred bind path is a Unix domain socket.
  - The toolbox endpoint path is deterministic per orchestration session.
  - Default permissions remain user-only.
- Optional loopback TCP:
  - If selected, TCP binds MUST be loopback-only.
  - TCP is a transport fallback, not a separate execution plane.

### Trace and event model
- Event types:
  - `toolbox_tool_call_start`
  - `toolbox_tool_call_complete`
- Required fields on all toolbox tool-call records:
  - `ts`
  - `event_type`
  - `component: "agent-toolbox"`
  - `session_id`
  - `orchestration_session_id`
  - `run_id`
  - `agent_id`
  - `client`
  - `router`
  - `protocol`
  - `role: "orchestrator"`
  - `backend_id`
  - `tool_call_id`
  - `toolbox_version`
  - `tool_name`
- Completion-only fields:
  - `outcome: "ok" | "error" | "denied"`
  - `duration_ms`
- Safe-by-default payload posture:
  - `args_omitted: true`
  - `result_omitted: true`
- Correlation rules:
  - Use the same correlation vocabulary as ADR-0028.
  - Do not invent a separate toolbox-specific join key when the existing session and run fields are sufficient.

### Concrete example
- Host orchestrator call:
  - `client=claude_code`
  - `backend_id=cli:claude_code`
  - `router=agent_toolbox`
  - `protocol=mcp.toolbox.v1`
  - toolbox tool: `substrate.get_effective_policy`
- Expected start record fields:
  - `event_type=toolbox_tool_call_start`
  - `tool_call_id=<stable id>`
  - `tool_name=substrate.get_effective_policy`
  - `component=agent-toolbox`
  - `session_id=<session>`
  - `orchestration_session_id=<orchestration session>`
  - `run_id=<run>`
  - `agent_id=<orchestrator agent>`
  - `client=claude_code`
  - `router=agent_toolbox`
  - `protocol=mcp.toolbox.v1`
  - `role=orchestrator`
  - `backend_id=cli:claude_code`
  - `args_omitted=true`
- Expected completion record fields:
  - same correlation fields
  - `event_type=toolbox_tool_call_complete`
  - `outcome=ok`
  - `duration_ms=<value>`
  - `result_omitted=true`
- Fields that must be absent in v1:
  - `provider`
  - `auth_authority`

## Architecture Shape
- Components:
  - `crates/agent-toolbox`: internal MCP server for toolbox reads and explanations.
  - the ADR-0044 successor surfaces in `crates/shell`, `crates/common`, and `crates/agent-api-*`: supply orchestrator identity, session state, and backend allowlist context without requiring ADR-0044 to introduce a dedicated `crates/agent-hub` crate.
  - `crates/broker`: source of policy views and explainability.
  - `crates/trace`: canonical sink for toolbox call start/complete records.
  - `crates/substrate-graph`: query surface for graph-backed introspection where available.
- End-to-end flow:
  - Inputs:
    - orchestrator identity
    - toolbox tool name and request payload
    - config and policy gates
    - auth token for the current session
  - Derived state:
    - allow/deny decision
    - stable `tool_call_id`
    - trace correlation fields
  - Actions:
    - execute a read-only handler through existing Substrate internals
    - emit start and complete records
    - redact sensitive data by default
  - Outputs:
    - MCP response payload
    - canonical trace record pair

## Sequencing / Dependencies
- Sequencing entry: `docs/project_management/packs/sequencing.json` → `orchestration-toolbox-mcp`
- Prerequisite integration task IDs:
  - ADR-0042 must exist before this successor is considered complete.
  - ADR-0043 must exist before policy/config interpretation is considered complete.
  - ADR-0044 must exist before backend-id and Agent Hub semantics are considered complete.
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
  - If `agents.enabled=false`, the toolbox MUST be disabled.
  - If `agents.toolbox.enabled=false`, the toolbox MUST be disabled.
  - If the orchestrator backend id is not allowed by `agents.allowed_backends`, the toolbox MUST be disabled.
  - If the session auth token is missing or invalid, the toolbox MUST deny the request.
  - If the bind transport is not permitted by `agents.toolbox.bind.transport`, the toolbox MUST fail closed.
  - v1 MUST NOT expose mutating tools.
- Protected invariants:
  - `backend_id` is adapter-only.
  - `provider` and `auth_authority` are absent in v1 toolbox calls.
  - `router=agent_toolbox` does not imply a second execution plane.
  - World state access must stay inside existing Substrate internals.

## Validation Plan (Authoritative)

### Tests
- Unit tests:
  - identity projection for toolbox calls
  - config and policy gating
  - trace record shape and omission defaults
  - transport selection and loopback-only enforcement
- Integration tests:
  - orchestrator can call `substrate.get_effective_policy`
  - non-orchestrator callers are denied
  - toolbox-disabled configs fail closed
  - start and complete trace records are emitted as a pair

### Manual validation
- Run `substrate agent toolbox status --json` and confirm the enabled flag, transport, endpoint, and orchestrator identity.
- Run `substrate agent toolbox env --json` and confirm the exported endpoint/version values.
- Invoke `substrate.get_effective_policy` through the toolbox and confirm trace fields include `tool_call_id`, `tool_name`, and correlation metadata.
- Confirm `provider` and `auth_authority` are absent from the toolbox record in v1.

## Rollout / Backwards Compatibility
- This ADR supersedes ADR-0026 semantically.
- The toolbox remains internal and read-only in v1, so there is no external API migration.
- Existing operator flows should treat the new identity fields as additive clarity, not as a change in execution behavior.
- Any later mutating toolbox surface would require a separate ADR and a separate rollout plan.

## Decision Summary
- Options (required; at least two):
  - A) Keep ADR-0026 as the source of truth and rely on implementation notes to explain identity and trace semantics.
  - B) Supersede ADR-0026 with an explicit control-plane contract that names `client`, `router`, `protocol`, `backend_id`, trace correlation, and the absence of nested LLM fields in v1.
- Selection:
  - Chosen: B
  - Rationale: operators need the toolbox to be obviously internal, read-only, and identity-explicit so it cannot be confused with a second execution plane or with nested gateway behavior.
