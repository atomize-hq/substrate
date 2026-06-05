Status: synthesis
Scope: program-wide
Authority: non-canonical
Artifact-boundary impact: possible

# Structural Intelligence As Evidence

This note synthesizes:

- [tooling-patterns-gitnexus-deepwiki-serena.md](../../raw/2026-06-05/tooling-patterns-gitnexus-deepwiki-serena.md)
- [gitnexus-agent-feedback.md](../../raw/2026-06-05/gitnexus-agent-feedback.md)
- the current authority split across the code-intelligence program

## Durable takeaways

1. Graph-backed repository intelligence is a high-value evidence family.
2. The same structural signals are useful both before execution and during
   execution.
3. Structural evidence can improve planning and runtime discipline without
   becoming the correctness oracle.
4. Contract truth still lives in locked claims plus evaluated evidence, not in
   Lift scoring or impact output by itself.

## Implications for `lift`

`lift` should favor export surfaces that are:

- path-qualified
- freshness-scoped
- confidence-bearing
- explainable
- backed by evidence refs rather than opaque scalar summaries alone

Useful examples include impact, changed-scope, test topology, contract/policy
findings, and candidate conflict surfaces.

## Implications for `effort`

`effort` should consume structural intelligence as planning evidence:

- constrain lane partitioning with it
- reduce parallelism when confidence is low or signals are missing
- preserve the evidence links that justified a plan boundary

## Implications for `exec`

`exec` should be able to reuse the same class of signals for:

- scope drift detection
- stale-assumption checks
- runtime sanity checks around ownership and touched surfaces

That reuse should stay subordinate to runtime evidence and verdict evaluation.

## Relationship to contracts and gates

The contract layer should treat Lift-derived structural facts as evidence
inputs, not as final truth.

This preserves the rule:

> Lift may improve correctness-oriented workflows without owning correctness
> verdict semantics.

## Boundary caution

Do not answer final artifact naming or field taxonomy from this note alone.
Its purpose is to establish the layering rule, not to freeze schema shape.
