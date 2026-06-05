Status: synthesis
Scope: program-wide
Authority: non-canonical
Artifact-boundary impact: none

# Code-Intelligence Research

This directory keeps research available without letting it silently become architecture authority.

## Authority split

The only canonical docs for this program remain:

- [code-intelligence-program.md](../code-intelligence-program.md)
- [code-intelligence-contracts-and-gates.md](../code-intelligence-contracts-and-gates.md)
- [code-intelligence-workstream-orchestration.md](../code-intelligence-workstream-orchestration.md)

Everything under `docs/code-intelligence-research/` is supporting material.

## Directory structure

- `raw/`
  - source captures, imported chats, paper summaries, and evidence bundles
- `overlays/`
  - `program-wide/` for cross-crate synthesis that spans Lift plus one or more peer crates, or the contract/evidence membrane itself
  - `effort-exec/` for cross-cutting synthesis specific to planning/runtime concerns
- `crates/`
  - crate-local implications and open questions derived from the research
- `archive/`
  - older or superseded research notes that should remain available

## Active material

### Raw captures

- [2026-06-04](./raw/2026-06-04/)
- [2026-06-05](./raw/2026-06-05/)

### Overlays

- [program-wide](./overlays/program-wide/README.md)
- [effort-exec](./overlays/effort-exec/README.md)

### Modern paper addendum

- [hf-papers-modern-reference-addendum.md](./overlays/effort-exec/hf-papers-modern-reference-addendum.md)

### Crate-local notes

- [lift](./crates/lift/README.md)
- [effort](./crates/effort/README.md)
- [exec](./crates/exec/README.md)

## Promotion rules

1. `raw/` is evidence only.
2. `overlays/` may propose semantics but do not freeze ownership, artifact names, or rollout rules by themselves.
3. `crates/*/implications.md` may recommend design moves for one crate, but remain non-canonical.
4. Only the three top-level authority docs may freeze artifact boundaries, ownership, and rollout semantics.
5. A provisional artifact should be promoted only after repeated use across multiple notes and an implementation need that cannot be handled by embedded substructures.

## Header convention

Research notes should begin with:

```md
Status: exploratory | synthesis | candidate-direction | adopted | superseded
Scope: lift | effort | exec | effort+exec | program-wide
Authority: non-canonical
Artifact-boundary impact: none | possible | proposed
```

## Current guidance

For the present research set, the clean default is:

- keep imported captures in `raw/`
- use `overlays/program-wide/` when the concern spans Lift plus downstream consumers or the contract/evidence membrane
- keep `effort` + `exec` shared concerns in `overlays/effort-exec/`
- keep Lift-specific takeaways in `crates/lift/`
- keep crate-specific takeaways in `crates/effort/` and `crates/exec/`
- promote only stable conclusions into the authority docs
