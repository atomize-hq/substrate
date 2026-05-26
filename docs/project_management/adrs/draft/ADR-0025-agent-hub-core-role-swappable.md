# ADR-0025 — Agent Hub Core (Role-Swappable Agent Backends)

## Status
- Status: Draft
- Date (UTC): 2026-02-09
- Owner(s): Spenser McConnell (Substrate)
- Superseded by: `docs/project_management/adrs/draft/ADR-0044-agent-hub-core-successor-identity-tuple-compatible.md`

## Scope
- Feature directory: `docs/project_management/_archived/next/agent_hub_core/`
- Sequencing spine: `docs/project_management/packs/sequencing.json`
- Standards:
  - `docs/project_management/system/standards/planning/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md`
  - `docs/project_management/system/standards/triad/TASK_TRIADS_AND_FEATURE_SETUP.md`
  - `docs/project_management/system/standards/triad/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Related Docs
- Decision Register: `docs/project_management/_archived/next/agent_hub_core/decision_register.md`
- Sequencing spine: `docs/project_management/packs/sequencing.json`

## Executive Summary (Operator)

ADR_BODY_SHA256: a1beafeadd0d0cb480745724dfb520a52ef5ae886dcd2174c2d66b4567db63dc
### Changes (operator-facing)
- Agent Hub provides a stable registry + session router for CLI and API agents
  - Existing: Substrate can run worlds, trace commands, and call a world-service API, but “agents” are not uniformly registered/routed as role-swappable backends.
  - New: Substrate maintains an Agent Hub registry where any backend (CLI or API) can assume the `orchestrator` role (privileged) or a non-orchestrator role (taxonomy label), and emits stable event attribution for concurrent routing.
  - Why: Enable consistent orchestration across Codex/Claude/Gemini CLIs and API agents without hardcoding roles into types.
  - Links:
    - `docs/project_management/adrs/draft/ADR-0025-agent-hub-core-role-swappable.md#L1`
    - `docs/project_management/_archived/next/agent_hub_core/decision_register.md`

## Problem / Context
- Substrate requires a central, deterministic orchestration layer that can assign roles and toolsets to agents.
- Agents must be role-swappable: the assigned `role` taxonomy label (e.g., `orchestrator`, `planning`, `quality_gate`) is determined by config + prompt + tools + guardrails, not by implementing a different interface.
- To support concurrent output and reliable routing, Agent Hub needs stable IDs and attribution aligned with ADR-0017.

Clarification (v1; non-negotiable):
- Role assignment is deterministic and config-driven in v1:
  - `orchestrator` is selected explicitly via `agents.hub.orchestrator_agent_id`.
  - All other eligible agents default to `member`.
  - Prompts/tools influence *behavior*, but do not implicitly assign privileged roles.

## Goals
- Define a stable Agent Backend interface usable by both CLI and API-based agents.
- Provide an in-process Agent Hub registry with:
  - agent registration/discovery,
  - session lifecycle tracking,
  - event bus for structured events,
  - stable attribution (`orchestration_session_id`, `thread_id`, `run_id`, `agent_id`, `role`).
- Specify world-session reuse semantics for multi-agent operation (default: one shared `world_id` per `orchestration_session_id` for all world-scoped agents; see Decision Register DR-0004).
- Ensure orchestration toolbelt access is restricted to agents operating in orchestrator role (via MCP/tool gating).

## Non-Goals
- Full UI/UX for agent lifecycle management in v1 (basic CLI only).
- Multi-tenant remote agent registration in v1.
- Persisting third-party agent credentials or internal session memory.

## Control plane vs event plane (v1; Phase 8 lock)

This ADR explicitly separates two planes to prevent “second execution plane” drift.

- **Control plane**: request/response operations that ask Substrate to perform orchestration-adjacent work (v1: internal toolbox tool calls; ADR-0026).
  - Control-plane operations MUST be explicitly gated (role + policy + auth token) and MUST be attributable to a concrete caller identity (no heuristic inference).
  - In v1, the control-plane tool surface is introspection-only (read-only); no mutating tools exist (ADR-0026, Decision Register DR-0010).

- **Event plane**: append-only structured events and trace records used for observability and deterministic joins (ADR-0017 envelope + ADR-0028 trace vocabulary).
  - Event-plane records are consumable for UI/REPL rendering and for routing/analytics, but MUST NOT be treated as a general-purpose execution trigger.

