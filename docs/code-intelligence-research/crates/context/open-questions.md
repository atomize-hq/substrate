Status: exploratory
Scope: context
Authority: non-canonical
Artifact-boundary impact: possible

# Context Open Questions

These questions remain open on purpose and should not be treated as settled
boundaries.

## Open questions

1. What is the smallest handbook-derived engine surface that `substrate-context`
   can consume first without importing handbook product-shell assumptions?
2. Should the first stable boundary between handbook-engine and `context` be a
   typed Rust API, artifact refs plus loaders, or both?
3. Which reusable pipeline pieces belong beside handbook-engine versus inside
   `substrate-context` once storage roots become configurable?
4. How much route-state or pipeline-state detail should enter `ContextPacketV1`
   versus remain behind provider-specific read APIs?
5. Should a handbook-derived engine land as `crates/handbook-engine`,
   `crates/handbook-provider`, or another workspace crate shape once the split
   is real?

## Keep provisional

Do not answer these by directory structure alone.
They should be resolved only when the handbook split work and the first
`context` implementation force a narrower reviewed boundary.
