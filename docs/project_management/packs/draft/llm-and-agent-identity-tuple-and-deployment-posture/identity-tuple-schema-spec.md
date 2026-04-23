# llm-and-agent-identity-tuple-and-deployment-posture — identity tuple schema spec

This document defines the canonical machine-readable objects introduced by ADR-0042.

Authoritative inputs:
- `docs/project_management/adrs/draft/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/contract.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/pre-planning/spec_manifest.md`
- `docs/project_management/adrs/draft/ADR-0043-adr-0027-identity-tuple-policy-surface.md`

## Canonical object names

The canonical machine-readable object names are:
- `identity_tuple`
- `placement_posture`

Object-name rules:
- Status, diagnostics, and trace surfaces publish these names unchanged when they include tuple or posture metadata.
- `client_wiring.*` remains outside this contract.
- This document owns object names and object shapes only. `telemetry-spec.md` owns surface-specific placement and projection.

## `identity_tuple`

Concrete shape:

```text
{
  client: snake_case_id
  router: snake_case_id
  protocol: dotted_id
  provider?: snake_case_id
  auth_authority?: snake_case_id
}
```

Required-field rules:
- `client`, `router`, and `protocol` are required whenever `identity_tuple` is published.
- `provider` is omitted until the effective upstream provider is fixed for the reported surface.
- `auth_authority` is omitted until the effective credential or billing authority is fixed for the reported surface.
- `provider` and `auth_authority` omit independently.
- Omission uses field absence only. Writers do not emit `null`, empty strings, or placeholder tokens such as `unknown`.
- Writers do not publish a partial `identity_tuple` object that is missing `client`, `router`, or `protocol`.

Token grammar:
- `snake_case_id` regex:

```text
^[a-z][a-z0-9]*(?:_[a-z0-9]+)*$
```

- `dotted_id` regex:

```text
^[a-z][a-z0-9]*(?:\.[a-z0-9]+)+$
```

Grammar invariants:
- Tuple fields do not use the backend-id grammar `<kind>:<name>`.
- Tuple ids do not contain whitespace.
- Tuple ids do not contain uppercase characters.

Current locked router ids:
- `substrate_gateway`
- `direct_provider_path`

Router-id rules:
- `direct_provider_path` records direct provider fulfillment with no `substrate_gateway` mediation.
- `direct_provider_path` does not name a standing daemon or a second control plane.
- A later router id remains additive only when it reuses the same field meaning and token grammar.

## `placement_posture`

Concrete shape:

```text
{
  execution: "in_world" | "host_only"
  host_to_world_bridge?: true
}
```

Field rules:
- `execution` is required whenever `placement_posture` is published.
- `host_to_world_bridge` is omitted when no bridge transport participates.
- `host_to_world_bridge` is published only as the literal boolean `true`.
- Writers do not emit `host_to_world_bridge: false`.
- Writers do not publish a partial `placement_posture` object that omits `execution`.

Cross-field invariants:
- `execution="host_only"` with `host_to_world_bridge=true` is invalid.
- `host_to_world_bridge=true` records transport only. It does not redefine router identity.

## Cross-object invariants

When `identity_tuple` and `placement_posture` appear together:
- `router="direct_provider_path"` requires `placement_posture.execution="host_only"`.
- `router="direct_provider_path"` is invalid when `placement_posture.host_to_world_bridge=true`.
- `router="substrate_gateway"` is valid with `placement_posture.execution="in_world"` or `placement_posture.execution="host_only"`.
- `provider` and `auth_authority` omission does not weaken the meaning of `client`, `router`, or `protocol`.

## Compatibility rules

- Existing object names are fixed: `identity_tuple` and `placement_posture`.
- Existing field names, requiredness, and types are fixed.
- A later revision extends these objects additively through new optional fields only.
- Existing fields are not renamed, widened into union types, or repurposed.
- Consumers treat omitted optional fields as unresolved or not applicable. They do not infer substitute values.

## Examples

### Gateway-routed in-world fulfillment

```json
{
  "identity_tuple": {
    "client": "codex",
    "router": "substrate_gateway",
    "provider": "openai",
    "auth_authority": "codex_subscription",
    "protocol": "openai.responses"
  },
  "placement_posture": {
    "execution": "in_world"
  }
}
```

### Host-only direct provider fulfillment

```json
{
  "identity_tuple": {
    "client": "codex",
    "router": "direct_provider_path",
    "provider": "openai",
    "auth_authority": "codex_subscription",
    "protocol": "openai.responses"
  },
  "placement_posture": {
    "execution": "host_only"
  }
}
```

### Pre-provider-selection publication

```json
{
  "identity_tuple": {
    "client": "codex",
    "router": "substrate_gateway",
    "protocol": "openai.responses"
  },
  "placement_posture": {
    "execution": "in_world"
  }
}
```
