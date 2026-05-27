# ADR-0042 — LLM and Agent Identity Tuple and Deployment Posture

## Status

- Status: Implemented
- Original date (UTC): 2026-04-02
- Curated into `docs/adr/implemented/`: 2026-05-26
- Owner(s): Spenser McConnell (Substrate)

## Curated From

- Historical planning ADR:
  - `docs/project_management/adrs/draft/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md`

This curated ADR is the stable decision record. The project-management ADR remains as the
planning-rich historical source.

## Decision

Operator-visible identity must be expressed as an explicit tuple rather than inferred from one
overloaded backend label.

The stable tuple fields are:

- `client`
- `router`
- `provider`
- `auth_authority`
- `protocol`

The stable placement posture is:

- `in_world`
- `host_only`
- `host_to_world_bridge` as a transport-only adjunct, not a second control plane

This keeps operator, status, and trace semantics aligned with the actual runtime split between
Substrate and `substrate-gateway`.

## Stable Owned Surface

This ADR remains the semantic owner for:

- `identity_tuple`
- `placement_posture`

Current stable contract references that depend on this ownership include:

- `docs/reference/policy/contract.md`
- `docs/reference/policy/tuple_constraints.md`
- `docs/contracts/gateway/status-schema.md`

## Current Implementation Anchors

The decision is materially implemented and verified through:

- `crates/shell/src/builtins/world_gateway.rs`
- `crates/world-service/src/service.rs`
- `crates/shell/tests/world_gateway.rs`
- `crates/shell/tests/agent_successor_contract_ahcsitc0.rs`

## Related ADRs

- `docs/adr/implemented/ADR-0027-llm-and-agent-config-policy-surface.md`
- `docs/adr/implemented/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md`
- `docs/adr/implemented/ADR-0041-substrate-gateway-backend-adapter-contract.md`
- `docs/adr/implemented/ADR-0043-adr-0027-identity-tuple-policy-surface.md`

## Historical Note

The original ADR includes planning-pack references and larger example context. Keep using this
curated ADR as the stable semantic owner of tuple and placement-posture meaning.
