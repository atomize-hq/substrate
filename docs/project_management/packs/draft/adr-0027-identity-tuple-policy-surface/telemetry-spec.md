# adr-0027-identity-tuple-policy-surface — telemetry spec

Owner standard:
- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Scope

- This spec is authoritative for tuple-aware policy-evaluation publication on status, diagnostics, and trace surfaces for ADR-0043.
- This spec owns the additive field placement, allow-versus-deny publication rules, and redaction posture for tuple-policy outcomes.
- This spec reuses the existing `identity_tuple` and `placement_posture` object names.
- This spec does not redefine tuple semantics, tuple object shape, gateway status envelope ownership, or ADR-0028 correlation keys.

Canonical references:
- `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/contract.md`
- `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/policy-spec.md`
- `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/compatibility-spec.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/identity-tuple-schema-spec.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/telemetry-spec.md`
- `docs/contracts/gateway/status-schema.md`
- `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md`
- `docs/TRACE.md`

## Stability guarantees

- Publication is additive only.
- `identity_tuple` and `placement_posture` remain the only tuple-aware machine-readable object names in scope.
- `status`, `client_wiring.*`, `session_id`, `span_id`, `cmd_id`, `world_id`, `backend_id`, and `policy_decision` keep their existing meanings.
- Consumers that ignore tuple-aware metadata continue to parse existing status and trace records.

## Publication surfaces

Tuple-aware policy evaluation publishes on these existing operator surfaces:

- `substrate world gateway status --json`
- `substrate world gateway status`
- `substrate world gateway sync`
- `substrate world gateway restart`
- canonical trace records appended to `~/.substrate/trace.jsonl`

This feature does not introduce a second telemetry channel or a trace-only tuple schema.

## Machine-readable status placement

`substrate world gateway status --json` continues to emit the status-envelope fields owned elsewhere and widens additively with tuple-aware policy metadata:

```json
{
  "status": "available",
  "client_wiring": {
    "openai_base_url": "http://gateway.test/openai",
    "anthropic_base_url": "http://gateway.test/anthropic"
  },
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

Placement rules:

- `identity_tuple` and `placement_posture` are top-level siblings of `status` and `client_wiring`.
- `identity_tuple` and `placement_posture` never publish under `client_wiring.*`.
- `identity_tuple` keeps the required fields `client`, `router`, and `protocol` whenever the object is present.
- `provider` and `auth_authority` omit by field absence when unresolved or inapplicable.
- `placement_posture` keeps the required field `execution` whenever the object is present.
- `status="unavailable"` may still emit `identity_tuple` and `placement_posture` when tuple derivation completed before runtime unavailability.

## Human-readable diagnostics

Human-readable gateway policy output uses this stable label order:

1. `originating client`
2. `routing authority`
3. `fulfillment provider`
4. `auth authority`
5. `protocol`
6. `deployment posture`
7. `bridge transport`

Rendering rules:

- Each emitted line uses the existing label text and the normalized id value.
- Missing optional fields produce no placeholder line.
- `bridge transport: host_to_world_bridge` appears only when `placement_posture.host_to_world_bridge=true`.
- Human-readable output does not rename `router` to `backend`.
- Human-readable output does not rename `provider` to backend inventory identity.

## Trace publication

Tuple-aware trace publication augments existing trace records with the same top-level object names:

```json
{
  "event_type": "command_complete",
  "session_id": "ses_123",
  "span_id": "spn_123",
  "backend_id": "api:openai",
  "policy_decision": {
    "action": "deny",
    "reason": "gateway_policy_blocked"
  },
  "identity_tuple": {
    "client": "codex",
    "router": "substrate_gateway",
    "provider": "openai",
    "auth_authority": "codex_subscription",
    "protocol": "openai.responses"
  },
  "placement_posture": {
    "execution": "in_world"
  },
  "tuple_policy": {
    "result": "deny",
    "denied_by": "llm.constraints.providers",
    "detail": "effective gateway provider 'openai' is not allowlisted by llm.constraints.providers"
  }
}
```

Trace rules:

- `identity_tuple` and `placement_posture` stay top-level additive objects on records that already carry ADR-0028 correlation keys.
- `backend_id` remains a separate selector and correlation field and never becomes a tuple surrogate.
- `tuple_policy.result` is `allow` or `deny`.
- `tuple_policy.denied_by` is omitted on allow records and required on deny records.
- `tuple_policy.detail` is omitted on allow records and required on deny records.
- `tuple_policy.denied_by` uses one of:
  - `llm.allowed_backends`
  - `llm.constraints.routers`
  - `llm.constraints.protocols`
  - `llm.constraints.providers`
  - `llm.constraints.auth_authorities`
  - `llm.secrets.env_allowed`
  - `agents.host_credentials.read.allowed_backends`
- Trace publication uses the same deny-detail wording family already defined by `policy-spec.md` and `contract.md`.

## Allow-versus-deny rules

### Allow records

Allow-path publication includes:

- `identity_tuple`
- `placement_posture`
- existing ADR-0028 correlation keys
- `backend_id` when a concrete backend is selected
- `tuple_policy.result = "allow"`

Allow-path publication omits:

- `tuple_policy.denied_by`
- `tuple_policy.detail`

### Deny records

Deny-path publication includes:

- `identity_tuple` when tuple derivation completed before the denial
- `placement_posture` when placement posture was already resolved
- existing ADR-0028 correlation keys
- `backend_id` when backend selection completed before the denial
- `tuple_policy.result = "deny"`
- `tuple_policy.denied_by`
- `tuple_policy.detail`
- existing `policy_decision` metadata

Deny-path publication does not emit:

- a second tuple object family
- placeholder values for unresolved `provider` or `auth_authority`
- secret-bearing auth-source details

## Redaction and omission rules

- `client`, `router`, `provider`, and `auth_authority` publish normalized lowercase snake_case ids only.
- `protocol` publishes normalized lowercase dotted ids only.
- `auth_authority` publishes the authority class only, not the credential value or token source payload.
- Telemetry does not emit:
  - API keys
  - access tokens
  - session cookies
  - gateway-delivered secret bundles
  - raw credential files
  - raw credential paths such as `~/.codex/auth.json`
  - provider endpoints copied from secret-bearing inputs
- When `provider` or `auth_authority` is unresolved, the field is omitted rather than replaced with `unknown`, `n/a`, or an empty string.

## Collision-avoidance boundary

- ADR-0028 remains the owner of canonical trace correlation keys and record-family semantics.
- The LAITDP pack remains the owner of tuple object names, required-field rules, and omission rules.
- This feature adds tuple-policy outcome metadata around those reused objects.
- This feature does not rename, nest, or widen the existing correlation-key surface.

## Consumer expectations

- Status readers continue to discover endpoints through `client_wiring.*`.
- Policy readers use `tuple_policy.*` only for tuple-aware allow and deny classification.
- Trace readers continue to join through ADR-0028 correlation keys and treat tuple-aware metadata as descriptive enrichment.

## Acceptance criteria

- Status and trace publication reuse `identity_tuple` and `placement_posture` without introducing a second tuple schema.
- Allow records publish tuple-aware metadata without deny-only fields.
- Deny records publish the denying policy key and exact deny-detail text while keeping existing correlation keys unchanged.
- No status, diagnostics, or trace record emits secret material or illustrative credential paths.
