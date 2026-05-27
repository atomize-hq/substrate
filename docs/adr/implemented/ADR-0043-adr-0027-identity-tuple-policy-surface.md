# ADR-0043 — ADR-0027 Identity Tuple Policy Surface

## Status

- Status: Implemented
- Original date (UTC): 2026-04-03
- Curated into `docs/adr/implemented/`: 2026-05-26
- Owner(s): Spenser McConnell (Substrate)

## Curated From

- Historical planning ADR:
  - `docs/project_management/adrs/draft/ADR-0043-adr-0027-identity-tuple-policy-surface.md`

This curated ADR is the stable decision record. The project-management ADR remains as the
planning-rich historical source.

## Decision

ADR-0027 remains the only config/policy root, and tuple-aware narrowing is added under
`llm.constraints.*` rather than through a second config system or overloaded backend ids.

The stable additive keys are:

- `llm.constraints.routers`
- `llm.constraints.providers`
- `llm.constraints.protocols`
- `llm.constraints.auth_authorities`

The stable behavior is:

- each axis narrows an already-selected backend path
- empty lists mean unconstrained on that axis
- tuple-axis mismatch is a policy denial
- tuple-policy inspection belongs on `substrate policy current show --explain`

## Stable Owned Surface

The stable references for this ADR are:

- `docs/reference/policy/tuple_constraints.md`
- `docs/reference/policy/contract.md`
- `docs/reference/policy/schema.md`

## Current Implementation Anchors

The decision is materially implemented and verified through:

- `crates/broker/src/policy.rs`
- `crates/broker/src/effective_policy.rs`
- `crates/shell/src/execution/policy_model.rs`
- `crates/broker/src/tests.rs`

## Related ADRs

- `docs/adr/implemented/ADR-0027-llm-and-agent-config-policy-surface.md`
- `docs/adr/implemented/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md`
- `docs/adr/implemented/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md`
- `docs/adr/implemented/ADR-0041-substrate-gateway-backend-adapter-contract.md`

## Historical Note

The original ADR contains planning-pack references and broader local execution context. Keep using
this curated ADR and the stable policy reference docs for current tuple-policy truth.
