# ADR-0026 — Orchestration Toolbox via Internal MCP Server

## Status
- Status: Draft
- Date (UTC): 2026-02-09
- Owner(s): Spenser McConnell (Substrate)

## Scope
- Feature directory: `docs/project_management/next/orchestration_mcp_toolbox/`
- Sequencing spine: `docs/project_management/next/sequencing.json`
- Standards:
  - `docs/project_management/standards/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md`
  - `docs/project_management/standards/TASK_TRIADS_AND_FEATURE_SETUP.md`
  - `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Related Docs
- Decision Register: `docs/project_management/next/orchestration_mcp_toolbox/decision_register.md`
- Sequencing spine: `docs/project_management/next/sequencing.json`
- Dependency ADR: `docs/project_management/adrs/draft/ADR-0025-agent-hub-core-role-swappable.md`

## Executive Summary (Operator)

ADR_BODY_SHA256: b67a0d37f814f77c05a9555f9918eedc640407ee9dbebc10d779ef5c06ebfbea
ADR_BODY_SHA256: <run `make adr-fix ADR=<this-file>` after drafting>

### Changes (operator-facing)
- Substrate exposes an internal MCP server providing orchestration-only tools
  - Existing: Orchestration context and levers are primarily internal; CLI agents cannot uniformly access Substrate orchestration functions.
  - New: Substrate runs an internal MCP server (“substrate tools”) and exposes orchestration tools to whichever agent is assigned orchestrator role.
  - Why: Make orchestration levers/context available via tool calls, enabling role-swappable orchestrators (CLI or API) without bespoke SDK coupling.
  - Links:
    - `docs/project_management/adrs/draft/ADR-0026-orchestration-toolbox-mcp.md#L1`
    - `docs/project_management/next/orchestration_mcp_toolbox/decision_register.md`

## Problem / Context
- Substrate needs a clean mechanism to expose orchestration functions (agent/session discovery, trace/graph queries, policy views) to the orchestrator agent.
- MCP is the natural tool surface for agent interoperability.
- Access must be restricted by role and policy: non-orchestrator agents must not receive orchestration-only tools.

## Goals
- Implement an internal “substrate MCP server” that exposes orchestration tools.
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

## User Contract (Authoritative)

### CLI
- Commands:
  - `substrate mcp status [--json]`
    - Behavior: report whether the internal MCP toolbox is enabled and (when enabled) how to reach it for the current orchestration session:
      - `toolbox_enabled` (effective config)
      - `toolbox_version` (Decision Register DR-0003; v1 starts at `1`)
      - bind transport:
        - `unix://<absolute-path>` (UDS), or
        - `tcp://127.0.0.1:<port>` (loopback)
      - orchestrator identity and scope (`agent_id`, `role=orchestrator`, `execution.scope=host|world`)
    - Exit codes:
      - Exit code taxonomy: `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`
      - `0`: success (including “disabled”)
      - `2`: config/schema error (strict parsing)
      - `3`: required dependency unavailable (e.g., world boundary required but unavailable)
      - `4`: unsupported / missing prerequisites for required posture
  - `substrate mcp env [--json]`
    - Behavior: emit environment/config hints for orchestrator agents to connect to the toolbox.
      - Default output is shell-compatible exports.
      - `--json` outputs a structured object.
    - Output keys (authoritative):
      - `SUBSTRATE_MCP_ENDPOINT`: `unix://...` or `tcp://127.0.0.1:...`
      - `SUBSTRATE_MCP_TOOLBOX_VERSION`: `1`
    - Exit codes:
      - Exit code taxonomy: `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`
      - `0`: success
      - `2`: config/schema error (strict parsing)
      - `3`: required dependency unavailable (world boundary required but unavailable)
      - `4`: unsupported / missing prerequisites for required posture

### Config
- This ADR does not define new config file families. It MUST use the Phase 3 config/policy surface defined by ADR-0027.
- Source of truth (key paths + precedence + defaults):
  - `docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md`
  - `docs/project_management/next/llm_and_agent_config_policy_surface/SCHEMA.md`

