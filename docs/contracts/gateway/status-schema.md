# Substrate Gateway Status Schema

This document is the durable canonical contract reference for the Substrate gateway status schema.
It mirrors the feature-local boundary contract and names the owned machine-readable status surface
without expanding into implementation detail.

## Contract

The operator-owned machine-readable gateway status surface is:
- `substrate world gateway status --json`

Owned contract points:
- the minimum owned portion of the top-level JSON object for gateway status
- the required `status` availability field
- the `client_wiring` object and its `client_wiring.*` field family
- the rule that the JSON surface is the authoritative wiring discovery surface
- the non-secret posture for status output
- the ownership of absence semantics for gateway status fields
- the boundary against additive metadata outside `client_wiring.*`

Concrete shape:

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
- No additive adapter-visible field family is currently published beyond `status` and `client_wiring.*`.
- Any future additive adapter-visible status metadata requires an explicit update to this document before code,
  tests, or typed runtime models widen the JSON shape.
- Additional top-level fields may coexist only when they are owned by another contract surface.
- This contract does not define, version-control, or widen the meaning of those externally owned additive fields.

Not defined here:
- policy-evaluation decisions or trust-boundary logic
- transport mechanics or endpoint framing
- implementation serialization details
- additive identity-tuple or placement-posture metadata outside `client_wiring.*`

## Boundary rules

- `status --json` is the canonical machine-readable authority for gateway wiring discovery.
- `client_wiring.*` is the only owned gateway-wiring field family in this contract.
- absence semantics are part of the contract and must remain stable.
- the surface must not emit secrets.
- adapter-visible status metadata beyond the current envelope must not ship by implication; it needs an explicit
  schema-owner update here first.
- additive metadata outside `client_wiring.*` is outside this contract.
- identity-tuple and placement-posture metadata beyond `client_wiring.*` belong to ADR-0042, not this schema contract.
