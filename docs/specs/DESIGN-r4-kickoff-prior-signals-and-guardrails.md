# Design: R4 Kickoff Prior Signals And Guardrails

Status: draft design input. This document defines the bounded kickoff-prior seam for `R4`
session-archetype classification. It exists to prevent prompt-shape information from either being
ignored completely or from quietly overpowering observed behavior.

## Why This Doc Exists

The current `R4` authority stack already says kickoff prompt shape, skill invocations, and
orchestration markers may be useful, but only as bounded priors.

That statement is directionally correct but still too loose for implementation.

Without a dedicated design:

1. priors may leak into prompt-keyword classification,
2. different packets may treat kickoff cues inconsistently,
3. later reviews will not be able to tell whether the classifier is still behavior-first.

## Relationship To Existing Decisions

This design must compose with:

1. [docs/specs/agent-drift-analyzer-session-archetype-r4-spec.md](./agent-drift-analyzer-session-archetype-r4-spec.md):
   kickoff cues are bounded priors only.
2. [docs/specs/DESIGN-r4-intent-evidence-layer.md](./DESIGN-r4-intent-evidence-layer.md):
   the lower layer remains behavior-first.
3. [docs/specs/DESIGN-r4-session-archetype-aggregation.md](./DESIGN-r4-session-archetype-aggregation.md):
   priors may contribute only if they survive explicit caps and contradiction rules.

## Problem Statement

How might the analyzer use kickoff-time signals when behavioral evidence is sparse without letting
those signals become a hidden source of final truth?

## Frozen Direction

The direction frozen by this design is:

1. kickoff priors are optional inputs to archetype aggregation,
2. they should be low-confidence and explicitly capped,
3. they should decay quickly once behavior accumulates,
4. contradiction from observed behavior should neutralize them,
5. the classifier must remain fully functional without priors.

## Non-Goals

This design does not:

1. justify prompt-only classification,
2. create a large free-text keyword ontology,
3. require kickoff priors in the first code landing,
4. let priors modify drift severity or scheduler behavior.

## Allowed Prior Sources

The first design pass should allow only structured or narrowly interpretable kickoff sources:

1. explicit skill invocation markers already present in the transcript,
2. orchestration-prompt markers or packet scaffolds,
3. normalized objective wording when it clearly expresses session mode,
4. explicit user framing such as "plan", "debug", "investigate", "implement", or "verify" when
   that framing is direct and not merely incidental prose.

The design should not depend on latent semantic interpretation or broad keyword bags.

## Prior Output Shape

The cleanest conceptual output is one small bias vector, for example:

```text
KickoffPrior
- planning_bias: none | weak
- troubleshooting_bias: none | weak
- autonomous_implementation_bias: none | weak
- verification_closeout_bias: none | weak
- rationale: Vec<EvidenceRef>
```

Guardrail:

1. no prior should start above `weak`,
2. no prior alone should justify `medium` or `high` final confidence.

## Guardrails

### Rule 1: Behavior always outranks priors

Once the session shows concentrated edits, verification cadence, working-set behavior, or repeated
diagnostic patterns, the classifier must let observed behavior dominate.

### Rule 2: Priors decay fast

The prior should matter most at the earliest checkpoints and decay after a small number of
behavioral checkpoints or once sufficient lower-layer evidence exists.

### Rule 3: Contradiction cancels

If kickoff cues suggest `planning` but the recent prefix shows stable implementation loops, the
planning prior should stop contributing materially.

### Rule 4: Priors must remain auditable

If a prior contributes at all, its rationale should be representable in evidence form so tests and
future summaries can explain that influence.

### Rule 5: Priors must not backdoor label expansion

The prior seam may bias only the existing four labels. It must not create quasi-labels such as
`brainstorming`, `review`, `docs_only`, or `meta_orchestration`.

## Recommended First Landing

The initial first-landing recommendation is:

1. freeze this design now,
2. keep the first active `R4` code landing behavior-first,
3. defer prior contribution entirely or keep it disabled by default behind a narrow internal gate
   until behavior-only regressions are stable.

This keeps the first classifier easier to reason about while preserving a clear place for priors
later if they prove useful.

## Cases That Should Not Over-Trigger Priors

1. long packet prompts that mention planning, implementation, testing, and review all at once,
2. resume or handoff prompts that summarize prior work without describing the current dominant
   behavior,
3. generic instructions like "fix this" or "take a look" without other structured cues,
4. skill invocations whose contract is broad enough to cover multiple later behaviors.

## Open Questions

1. Should direct skill invocation be treated as stronger than ordinary objective wording, or should
   all kickoff signals stay equally capped at `weak`?
2. If priors are deferred from the first code landing, should the spec/plan/tasks say that
   explicitly now?
3. Should resume or handoff prompts contribute any prior signal at all, or should they be treated
   as too noisy for the first version?
