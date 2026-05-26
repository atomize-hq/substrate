# llm-and-agent-identity-tuple-and-deployment-posture — policy spec

Owner standard:
- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Scope

- This spec is authoritative for routing-hint evaluation, the direct-provider permission boundary, and the transport-only bridge rule for ADR-0042.
- This spec reuses existing ADR-0027 and ADR-0043 policy inputs. It does not create new config files, policy files, or operator flags.
- This spec does not own exit-code taxonomy, gateway-local trust boundaries, or the tuple-axis key definitions themselves.

Canonical references:
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/contract.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/identity-tuple-schema-spec.md`
- `docs/contracts/gateway/policy-evaluation.md`
- `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/contract.md`
- `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/SCHEMA.md`
- `docs/project_management/adrs/draft/ADR-0043-adr-0027-identity-tuple-policy-surface.md`

## Governing inputs

The ordered evaluation in this spec uses only these pre-existing inputs:

- placement and fail-closed posture:
  - `llm.gateway.mode`
  - `llm.fail_closed.routing`
- backend and credential-read gates:
  - `llm.allowed_backends`
  - `agents.allowed_backends`
  - `llm.secrets.env_allowed`
  - `agents.host_credentials.read.allowed_backends`
- tuple-axis narrowing keys owned by ADR-0043:
  - `llm.constraints.routers`
  - `llm.constraints.providers`
  - `llm.constraints.protocols`
  - `llm.constraints.auth_authorities`
- request-scoped metadata:
  - originating `client`
  - selected `backend_id`
  - requested routing hint
  - candidate `provider`
  - candidate `auth_authority`
  - candidate `protocol`

Explicit non-inputs:

- gateway-local config files
- gateway-local admin mutation surfaces
- gateway-local token persistence
- provider-specific wrapper hints outside the normalized request metadata
- `client_wiring.*`

## Ordered evaluation flow

Policy evaluation for tuple-aware routing follows this order:

1. Resolve the effective placement floor from `llm.gateway.mode` and `llm.fail_closed.routing`.
2. Resolve the selected `backend_id` and apply the existing backend allowlists from ADR-0027 before tuple evaluation begins.
3. Derive the candidate identity tuple from the request and selected backend:
   - `client` comes from the originating runtime.
   - `router` starts as the requested router when the request names one, otherwise the placement-aware default:
     - `substrate_gateway` for gateway-mediated fulfillment
     - `direct_provider_path` only when host-only direct fulfillment is explicitly selected
   - `protocol`, `provider`, and `auth_authority` come from the routed request context and selected credential path.
4. If a routing hint is present, treat it as a requested provider selection only. A hint never supplies `client`, `router`, `auth_authority`, or placement authority by itself.
5. Validate the candidate tuple against the ADR-0043 narrowing keys:
   - `llm.constraints.routers` gates `router`
   - `llm.constraints.providers` gates `provider`
   - `llm.constraints.protocols` gates `protocol`
   - `llm.constraints.auth_authorities` gates `auth_authority`
6. Apply the direct-provider gate:
   - `router=direct_provider_path` is valid only when the effective placement is `host_only`
   - `router=direct_provider_path` is invalid when a bridge transport participates
   - `router=direct_provider_path` still requires backend allowlist success and host credential-read or env-read success
7. Finalize the route and only then allow publication or execution.

## Hint outcomes

### Accepted hint

- An accepted routing hint narrows provider selection after all policy gates above pass.
- An accepted hint can change the effective `provider`.
- An accepted hint does not rename `client`.
- An accepted hint does not rename `router` unless the effective routed path itself changed through ordinary policy evaluation.
- An accepted hint does not create a new `auth_authority`.

### Rejected hint

- A rejected routing hint is discarded.
- Evaluation continues on the baseline route selected from existing config, policy, and capability inputs.
- Rejection does not rewrite `client`, `router`, or `auth_authority`.
- Rejection does not establish implicit provider authority.
- Rejection does not bypass any backend allowlist, tuple-axis constraint, secret-read gate, or fail-closed placement rule.

### Denied path

- The request is denied when no baseline route remains after hint rejection.
- The request is denied when `llm.gateway.mode` requires in-world execution and the world boundary is unavailable.
- The request is denied when `router=direct_provider_path` is selected without an effective `host_only` placement.
- The request is denied when any configured tuple-axis constraint does not match the effective tuple.

## Unresolved-field denial rules

- `provider` and `auth_authority` may remain omitted on publication surfaces until their effective values are fixed.
- An allow decision cannot bypass a configured tuple-axis constraint through omission.
- If `llm.constraints.providers` is non-empty and the effective `provider` is unresolved, the request is denied.
- If `llm.constraints.auth_authorities` is non-empty and the effective `auth_authority` is unresolved, the request is denied.
- Rejected hints do not backfill placeholder values for unresolved fields.

## Direct-provider boundary

- `host_only` does not imply `direct_provider_path`.
- `router=substrate_gateway` remains valid for `in_world` and `host_only`.
- `router=direct_provider_path` records routing authority only. It does not name a second standing daemon or control plane.
- `provider`, `protocol`, or `auth_authority` alone cannot imply `direct_provider_path`.
- A bridge transport cannot upgrade a `substrate_gateway` route into `direct_provider_path`.

## Transport-only bridge rule

- `host_to_world_bridge` is transport only.
- The bridge does not authorize host fallback.
- The bridge does not authorize direct-provider fulfillment.
- The bridge does not authorize new credential sources.
- The bridge does not change ownership of `net_allowed` or any other world-egress control.

## Failure buckets reused from existing owners

- Invalid integration state, dependency unavailability, and policy denial remain separate outcomes.
- This spec fixes the tuple-aware decision points that feed those existing buckets.
- `docs/contracts/gateway/policy-evaluation.md` remains the owner of the final bucket taxonomy and operator explanation boundary.

## Acceptance criteria

- Routing hints are evaluated after backend allowlist resolution and before final route selection.
- Rejected hints never rewrite `client`, `router`, or `auth_authority`.
- `router=direct_provider_path` requires `host_only` and forbids bridge transport.
- Configured provider and auth-authority constraints deny unresolved effective values.
- The bridge stays transport-only and never creates policy authority.
