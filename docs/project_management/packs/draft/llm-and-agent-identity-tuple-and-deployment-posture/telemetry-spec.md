# llm-and-agent-identity-tuple-and-deployment-posture — telemetry spec

Owner standard:
- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Scope

- This spec is authoritative for additive publication of `identity_tuple` and `placement_posture` on gateway status, diagnostics, and trace surfaces introduced by ADR-0042.
- This spec owns field placement, emission rules, omission rules, and redaction posture for those two objects.
- This spec does not redefine the object shapes from `identity-tuple-schema-spec.md`.
- This spec does not redefine the owned `status` or `client_wiring.*` fields from `docs/contracts/substrate-gateway-status-schema.md`.
- This spec does not redefine the canonical correlation keys from ADR-0028.

Canonical references:
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/contract.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/identity-tuple-schema-spec.md`
- `docs/contracts/substrate-gateway-status-schema.md`
- `docs/contracts/substrate-gateway-operator-contract.md`
- `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md`
- `docs/TRACE.md`

## Stability guarantees

- Publication is additive only.
- Existing `status` and `client_wiring.*` semantics remain unchanged.
- Existing ADR-0028 join keys remain unchanged.
- Consumers that ignore unknown top-level fields continue to parse status and trace output correctly.

## Status JSON placement

`substrate world gateway status --json` widens additively with two top-level fields:

```json
{
  "status": "available",
  "client_wiring": {
    "openai_base_url": "http://127.0.0.1:18080",
    "anthropic_base_url": "http://127.0.0.1:18080"
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
- `identity_tuple` and `placement_posture` never appear inside `client_wiring.*`.
- `status` and `client_wiring.*` remain externally owned.
- `identity_tuple` follows the required and optional field rules from `identity-tuple-schema-spec.md`.
- `placement_posture` follows the `execution` and `host_to_world_bridge` rules from `identity-tuple-schema-spec.md`.

Emission rules:

- Gateway lifecycle responses emit `identity_tuple` when request resolution determines `client`, `router`, and `protocol`.
- Gateway lifecycle responses emit `placement_posture` when request resolution determines the effective placement.
- `status="unavailable"` omits `client_wiring` per the external status-schema owner and still emits `identity_tuple` or `placement_posture` when request resolution already determined them.
- `provider` and `auth_authority` omit by field absence only when their effective values are unresolved or not applicable.

## Human-readable status and diagnostics

Human-readable gateway status and diagnostics publish tuple and posture metadata in this order:

1. `originating client`
2. `routing authority`
3. `fulfillment provider`
4. `auth authority`
5. `protocol`
6. `deployment posture`
7. `bridge transport`

Rendering rules:

- Each published field renders as one line with the contract-owned label followed by the normalized id.
- Omitted tuple fields produce no placeholder lines.
- `bridge transport` renders only when `placement_posture.host_to_world_bridge=true`.
- Human-readable output does not rename `router` to `backend`.
- Human-readable output does not rename `host_only` to `host gateway`.

## Trace publication

Trace records with one stable routed subject emit the same two additive top-level objects:

```json
{
  "event_type": "command_complete",
  "session_id": "ses_123",
  "span_id": "spn_123",
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

Trace rules:

- `identity_tuple` and `placement_posture` are top-level additive objects on the same record as the existing ADR-0028 correlation keys.
- The tuple and posture objects augment the canonical trace vocabulary. They do not replace `session_id`, `span_id`, `cmd_id`, `world_id`, `backend_id`, or any other correlation key.
- Trace records emit `identity_tuple` when `client`, `router`, and `protocol` are known for that record.
- Trace records emit `placement_posture` when the effective placement is known for that record.
- `provider` and `auth_authority` omit by field absence until fixed for that record.
- Gateway-backed shell and runtime traces are in scope here.
- Pure agent and toolbox tuple publication remains outside this spec and stays owned by ADR-0044 and ADR-0045 follow-on work.

## Redaction and non-secret rules

- `client`, `router`, `provider`, and `auth_authority` publish normalized snake_case ids only.
- `protocol` publishes normalized dotted ids only.
- `auth_authority` identifies the authority class, not the secret material or credential value.
- Status, diagnostics, and trace records do not emit:
  - access tokens
  - API keys
  - session cookies
  - raw credential files
  - raw credential paths such as `~/.codex/auth.json`
  - upstream provider endpoints copied from secret-bearing sources
- `client_wiring.*` remains the only endpoint-discovery family in the status schema. Tuple publication does not duplicate those URLs into `identity_tuple`.

## Consumer impact

- Status readers continue to rely on `status` and `client_wiring.*` for gateway wiring discovery.
- Readers that need tuple or posture semantics consume `identity_tuple` and `placement_posture` directly.
- Trace readers continue to join through ADR-0028 correlation keys and treat tuple metadata as descriptive enrichment only.

## Acceptance criteria

- `status --json` publishes `identity_tuple` and `placement_posture` as top-level additive siblings outside `client_wiring.*`.
- Human-readable status and diagnostics render contract-owned labels in one stable order and omit missing optional fields without placeholders.
- Trace records carry the same object names as top-level additive metadata and keep ADR-0028 join keys unchanged.
- No status, diagnostics, or trace publication emits secret material or illustrative credential paths.
