Status: synthesis
Scope: program-wide
Authority: non-canonical
Artifact-boundary impact: possible

# Freshness, Provenance, And Trust

This note captures the cross-crate implications of stale indexes, duplicate
repo identity, dirty-tree ambiguity, and evidence provenance.

## Durable takeaways

1. Freshness is artifact quality, not just tool UX.
2. Repo identity must be canonical enough to disambiguate same-named indexes.
3. Changed-scope output is only interpretable relative to an explicit baseline.
4. Provenance and trust metadata should survive across planning and runtime,
   not disappear after retrieval.

## Cross-crate implications

### `context`

`context` remains the natural home for broader provenance, trust, and freshness
filtering policies when assembling packets.

### `lift`

`lift` should report freshness and identity facts for the structural signals it
exports, such as:

- repository instance identity
- basis commit or comparable revision anchor
- index freshness or stale-state markers
- changed-scope baseline
- evidence provenance for major findings

### `effort`

`effort` should treat stale, ambiguous, or low-trust structural signals as a
reason to reduce parallelism or require additional evidence.

### `exec`

`exec` should preserve enough of the same freshness and baseline context to
detect drift between planning-time assumptions and runtime reality.

## Practical examples from the GitNexus feedback

- duplicate repo names made path qualification necessary
- stale index detection changed whether impact output was trustworthy
- dirty-tree noise weakened changed-scope summaries
- formatter churn blurred semantic change unless interpreted carefully

## Boundary caution

This note argues for carrying freshness and provenance across crate boundaries.
It does not settle whether every fact lives in one artifact, multiple artifact
families, or shared metadata envelopes.
