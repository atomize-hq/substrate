Status: synthesis
Scope: lift
Authority: non-canonical
Artifact-boundary impact: possible

# Lift Implications

This note records the strongest Lift-specific implications from the new
GitNexus, deepwiki-rs, Serena, and implementation-feedback research.

## Immediate implications

1. Lift export surfaces should be freshness-scoped and path-qualified rather
   than relying on display names or timeless summaries alone.
2. Lift should emit explicit graph-backed and schema-backed repository
   intelligence artifacts rather than prose-heavy handoffs when downstream
   planning or runtime consumers need structured input.
3. Lift should emit explainable structural intelligence:
   changed scope, impact, topology, reuse, policy/contract findings, and
   evidence refs for major claims.
4. Dependency structure and conflict-risk structure should remain distinct
   exported signal families rather than collapsing into one aggregate risk
   summary.
5. Confidence and missing-input markers should travel with Lift-produced
   signals, not stay implicit in tool logs.
6. Lift should distinguish production-facing versus test-facing touched
   surfaces where the evidence supports that split.
7. History should be encoded as deterministic bias:
   causal features, risk features, retrieval features, or no-good style
   constraints, not as stochastic planner behavior.
8. AI-authored judgments should be fallback annotations:
   semantic clustering, behavior-delta summarization, or ambiguity labeling
   when deterministic extraction cannot fully resolve the shape.
9. Lift should help downstream correctness-oriented workflows without claiming
   correctness verdict ownership for itself.

## MVP design bias

Preferred early Lift posture:

1. report repository instance identity and basis revision clearly
2. report freshness or stale-state markers for graph-backed findings
3. emit typed entities, typed dependency structure, typed conflict-risk
   structure or conflict evidence, and source mappings where available
4. attach evidence refs and rationale to major impact or changed-scope claims
5. preserve confidence and missing-signal markers
6. preserve deterministic history-derived signals as explicit features or
   hints rather than hidden model behavior
7. keep downstream consumption artifact-based rather than deep-import based
8. treat aggregate score output as advisory when richer structural exports are
   available

## Useful compression

The cleanest Lift-side compression of the current research is:

- Lift owns the code-intelligence fact layer.
- Lift should export typed graph and evidence surfaces, not only summaries.
- Score remains useful, but should not stand in for the full planning
  substrate when richer structural signals are available.
- AI can explain, cluster, and summarize, but deterministic extraction should
  own the primary repository-intelligence substrate whenever possible.

## Promotion caution

This note supports stronger Lift export content and clearer cross-crate reuse.
It does not by itself freeze the final `LiftEffortSignalV1` field taxonomy or
settle which downstream artifact families should reference Lift evidence
directly.