Non-negotiable invariants:
- The Agent Hub MUST NOT perform host/world actions merely because it observed event-plane records.
  - The only v1 indirect-execution surface is the workflow-router daemon (ADR-0029), which is separately policy-gated under `workflow.router.*` (ADR-0027) and has its own derived-event families (ADR-0028).
- Every authenticated toolbox tool invocation MUST emit tool-call trace records with a stable `tool_call_id` so control-plane activity is auditable and joinable without heuristics (ADR-0026, ADR-0028).

Control-plane enablement gates (v1; fail closed):
- Toolbox is enabled only when:
  - effective config has `agents.enabled=true` AND `agents.toolbox.enabled=true` (ADR-0027), AND
  - the orchestrator backend id is allowlisted by effective policy `agents.allowed_backends[*]` (ADR-0027), AND
  - the caller presents a valid per-session auth token (ADR-0026).

## User Contract (Authoritative)

### CLI
- Commands:
  - `substrate agent list [--json] [--scope <host|world|any>] [--role <ROLE>]`
    - Behavior: list agent inventory items visible in the effective scope (workspace overrides global), including:
      - `agent_id`
      - `backend_id` (derived; see Config)
      - `kind` (`cli|api`)
      - declared capabilities (from inventory)
      - eligibility (`allowed|denied`) with a concise reason (policy allowlist, disabled, invalid schema)
      - assigned `role` (taxonomy label; see Config) for the current hub process
      - filter semantics:
        - `--scope` is a view filter based on each agent’s configured `execution.scope` (it MUST NOT change global world isolation toggles)
        - `--role` filters by assigned `role` (taxonomy label)
    - Exit codes:
      - Exit code taxonomy: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
      - `0`: success (including “agents disabled” with empty list and an explicit `disabled=true` flag in `--json`)
      - `2`: config/schema error in agent inventory or effective config (strict parsing)
      - `4`: feature unavailable on this platform/build
  - `substrate agent status [--json] [--scope <host|world|any>] [--role <ROLE>]`
    - Behavior: show live hub status for the current process, including:
      - effective orchestrator selection (the `agent_id` assigned `role=orchestrator`; configured by `agents.hub.orchestrator_agent_id`)
      - active sessions keyed by `(orchestration_session_id, agent_id)`
      - last event timestamp per session
      - world linkage when applicable:
        - `world_id`
        - `world_generation` (monotonic counter per `orchestration_session_id`; starts at `0`, increments on each hub-driven world restart)
      - filter semantics:
        - `--scope` and `--role` apply as view filters to the returned sessions/agents
    - Exit codes:
      - Exit code taxonomy: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
      - `0`: success (including “agents disabled”)
      - `2`: config/schema error (strict parsing) or invalid orchestrator selection
      - `4`: feature unavailable on this platform/build
  - `substrate agent doctor [--json]`
    - Behavior: validate that the Agent Hub can start deterministically and fail-closed where required:
      - inventory scan succeeds (strict schema)
      - orchestrator agent exists, is enabled, and is eligible
      - policy allowlist contains the orchestrator backend id
      - if any member agents are configured `execution.scope=world` and `agents.fail_closed.routing=true`, the world boundary is available
    - Exit codes:
      - Exit code taxonomy: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
      - `0`: all checks pass
      - `2`: config/schema error or invalid orchestrator selection
      - `3`: required dependency unavailable (e.g., world backend/socket missing when required)
      - `4`: unsupported / missing prerequisites for required posture
      - `5`: policy/safety violation (explicit deny)

### Config
- This ADR does not define new config file families. It MUST use the Phase 3 config/policy surface defined by ADR-0027.
- Source of truth (key paths + precedence + defaults):
  - `docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md`
  - `docs/reference/policy/schema.md`
- Agent backends are registered via the agent inventory directory (one file per backend), per ADR-0027:
  - Global: `$SUBSTRATE_HOME/agents/<agent_id>.yaml` (default `~/.substrate/agents/<agent_id>.yaml`)
  - Workspace: `<workspace_root>/.substrate/agents/<agent_id>.yaml`

- Backend id mapping (authoritative; see Decision Register DR-0001):
  - For an agent inventory item with `id=<agent_id>` and `config.kind=<kind>`, the derived backend id is:
    - `backend_id = "<kind>:<agent_id>"`
  - This derived `backend_id` is the value used for:
    - policy allowlist checks (`agents.allowed_backends[*]`), and
    - trace attribution (`backend_id` field on structured agent events).

