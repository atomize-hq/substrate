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

#### `agents.hub` (additive; Phase 5)
- `agents.hub.orchestrator_agent_id: string`
  - Meaning: selects which agent inventory item id is assigned `role=orchestrator` for the current process/session.
  - Default (effective): empty string.
  - Constraint: if `agents.enabled=true`, this key MUST be non-empty and MUST refer to an eligible, allowlisted agent inventory item (enforced by the Agent Hub; ADR-0025).
- `agents.hub.world_restart.on_drift: auto_restart|fail_closed`
  - Meaning: how Agent Hub handles “world-relevant drift” during a long-running orchestration session.
  - Default (effective): `auto_restart`.

#### `agents.toolbox` (additive; Phase 5)
- `agents.toolbox.enabled: bool`
  - Meaning: whether the internal orchestration toolbox may run at all for the effective config.
  - Default (effective): `false`.
- `agents.toolbox.bind.transport: uds|tcp`
  - Meaning: preferred bind transport for the toolbox endpoint.
  - Default (effective): `uds`.

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
- `llm.secrets.env_allowed: [string]`
  - Meaning: allowlist of secret env var *names* that Substrate is permitted to read from the host process environment for host→world secret delivery to the in-world gateway/engine.
  - Default (effective): `[]` (deny-by-default; no secret host env reads allowed for LLM secret delivery).
  - Constraints: names only; values must never be stored in Substrate YAML; missing names fail closed with actionable errors.

### `agents`
- `agents.allowed_backends: [string]`
  - Default (effective): `[]` (deny-by-default).
- `agents.fail_closed.routing: bool`
  - Meaning: when `true`, agent executions that are configured/routed to run in-world MUST fail closed if a world boundary is not available (no host fallback).
  - Default (effective): `true`.
- `agents.host_credentials.read.allowed_backends: [string]`
  - Meaning: allowlist of backend ids permitted to read *host* credential material as part of a backend adapter’s host-side preparation step (e.g., reading a CLI’s existing login state so required auth fields can be delivered to an in-world component over a Substrate-owned secret channel).
  - Elements format: `<kind>:<name>`.
  - Default (effective): `[]` (deny-by-default).
  - Notes:
    - This key gates **host credential reads**, not network egress. Egress remains governed by `net_allowed` at the world boundary.
    - This key is intentionally backend-generic (not Codex-specific) so additional `cli:*` adapters can be added later without reshaping the policy surface.

### `workflow.router` (router daemon indirect execution)

Phase 8 additive note: ADR-0029 introduces an indirect execution path (trace event → request → action). This path MUST be explicitly policy-gated and fail-closed by default.

- `workflow.router.enabled: bool`
  - Default (effective): `false` (fail-closed).
- `workflow.router.allow_cross_workspace: bool`
  - Default (effective): `false` (fail-closed).
- `workflow.router.allowed_rule_ids: [string]`
  - Default (effective): `[]` (deny-by-default).
- `workflow.router.allowed_workflow_ids: [string]`
  - Default (effective): `[]` (deny-by-default).
- `workflow.router.allowed_target_workspace_ids: [string]`
  - Default (effective): `[]` (deny-by-default).

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

API agents:
- `config.api.base_url: string` (required for `config.kind=api`)
  - Meaning: upstream provider base URL used by this `api:*` backend (non-secret; auth is handled separately).
  - Constraints: MUST be an `https://` URL; MUST NOT include userinfo (`user:pass@`); MUST NOT include query params.
- `config.api.auth.env: [string]` (required for `config.kind=api`)
  - Meaning: required secret env var *names* that must be present on the host so Substrate can deliver their values to the in-world gateway/engine over a Substrate-owned secret channel.
  - Default: `[]` (but for `config.kind=api` it MUST be non-empty).
  - Constraints: names only; values must never be stored in Substrate YAML; missing names fail closed with actionable errors.

Capabilities:
- `config.capabilities.llm: bool` (default: `false`)
- `config.capabilities.mcp_client: bool` (default: `false`)

### `policy_overlay` (restriction-only; allowed subset)
The overlay may include only these keys (all optional):
- `world_fs.*` (ADR-0018 keys; may only tighten)
- `agents.fail_closed.routing`
- `agents.host_credentials.read.allowed_backends`
- `llm.fail_closed.routing`
- `llm.secrets.env_allowed`
- `net_allowed`
- `cmd_allowed`
- `cmd_denied`
- `cmd_isolated`
- `require_approval`
- `limits.*`

### `policy_overlay` composition rules (restriction-only AND semantics)

Agent `policy_overlay` MUST be restriction-only: it MUST NOT broaden permissions beyond the effective base policy derived from `policy.yaml` (global/workspace + defaults).

Error posture:
- If an overlay attempts to broaden a permission (by value or by list contents), Substrate MUST reject the agent file as invalid and MUST fail closed with exit code `2`.

General rules:
- Omitted keys in the overlay have no effect; the base policy governs.
- For keys composed by AND/OR rules below, the effective behavior is defined as “most restrictive wins”.
- For list keys where “subset-only” is required, the overlay list MUST be a subset of the base list (exact string match). If it is not, it is a broadening attempt and MUST be rejected.

Key-family rules:

1) **Subset-only allowlists (overlay can only narrow)**
- `llm.secrets.env_allowed`:
  - Overlay MAY further restrict the set of secret env var names eligible for host→world secret delivery.
  - Overlay list MUST be a subset of the base `llm.secrets.env_allowed`.
- `agents.host_credentials.read.allowed_backends`:
  - Overlay MAY further restrict which backends may read host credential material during adapter preparation.
  - Overlay list MUST be a subset of the base `agents.host_credentials.read.allowed_backends`.

2) **Boolean “tighten-only” constraints (overlay can require stricter posture)**
- `llm.fail_closed.routing`: effective is `base OR overlay` (overlay may require fail-closed routing).
- `agents.fail_closed.routing`: effective is `base OR overlay` (overlay may require fail-closed routing).
- `require_approval`: effective is `base OR overlay` (overlay may require approval).

3) **Allow/deny command surfaces**
- `cmd_denied`: union (overlay may deny additional commands).
- `cmd_isolated`: union (overlay may require isolation for additional commands).
- `cmd_allowed`: additional allow filter:
  - If the overlay `cmd_allowed` list is empty or omitted, it has no effect.
  - If the overlay `cmd_allowed` list is non-empty, a command MUST match BOTH:
    - the base allow rules (if any), AND
    - the overlay allow rules.
  - Rationale: overlay cannot broaden; it can only add additional allow constraints.

4) **Network allowlist**
- `net_allowed`: additional allow filter:
  - If the overlay `net_allowed` list is empty or omitted, it has no effect.
  - If the overlay `net_allowed` list is non-empty, an outbound host MUST be allowed by BOTH:
    - base `net_allowed`, AND
    - overlay `net_allowed`.

5) **Resource limits**
- `limits.*`: “more restrictive wins”:
  - Each limit field composes as: effective = `min(base, overlay)` when both are present.
  - If only one side specifies a given limit, that value is used.

6) **Filesystem policy (`world_fs.*`)**
- Overlay MAY only tighten `world_fs.*` constraints (ADR-0018); it MUST NOT broaden.
- Composition for `world_fs.*` is implementation-defined but MUST follow the restriction-only rule, and MUST fail closed on any broadening attempt.
