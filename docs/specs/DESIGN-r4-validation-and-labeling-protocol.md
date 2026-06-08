# Design: R4 Validation And Labeling Protocol

Status: draft design input. This document defines how `R4` session-archetype behavior should be
validated, labeled, and regression-tested before the packet is called review-clean. It is not a
benchmark paper and it is not a broad evaluation harness redesign.

## Why This Doc Exists

`R4` introduces a new semantic checkpoint field. The risk is not only code correctness; it is also
semantic drift:

1. the classifier may pass tests while encoding the wrong boundary,
2. fixtures may accidentally test commands in isolation instead of checkpoint-scoped session mode,
3. review may focus on green tests without proving the labels actually mean what the docs say.

This design freezes a narrow protocol for validating the mechanism against the intended `R4`
boundary.

## Relationship To Existing Decisions

This design must compose with:

1. [docs/specs/agent-drift-analyzer-session-archetype-r4-spec.md](./agent-drift-analyzer-session-archetype-r4-spec.md):
   public labels, confidence, evidence, and packet boundary.
2. [docs/specs/DESIGN-r4-intent-evidence-layer.md](./DESIGN-r4-intent-evidence-layer.md):
   lower-layer evidence should be testable and reviewable.
3. [docs/specs/DESIGN-r4-session-archetype-aggregation.md](./DESIGN-r4-session-archetype-aggregation.md):
   label transitions and ambiguity handling need explicit regression coverage.
4. [docs/internals/agent-drift-analyzer-current-checkpoint-logic.md](../internals/agent-drift-analyzer-current-checkpoint-logic.md):
   checkpoint construction already has natural test seams such as
   `build_session_checkpoint_from_analysis(...)`.

## Problem Statement

How might the repo validate checkpoint-scoped session labels so that:

1. the tests prove the intended four-label boundary rather than generic "some label exists",
2. ambiguous cases are reviewed honestly,
3. replay/live parity stays aligned with analyzer truth,
4. future plan/task revisions can point to a concrete validation matrix instead of vague coverage
   claims?

## Frozen Direction

The direction frozen by this design is:

1. validate at checkpoint scope, not per command and not per full session only,
2. combine narrow builder-level tests with a smaller number of end-to-end bundle fixtures,
3. require positive, negative, ambiguous, and transition cases,
4. verify both analyzer contract output and sentinel replay/live presentation parity,
5. treat manual review of a bounded corpus as part of the semantic validation story.

## Non-Goals

This design does not:

1. promise dataset-scale accuracy claims,
2. require new external evaluation infrastructure,
3. widen into `R5` progress validation or drift-score adjudication.

## Validation Units

The first implementation should validate across four unit types.

### 1. Builder-level classifier cases

Use focused analyzer helpers to prove lower-layer and aggregation behavior on tightly controlled
checkpoint inputs.

Good for:

1. direct label mapping,
2. confidence degradation,
3. evidence and counter-evidence population,
4. transition hysteresis.

### 2. End-to-end analyzer bundle fixtures

Use a smaller set of bundle-shaped fixtures to prove that realistic session prefixes produce the
same `v0.5` label and summary output after full analyzer execution.

Good for:

1. schema export,
2. summary rendering,
3. deterministic reruns,
4. realistic mixed-signal sessions.

### 3. Sentinel compatibility fixtures

Use replay/live fixtures to prove:

1. `v0.5` loads correctly,
2. legacy `v0.2` through `v0.4` still load,
3. replay/live surfaces present the same archetype view for matching checkpoints.

### 4. Manual bounded audit corpus

Keep a small screened set of real or realistic sessions for human review when packet semantics are
under debate.

This should stay bounded and reviewable rather than becoming a large evaluation program.

## Required Case Matrix

The first `R4` validation set should include at least:

1. one clear `planning` case,
2. one clear `autonomous_implementation` case,
3. one clear `troubleshooting` case,
4. one clear `verification_closeout` case,
5. at least one ambiguous mixed case that must stay `low` or `medium` confidence,
6. at least one legitimate mode-shift case across adjacent checkpoints,
7. at least one PR-response loop where targeted edits plus local verification stay
   `autonomous_implementation`,
8. at least one proof-oriented closeout case where successful verification beats residual
   implementation,
9. at least one failing verification loop where `troubleshooting` beats
   `verification_closeout`,
10. at least one delegated-parent plus opaque-child case that caps confidence conservatively,
11. at least one case proving replay/live parity for the same checkpoint output,
12. at least one case proving legacy schema fallback remains intact,
13. at least one case proving `v0.5` requiredness fails closed when `session_archetype` is
   missing,
14. at least one case where kickoff priors are contradicted by later behavior and therefore do not
   win the final label.

## Labeling Rules For Fixtures

Fixture labels should be assigned using the `R4` docs, not intuition alone.

Each labeled fixture should record:

1. expected public label,
2. expected confidence ceiling or floor when the case is intentionally ambiguous,
3. the decisive supporting evidence categories,
4. the main counter-evidence categories,
5. why nearby competing labels do not win.

This makes later review about documented boundaries rather than post-hoc taste.

## Recommended Test Placement

The cleanest initial placement is:

1. builder-level and lower-layer coverage in
   `crates/agent-drift-analyzer/tests/checkpoints.rs`,
2. summary and export coverage in
   `crates/agent-drift-analyzer/tests/export_bundle.rs` and
   `crates/agent-drift-analyzer/tests/end_to_end.rs`,
3. replay/live coverage in
   `crates/agent-drift-sentinel/tests/replay_input.rs`,
   `crates/agent-drift-sentinel/tests/live_checkpoint_compatibility.rs`,
   `crates/agent-drift-sentinel/tests/operator_surface.rs`, and
   `crates/agent-drift-sentinel/tests/live_end_to_end.rs`.
4. fixture labeling authority in
   `docs/specs/agent-drift-analyzer-session-archetype-r4-fixtures.md`.

## Review Protocol

Before calling `R4` review-clean, the packet should show:

1. the doc boundary still matches the implemented classifier,
2. the required case matrix exists,
3. at least one bounded manual audit pass was performed on realistic session prefixes,
4. analyzer and sentinel verification commands both passed,
5. any misclassification known during the bounded audit is either fixed or called out explicitly as
   an accepted limitation.

## Open Questions

1. Should the first bounded audit corpus be entirely synthetic fixtures, or should it include a
   small number of screened real-session prefixes?
2. Should expected lower-layer bucket strengths be asserted directly in tests, or only indirectly
   through final label and evidence output?
3. Do we want to assert debug-only lower-layer bucket strengths directly, or is the fixture
   manifest plus final public label enough for the first landing?
