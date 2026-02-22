# ADR-0026 — Orchestration Toolbox (Internal; MCP Protocol)

## Status
- Status: Draft
- Date (UTC): 2026-02-09
- Owner(s): Spenser McConnell (Substrate)

## Scope
- Feature directory: `docs/project_management/_archived/next/orchestration_mcp_toolbox/`
- Sequencing spine: `docs/project_management/packs/sequencing.json`
- Standards:
  - `docs/project_management/system/standards/planning/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md`
  - `docs/project_management/system/standards/triad/TASK_TRIADS_AND_FEATURE_SETUP.md`
  - `docs/project_management/system/standards/triad/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Related Docs
- Decision Register: `docs/project_management/_archived/next/orchestration_mcp_toolbox/decision_register.md`
- Sequencing spine: `docs/project_management/packs/sequencing.json`
- Dependency ADR: `docs/project_management/adrs/draft/ADR-0025-agent-hub-core-role-swappable.md`

## Executive Summary (Operator)

ADR_BODY_SHA256: fb909753c2550d697895759d7e68567ba2536457086afb5414d34bbfa8448889
### Changes (operator-facing)
- Substrate exposes an internal orchestration toolbox providing orchestrator-only tools (speaks MCP)
  - Existing: Orchestration context and levers are primarily internal; agents cannot uniformly access Substrate orchestration functions.
  - New: Substrate runs an internal toolbox server (MCP protocol) and exposes orchestration tools to whichever agent is assigned `role=orchestrator`.
  - Why: Make orchestration levers/context available via tool calls, enabling role-swappable orchestrators (CLI or API) without bespoke SDK coupling while keeping the `mcp` CLI namespace reserved for external MCP servers.
  - Links:
    - `docs/project_management/adrs/draft/ADR-0026-orchestration-toolbox-mcp.md#L1`
    - `docs/project_management/_archived/next/orchestration_mcp_toolbox/decision_register.md`

## Problem / Context
- Substrate needs a clean mechanism to expose orchestration functions (agent/session discovery, trace/graph queries, policy views) to the orchestrator agent.
- MCP is the natural tool surface for agent interoperability.
- Access must be restricted by role and policy: non-orchestrator agents must not receive orchestration-only tools.

## Goals
- Implement an internal toolbox server (MCP protocol) that exposes orchestration tools.
- Support tool gating by role:
  - orchestrator-role sessions: allowed
  - non-orchestrator sessions: denied
- Provide a stable tool namespace and schemas for:
  - session history retrieval
  - agent registry queries
  - policy introspection
  - trace/graph queries
- Ensure tool execution posture is deterministic and does not silently change the enforcement boundary:
  - tools inherit the orchestrator’s execution boundary (Decision Register DR-0005),
  - and tool invocation is attributed to the current `(orchestration_session_id, agent_id, role, world_id?)` context.

## Non-Goals
- Full external MCP marketplace/registry UX in v1 (this ADR focuses on internal toolbox exposure).
- Exposing privileged host operations directly (tool calls must respect world + policy constraints).
- Supporting in-world orchestrator processes in v1 (the v1 orchestrator is host-scoped; see ADR-0025 Decision Register DR-0007).

## User Contract (Authoritative)