- Role taxonomy and assignment (authoritative; see Decision Register DR-0003):
  - `role` is an attribution + permissioning label used by the Agent Hub and downstream tooling.
  - `role` is NOT a closed enum at the architecture level. It is a taxonomy label that can grow over time (e.g., `planning`, `quality_gate`, `devops`, `triage`, …).
  - Contract stability rules:
    - Substrate reserves certain well-known role strings for core gating surfaces (v1: `orchestrator`).
    - Other role strings MAY be introduced later as additive extensions, but MUST be strict, documented, and policy-gated where they affect privileges.

  - v1 role set (this ADR):
    - `orchestrator`: the single agent permitted to access orchestration-only tools (see ADR-0026).
    - `member`: the default role for other eligible agents in the session (non-privileged; taxonomy label).
    - `unassigned`: not eligible (disabled, denied by policy allowlist, or invalid).

  - New additive config key:
    - `agents.hub.orchestrator_agent_id: <agent_id>`
      - Meaning: selects the agent inventory item with `id=<agent_id>` (resolved via the agent inventory directory precedence above).
  - If `agents.enabled=true` and this key is missing or points to an ineligible agent, Agent Hub MUST fail closed with a config error (exit code `2`).
  - Orchestrator execution scope posture (authoritative; see Decision Register DR-0007):
    - The orchestrator agent selected by `agents.hub.orchestrator_agent_id` MUST have `config.execution.scope=host`.
    - If the selected orchestrator has `config.execution.scope=world`, Agent Hub MUST fail closed with an actionable config error (exit code `2`).
    - In-world execution is driven indirectly by dispatching world-scoped member agents with their own toolsets/policy overlays (the orchestrator does not need to run in-world).

  - World drift handling (authoritative; see Decision Register DR-0008):
    - New additive config key:
      - `agents.hub.world_restart.on_drift: auto_restart|fail_closed`
        - Meaning: how the Agent Hub handles “world-relevant drift” during a long-running orchestration session.
        - Default: `auto_restart`.
    - Minimum required `reason` taxonomy (v1; strings):
      - `policy_snapshot_changed`
      - `workspace_root_changed`
      - `world_fs_policy_changed`
      - `net_policy_changed`
      - `execution_scope_changed`
    - Required behavior:
      - If `auto_restart`: hub MUST restart the world, increment `world_generation`, and emit a structured `world_restarted` event (Decision Register DR-0010) with a non-empty `reason`.
      - If `fail_closed`: hub MUST NOT restart automatically; it MUST fail closed with an actionable error (exit code `3`) AND MUST emit a structured “restart required” alert event (Decision Register DR-0009).

  - `world_restarted` alert event (authoritative; see Decision Register DR-0010):
    - When drift is detected under `agents.hub.world_restart.on_drift=auto_restart` and the hub restarts the world, the hub MUST emit a structured agent event with:
      - Envelope:
        - `kind: "alert"`
        - `orchestration_session_id` (required)
        - `run_id` (required)
        - `agent_id` (the orchestrator’s agent id; required)
        - `role: "orchestrator"` (required)
      - `data` (required fields):
        - `code: "world_restarted"`
        - `reason: <one of the DR-0008 taxonomy strings>`
        - `on_drift: "auto_restart"`
        - `previous_world_id: <string>`
        - `new_world_id: <string>`
        - `previous_world_generation: <int>`
        - `new_world_generation: <int>` (MUST equal `previous_world_generation + 1`)
        - `message: <human readable string>`

  - Drift “restart required” alert event (authoritative; see Decision Register DR-0009):
    - When drift is detected under `agents.hub.world_restart.on_drift=fail_closed`, the hub MUST emit a structured agent event with:
      - Envelope:
        - `kind: "alert"`
        - `orchestration_session_id` (required)
        - `run_id` (required)
        - `agent_id` (the orchestrator’s agent id; required)
        - `role: "orchestrator"` (required)
      - `data` (required fields):
        - `code: "world_restart_required"`
        - `reason: <one of the DR-0008 taxonomy strings>`
        - `required_action: "restart_world"`
        - `on_drift: "fail_closed"`
        - `world_id: <string>`
        - `world_generation: <int>`
        - `message: <human readable string>`
  - Default role assignment:
    - Every other *eligible* agent is assigned `member` (v1 default taxonomy label).
    - Non-eligible agents are `unassigned` and MUST NOT receive orchestration tools.

  - Future extensibility (non-normative; does not change v1 contract):
    - A future ADR may introduce explicit role taxonomy assignment (e.g., `agents.hub.role_overrides.<agent_id>=<role>`) and/or role→toolset policy mapping.

