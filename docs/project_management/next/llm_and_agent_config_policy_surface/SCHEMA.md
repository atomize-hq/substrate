# llm_and_agent_config_policy_surface — SCHEMA (config + policy)

This document defines the authoritative key paths, types, defaults, and merge strategies introduced by ADR-0027.

Authoritative ADR:
- `docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md`

## General rules (strictness + errors)
- Unknown keys in config/policy patches MUST be rejected (exit code `2`).
- Invalid values/types for known keys MUST be rejected (exit code `2`).
- All merge strategies below are **per-key** and are unchanged from the existing model:
  - Config: workspace overrides global; env overrides apply only when no workspace exists.
  - Policy: workspace overrides global.

## Backend id format (shared)
Applies to:
- `llm.routing.default_backend`
- `llm.allowed_backends[*]`
- `agents.allowed_backends[*]`

Format:
- `<kind>:<name>`
  - `<kind>`: lowercase ascii `[a-z0-9_]+`
  - `<name>`: lowercase ascii `[a-z0-9_\\-]+`
Examples:
- `cli:codex`
- `cli:claude_code`
- `cli:gemini_cli`
- `api:openai`

Notes:
- This ADR does **not** enumerate the complete set of available backends. It only defines the id format and the config/policy key paths that select/allowlist them.
- The authoritative “what backends exist and what they do” contracts live in the gateway/engine and agent hub ADRs (Phase 4/5), and can be referenced during the Phase 8 circle-back once those contracts are accepted.

## Agent inventory directory (new)

Agent definitions are inventory items stored as one file per agent, similar to the deps inventory model (ADR-0011).

Locations (by scope):
- Global: `$SUBSTRATE_HOME/agents/<agent_id>.yaml` (default `$HOME/.substrate/agents/<agent_id>.yaml`)
- Workspace: `<workspace_root>/.substrate/agents/<agent_id>.yaml`

Requirements:
- The filename-derived `<agent_id>` MUST match the YAML field `id` exactly.
- Unknown keys in agent files MUST be rejected (strict schema).
- Agent files MUST NOT contain secrets.

Inventory precedence (per `id`):
1. Workspace agent file (if present)
2. Global agent file (if present)
3. Built-in defaults (if any; optional future)

## Config schema additions

Files:
- Global patch: `$SUBSTRATE_HOME/config.yaml`
- Workspace patch: `<workspace_root>/.substrate/workspace.yaml`

Merge strategy:
- All keys below are `replace` (workspace overrides global). No list merge across layers.

### `llm`
- `llm.enabled: bool`
  - Default (effective): `false`.
- `llm.gateway.enabled: bool`
  - Default (effective): `false`.
- `llm.gateway.mode: in_world|host_only`
  - Default (effective): `in_world`.
  - Constraint: `host_only` is only permissible when effective policy has `llm.fail_closed.routing=false`.
- `llm.routing.default_backend: string`
  - Default (effective): empty string (meaning “no default backend selected”).

### `agents`
- `agents.enabled: bool`
  - Default (effective): `false`.
- `agents.defaults.execution.scope: host|world`
  - Default (effective): `world`.
- `agents.defaults.cli.mode: persistent|per_request`
  - Default (effective): `persistent`.

## Policy schema additions

Files:
- Global patch: `$SUBSTRATE_HOME/policy.yaml`
- Workspace patch: `<workspace_root>/.substrate/policy.yaml`

Merge strategy:
- All keys below are `replace` (workspace overrides global). No list merge across layers.

### `llm`
- `llm.fail_closed.routing: bool`
  - Meaning: when `true`, LLM operations MUST fail closed if they cannot be executed inside a world boundary (no host fallback).
  - Default (effective): `true`.
- `llm.require_approval: bool`
  - Default (effective): `false`.
- `llm.allowed_backends: [string]`
  - Default (effective): `[]` (deny-by-default).

### `agents`
- `agents.allowed_backends: [string]`
  - Default (effective): `[]` (deny-by-default).
- `agents.fail_closed.routing: bool`
  - Meaning: when `true`, agent executions that are configured/routed to run in-world MUST fail closed if a world boundary is not available (no host fallback).
  - Default (effective): `true`.

## Interaction with existing policy keys
- Network egress remains governed by `net_allowed`.
  - LLM backends that egress MUST be constrained by `net_allowed` at the world boundary.
  - Empty `net_allowed` means deny-all outbound.

## Agent file schema (new)

Each agent file `<agent_id>.yaml` has:

Top-level:
- `version: 1` (required)
- `id: <agent_id>` (required; MUST match filename)
- `config: <agent_config>` (required)
- `policy_overlay: <policy_overlay>` (optional; restriction-only; see decision register DR-0007)

### `config` (agent-local config overlay)
These keys are interpreted as overrides on top of effective `config.yaml` defaults:
- `config.enabled: bool` (default: `true`)
- `config.kind: cli|api` (required)
- `config.execution.scope: host|world` (default: inherit `agents.defaults.execution.scope`)

CLI agents:
- `config.cli.binary: string` (default: empty meaning “resolve via PATH”)
- `config.cli.mode: persistent|per_request` (default: inherit `agents.defaults.cli.mode`)

Capabilities:
- `config.capabilities.llm: bool` (default: `false`)
- `config.capabilities.mcp_client: bool` (default: `false`)

### `policy_overlay` (restriction-only; allowed subset)
The overlay may include only these keys (all optional):
- `world_fs.*` (ADR-0018 keys; may only tighten)
- `agents.fail_closed.routing`
- `llm.fail_closed.routing`
- `net_allowed`
- `cmd_allowed`
- `cmd_denied`
- `cmd_isolated`
- `require_approval`
- `limits.*`
