# Design: R4 Session-Archetype Aggregation

Status: draft design input. This document defines how the public `R4` session-archetype decision
should be derived from the lower intent-evidence layer plus existing checkpoint-local analyzer
state. It is not a drift-scoring redesign and it is not `R5` progress semantics.

## Why This Doc Exists

The `R4` spec already freezes the public output shape:

1. one checkpoint-local `session_archetype`,
2. one of four labels,
3. explicit confidence and evidence.

What still needs a frozen mechanism is the aggregation rule: how a full session prefix at one
checkpoint becomes one session-type judgment without collapsing back to turn-local labeling or
quietly importing `R5` progress semantics.

## Relationship To Existing Decisions

This design must compose with:

1. [docs/specs/agent-drift-analyzer-session-archetype-r4-spec.md](./agent-drift-analyzer-session-archetype-r4-spec.md):
   public labels and boundary.
2. [docs/specs/DESIGN-r4-intent-evidence-layer.md](./DESIGN-r4-intent-evidence-layer.md):
   lower evidence buckets feed this aggregation step.
3. [docs/specs/agent-drift-analyzer-turn-context-r3-spec.md](./agent-drift-analyzer-turn-context-r3-spec.md):
   turn context remains input evidence, not the final session label.
4. [docs/internals/agent-drift-analyzer-current-checkpoint-logic.md](../internals/agent-drift-analyzer-current-checkpoint-logic.md):
   checkpoint windows are cumulative prefixes already, which fits session-prefix aggregation.

## Problem Statement

How might the analyzer map a mixed intent-evidence profile plus existing checkpoint-local context
into one checkpoint-scoped session label so that:

1. the label reflects the current session mode, not merely the last turn or last command,
2. genuine mode shifts can occur across checkpoints,
3. ambiguous cases lower confidence rather than inventing more labels,
4. the result stays descriptive rather than acting as a success or failure judgment?

## Frozen Direction

The direction frozen by this design is:

1. aggregate over the session prefix through the current checkpoint,
2. use a recency-aware view where recent checkpoints may change the label but earlier established
   evidence still matters,
3. emit only the four public `R4` labels:
   - `troubleshooting`
   - `planning`
   - `autonomous_implementation`
   - `verification_closeout`
4. keep the output descriptive and mode-oriented,
5. separate confidence from severity or drift judgment,
6. attach supporting and counter-evidence to every public decision.

## Non-Goals

This design does not:

1. define whether the session is succeeding,
2. define frontier advance or convergence semantics,
3. redefine the existing drift classes,
4. create a fifth fallback label such as `mixed` or `unknown`.

## Aggregation Model

The aggregator should consume:

1. the current checkpoint's `IntentEvidenceProfile`,
2. recent checkpoint-local evidence trends,
3. current task-frame and working-set concentration,
4. turn-context execution mode and activity mix,
5. diagnostics such as verification density and task-frame transitions,
6. optional kickoff priors only if they survive the guardrails in
   [docs/specs/DESIGN-r4-kickoff-prior-signals-and-guardrails.md](./DESIGN-r4-kickoff-prior-signals-and-guardrails.md).

The output should remain conceptually equivalent to:

```text
SessionArchetype
- label: SessionArchetypeLabel
- confidence: Confidence
- supporting_evidence: Vec<EvidenceRef>
- counter_evidence: Vec<EvidenceRef>
```

## Label Mapping Rules

### 1. `planning`

Prefer `planning` when the prefix is dominated by:

1. `exploration_like` and `orchestration_like` evidence,
2. directive synthesis or scope-shaping behavior,
3. lower write/test density,
4. broader working-set exploration while the objective sharpens.

Counter-evidence should rise when:

1. stable source-edit plus verification cadence is already established,
2. proof-oriented validation dominates the recent checkpoints.

### 2. `autonomous_implementation`

Prefer `autonomous_implementation` when the prefix is dominated by:

1. sustained `implementation_like` evidence,
2. concentrated source edits on a stable objective,
3. local verification that supports the active implementation scope,
4. turn-context that is `autonomous` or implementation-heavy.

Counter-evidence should rise when:

1. the recent prefix is mostly diagnostic churn,
2. source editing has largely stopped and proof gathering dominates,
3. the session remains broad and directive-heavy instead of settled on one implementation scope.

### 3. `verification_closeout`

Prefer `verification_closeout` when the recent prefix is dominated by:

1. strong `verification_like` evidence,
2. narrowing scope,
3. review, validation, proof, or handoff-oriented activity,
4. little new source editing relative to proof gathering.

Counter-evidence should rise when:

1. the session is still actively exploring root causes,
2. new source edits remain the main behavior,
3. verification is failing in a way that forces renewed diagnosis rather than closeout.

### 4. `troubleshooting`

Prefer `troubleshooting` when the prefix shows:

1. strong `verification_like` evidence,
2. meaningful `exploration_like` evidence,
3. repeated diagnostic or failing verification behavior on a constrained scope,
4. insufficient stable implementation cadence to call the current mode
   `autonomous_implementation`,
5. insufficient proof-oriented narrowing to call the current mode `verification_closeout`.

Counter-evidence should rise when:

1. the recent prefix has shifted into stable write/test implementation loops,
2. the recent prefix is mostly closeout-style proof gathering,
3. the work is still mostly broad planning and scope design.

## Transition Rules

The label may change across checkpoints, but not on a single weak contradiction.

The first implementation should apply simple hysteresis:

1. keep the current dominant label when new contradictory evidence is weak,
2. allow a label shift when recent evidence is sustained and materially stronger than the prior
   mode,
3. degrade confidence during transition periods instead of pretending the mode is fully stable.

Examples of legitimate shifts:

1. `planning` -> `autonomous_implementation`
   - when concentrated source-edit plus local verification cadence becomes the dominant recent mode.
2. `autonomous_implementation` -> `troubleshooting`
   - when repeated failing verification plus renewed diagnosis dominates the recent prefix.
3. `autonomous_implementation` -> `verification_closeout`
   - when new editing tapers and proof-oriented verification becomes dominant.
4. `troubleshooting` -> `autonomous_implementation`
   - when the session returns to stable implementation loops on a concentrated scope.

## Confidence Rules

Confidence should be a function of evidence coherence, not merely evidence volume.

High confidence requires:

1. one label clearly dominating the recent prefix,
2. limited counter-evidence from competing labels,
3. alignment between lower intent evidence and higher checkpoint-local context.

Low confidence is expected when:

1. the session is transitioning,
2. strong evidence exists for multiple labels,
3. only kickoff priors exist and behavioral evidence is sparse.

## Recommended Default For PR-Response Loops

The initial draft recommendation is:

1. default PR-comment or review-response loops to `autonomous_implementation` when the session is
   still making targeted code changes plus local verification,
2. shift to `verification_closeout` only when proof-oriented validation and narrowing dominate over
   new implementation work.

This keeps `verification_closeout` narrower and avoids turning any review-shaped work into
closeout by default.

## Open Questions

1. Should the first implementation expose any explicit transition reason in debug/test output, or
   is label plus evidence enough?
2. Is the recommended default for PR-response loops the right one, or should proof-oriented review
   response be considered `verification_closeout` earlier?
3. How sticky should the label be across adjacent checkpoints before we consider it flapping?

