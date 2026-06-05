Status: exploratory
Scope: program-wide
Authority: non-canonical
Artifact-boundary impact: none

# GitNexus Agent Feedback

This note captures the implementation-agent findings about how GitNexus helped
or failed during real packet execution work in Substrate checkouts.

## Durable takeaways

1. GitNexus was most useful as a scope-control and safety-rail tool:
   index freshness checks, impact analysis, and pre-commit change detection.
2. Stale-index state is not cosmetic; it changes whether graph-derived output
   should be trusted.
3. Repo identity must be path-qualified or otherwise disambiguated when
   multiple indexed repos share the same display name.
4. Symbol-level impact is directionally useful, but symbol targeting can be
   brittle and test fanout can distort apparent production risk.
5. `detect_changes` is valuable, but its signal becomes noisier when formatter
   churn, adjacent tests, or unrelated dirty-tree files are mixed into the
   same surface.

## Product-quality gaps that matter architecturally

- explain semantic change separately from formatter-only churn
- distinguish production blast radius from test-only fanout
- record the baseline used for changed-scope analysis:
  staged, commit-relative, or dirty-tree relative
- attach clearer freshness metadata:
  indexed commit, current commit, interrupted-refresh state
- connect changed symbols to likely verification surfaces where possible

## Architecture implications worth carrying forward

- `lift` should export freshness-scoped and explainable structural signals
- `effort` should treat stale or missing graph signals conservatively
- `exec` should be able to reuse changed-scope and risk signals for runtime
  drift and sanity checks without treating them as correctness verdicts

## Boundary caution

These findings describe workflow value and pain points around GitNexus.
They justify cross-crate research notes, but they do not by themselves freeze
field names or top-level artifact families.