### CLI
- Commands:
  - `substrate agent toolbox status [--json]`
    - Behavior: report whether the internal orchestration toolbox is enabled and (when enabled) how to reach it for the current orchestration session.
      - Naming note: the `mcp` subcommand namespace is reserved for external MCP servers; internal orchestration tools live under `agent toolbox`.
      - `toolbox_enabled` (effective config)
      - `toolbox_version` (Decision Register DR-0003; v1 starts at `1`)
      - bind transport:
        - `unix://<absolute-path>` (UDS), or
        - `tcp://127.0.0.1:<port>` (loopback)
      - orchestrator identity and scope (`agent_id`, `role=orchestrator`, `execution.scope=host`)
    - Exit codes:
      - Exit code taxonomy: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
      - `0`: success (including “disabled”)
      - `2`: config/schema error (strict parsing)
      - `3`: required dependency unavailable (e.g., world boundary required but unavailable)
      - `4`: unsupported / missing prerequisites for required posture
  - `substrate agent toolbox env [--json]`
    - Behavior: emit environment/config hints for orchestrator agents to connect to the toolbox endpoint.
      - Default output is shell-compatible exports.
      - `--json` outputs a structured object.
    - Output keys (authoritative):
      - `SUBSTRATE_AGENT_TOOLBOX_ENDPOINT`: `unix://...` or `tcp://127.0.0.1:...`
      - `SUBSTRATE_AGENT_TOOLBOX_VERSION`: `1`
    - Exit codes:
      - Exit code taxonomy: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
      - `0`: success
      - `2`: config/schema error (strict parsing)
      - `3`: required dependency unavailable (world boundary required but unavailable)
      - `4`: unsupported / missing prerequisites for required posture

### Config
- This ADR does not define new config file families. It MUST use the Phase 3 config/policy surface defined by ADR-0027.
- Source of truth (key paths + precedence + defaults):
  - `docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md`
  - `docs/project_management/_archived/next/llm_and_agent_config_policy_surface/SCHEMA.md`

- Additive config keys (authoritative):
  - `agents.toolbox.enabled: bool`
    - Meaning: whether the internal orchestration toolbox may run at all for the effective config.
    - Default: `false` (explicit enable required).
  - `agents.toolbox.bind.transport: uds|tcp`
    - Meaning: preferred bind transport for the toolbox endpoint (UDS-first; loopback TCP fallback).
    - Default: `uds` (Decision Register DR-0001).

Constraints:
- Toolbox enable is gated by both config and policy allowlists:
  - If `agents.toolbox.enabled=false`, the toolbox MUST be disabled.
  - If `agents.toolbox.enabled=true` but `agents.enabled=false`, the toolbox MUST be disabled (agent hub is prerequisite).
  - If the orchestrator backend id is not allowlisted by effective policy (`agents.allowed_backends[*]`), the toolbox MUST be disabled (fail closed with a policy error).

### Platform guarantees
- The toolbox endpoint is internal-only:
  - UDS endpoints are created with user-only filesystem permissions by default.
  - TCP endpoints (when used) MUST bind to `127.0.0.1` only by default.
- UDS endpoint placement + permissions (Decision Register DR-0007):
  - Deterministic per-session path (v1; host-scoped orchestrator):
    - Host-scoped toolbox: `$SUBSTRATE_HOME/run/agent-toolbox/<orchestration_session_id>.sock`
  - Default permissions:
    - parent directory: `0700`
    - socket file: `0600`
  - Stale socket handling:
    - On startup, if the target socket path already exists, the toolbox server MUST attempt to detect staleness and MUST unlink stale sockets before binding.
- Caller identity + authentication (Decision Registers DR-0008, DR-0009):
  - The Agent Hub MUST mint a per-orchestration-session toolbox auth token at session start.
  - The toolbox server MUST require the token for all connections/requests and MUST deny requests with missing/invalid tokens.
  - Token injection mechanism (default):
    - For Substrate-spawned orchestrator backends, the token MUST be injected via an inherited one-time pipe/FD (not via env) by default.
      - The orchestrator process MUST be told which FD contains the token via `SUBSTRATE_AGENT_TOOLBOX_TOKEN_FD: int` (Decision Register DR-0011). This env var carries only the FD number, not the token.
    - `substrate agent toolbox env --include-token` exists as a debug-only escape hatch for manual wiring and MUST NOT be required for normal operation.
- Boundary inheritance (Decision Register DR-0005):
  - Tools inherit the orchestrator’s execution boundary (`execution.scope=host` in v1) and MUST NOT silently change it.
  - Any world-only actions MUST be performed by dispatching world-scoped member agents; the toolbox server remains host-scoped in v1.

