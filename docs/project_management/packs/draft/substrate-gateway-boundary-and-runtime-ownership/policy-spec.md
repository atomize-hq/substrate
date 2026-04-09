# substrate-gateway-boundary-and-runtime-ownership - policy evaluation boundary

This document defines the boundary-level contract for `C-03`.
It is intentionally narrow: it names the existing ADR-0027 inputs that govern placement and secret
delivery, the fail-closed no-host-fallback rule, and the non-trust rule for gateway-local surfaces.
The complete routing matrix and any runtime transport details remain out of scope.

## Contract boundary

The owned surface is the gateway-integration evaluation flow over existing ADR-0027 inputs.

Owned here:
- the no-host-fallback rule when in-world execution is required
- the host-to-world secret delivery boundary
- the rule that gateway-local config, admin, and persistence surfaces are not trusted policy inputs
- the rule that policy evaluation remains Substrate-owned
- the distinction between invalid integration state, dependency unavailability, and policy denial

Governed inputs:
- `llm.gateway.mode`: determines whether gateway execution is required to stay in-world or may remain host-only.
- `llm.fail_closed.routing`: determines whether policy may permit any host fallback when in-world execution is required.
- `llm.secrets.env_allowed`: determines which host environment variable names Substrate may read for secret delivery.
- `agents.host_credentials.read.allowed_backends`: determines which backend identities may participate in host-side credential reads for the delivery path.

Concrete rules:
- When `llm.gateway.mode` requires in-world execution and the world boundary is unavailable, policy must fail closed rather than fall back to a host gateway.
- When `llm.fail_closed.routing` is true, the absence of a world boundary is a denial condition, not an automatic downgrade to host execution.
- Host secret sourcing is only a preparation step for Substrate-owned delivery and only for env-var names allowed by `llm.secrets.env_allowed` and backend identities allowed by `agents.host_credentials.read.allowed_backends`.
- Gateway-local config files, admin mutation endpoints, and token persistence remain implementation details of `substrate-gateway`; they are not trusted policy inputs and cannot authorize or override Substrate policy.
- Policy explanations remain part of the Substrate operator surface and must not be delegated to gateway-local state.
- Invalid integration state means the configuration or wiring is structurally wrong for the requested mode; dependency unavailability means the required world or gateway component is missing; policy denial means the requested action is rejected even though the deployment is otherwise structurally valid.

Not owned here:
- a full policy decision table
- exhaustive examples for every routing combination
- implementation-level adapter or transport behavior
- config schema definitions already owned by ADR-0027 and the config-policy surface
- `client_wiring.*` status-schema detail

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
- separate invalid integration state, dependency unavailability, and policy denial in operator-facing reasoning

## Deferred to later slices

The later policy slice owns:
- the complete decision flow
- specific routing and denial matrices
- concrete input-to-outcome examples
- any expanded validation notes needed for implementation
