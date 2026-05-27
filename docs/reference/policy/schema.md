# Policy Schema

This document defines the stable ADR-0027 config and policy key paths, defaults, and merge
strategies.

Related references:
- `docs/reference/policy/contract.md`
- `docs/reference/policy/tuple_constraints.md`

## General rules

- Unknown config or policy keys MUST be rejected with exit code `2`.
- Invalid values for known keys MUST be rejected with exit code `2`.
- Merge remains per-key:
  - Config: workspace overrides global; environment overrides apply only when no workspace exists.
  - Policy: workspace overrides global.
- Keys defined here use replace semantics across layers unless noted otherwise.

## Backend id format

Applies to:
- `llm.routing.default_backend`
- `llm.allowed_backends[*]`
- `agents.allowed_backends[*]`
- `agents.host_credentials.read.allowed_backends[*]`

Format:
- `<kind>:<name>`
  - `<kind>`: lowercase ASCII `[a-z0-9_]+`
  - `<name>`: lowercase ASCII `[a-z0-9_-]+`

Backend ids are selector ids only. They are not substitutes for `client`, `router`, `provider`,
`auth_authority`, or `protocol`.

## Agent inventory

Agent definitions are stored as one file per agent:

- Global: `$SUBSTRATE_HOME/agents/<agent_id>.yaml`
- Workspace: `<workspace_root>/.substrate/agents/<agent_id>.yaml`

Requirements:
- The filename-derived `<agent_id>` MUST match the YAML field `id`.
- Unknown keys in agent files MUST be rejected.
- Agent files MUST NOT contain secrets.

Inventory precedence per `id`:
1. Workspace agent file
2. Global agent file
3. Built-in defaults, if any

## Config schema additions

### `llm`

- `llm.enabled: bool`
  - Default: `false`
- `llm.gateway.enabled: bool`
  - Default: `false`
- `llm.gateway.mode: in_world|host_only`
  - Default: `in_world`
  - Constraint: `host_only` is only valid when effective policy has
    `llm.fail_closed.routing=false`
- `llm.routing.default_backend: string`
  - Default: empty string

### `agents`

- `agents.enabled: bool`
  - Default: `false`
- `agents.defaults.execution.scope: host|world`
  - Default: `world`
- `agents.defaults.cli.mode: persistent|per_request`
  - Default: `persistent`
- `agents.hub.orchestrator_agent_id: string`
  - Default: empty string
- `agents.hub.world_restart.on_drift: auto_restart|fail_closed`
  - Default: `auto_restart`
- `agents.toolbox.enabled: bool`
  - Default: `false`
- `agents.toolbox.bind.transport: uds|tcp`
  - Default: `uds`

## Policy schema additions

### `llm`

- `llm.fail_closed.routing: bool`
  - Default: `true`
- `llm.require_approval: bool`
  - Default: `false`
- `llm.allowed_backends: [string]`
  - Default: `[]`
- `llm.secrets.env_allowed: [string]`
  - Default: `[]`

### `agents`

- `agents.allowed_backends: [string]`
  - Default: `[]`
- `agents.fail_closed.routing: bool`
  - Default: `true`
- `agents.host_credentials.read.allowed_backends: [string]`
  - Default: `[]`

### `workflow.router`

- `workflow.router.enabled: bool`
  - Default: `false`
- `workflow.router.allow_cross_workspace: bool`
  - Default: `false`
- `workflow.router.allowed_rule_ids: [string]`
  - Default: `[]`
- `workflow.router.allowed_workflow_ids: [string]`
  - Default: `[]`
- `workflow.router.allowed_target_workspace_ids: [string]`
  - Default: `[]`

## Additive tuple-policy note

ADR-0043 extends this policy family additively with tuple-axis narrowing constraints under
`llm.constraints`:

- `llm.constraints.routers`
- `llm.constraints.providers`
- `llm.constraints.protocols`
- `llm.constraints.auth_authorities`

Those keys narrow an already-selected backend path. They do not replace backend allowlists or
introduce a standalone `client` policy key. The authoritative grammar, defaults, evaluation order,
and deny wording for that additive surface live in `docs/reference/policy/tuple_constraints.md`.
