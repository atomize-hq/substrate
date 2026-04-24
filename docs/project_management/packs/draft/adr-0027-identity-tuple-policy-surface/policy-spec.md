# adr-0027-identity-tuple-policy-surface — policy spec

Owner standard:
- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Scope

- This spec is authoritative for tuple-aware runtime policy evaluation for ADR-0043.
- This spec owns the ordered interaction between:
  - gateway lifecycle config gates
  - backend allowlists
  - tuple-axis narrowing under `llm.constraints.*`
  - fail-closed routing posture
  - host env and host credential-read gates
  - downstream world-boundary and network gates
- This spec does not own tuple-field meanings, tuple-key grammar, telemetry field inventories, or cross-platform validation evidence.

Canonical references:
- `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/contract.md`
- `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/tuple-policy-schema-spec.md`
- `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/decision_register.md`
- `docs/contracts/substrate-gateway-policy-evaluation.md`
- `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/contract.md`
- `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/SCHEMA.md`
- `docs/project_management/adrs/draft/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md`
- `docs/project_management/adrs/draft/ADR-0043-adr-0027-identity-tuple-policy-surface.md`

## Governing inputs

The ordered evaluation in this spec uses these effective inputs and no others:

- config-gated lifecycle inputs:
  - `llm.gateway.enabled`
  - `llm.gateway.mode`
  - `llm.routing.default_backend`
- policy backend and routing inputs:
  - `llm.allowed_backends`
  - `llm.fail_closed.routing`
  - `llm.constraints.routers`
  - `llm.constraints.providers`
  - `llm.constraints.protocols`
  - `llm.constraints.auth_authorities`
- policy secret and credential-read gates:
  - `llm.secrets.env_allowed`
  - `agents.host_credentials.read.allowed_backends`
- downstream transport and egress gates:
  - `net_allowed`
  - `host_to_world_bridge`
- request-derived identity inputs:
  - originating `client`
  - selected `backend_id`
  - derived `router`
  - derived `provider`
  - derived `protocol`
  - derived `auth_authority`

Explicit non-inputs:

- backend inventory filenames
- gateway-local mutable config
- gateway-local token persistence
- `client` as a standalone tuple-policy key
- any new tuple-specific CLI flag or environment override

## Ordered evaluation flow

Tuple-aware runtime policy evaluation follows this fixed order:

1. Validate gateway lifecycle config:
   - `llm.gateway.enabled` must remain enabled on the effective config surface.
   - `llm.gateway.mode` must remain `in_world` for gateway lifecycle commands.
   - `llm.routing.default_backend` must resolve to a non-empty backend id.
2. Resolve the selected backend inventory entry and apply `llm.allowed_backends` before tuple derivation begins.
3. Derive the candidate identity tuple from the selected backend and the integrated auth source:
   - `router` resolves to `substrate_gateway`.
   - `protocol` resolves from the backend family.
   - `provider` resolves from the backend family.
   - `auth_authority` resolves from the integrated auth source when one is present.
4. Apply tuple-axis narrowing in this exact order:
   - `llm.constraints.routers`
   - `llm.constraints.protocols`
   - `llm.constraints.providers`
   - `llm.constraints.auth_authorities`
5. Resolve integrated auth source material:
   - complete allowlisted env auth material wins over host credential-file reads
   - blocked env auth is a policy denial
   - partial env auth is invalid integration
   - host credential-file reads are permitted only when env auth is absent and `agents.host_credentials.read.allowed_backends` allows the selected backend
6. Apply world-boundary posture:
   - missing required gateway/world components remain component unavailability
   - transient connection or timeout failures remain transient runtime failures
   - `llm.fail_closed.routing=true` preserves denial over host fallback when the required in-world route is unavailable
7. Apply downstream transport and egress gates:
   - `host_to_world_bridge` remains transport only
   - `net_allowed` remains the egress gate
   - neither surface can authorize a tuple-axis mismatch

## Tuple-axis deny taxonomy

### Backend gate denial

- If the selected backend id is absent from `llm.allowed_backends`, evaluation stops before tuple-axis narrowing.
- The deny explanation must use this message family:
  - `"<backend_id> is not allowlisted by effective policy llm.allowed_backends"`

### Router mismatch denial