### v1 tool surface (authoritative; introspection-only; Decision Register DR-0010)
- All v1 tools are introspection-only (read-only) and MUST NOT directly mutate host/world state.
- Stable naming (Decision Register DR-0003):
  - Tool names are stable under `substrate.*`.
  - `toolbox_version=1` indicates the v1 schema set.

v1 tool list (stable names; minimum required set):
- `substrate.list_agents`
  - Return: the effective agent registry view (agent ids, derived backend ids, kinds, capabilities, eligibility, assigned role).
- `substrate.get_agent`
  - Input: `agent_id`
  - Return: the full inventory + effective runtime view for one agent (with redaction as applicable).
- `substrate.list_sessions`
  - Return: active sessions for the current orchestration context (including `orchestration_session_id`, `world_id` when applicable, and `world_generation`).
- `substrate.get_session_history`
  - Input: session identity (at minimum `orchestration_session_id`; optionally `agent_id`)
  - Return: recent structured events for the session (aligned to ADR-0017; redaction applied).
- `substrate.get_effective_policy`
  - Return: effective policy view (redacted; never includes secrets).
- `substrate.explain_policy`
  - Input: a policy query payload (e.g., “why is backend X denied?”)
  - Return: a structured explanation payload suitable for operator/agent debugging (redacted).
- `substrate.query_trace`
  - Input: a bounded query (filters + limit)
  - Return: trace records that the caller is permitted to view (redaction applied).
- `substrate.query_graph`
  - Input: a bounded query (filters + limit)
  - Return: graph/query results derived from trace data where available (redaction applied).

Explicit exclusions (v1):
- No mutating tools such as `substrate.run_agent`, `substrate.cancel_run`, `substrate.restart_world`, or config/policy mutation tools.

### Tool-call trace events (Phase 8; authoritative; CC-0009)

Every authenticated toolbox tool invocation MUST emit canonical trace records so control-plane activity is auditable and joinable without heuristics.

Event types (v1; additive-only list):
- `toolbox_tool_call_start`
- `toolbox_tool_call_complete`

Emission rules (non-negotiable):
- The toolbox server MUST generate a stable `tool_call_id` for each invocation and MUST reuse it on the corresponding completion record.
- The toolbox server MUST append exactly one start record and exactly one completion record for each invocation.
- Tool-call trace records MUST be appended to canonical `trace.jsonl` (same file as command spans) and MUST follow ADR-0028 correlation vocabulary.
- Unauthorized/unauthenticated connection attempts (missing/invalid token) MUST be denied, but are not required to be written to canonical trace because they are not attributable to a stable `(agent_id, role, orchestration_session_id)` identity.

Required fields (all tool-call records):
- `ts` (RFC3339 UTC timestamp)
- `event_type` (`toolbox_tool_call_start` or `toolbox_tool_call_complete`)
- `component: "agent-toolbox"`
- `session_id`
- `orchestration_session_id`
- `run_id`
- `agent_id` (the orchestrator agent id)
- `role: "orchestrator"`
- `backend_id` (the orchestrator backend id; `<kind>:<name>`)
- `tool_call_id`
- `toolbox_version` (`1` in v1)
- `tool_name` (one of the stable `substrate.*` tool names listed above)

Completion-only required fields:
- `outcome: "ok" | "error" | "denied"`
- `duration_ms`

Denies (role/policy/tool gating; authenticated only):
- If the toolbox denies a request after authentication (e.g., wrong role, backend not allowlisted, tool not supported), it MUST:
  - emit `toolbox_tool_call_complete` with `outcome="denied"`, and
  - include a redacted, capped `error_summary` string (safe to print/persist; MUST NOT contain secrets).

Safe-by-default payload posture (non-negotiable):
- Tool-call trace records MUST NOT embed full tool request arguments or tool response bodies in v1.
  - Records MUST include:
    - `args_omitted: true`
    - `result_omitted: true`

