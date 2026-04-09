# substrate-gateway-boundary-and-runtime-ownership - policy evaluation boundary

This document defines the boundary-level contract for `C-03`.
It is intentionally narrow: it names the fail-closed placement rule, the host-to-world secret
boundary, and the non-trust rule for gateway-local surfaces, while leaving the full decision matrix
to later slices.

## Contract boundary

The owned surface is the gateway-integration evaluation flow over existing ADR-0027 inputs.

Owned here:
- the no-host-fallback rule when in-world execution is required
- the host-to-world secret delivery boundary
- the rule that gateway-local config, admin, and persistence surfaces are not trusted policy inputs
- the rule that policy evaluation remains Substrate-owned
- the distinction between invalid integration state, dependency unavailability, and policy denial

Not owned here:
- a full policy decision table
- exhaustive examples for every routing combination
- implementation-level adapter or transport behavior
- config schema definitions already owned by ADR-0027 and the config-policy surface

## Publication surface

The policy boundary is published through:
- this feature-local spec
- `docs/contracts/substrate-gateway-policy-evaluation.md`
- later implementation and verification slices

## Required contract statements

This boundary must make the following statements explicit:
- fail closed when policy requires in-world execution and no world boundary is available
- do not treat gateway-local config, admin, or persistence as Substrate trust inputs
- host secret sourcing and delivery remain policy-gated and Substrate-owned
- policy explanations remain part of the Substrate operator surface

## Deferred to later slices

The later policy slice owns:
- the complete decision flow
- specific routing and denial matrices
- concrete input-to-outcome examples
- any expanded validation notes needed for implementation
