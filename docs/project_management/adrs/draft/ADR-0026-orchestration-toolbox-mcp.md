# ADR-0026 — Orchestration Toolbox via Internal MCP Server

## Status
- Status: Draft
- Date (UTC): 2026-02-03
- Owner(s): Spenser McConnell (Substrate)

## Scope
- Feature directory: `docs/project_management/next/orchestration_mcp_toolbox/`
- Sequencing spine: `docs/project_management/next/sequencing.json`
- Standards:
  - `docs/project_management/standards/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md`
  - `docs/project_management/standards/TASK_TRIADS_AND_FEATURE_SETUP.md`
  - `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Related Docs
- Plan: `docs/project_management/next/orchestration_mcp_toolbox/plan.md`
- Tasks: `docs/project_management/next/orchestration_mcp_toolbox/tasks.json`
- Spec manifest: `docs/project_management/next/orchestration_mcp_toolbox/spec_manifest.md`
- Specs: `docs/project_management/next/orchestration_mcp_toolbox/specs/*`
- Decision Register: `docs/project_management/next/orchestration_mcp_toolbox/decision_register.md`

## Executive Summary (Operator)

ADR_BODY_SHA256: <run `make adr-fix ADR=<this-file>` after drafting>

### Changes (operator-facing)
- Substrate exposes an internal MCP server providing orchestration-only tools
  - Existing: Orchestration context and levers are primarily internal; CLI agents cannot uniformly access Substrate orchestration functions.
  - New: Substrate runs an internal MCP server (“substrate tools”) and exposes orchestration tools to whichever agent is assigned orchestrator role.
  - Why: Make orchestration levers/context available via tool calls, enabling role-swappable orchestrators (CLI or API) without bespoke SDK coupling.
  - Links:
    - `docs/project_management/adrs/draft/ADR-0026-orchestration-toolbox-mcp.md#L1`

## Problem / Context
- Substrate needs a clean mechanism to expose orchestration functions (agent/session discovery, trace/graph queries, policy views) to the orchestrator agent.
- MCP is the natural tool surface for agent interoperability.
- Access must be restricted by role and policy: executor agents must not receive orchestration-only tools.

## Goals
- Implement an internal “substrate MCP server” that exposes orchestration tools.
- Support tool gating by role:
  - orchestrator-role sessions: allowed
  - executor-role sessions: denied
- Provide a stable tool namespace and schemas for:
  - session history retrieval
  - agent registry queries
  - policy introspection
  - trace/graph queries

## Non-Goals
- Full external MCP marketplace/registry UX in v1 (this ADR focuses on internal toolbox exposure).
- Exposing privileged host operations directly (tool calls must respect world + policy constraints).

## User Contract (Authoritative)

### CLI
- Commands:
  - `substrate mcp status`
    - Behavior: show whether internal MCP server is running and where it is bound (UDS/loopback inside world).
    - Exit codes: `0` success; `4` unavailable.
  - `substrate mcp env`
    - Behavior: print exports / config hints for agents to connect to the internal MCP server.
    - `--json`: structured output.

### Config
- Files and locations: project `.substrate/config.toml` then global `~/.substrate/config.toml`.
- Schema constraints (minimum required):
  - `[mcp.internal]`
    - `enabled = true|false` (default true when orchestrator role is active)
    - `bind = \"unix://~/.substrate/sock/mcp.sock\"` or `http://127.0.0.1:<port>` (inside world)
    - `allowed_tools = [\"substrate.list_agents\", \"substrate.get_session_history\", ...]`
  - `[mcp.external]` (future-compatible placeholder)
    - external server registry is handled in a separate ADR if needed.

### Platform guarantees
- Internal MCP server runs inside world when worlds enabled; fail closed if world required.

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
  - If caller is not in orchestrator role, orchestration tools return a deny error.
  - Tools that expose sensitive data apply redaction per policy.
- Protected paths/invariants:
  - Internal MCP bind endpoint is not exposed publicly by default.
  - Tool schemas are stable and versioned.

## Validation Plan (Authoritative)

### Tests
- Unit tests:
  - role gating logic for each tool
  - schema validation and deterministic outputs
- Integration tests:
  - orchestrator agent calls internal MCP tools; executor agent denied

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
    - DR-0001 (Bind transport: UDS vs loopback TCP)
    - DR-0002 (Tool namespace + versioning strategy)

