# Substrate Gateway Status Schema

This document is the durable canonical contract reference for the Substrate gateway status schema.
It mirrors the feature-local boundary contract and names the owned machine-readable status surface
without expanding into implementation detail.

## Contract

The operator-owned machine-readable gateway status surface is:
- `substrate world gateway status --json`

Owned contract points:
- the top-level JSON envelope for gateway status
- the `client_wiring.*` field family boundary
- the rule that the JSON surface is the authoritative wiring discovery surface
- the non-secret posture for status output
- the ownership of absence semantics for gateway status fields
- the boundary against additive metadata outside `client_wiring.*`

Not defined here:
- full field tables
- final examples
- transport mechanics
- implementation serialization details

## Boundary rules

- `status --json` is the canonical machine-readable authority for gateway wiring discovery.
- `client_wiring.*` is the only owned gateway-wiring field family in this contract.
- absence semantics are part of the contract and must remain stable.
- the surface must not emit secrets.
- additive metadata outside `client_wiring.*` is outside this contract.