### Platform guarantees
- Host-only operation is supported:
  - Agents configured with `execution.scope=host` MUST execute on the host (even when world isolation is disabled via effective config/env/flags).
- Agent backends configured to execute in-world MUST execute inside a world boundary.
- If effective policy has `agents.fail_closed.routing=true`, agent executions configured/routed to run in-world MUST fail closed when a world boundary is unavailable (no host fallback).
- World session reuse (authoritative; see Decision Register DR-0004):
  - For a single `orchestration_session_id`, all agents with `execution.scope=world` share the same `world_id` by default.
  - If the hub restarts the world (changing `world_id`), it MUST emit a structured `world_restarted` event with a non-empty `reason`.

## Architecture Shape
- Components:
  - `crates/agent-hub` (new): registry + session manager + event bus.
  - `crates/agent-core` (new/small): shared types for IDs, roles, events (aligned to ADR-0017).
  - `crates/shell` (existing): consumes event bus for rendering; dispatches CLI commands `agents list/status`.
  - `crates/trace` (existing): records agent events with stable attribution.

- End-to-end flow:
  - Inputs:
    - config-defined backends
    - runtime registration (from wrappers)
    - orchestrator role assignment decision
  - Derived state:
    - active sessions keyed by `(agent_id, orchestration_session_id)`
  - Actions:
    - route tasks to selected backend session
    - publish structured events to hub bus and trace pipeline
  - Outputs:
    - `agents list/status` results
    - event stream for UI/REPL

## Sequencing / Dependencies
- Sequencing entry: `docs/project_management/packs/sequencing.json` → `agent-hub-core` (to be scheduled)
- Prerequisite integration task IDs:
  - ADR-0017 (Output/Event Contract) is prerequisite (field set must include stable routing attribution).
  - ADR-0027 schema extension is prerequisite:
    - Additive key `agents.hub.orchestrator_agent_id` MUST be added to the strict config schema before implementation.

## Security / Safety Posture
- Fail-closed rules:
  - If a backend is configured but not present/healthy, it is not eligible for role assignment.
  - Orchestration-only tools are not exposed to non-orchestrator sessions.
- Protected paths/invariants:
  - Hub registry state is in-memory (Decision Register DR-0002).
  - Any on-disk artifacts introduced in future (e.g., per-session endpoints, logs) MUST live under `$SUBSTRATE_HOME/` and MUST be created with user-only permissions by default.

## Validation Plan (Authoritative)

### Tests
- Unit tests:
  - role assignment mapping (config + overrides)
  - event attribution correctness (run_id/session_id)
- Integration tests:
  - stub backend registration and routing
  - concurrent event routing does not misattribute sessions

### Manual validation
- Manual playbook: `docs/project_management/_archived/next/agent_hub_core/manual_testing_playbook.md`

### Smoke scripts
- Linux: `docs/project_management/_archived/next/agent_hub_core/smoke/linux-smoke.sh`
- macOS: `docs/project_management/_archived/next/agent_hub_core/smoke/macos-smoke.sh`
- Windows: `docs/project_management/_archived/next/agent_hub_core/smoke/windows-smoke.ps1`

## Rollout / Backwards Compatibility
- Policy: greenfield breaking is allowed
- Compat work: none

## Decision Summary
- Decision Register entries:
  - `docs/project_management/_archived/next/agent_hub_core/decision_register.md`:
    - DR-0001 (Backend id mapping: derived vs explicit backend_id field)
    - DR-0002 (Registry persistence: in-memory vs file-backed runtime registry)
    - DR-0003 (Role assignment: explicit config selection vs implicit heuristics)
    - DR-0004 (World session reuse: shared per orchestration session vs per-agent worlds)
    - DR-0005 (Backend event streaming model: push vs pull)
    - DR-0006 (CLI command placement: top-level `substrate agent` vs `substrate host|world agent`)
    - DR-0007 (Orchestrator execution scope: host-scoped orchestrator vs allow in-world orchestrator)
    - DR-0008 (World drift handling: config lever `auto_restart` vs `fail_closed`)
    - DR-0009 (“Restart required” alert event schema: `kind=alert` + `data.code` vs new enum kind)
    - DR-0010 (`world_restarted` alert event schema: `kind=alert` + `data.code` vs `kind=status`)
