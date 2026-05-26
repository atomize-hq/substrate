# Substrate Gateway Backend Adapter Policy Spec

This document is the feature-local mirror of the S00 selection-policy baseline for ADR-0041.
It records only the minimum ordered-input, fail-closed, and trusted-input wording needed to
keep the C-01 baseline internally consistent.

Canonical references:

- `docs/contracts/gateway/backend-adapter-selection.md`
- `docs/contracts/gateway/policy-evaluation.md`
- `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/contract.md`
- `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/SCHEMA.md`

## Ordered evaluation inputs

Selection over existing Substrate surfaces follows this order:

1. Resolve the requested or default backend id from the existing config surface.
2. Validate the backend id against the shared `<kind>:<name>` grammar.
3. Resolve the matching inventory entry through the existing one-file-per-backend model and
   filename/id match rule.
4. Apply deny-by-default allowlist policy before adapter dispatch.
5. If policy and placement requirements allow execution, hand the selected backend id to the
   gateway adapter/runtime boundary.

## Trusted-input boundary

Trusted selection inputs:

- `llm.routing.default_backend`
- `llm.allowed_backends`
- the existing backend inventory files governed by ADR-0027
- the existing policy inputs owned by the gateway policy-evaluation contract

Explicit non-inputs:

- gateway-local config files
- gateway-local admin mutation surfaces
- gateway-local token persistence
- adapter session state or session handles
- provider- or wrapper-private routing hints
- any gateway-local authorization source

## Failure classification

- Invalid selection:
  - malformed backend id
  - unknown backend id
  - inventory mismatch or invalid selection input
- Dependency unavailable:
  - selected backend id is valid and allowed, but the required adapter/runtime component is missing or unsupported
- Policy denial:
  - selection is denied by allowlist, fail-closed routing, or another safety/policy rule

These outcomes must remain distinct in docs, code, and tests.

## Verification plan

Evidence already present:

- `crates/broker/src/policy.rs` and `crates/broker/src/effective_policy.rs` enforce backend-id validation
- `docs/contracts/gateway/policy-evaluation.md` already separates invalid integration state,
  dependency unavailability, and policy denial
- `crates/shell/tests/world_gateway.rs` and `crates/world-service/tests/gateway_runtime_parity.rs`
  already prove the current unavailable and available status posture

Execution checklist for landing this seam:

- keep the ordered evaluation flow aligned with the implemented ADR-0027 contract surfaces
- add targeted tests where the boundary changes:
  - `crates/broker/src/policy/tests.rs` for malformed or disallowed backend ids
  - `crates/shell/tests/world_gateway.rs` for CLI/operator-visible selection classification
  - `crates/world-service/tests/gateway_runtime_parity.rs` if the typed runtime status posture changes
- pass when invalid selection, dependency unavailability, and policy denial each terminate in one explicit contract bucket
- fail when any gateway-local surface is required to authorize backend selection or when allowlist gating happens after dispatch
