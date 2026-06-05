Status: exploratory
Scope: program-wide
Authority: non-canonical
Artifact-boundary impact: none

# Tooling Patterns: GitNexus, deepwiki-rs, Serena

This note captures the external-tool precedent pulled in before the next
`lift` / `effort` drafting pass.

## Source repos

- [abhigyanpatwari/GitNexus](https://github.com/abhigyanpatwari/GitNexus)
- [sopaco/deepwiki-rs](https://github.com/sopaco/deepwiki-rs)
- [oraios/serena](https://github.com/oraios/serena)

## Highest-signal patterns

1. GitNexus is strongest on deterministic structural extraction:
   graph-backed symbols, relations, process traces, impact surfaces, and
   freshness-aware index discipline.
2. deepwiki-rs is strongest on compilation into durable artifacts:
   multi-stage analysis, categorized context, verification passes, and
   publishable documentation surfaces.
3. Serena is strongest on workflow discipline:
   explicit project activation, memory references, symbol-aware operations, and
   harness-boundary clarity when overlapping tools already exist.

## Architecture implications worth carrying forward

- treat graph-backed repository intelligence as a compiled substrate rather than
  ad hoc planner inference
- keep external knowledge mounted as categorized side input, not merged into
  core repository truth
- preserve a verification step between analysis output and operator-facing
  artifacts
- keep memory and workflow context explicit and referenceable rather than
  relying on broad latent recall
- hold clean boundaries between structural intelligence, planning semantics,
  and runtime mutation ownership

## Boundary caution

These repos are useful precedent, not target templates.
Do not copy their product surfaces or exact tool taxonomies into the canonical
Substrate architecture without narrower repo-local justification.
