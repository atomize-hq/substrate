Status: synthesis
Scope: effort+exec
Authority: non-canonical
Artifact-boundary impact: none

# Effort And Exec Overlay

This overlay groups research that spans `crates/effort` and `crates/exec`.

Use this directory when the concern is cross-cutting enough that forcing it into one crate too early would accidentally freeze a boundary.

## Governing authority docs

- [code-intelligence-program.md](../../../code-intelligence-program.md)
- [code-intelligence-workstream-orchestration.md](../../../code-intelligence-workstream-orchestration.md)
- [code-intelligence-contracts-and-gates.md](../../../code-intelligence-contracts-and-gates.md)

## Active notes

- [architecture-synthesis.md](./architecture-synthesis.md)
- [hf-papers-modern-reference-addendum.md](./hf-papers-modern-reference-addendum.md)
- [parallel-planning-and-conflict.md](./parallel-planning-and-conflict.md)
- [ownership-freeze-and-boundaries.md](./ownership-freeze-and-boundaries.md)
- [runtime-validation-and-fail-closed.md](./runtime-validation-and-fail-closed.md)

## Scope boundary

Good overlay topics:

- safe parallelism rules
- conflict and dependency reasoning that spans planning and runtime
- ownership, forbidden surfaces, and freeze semantics
- runtime validation and merged-tree truth

Not good overlay topics:

- broader Lift-plus-peer layering or contract-membrane semantics that belong in `../program-wide/`
- one crate's internal API design
- one crate's implementation checklist
- frozen artifact names that still need validation
