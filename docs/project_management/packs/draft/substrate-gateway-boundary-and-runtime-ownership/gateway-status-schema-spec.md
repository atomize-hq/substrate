# substrate-gateway-boundary-and-runtime-ownership - gateway status schema boundary

This document defines the boundary-level contract for `C-02`.
It is intentionally narrow: it names the owned status envelope surface, the `client_wiring.*`
boundary, and the absence-semantics and non-secret posture that downstream slices must preserve.

## Contract boundary

The owned surface is the structured output contract for `substrate world gateway status --json`.

Owned here:
- the minimum owned portion of the top-level JSON object for machine-readable gateway status
- the required `status` availability field
- the `client_wiring` object and its `client_wiring.*` field family
- the rule that `status --json` is the authoritative wiring discovery surface
- the rule that status output must not expose secret material
- the ownership of absence semantics for gateway status fields
- the boundary against additive metadata that belongs outside `client_wiring.*`

Not owned here:
- policy-evaluation decisions or trust-boundary logic
- a runtime transport protocol or endpoint framing contract
- implementation-specific serialization code
- additive identity-tuple or placement-posture metadata outside `client_wiring.*`

## Concrete shape

The top-level JSON object is:

```text
owned envelope portion:
{
  status: "available" | "unavailable"
  client_wiring?: {
    openai_base_url: string
    anthropic_base_url: string
  }
}
```

Rules:
- `status` is required and reports whether gateway wiring can be published for this entrypoint.
- `client_wiring` is required when wiring can be published and omitted when the gateway/world component is unavailable.
- `client_wiring.openai_base_url` and `client_wiring.anthropic_base_url` are the only owned wiring-discovery leaves in this contract.
- The two wiring leaves are published together or omitted together; partial publication is not allowed.
- When present, the wiring leaves are non-empty strings pointing to Substrate-managed gateway endpoints.
- When omitted, the contract uses absence rather than placeholder values, nulls, or empty strings.
- Additional top-level fields may coexist only when they are owned by another contract surface.
- This contract does not define, version-control, or widen the meaning of those externally owned additive fields.

## Publication surface

The schema boundary is published through:
- this feature-local spec
- `docs/contracts/substrate-gateway-status-schema.md`
- later implementation and verification slices

## Required contract statements

This boundary must make the following statements explicit:
- `status --json` is the machine-readable authority for gateway wiring discovery
- `client_wiring.*` is the only gateway-wiring field family owned by this contract surface
- absence rules are contract behavior, not incidental implementation detail
- the JSON output must remain non-secret
- additive metadata outside `client_wiring.*` is not owned here
- identity-tuple and placement-posture metadata beyond `client_wiring.*` remain outside this contract and belong to ADR-0042

## Deferred to later slices

The later schema slice owns:
- any expansion beyond the concrete top-level envelope and `client_wiring` leaves defined here
- any per-field validation notes or examples beyond the minimal contract sketch
- any expanded validation notes needed for implementation
