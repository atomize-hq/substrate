Status: synthesis
Scope: program-wide
Authority: non-canonical
Artifact-boundary impact: none

# Program-Wide Overlay

This overlay groups research that spans `crates/lift` plus one or more peer
crates, or that crosses the contract/evidence/gate membrane itself.

Use this directory when forcing the concern into one crate or into the
`effort-exec` overlay would freeze the boundary too early.

## Governing authority docs

- [code-intelligence-program.md](../../../code-intelligence-program.md)
- [code-intelligence-contracts-and-gates.md](../../../code-intelligence-contracts-and-gates.md)
- [code-intelligence-workstream-orchestration.md](../../../code-intelligence-workstream-orchestration.md)
- [crates/lift/README.md](../../../crates/lift/README.md)

## Active notes

- [structural-intelligence-as-evidence.md](./structural-intelligence-as-evidence.md)
- [freshness-provenance-and-trust.md](./freshness-provenance-and-trust.md)
- [planning-and-execution-continuity.md](./planning-and-execution-continuity.md)

## Key raw captures

- [scibot-lift-effort-question-corpus.md](../../raw/2026-06-05/scibot-lift-effort-question-corpus.md)
- [archived-work-lift-v1-precedent.md](../../raw/2026-06-05/archived-work-lift-v1-precedent.md)
- [tooling-patterns-gitnexus-deepwiki-serena.md](../../raw/2026-06-05/tooling-patterns-gitnexus-deepwiki-serena.md)
- [gitnexus-agent-feedback.md](../../raw/2026-06-05/gitnexus-agent-feedback.md)

## Scope boundary

Good overlay topics:

- structural repository intelligence reused across planning and runtime
- freshness, provenance, identity, and trust semantics that span multiple crates
- contract/evidence/gate questions where Lift-produced signals matter but do not own verdict semantics
- external-tool precedent that changes how cross-crate layering should be reasoned about

Not good overlay topics:

- one crate's internal API design
- effort-only graph heuristics that never leave planner scope
- exec-only runtime implementation checklists
- freezing final artifact names just because a non-canonical note mentions them