- Additive config keys (authoritative):
  - `mcp.toolbox.enabled: bool`
    - Meaning: whether the internal MCP toolbox may run at all for the effective config.
    - Default: `false` (explicit enable required).
  - `mcp.toolbox.bind.transport: uds|tcp`
    - Meaning: preferred bind transport for the toolbox endpoint.
    - Default: `uds` (Decision Register DR-0001).

Constraints:
- Toolbox enable is gated by both config and policy allowlists:
  - If `mcp.toolbox.enabled=false`, the toolbox MUST be disabled.
  - If `mcp.toolbox.enabled=true` but `agents.enabled=false`, the toolbox MUST be disabled (agent hub is prerequisite).
  - If the orchestrator backend id is not allowlisted by effective policy (`agents.allowed_backends[*]`), the toolbox MUST be disabled (fail closed with a policy error).

### Platform guarantees
- The toolbox endpoint is internal-only:
  - UDS endpoints are created with user-only filesystem permissions by default.
  - TCP endpoints (when used) MUST bind to `127.0.0.1` only by default.
- Boundary inheritance (Decision Register DR-0005):
  - Tools inherit the orchestrator’s execution boundary (`execution.scope=host|world`) and MUST NOT silently change it.
- If the effective posture requires in-world execution (fail-closed routing), the MCP toolbox MUST fail closed when a world boundary is unavailable (no host fallback).

## Architecture Shape
- Components:
  - `crates/mcp-internal` (new): internal MCP server implementation (pmcp-based).
  - `crates/agent-hub` (from ADR-0025): provides agent/session inventory used by MCP tools.
  - `crates/trace` / `crates/substrate-graph` (existing): query surfaces for trace/graph tools.
  - `crates/broker` (existing): policy view/explain tool.

- End-to-end flow:
  - Inputs:
    - orchestrator role assignment
    - MCP tool invocation
  - Derived state:
    - role gating decision (allow/deny)
  - Actions:
    - execute tool handler using Substrate internal APIs
    - emit tool call event into trace pipeline
  - Outputs:
    - MCP tool response payload (JSON)
    - trace event record

## Sequencing / Dependencies
- Sequencing entry: `docs/project_management/next/sequencing.json` → `orchestration-mcp-toolbox` (to be scheduled)
- Prerequisite integration task IDs:
  - ADR-0025 Agent Hub Core (required for `list_agents`, `get_session_history`)
  - ADR-0017 Output/Event Contract (required for attribution + auditing)

## Security / Safety Posture
- Fail-closed rules:
  - Role gate (Decision Register DR-0004): if caller is not in orchestrator role, orchestration tools return a deny error.
  - Policy allowlist: orchestrator backend id MUST be allowlisted by `agents.allowed_backends[*]` or the toolbox is disabled.
  - Tools that expose sensitive data apply redaction per policy (reuse ADR-0028 redaction/caps rules).
- Protected paths/invariants:
  - Internal MCP bind endpoint is not exposed publicly by default.
  - Tool schemas are stable and versioned (Decision Register DR-0003).

## Validation Plan (Authoritative)

### Tests
- Unit tests:
  - role gating logic for each tool
  - schema validation and deterministic outputs
- Integration tests:
  - orchestrator agent calls internal MCP tools; non-orchestrator agent denied

### Manual validation
- Manual playbook: `docs/project_management/next/orchestration_mcp_toolbox/manual_testing_playbook.md`

### Smoke scripts
- Linux: `docs/project_management/next/orchestration_mcp_toolbox/smoke/linux-smoke.sh`
- macOS: `docs/project_management/next/orchestration_mcp_toolbox/smoke/macos-smoke.sh`
- Windows: `docs/project_management/next/orchestration_mcp_toolbox/smoke/windows-smoke.ps1`

## Rollout / Backwards Compatibility
- Policy: greenfield breaking is allowed
- Compat work: none

## Decision Summary
- Decision Register entries:
  - `docs/project_management/next/orchestration_mcp_toolbox/decision_register.md`:
    - DR-0001 (Bind transport: UDS-first vs TCP-only)
    - DR-0002 (MCP server lifecycle: per-session vs global singleton)
    - DR-0003 (Tool namespace + versioning: server-level version vs tool-name version)
    - DR-0004 (Authorization enforcement: central role gate vs per-tool ad-hoc checks)
    - DR-0005 (Tool execution boundary inheritance: inherit orchestrator scope vs always in-world)
