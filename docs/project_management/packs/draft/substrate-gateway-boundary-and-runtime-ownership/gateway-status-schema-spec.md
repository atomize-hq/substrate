# substrate-gateway-boundary-and-runtime-ownership - gateway status schema boundary

This document defines the boundary-level contract for `C-02`.
It is intentionally narrow: it names the owned status envelope surface, the `client_wiring.*`
boundary, and the absence-semantics and non-secret posture that downstream slices must preserve.

## Contract boundary

The owned surface is the structured output contract for `substrate world gateway status --json`.

Owned here:
- the top-level envelope owned by Substrate for machine-readable gateway status
- the `client_wiring.*` field family boundary
- the rule that the JSON surface is the authoritative wiring discovery surface
- the rule that status output must not expose secret material
- the ownership of absence semantics for gateway status fields
- the boundary against additive metadata that belongs outside `client_wiring.*`

Not owned here:
- a full field-by-field schema table
- final JSON examples
- transport details or world-agent endpoint shapes
- implementation-specific serialization code

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

## Deferred to later slices

The later schema slice owns:
- the complete top-level field inventory
- exact field types and conditional presence rules
- canonical examples
- any expanded validation notes needed for implementation
