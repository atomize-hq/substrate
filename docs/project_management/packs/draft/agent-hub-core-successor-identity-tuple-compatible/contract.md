# agent-hub-core-successor-identity-tuple-compatible - contract

This file is the single authoritative operator-facing contract for ADR-0044 inside this feature pack. It defines `C-01`, the additive command-surface contract for `substrate agent list`, `substrate agent status`, and `substrate agent doctor`, and it closes the required decisions for CLI namespace, implementation placement, and nested LLM publication separation.

Authoritative inputs:
- `docs/project_management/adrs/draft/ADR-0044-agent-hub-core-successor-identity-tuple-compatible.md`
- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/pre-planning/spec_manifest.md`
- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/pre-planning/impact_map.md`
- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/pre-planning/workstream_triage.md`
- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/pre-planning/alignment_report.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/contract.md`
- `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/contract.md`
- `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/SCHEMA.md`
- `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`

## Authority handoff

- `C-01` is authoritative for:
  - the canonical command namespace for the successor operator surfaces
  - the exact human-readable and machine-readable output contract for `substrate agent list`, `substrate agent status`, and `substrate agent doctor`
  - the exact render and omission rules for `backend_id`, `execution.scope`, `role`, capability summary, `world_id`, `world_generation`, `provider`, and `auth_authority` on those command surfaces
  - the exact `agent_id -> backend_id` derivation rule
  - the exact config, exit-code, implementation-placement, and platform-guarantee boundaries reused by this feature