## Architecture Shape
- Components:
  - `crates/agent-toolbox` (new): internal toolbox server implementation (MCP protocol; pmcp-based).
  - `crates/agent-hub` (from ADR-0025): provides agent/session inventory used by toolbox tools.
  - `crates/trace` / `crates/substrate-graph` (existing): query surfaces for trace/graph tools.
  - `crates/broker` (existing): policy view/explain tool.

- End-to-end flow:
  - Inputs:
    - orchestrator role assignment
    - toolbox tool invocation (MCP protocol)
  - Derived state:
    - role gating decision (allow/deny)
  - Actions:
    - execute tool handler using Substrate internal APIs
    - emit tool call event into trace pipeline
  - Outputs:
    - tool response payload (JSON; MCP protocol)
    - trace event record

## Sequencing / Dependencies
- Sequencing entry: `docs/project_management/packs/sequencing.json` → `orchestration-mcp-toolbox` (to be scheduled)
- Prerequisite integration task IDs:
  - ADR-0025 Agent Hub Core (required for `list_agents`, `get_session_history`)
  - ADR-0017 Output/Event Contract (required for attribution + auditing)

## Security / Safety Posture
- Fail-closed rules:
  - Role gate (Decision Register DR-0004): if caller is not in orchestrator role, orchestration tools return a deny error.
  - Policy allowlist: orchestrator backend id MUST be allowlisted by `agents.allowed_backends[*]` or the toolbox is disabled.
  - Tools that expose sensitive data apply redaction per policy (reuse ADR-0028 redaction/caps rules).
  - Toolbox auth token (Decision Registers DR-0008, DR-0009):
    - Toolbox requests MUST be denied if the token is missing/invalid.
    - The token MUST NOT be written to trace logs or printed to stdout by default.
- Protected paths/invariants:
  - Internal toolbox bind endpoint is not exposed publicly by default.
  - Tool schemas are stable and versioned (Decision Register DR-0003).

## Validation Plan (Authoritative)

### Tests
- Unit tests:
  - role gating logic for each tool
  - schema validation and deterministic outputs
- Integration tests:
  - orchestrator agent calls toolbox tools; non-orchestrator agent denied

### Manual validation
- Manual playbook: `docs/project_management/_archived/next/orchestration_mcp_toolbox/manual_testing_playbook.md`

### Smoke scripts
- Linux: `docs/project_management/_archived/next/orchestration_mcp_toolbox/smoke/linux-smoke.sh`
- macOS: `docs/project_management/_archived/next/orchestration_mcp_toolbox/smoke/macos-smoke.sh`
- Windows: `docs/project_management/_archived/next/orchestration_mcp_toolbox/smoke/windows-smoke.ps1`

## Rollout / Backwards Compatibility
- Policy: greenfield breaking is allowed
- Compat work: none

## Decision Summary
- Decision Register entries:
  - `docs/project_management/_archived/next/orchestration_mcp_toolbox/decision_register.md`:
    - DR-0001 (Bind transport: UDS-first vs TCP-only)
    - DR-0002 (Toolbox server lifecycle: per-session vs global singleton)
    - DR-0003 (Tool namespace + versioning: server-level version vs tool-name version)
    - DR-0004 (Authorization enforcement: central role gate vs per-tool ad-hoc checks)
    - DR-0005 (Tool execution boundary inheritance: inherit orchestrator scope vs always in-world)
    - DR-0006 (CLI namespace: `agent toolbox` vs `mcp`)
    - DR-0007 (UDS endpoint placement + permissions: deterministic vs temp/random)
    - DR-0008 (Caller auth: per-session token vs implicit trust)
    - DR-0009 (Token injection: inherited one-time FD vs env var)
    - DR-0010 (Tool surface: introspection-only v1 vs includes orchestration actions)
    - DR-0011 (Token FD discovery: advertised FD via env var vs fixed FD)
