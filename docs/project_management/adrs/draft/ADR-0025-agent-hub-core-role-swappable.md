# ADR-0025 — Agent Hub Core (Role-Swappable Agent Backends)

## Status
- Status: Draft
- Date (UTC): 2026-02-03
- Owner(s): Spenser McConnell (Substrate)

## Scope
- Feature directory: `docs/project_management/next/agent_hub_core/`
- Sequencing spine: `docs/project_management/next/sequencing.json`
- Standards:
  - `docs/project_management/standards/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md`
  - `docs/project_management/standards/TASK_TRIADS_AND_FEATURE_SETUP.md`
  - `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Related Docs
- Plan: `docs/project_management/next/agent_hub_core/plan.md`
- Tasks: `docs/project_management/next/agent_hub_core/tasks.json`
- Spec manifest: `docs/project_management/next/agent_hub_core/spec_manifest.md`
- Specs: `docs/project_management/next/agent_hub_core/specs/*`
- Decision Register: `docs/project_management/next/agent_hub_core/decision_register.md`

## Executive Summary (Operator)

ADR_BODY_SHA256: <run `make adr-fix ADR=<this-file>` after drafting>

### Changes (operator-facing)
- Agent Hub provides a stable registry + session router for CLI and API agents
  - Existing: Substrate can run worlds, trace commands, and call a world-agent API, but “agents” are not uniformly registered/routed as role-swappable backends.
  - New: Substrate maintains an Agent Hub registry where any backend (CLI or API) can assume orchestrator/executor roles by profile/config, and emits stable event attribution for concurrent routing.
  - Why: Enable consistent orchestration across Codex/Claude/Gemini CLIs and API agents without hardcoding roles into types.
  - Links:
    - `docs/project_management/adrs/draft/ADR-0025-agent-hub-core-role-swappable.md#L1`

## Problem / Context
- Substrate requires a central, deterministic orchestration layer that can assign roles and toolsets to agents.
- Agents must be role-swappable: “orchestrator vs executor” is determined by config + prompt + tools + guardrails, not by implementing a different interface.
- To support concurrent output and reliable routing, Agent Hub needs stable IDs and attribution aligned with ADR-0017.

## Goals
- Define a stable Agent Backend interface usable by both CLI and API-based agents.
- Provide an in-process Agent Hub registry with:
  - agent registration/discovery,
  - session lifecycle tracking,
  - event bus for structured events,
  - stable attribution (`orchestration_session_id`, `thread_id`, `run_id`, `agent_id`, `role`).
- Ensure orchestration toolbelt access is restricted to agents operating in orchestrator role (via MCP/tool gating).

## Non-Goals
- Full UI/UX for agent lifecycle management in v1 (basic CLI only).
- Multi-tenant remote agent registration in v1.
- Persisting third-party agent credentials or internal session memory.

## User Contract (Authoritative)

### CLI
- Commands:
  - `substrate agents list`
    - Behavior: list registered agent backends (CLI + API), their declared capabilities, and current role assignments (if any).
    - Exit codes: `0` success; `4` hub unavailable.
  - `substrate agents status`
    - Behavior: show active sessions, last event time, and health of registered backends.
    - Exit codes: `0` success; `4` hub unavailable.

### Config
- Files and locations (precedence): project `.substrate/config.toml` then global `~/.substrate/config.toml`, env, CLI flags.
- Schema constraints (minimum required):
  - `[agents]`
    - `orchestrator_backend = \"<agent_id>\"` (default from global if not set)
  - `[agents.backends.<id>]`
    - `kind = \"cli\"|\"api\"`
    - `binary = \"<path>\"` (if kind=cli)
    - `endpoint = \"<url>\"` (if kind=api)
    - `enabled = true|false`
  - `[agents.roles]`
    - optional per-session override via CLI flags; default role assignment comes from config.

### Platform guarantees
- Any agent backend declared `world_required=true` must run inside world; fail closed otherwise.

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
- Sequencing entry: `docs/project_management/next/sequencing.json` → `agent-hub-core` (to be scheduled)
- Prerequisite integration task IDs:
  - ADR-0017 (Output/Event Contract) is prerequisite (field set must include stable routing attribution).

## Security / Safety Posture
- Fail-closed rules:
  - If a backend is configured but not present/healthy, it is not eligible for role assignment.
  - Orchestration-only tools are not exposed to executor-role sessions.
- Protected paths/invariants:
  - Hub state persisted only under `~/.substrate/` with user-only permissions if persistence is enabled (default: in-memory).

## Validation Plan (Authoritative)

### Tests
- Unit tests:
  - role assignment mapping (config + overrides)
  - event attribution correctness (run_id/session_id)
- Integration tests:
  - stub backend registration and routing
  - concurrent event routing does not misattribute sessions

### Manual validation
- Manual playbook: `docs/project_management/next/agent_hub_core/manual_testing_playbook.md`

### Smoke scripts
- Linux: `docs/project_management/next/agent_hub_core/smoke/linux-smoke.sh`
- macOS: `docs/project_management/next/agent_hub_core/smoke/macos-smoke.sh`
- Windows: `docs/project_management/next/agent_hub_core/smoke/windows-smoke.ps1`

## Rollout / Backwards Compatibility
- Policy: greenfield breaking is allowed
- Compat work: none

## Decision Summary
- Decision Register entries:
  - `docs/project_management/next/agent_hub_core/decision_register.md`:
    - DR-0001 (Registry persistence: in-memory vs file-backed)
    - DR-0002 (Backend interface: pull vs push streaming)

