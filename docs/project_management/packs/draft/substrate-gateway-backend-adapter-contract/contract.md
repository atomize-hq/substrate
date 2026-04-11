# Substrate Gateway Backend Adapter Contract

This document is the feature-local mirror of the S00 contract baseline for the adapter-selection
boundary in ADR-0041. It keeps the owned baseline close to the pack without expanding into
protocol, payload, or parity detail.

Canonical references:

- `docs/contracts/substrate-gateway-backend-adapter-selection.md`
- `docs/contracts/substrate-gateway-status-schema.md`
- `docs/contracts/substrate-gateway-operator-contract.md`

## Contract baseline

- Stable backend ids remain the only Substrate-facing backend identity.
- One stable backend id maps to one adapter identity at the Substrate boundary.
- Backend ids stay in `<kind>:<name>` form and remain selectors only; they do not encode
  provider, router, auth-authority, protocol, planner, or wrapper identity.
- Selection stays fail-closed and uses the existing ADR-0027 config, policy, and inventory surfaces.
- Invalid selection, dependency unavailability, and policy denial remain separate outcomes.
- Gateway-local config, admin mutation surfaces, persistence, and session state do not become
  Substrate authorization inputs.
- No additive adapter-visible `status --json` field family is published by this seam in v1.
  The currently published machine-readable surface remains the existing `status` plus
  `client_wiring.*` schema owned by `docs/contracts/substrate-gateway-status-schema.md`.
- Any future additive adapter-visible status metadata requires an explicit update to
  `docs/contracts/substrate-gateway-status-schema.md` before code or tests widen
  `GatewayLifecycleResponseV1`.

## Verification plan

Evidence already present:

- backend-id grammar validation exists in `crates/broker/src/policy.rs`
- effective-policy validation reuses the same backend-id rules in `crates/broker/src/effective_policy.rs`
- the current runtime status shape is limited to `status` plus `client_wiring` in
  `crates/agent-api-types/src/lib.rs`
- `crates/shell/tests/world_gateway.rs` currently proves the available and unavailable JSON shapes

Execution checklist for landing this seam:

- Update docs:
  - keep this file aligned with `docs/contracts/substrate-gateway-backend-adapter-selection.md`
  - keep the status-boundary statement aligned with `docs/contracts/substrate-gateway-status-schema.md`
- Add or update tests:
  - extend `crates/broker/src/policy/tests.rs` when backend-id grammar or invalid-selection rules change
  - extend `crates/shell/tests/world_gateway.rs` if gateway status output is widened after the schema owner is updated
  - extend `crates/world-agent/tests/gateway_runtime_parity.rs` if runtime unavailable/available status posture changes
- Pass/fail conditions:
  - pass when one backend-id contract, one failure taxonomy, and one status-publication boundary are reflected consistently in docs and consuming tests
  - fail when any new status field family appears without a status-schema owner update, or when selection semantics diverge from the backend-id grammar and allowlist contract