- If `llm.constraints.routers` is non-empty and the effective `router` is not listed, evaluation denies.
- The deny explanation must use this message family:
  - `"effective gateway routing authority '<value>' is not allowlisted by llm.constraints.routers"`

### Protocol mismatch denial

- If `llm.constraints.protocols` is non-empty and the effective `protocol` is not listed, evaluation denies.
- The deny explanation must use this message family:
  - `"effective gateway protocol '<value>' is not allowlisted by llm.constraints.protocols"`

### Provider mismatch or unresolved denial

- If `llm.constraints.providers` is non-empty and the effective `provider` is unresolved, evaluation denies.
- If `llm.constraints.providers` is non-empty and the effective `provider` is present but not listed, evaluation denies.
- The deny explanation must use one of these message families:
  - `"effective gateway provider is unresolved while llm.constraints.providers is constrained"`
  - `"effective gateway provider '<value>' is not allowlisted by llm.constraints.providers"`

### Auth-authority mismatch or unresolved denial

- If `llm.constraints.auth_authorities` is non-empty and the effective `auth_authority` is unresolved, evaluation denies.
- If `llm.constraints.auth_authorities` is non-empty and the effective `auth_authority` is present but not listed, evaluation denies.
- The deny explanation must use one of these message families:
  - `"effective gateway auth authority is unresolved while llm.constraints.auth_authorities is constrained"`
  - `"effective gateway auth authority '<value>' is not allowlisted by llm.constraints.auth_authorities"`

## Failure posture

### Invalid integration

These cases are invalid integration and map to the existing invalid-integration bucket:

- `llm.routing.default_backend` is empty
- the backend inventory entry is missing or does not match the selected backend id
- the selected backend cannot derive a supported tuple shape
- partial env auth is present for the selected backend
- required host auth state cannot be parsed into the expected integrated-auth payload

### Policy or safety failure

These cases are policy or safety failures and map to the existing policy-failure bucket:

- `llm.gateway.enabled=false` on the effective config surface for lifecycle commands
- `llm.gateway.mode=host_only` for lifecycle commands
- `llm.allowed_backends` denies the selected backend
- any non-empty `llm.constraints.*` axis denies the effective tuple
- `llm.secrets.env_allowed` denies an env auth source that is otherwise present
- `agents.host_credentials.read.allowed_backends` denies the selected backend for host credential-file reads

### Component unavailable

These cases remain component unavailable:

- the required world or gateway socket is missing
- the world backend reports unavailable before a request can complete

### Transient runtime failure

These cases remain transient runtime failures:

- connection refused
- timeout
- connection reset
- broken pipe

## Explain-surface requirements

- `substrate policy current show --explain` is the authoritative merged inspection surface for `llm.constraints.*`.
- Explain output for tuple-aware denials must identify the exact policy key that denied the route.
- Explain output must use tuple labels, not backend surrogates:
  - `routing authority`
  - `protocol`
  - `provider`
  - `auth authority`
- Explain output must not relabel `router` as `backend`.
- Explain output must not imply that `host_to_world_bridge` or `net_allowed` can override a tuple-axis denial.

## Invariants

- Backend ids remain adapter gates and never become surrogates for `router`, `provider`, `protocol`, or `auth_authority`.
- Tuple-axis constraints narrow an already selected backend path; they never widen a denied backend.
- `host_to_world_bridge` remains transport only and never becomes a second policy control plane.
- `net_allowed` remains downstream of tuple-axis evaluation and cannot authorize a mismatched tuple.
- Empty tuple-axis lists remain unconstrained on that axis.

## Acceptance criteria

- The ordered runtime evaluation runs config gating, backend allowlisting, tuple derivation, tuple-axis narrowing, auth-source gating, and world-boundary classification in the sequence defined by this spec.
- Tuple-axis narrowing uses the exact order `routers`, `protocols`, `providers`, `auth_authorities`.
- Non-empty provider or auth-authority constraints deny unresolved effective values instead of degrading to an unconstrained path.
- Blocked env auth does not fall through to host credential-file reads, and partial env auth fails as invalid integration.
- Tuple-aware denials use the exact message families defined in this spec and always name the denying policy key.
- Missing world components remain component unavailability, and connection or timeout failures remain transient runtime failures.