- `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/contract.md` remains authoritative for config and policy file families, precedence, and the deny-by-default allowlist posture.
- `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/SCHEMA.md` remains authoritative for the existing key families `agents.hub.orchestrator_agent_id`, `agents.hub.world_restart.on_drift`, `agents.allowed_backends`, and `llm.allowed_backends`.
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/contract.md` remains authoritative for the normalized meanings of `client`, `router`, `provider`, `auth_authority`, and `protocol`.
- The future `agent-hub-session-protocol-spec.md` remains authoritative for capability-descriptor and session-handle schemas. `C-01` only fixes the command-surface projection of those schemas.
- The future `policy-spec.md` remains authoritative for ordered deny evaluation.
- The future `telemetry-spec.md` remains authoritative for structured event and trace field placement.
- If any later feature-local document conflicts with `C-01` on operator command behavior, `C-01` wins.

## Decision closure

### DR-AHCSITC-01 - Canonical CLI namespace

- The canonical successor runtime namespace is `substrate agent ...`.
- The commands in that namespace are:
  - `substrate agent list [--json] [--scope <host|world|any>] [--role <ROLE>]`
  - `substrate agent status [--json] [--scope <host|world|any>] [--role <ROLE>]`
  - `substrate agent doctor [--json]`
- `substrate agents validate` remains supported as the inventory-validation compatibility leaf.
- `substrate agents validate` is inventory-only. It does not become an alias for `substrate agent list`, `substrate agent status`, or `substrate agent doctor`.
- This feature does not introduce `substrate agents list`, `substrate agents status`, or `substrate agents doctor`.

### DR-AHCSITC-02 - Successor implementation placement

- This feature does not introduce a new `crates/agent-hub` crate.
- The selected existing-crate owner set is:
  - `crates/shell` for CLI parsing, inventory rendering, status rendering, and doctor evaluation
  - `crates/common` for shared command-surface projection helpers and shared identity-field projection helpers reused across shell and telemetry work
  - `crates/agent-api-types`, `crates/agent-api-client`, and `crates/agent-api-core` for capability, session, and lifecycle models
- `substrate_gateway` remains the owner of nested LLM fulfillment.
- `crates/trace` remains outside this contract file's ownership. Trace placement is locked by `telemetry-spec.md`.

### DR-AHCSITC-03 - Nested LLM publication separation

- A pure-agent orchestration record and a nested gateway-backed LLM record are two separate records.
- A nested LLM record never mutates, widens, or retroactively annotates the base pure-agent record.
- `provider` and `auth_authority` appear only on the nested LLM record.
- `world_id` and `world_generation` remain on the world-scoped pure-agent session record and do not migrate onto the nested LLM record.

## `C-01` contract summary

`C-01` binds the successor command surfaces to one operator truth:

- `backend_id` remains the adapter identifier in `<kind>:<agent_id>` form.
- `execution.scope` remains the visibility and placement indicator for agent inventory and session rows.
- `role` remains a command-surface attribution label. The required core label in this feature is `orchestrator`.
- Pure-agent session rows use `router=agent_hub` and `protocol=uaa.agent.session`.
- Pure-agent rows omit `provider` and `auth_authority`.
- Nested gateway-backed rows use `router=substrate_gateway` and carry `provider` plus `auth_authority`.
- `substrate agent doctor` fails closed whenever orchestrator state, allowlist posture, or required world-boundary posture is invalid.
- No new config file family, env var family, or exit-code taxonomy override is introduced.

## CLI

### Common rules

- All three commands support `--json` additive output.
- `--json` output is machine-readable and stable for the keys defined in `C-01`.
- Later work may add keys. Later work does not remove, rename, or repurpose the keys defined here.
- Human-readable output uses the labels and omission rules defined in this file.
- `--scope` and `--role` are view filters only. They do not change effective config, world posture, or policy state.
- Stable ordering rules:
  - inventory rows sort by `agent_id` ascending byte order
  - session rows sort by `orchestration_session_id`, then `agent_id`, ascending byte order
  - nested LLM rows sort by parent `(orchestration_session_id, agent_id)`, then by nested `run_id` ascending byte order

### `substrate agent list`

#### Behavior

- `substrate agent list` lists the effective agent inventory after workspace-over-global overlay resolution.
- It shows one row per effective `agent_id`.
- It never renders nested LLM records.
- Exit `0` is valid when agents are disabled. In that case the command returns an empty list and an explicit disabled marker.

#### Human-readable columns

Human-readable list output renders these columns in this order:
1. `agent_id`
2. `backend_id`
3. `kind`
4. `execution.scope`
5. `role`
6. `capabilities`
7. `eligibility`
8. `protocol`

Human-readable render rules:
- `backend_id` renders exactly as `<kind>:<agent_id>`.
- `execution.scope` renders as `host` or `world`.
- `role` renders as `orchestrator` for the selected orchestrator and is blank for unassigned agents.
- `capabilities` renders the enabled capability tokens in this order: `llm`, `mcp_client`.
- If no capability token is enabled, `capabilities` renders `none`.
- `eligibility` renders `allowed` or `denied: <reason>`.
- `protocol` renders `uaa.agent.session`.
- `provider`, `auth_authority`, `world_id`, and `world_generation` do not appear in list output.

#### `--json` contract

```json
{
  "disabled": false,
  "scope_filter": "any",
  "role_filter": null,
  "agents": [
    {
      "agent_id": "codex",
      "backend_id": "cli:codex",
      "kind": "cli",
      "execution": {
        "scope": "world"
      },
      "role": null,
      "capabilities_summary": {
        "llm": true,
        "mcp_client": false
      },
      "eligibility": {
        "state": "allowed",
        "reason": null
      },
      "protocol": "uaa.agent.session"
    }
  ]
}
```

JSON rules:
- `disabled` is `true` only when effective config disables agents globally for the current command context.
- `scope_filter` is one of `host`, `world`, or `any`.
- `role_filter` is `null` when `--role` is absent.
- `execution.scope` is `host` or `world`.
- `role` is `null` for unassigned agents.
- `eligibility.state` is `allowed` or `denied`.
- `eligibility.reason` is `null` only when `state=allowed`.
- No list item includes `provider`, `auth_authority`, `world_id`, or `world_generation`.

### `substrate agent status`

#### Behavior

- `substrate agent status` reports the live successor view for the current process.
- It returns the selected orchestrator identity plus every active pure-agent session row that survives the requested filters.
- It renders nested gateway-backed LLM activity as separate correlated rows.

#### Human-readable sections

Human-readable status output renders:
1. one `orchestrator` summary block
2. one `sessions` section
3. one `nested_llm_records` section only when at least one nested record exists

Pure-agent session render rules:
- Each session row renders these fields in this order:
  1. `orchestration_session_id`
  2. `agent_id`
  3. `backend_id`
  4. `client`
  5. `router`
  6. `protocol`
  7. `execution.scope`
  8. `role`
  9. `last_event_at`
  10. `world_id`
  11. `world_generation`
- `client` renders exactly as the executing session's `agent_id`.
- `router` renders `agent_hub`.
- `protocol` renders `uaa.agent.session`.
- `world_id` and `world_generation` render only when `execution.scope=world`.
- Host-scoped rows omit `world_id` and `world_generation`.
- Pure-agent session rows omit `provider` and `auth_authority`.

Nested LLM render rules:
- Each nested row renders:
  1. `parent.orchestration_session_id`
  2. `parent.agent_id`
  3. `run_id`
  4. `backend_id`
  5. `client`
  6. `router`
  7. `provider`
  8. `auth_authority`
  9. `protocol`
- `router` renders `substrate_gateway`.
- `client` renders the parent agent session's `agent_id`.
- `run_id` renders the nested request correlation id.
- `provider` and `auth_authority` are required on every nested row.
- Nested rows omit `world_id` and `world_generation`.

#### `--json` contract

```json
{
  "disabled": false,
  "scope_filter": "any",
  "role_filter": null,
  "orchestrator_agent_id": "claude_code",
  "sessions": [
    {
      "orchestration_session_id": "sess_001",
      "agent_id": "codex",
      "backend_id": "cli:codex",
      "client": "codex",
      "router": "agent_hub",
      "protocol": "uaa.agent.session",
      "execution": {
        "scope": "world"
      },
      "role": null,
      "last_event_at": "2026-04-24T18:30:00Z",
      "world_id": "world-17",
      "world_generation": 3
    }
  ],
  "nested_llm_records": [
    {
      "parent": {
        "orchestration_session_id": "sess_001",
        "agent_id": "codex"
      },
      "run_id": "run_nested_001",
      "backend_id": "cli:codex",
      "client": "codex",
      "router": "substrate_gateway",
      "provider": "openai",
      "auth_authority": "codex_subscription",
      "protocol": "openai.responses"
    }
  ]
}
```

JSON rules:
- `orchestrator_agent_id` is the selected inventory id, not the derived `backend_id`.
- Each pure-agent session object includes `client`, `router`, and `protocol`.
- Each pure-agent session object omits `provider` and `auth_authority`.
- `world_id` and `world_generation` are both present or both absent.
- `world_generation` is an integer that starts at `0` for a fresh world allocation and increments by `1` on each hub-driven restart of that orchestration session's world.
- Each nested record includes `run_id` as the nested request correlation id.
- Each nested record includes `provider` and `auth_authority`.
- Each nested record omits `world_id` and `world_generation`.

### `substrate agent doctor`

#### Behavior

- `substrate agent doctor` validates deterministic startability of the successor control plane.
- It evaluates checks in this order:
  1. `inventory_scan`
  2. `orchestrator_selection`
  3. `policy_allowlist`
  4. `world_boundary`
- If a check fails, the command exits immediately with the mapped exit code. Later checks are omitted.
- `world_boundary` runs only when the effective config and effective inventory require a world-scoped member posture.
- `world_boundary` is `not_applicable` when no world-scoped member posture is required.

#### Human-readable output

Human-readable doctor output renders:
- one header line with `healthy` or `fail_closed`
- one orchestrator summary block
- one ordered checklist section using the four check ids above

Render rules:
- The orchestrator summary block includes `agent_id`, `backend_id`, and `execution.scope` when orchestrator selection succeeded.
- The orchestrator summary block never includes `provider` or `auth_authority`.
- A failing check renders `fail: <reason>`.
- A passing check renders `pass`.
- A not-applicable world-boundary check renders `not_applicable`.

#### `--json` contract

```json
{
  "healthy": true,
  "fail_closed": false,
  "orchestrator": {
    "agent_id": "claude_code",
    "backend_id": "cli:claude_code",
    "execution": {
      "scope": "host"
    }
  },
  "checks": [
    {
      "check": "inventory_scan",
      "status": "pass",
      "reason": null
    },
    {
      "check": "orchestrator_selection",
      "status": "pass",
      "reason": null
    },
    {
      "check": "policy_allowlist",
      "status": "pass",
      "reason": null
    },
    {
      "check": "world_boundary",
      "status": "not_applicable",
      "reason": null
    }
  ]
}
```

JSON rules:
- `healthy=true` implies `fail_closed=false`.
- `healthy=false` implies `fail_closed=true`.
- `orchestrator` is omitted only when `inventory_scan` fails before orchestrator selection runs.
- `checks[*].check` uses exactly the four check ids defined above.
- `checks[*].status` is `pass`, `fail`, or `not_applicable`.
- `checks[*].reason` is `null` only when `status` is `pass` or `not_applicable`.

## Config

- This feature introduces no new config file family.
- This feature introduces no new policy file family.
- This feature introduces no new environment variable.
- The reused config and policy roots remain:
  - global config: `$SUBSTRATE_HOME/config.yaml`
  - workspace config: `<workspace_root>/.substrate/workspace.yaml`
  - global policy: `$SUBSTRATE_HOME/policy.yaml`
  - workspace policy: `<workspace_root>/.substrate/policy.yaml`
  - global agent inventory: `$SUBSTRATE_HOME/agents/<agent_id>.yaml`
  - workspace agent inventory: `<workspace_root>/.substrate/agents/<agent_id>.yaml`
- Reused keys and exact meanings in this feature:
  - `agents.hub.orchestrator_agent_id`
    - selects the orchestrator by effective inventory `agent_id`
    - the selected orchestrator must resolve to `execution.scope=host`
  - `agents.allowed_backends`
    - remains the only agent-side backend allowlist key
    - every agent-side allowlist comparison uses the derived `backend_id`
  - `agents.hub.world_restart.on_drift`
    - `auto_restart` permits a hub-driven restart and increments `world_generation`
    - `fail_closed` converts world-relevant drift into a fail-closed routing outcome instead of a restart
  - `llm.allowed_backends`
    - remains the nested gateway-side allowlist key
    - it does not authorize pure-agent orchestration by implication

### Backend id derivation

- For an effective inventory item with:
  - `id=<agent_id>`
  - `config.kind=<kind>`
- the derived backend id is:
  - `backend_id = "<kind>:<agent_id>"`
- This is the only agent-side value used for:
  - allowlist checks under `agents.allowed_backends`
  - operator-facing `backend_id` rendering on list, status, and doctor
  - correlated attribution between pure-agent status output and later telemetry surfaces
- `backend_id` never stands in for `provider`, `auth_authority`, or `protocol`.

## Exit codes

- Exit code taxonomy: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
- This feature introduces no taxonomy override.
- `substrate agent list`
  - `0`: success, including the disabled-empty-list path
  - `2`: invalid CLI usage, invalid effective config, or invalid effective inventory
  - `4`: feature unsupported on the current platform or build
  - `1`: unexpected internal error
- `substrate agent status`
  - `0`: success, including the disabled-empty-status path
  - `2`: invalid CLI usage, invalid effective config, or invalid effective inventory
  - `4`: feature unsupported on the current platform or build
  - `1`: unexpected internal error
- `substrate agent doctor`
  - `0`: all required checks passed
  - `2`: invalid effective config, invalid effective inventory, missing orchestrator selection, unknown orchestrator id, disabled orchestrator, or world-scoped orchestrator selection
  - `3`: a required world dependency is unavailable for the effective world-scoped posture
  - `4`: the current platform or build cannot satisfy the required posture
  - `5`: explicit policy deny blocks the selected orchestrator or required world-scoped member path
  - `1`: unexpected internal error
- `substrate agents validate`
  - remains governed by the existing inventory-validation contract
  - does not inherit the `substrate agent doctor` dependency checks

## Platform guarantees

| Platform | `substrate agent list` | `substrate agent status` | `substrate agent doctor` |
| --- | --- | --- | --- |
| Linux | Same command namespace, same JSON keys, same omission rules, same `backend_id` derivation, same role and capability-summary rules. | Same pure-agent and nested-record split. World-scoped session rows render `world_id` and `world_generation`. | Same ordered checks and same fail-closed posture. Exit `3` when required world dependencies are unavailable. |
| macOS | Same command namespace, same JSON keys, same omission rules, same `backend_id` derivation, same role and capability-summary rules. | Same pure-agent and nested-record split. World-scoped session rows render `world_id` and `world_generation`. | Same ordered checks and same fail-closed posture. Exit `3` when required world dependencies are unavailable. |
| Windows | Same command namespace, same JSON keys, same omission rules, same `backend_id` derivation, same role and capability-summary rules. | Same pure-agent and nested-record split. World-scoped session rows render `world_id` and `world_generation`. | Same ordered checks and same fail-closed posture. Exit `3` when required world dependencies are unavailable. |

Platform invariants:
- Host-scoped orchestrator selection is required on every platform.
- World-scoped member visibility is additive. It does not change the host-scoped orchestrator requirement.
- A platform does not degrade by synthesizing host-only success when a required world-scoped posture is unavailable.

## Invariants and non-surfaces

- This feature adds no protected-path contract.
- This feature adds no filesystem-semantics contract.
- This feature adds no new top-level schema version for config or policy storage.
- Pure-agent command surfaces omit `provider` and `auth_authority`.
- Nested command surfaces that expose gateway-backed LLM activity require `provider` and `auth_authority`.
- `world_id` and `world_generation` belong to world-scoped pure-agent session rows only.
- `substrate agent list`, `substrate agent status`, and `substrate agent doctor` do not authorize control-plane actions from event-plane observation.
