Status: synthesis
Scope: effort+exec
Authority: non-canonical
Artifact-boundary impact: possible

# Ownership, Freeze, And Boundaries

This note captures the cross-crate semantics around ownership, forbidden surfaces, and frozen surfaces.

Primary inputs:

- [artifact-shapes-ownership-freeze.md](../../raw/2026-06-04/artifact-shapes-ownership-freeze.md)
- [architecture-synthesis.md](./architecture-synthesis.md)
- [code-intelligence-workstream-orchestration.md](../../../code-intelligence-workstream-orchestration.md)

## Durable takeaways

1. Ownership, forbidden surfaces, and frozen surfaces are coordination primitives, not optional operator garnish.
2. These semantics should exist early even if the final artifact boundary is still provisional.
3. The cleanest initial home is inside lane-, handoff-, and worktree-intent artifacts rather than in prematurely promoted standalone maps.

## Recommended early encoding

Useful embedded fields or substructures:

- `owned_surfaces`
- `read_only_surfaces`
- `forbidden_surfaces`
- `frozen_surfaces`
- `blocked_if_touched`
- `required_predecessors`

## Why this should stay provisional

The research strongly supports the semantics.
It does not strongly support one canonical top-level artifact split for those semantics.

That means:

- embed the semantics now where implementation needs them
- wait to promote `OwnershipMapV1` or `FreezeMapV1` until multiple orchestration patterns demand them

## Boundary caution

This note should not be read as freezing any particular artifact name.
It is an argument for preserving the meaning, not the top-level schema boundary.
