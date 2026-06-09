# Design: R5 Delegation Progress Guardrails

Status: canonical design authority locked in Packet R5-0 on 2026-06-09.

## Why This Doc Exists

R3.75 landed a bounded analyzer-local delegation boundary. R4 consumes that boundary to cap
archetype confidence when child work is opaque. R5 must do the same for progress. The risk is
stronger in R5: a parent checkpoint that only shows orchestration or waiting can look stalled, while
an opaque child may be doing successful implementation work that is not visible in the parent trace.

This doc freezes the R5 delegation rule: describe only visible progress, cap confidence under
opacity, and defer supported parent/child semantics to R7.

## Current Repo Reality

The current analyzer infers a private `DelegationContext` with:

```rust
pub(crate) struct DelegationContext {
    pub topology: Option<DelegationTopology>,
    pub child_work_visibility: Option<ChildWorkVisibility>,
    pub confidence: Option<Confidence>,
    pub markers: Vec<String>,
    pub supporting_evidence: Vec<EvidenceRef>,
    pub counter_evidence: Vec<EvidenceRef>,
}
```

The relevant private enums are:

```rust
pub(crate) enum DelegationTopology {
    SingleAgent,
    DelegatingParent,
    DelegatedChild,
    MixedOrAmbiguous,
}

pub(crate) enum ChildWorkVisibility {
    None,
    Partial,
    Opaque,
}
```

R3.75 keeps this analyzer-local. R5 should not widen the public checkpoint with delegation fields.
It should use the context internally to decide `SessionProgress` status, dimension, confidence, and
counter-evidence.

## Core Rule

R5 must never claim child progress unless the relevant child work is visible in the current analyzer
input.

Allowed:

```text
"parent-visible orchestration progressed after child result became visible"
"child-opaque delegation limited progress certainty"
"progress is insufficiently evidenced because child implementation is opaque"
```

Not allowed:

```text
"child implementation is advancing" when only parent wait/spawn/close rows are visible
"child troubleshooting frontier moved" without child diagnostic evidence
"parent waiting means stalled implementation" when child work is opaque
```

## Decision Table

| Topology | Visibility | Allowed R5 output |
|---|---|---|
| `single_agent` | `none` | normal archetype-native progress rules |
| `delegating_parent` | `partial` | visible evidence only; max `medium`; attach limiting signal when relevant |
| `delegating_parent` | `opaque` | `parent_visible_orchestration` or `insufficient_evidence`; max `low` |
| `delegated_child` | partial/opaque/unknown | treat as low-confidence unless this rollout itself contains direct child work; do not stitch parent context |
| `mixed_or_ambiguous` | `opaque` | prefer `insufficient_evidence`; max `low` |

## Public Signals

Opaque or partial cases should use:

```rust
ProgressSignalCode::DelegationVisibilityLimited
```

Recommended reason text:

```text
"delegating-parent plus child-opaque visibility prevented direct child progress claim"
"only parent-visible orchestration evidence was available for progress assessment"
"partial child visibility limited progress confidence"
```

## ParentVisibleOrchestration Dimension

Use `ProgressDimension::ParentVisibleOrchestration` when the progress evidence is about the parent
orchestrating work, not about the child executing implementation.

Positive signals:

1. child spawned with visible instructions,
2. child result link or summary appears in the parent trace,
3. parent incorporates visible child result into a plan, spec, handoff, or verification step,
4. parent closes child work after visible result.

Negative/stalled signals:

1. repeated wait/close attempts with no visible result,
2. repeated child spawn with no visible outcome,
3. parent changes delegation instructions without visible child output.

Insufficient evidence:

1. parent only waits,
2. parent mentions a separate child rollout but the child work is not loaded,
3. no visible child result or parent synthesis exists.

## Confidence Caps

```text
single_agent:
  normal confidence rules.

partial visibility:
  high is allowed only when the decisive evidence is directly visible in this checkpoint's rows;
  otherwise cap at medium.

opaque visibility:
  cap at low.

mixed_or_ambiguous:
  cap at low unless the checkpoint is clearly a single-agent child trace with direct work evidence.
```

For R5 first landing, prefer the simpler rule:

```text
Any delegating_parent + opaque => max low.
Any partial visibility with archetype-native progress => max medium unless explicitly child-visible.
```

## Interaction With Archetypes

When delegation is opaque:

1. `troubleshooting` should not emit `TroubleshootingFrontier` unless the failing diagnostics are
   parent-visible and parent-owned.
2. `autonomous_implementation` should not emit `ImplementationVerificationWall` for child work.
3. `verification_closeout` should not emit closeout proof progress for child proof unless proof
   output is visible.
4. `planning` may emit `PlanningConvergence` if the parent is visibly creating/refining a plan, but
   confidence should stay low/medium if the plan depends on opaque child results.

## Test Requirements

R5 should include at least these delegation cases:

1. single-agent troubleshooting still behaves normally,
2. delegating parent with opaque child work returns low-confidence `insufficient_evidence` or
   `parent_visible_orchestration`,
3. partial child visibility can produce parent-visible progress but includes limiting signal,
4. parent wait loop does not become child implementation stall,
5. parent spawn/result/close sequence can produce orchestration progress without claiming child
   code progress.

## Relationship To R7

R7 remains the first packet allowed to:

1. link parent and child rollout files,
2. interpret child-visible work as child progress,
3. represent parent progress and child progress separately in public schema,
4. add delegated-session-specific drift semantics.

R5's job is only to avoid overclaiming and provide a safe fallback dimension.

## Non-Goals

This design does not:

1. make `DelegationContext` public,
2. stitch multiple rollout files,
3. evaluate child agent quality,
4. infer unseen progress,
5. change sentinel interpretation consolidation.

## Locked Decisions After Packet R5-0

1. `ParentVisibleOrchestration` is an R5 delegation-only fallback dimension; non-delegated
   orchestration does not use it in this family.
2. Delegated cases never exceed `Medium` confidence in R5, and opaque visibility stays capped at
   `Low`.
3. If a debug artifact is emitted, it should include delegation markers internally, but the public
   contract remains limited to the final progress signal and evidence.
