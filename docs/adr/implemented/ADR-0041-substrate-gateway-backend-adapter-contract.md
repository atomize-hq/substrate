# ADR-0041 — Substrate Gateway Backend Adapter Contract

## Status

- Status: Implemented
- Original date (UTC): 2026-04-02
- Curated into `docs/adr/implemented/`: 2026-05-26
- Owner(s): Spenser McConnell (Substrate)

## Curated From

- Historical planning ADR:
  - `docs/project_management/adrs/draft/ADR-0041-substrate-gateway-backend-adapter-contract.md`

This curated ADR is the stable decision record. The project-management ADR remains as the
planning-rich historical source.

## Decision

Substrate selects and allowlists one stable backend id in `<kind>:<name>` form, then hands that
selection to a gateway-owned adapter boundary.

The stable decision is:

- backend ids are selector ids only
- one backend id maps to one adapter identity at the Substrate boundary
- selection uses the ADR-0027 config, policy, and inventory surfaces
- allowlisting happens before adapter dispatch
- gateway-local adapter internals remain implementation detail
- provider quirks, wrapper mechanics, and session details must not leak into the stable policy
  surface

## Stable Owned Surface

The stable references for this ADR are:

- `docs/contracts/gateway/backend-adapter-selection.md`
- `docs/contracts/gateway/backend-adapter-protocol.md`
- `docs/contracts/gateway/backend-adapter-schema.md`

## Current Implementation Anchors

The decision is materially implemented and verified through:

- `crates/shell/src/execution/config_model.rs`
- `crates/shell/src/execution/policy_model.rs`
- `crates/shell/src/builtins/world_gateway.rs`
- `crates/world-service/src/gateway_runtime.rs`

## Related ADRs

- `docs/adr/implemented/ADR-0027-llm-and-agent-config-policy-surface.md`
- `docs/adr/implemented/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md`
- `docs/adr/implemented/ADR-0046-gateway-backend-selection-runtime-integration.md`
- Historical predecessor kept for context:
  - `docs/project_management/adrs/draft/ADR-0024-cli-backend-provider-engine.md`

## Historical Note

The original ADR includes planning-pack and external evidence material that remains useful as
historical context, but stable backend-selection and adapter-contract truth now lives here and in
the gateway contract docs.
