# Design: Delegation-Aware Analyzer Boundary

Status: draft design input. This document defines the narrow analyzer-owned delegation boundary
that should sit underneath later `R4` through `R6` semantics. It is not full subagent support, it
is not a multi-trajectory evaluator, and it is not a sentinel redesign. It freezes what the
analyzer should know about delegation before `session_archetype`, `session_progress`, and
context-aware drift scoring harden single-agent assumptions into deeper logic.

## Why This Doc Exists

The current hybrid-drift stack has been intentionally proven on a screened non-subagent corpus.

That was the right call for `R1` through `R3.5`, because those packets were establishing:

1. typed outcome evidence,
2. deterministic checkpoint state,
3. turn-local context,
4. replay/live checkpoint compatibility.

But the current repo truth also says delegated runs were screened out rather than modeled:

1. `R1C` and `R1E` excluded sessions with `multi_agent_v1` delegation markers,
2. `R2` froze only the non-subagent acceptance corpus,
3. `R3.75` is now the immediate delegation-boundary packet, and `R4` through `R8` are the
   downstream semantic refinement family on top of that simpler world.
4. in real delegated/subagent runs, the child work may be emitted as its own ordinary
   `rollout-*.jsonl` with a distinct session id, so the visible parent rollout can be genuinely
   incomplete rather than merely sparse.

Without a delegation-aware boundary:

1. `R4` risks over-claiming session mode from parent-visible behavior alone,
2. `R5` risks claiming progress when the visible parent transcript only delegated and waited,
3. `R6` risks treating delegation wait periods or child opacity as parent-side thrash,
4. `R8` would consolidate replay/live interpretation on top of an analyzer seam that still
   pretends delegated and non-delegated sessions mean the same thing.

This design fills that exact gap while staying much narrower than full multi-agent support.

## Relationship To Existing Decisions

This design must compose with:

1. [HYBRID_DRIFT_REMAINING_GAPS_AND_LANDING_ORDER.md](../../HYBRID_DRIFT_REMAINING_GAPS_AND_LANDING_ORDER.md):
   the current acceptance and replay proof story is explicitly bounded to non-subagent sessions.
2. [docs/specs/agent-drift-analyzer-acceptance-fixture-hardening-r2-spec.md](./agent-drift-analyzer-acceptance-fixture-hardening-r2-spec.md):
   the frozen `R2` corpus deliberately excludes delegated sessions.
3. [docs/specs/agent-drift-analyzer-session-archetype-r4-spec.md](./agent-drift-analyzer-session-archetype-r4-spec.md):
   `R4` wants session-prefix state that is reusable by `R5` and `R6`.
4. [docs/specs/DESIGN-r4-intent-evidence-layer.md](./DESIGN-r4-intent-evidence-layer.md):
   lower intent evidence currently assumes one visible trajectory is a meaningful behavioral base.
5. [docs/specs/DESIGN-r4-session-archetype-aggregation.md](./DESIGN-r4-session-archetype-aggregation.md):
   archetype aggregation needs a way to degrade or cap confidence when child work is opaque.
6. [docs/specs/agent-drift-analyzer-outcome-evidence-r1-plan.md](./agent-drift-analyzer-outcome-evidence-r1-plan.md):
   the prior decision was to allow only the tiniest screening helper during `R1`, not to invent a
   supported delegation seam prematurely.

## Problem Statement

How might the analyzer become delegation-aware enough that:

1. delegated sessions are not silently treated as ordinary single-agent trajectories,
2. later archetype and progress modules can reason honestly about parent-visible versus child-opaque
   behavior,
3. the first delegation support remains narrow and auditable rather than exploding into cross-run
   orchestration analysis?

## Frozen Direction

The direction frozen by this design is:

1. add one analyzer-owned delegation boundary before or alongside early `R4` semantic work,
2. make that boundary primarily descriptive, not evaluative,
3. detect delegation topology and visibility limits first,
4. let downstream modules consume that boundary to cap confidence or suppress over-claims,
5. avoid claiming child progress or child intent when the child trajectory is not visible.

First-landing decisions now frozen for `R3.75-1`:

1. `DelegationContext`, `DelegationTopology`, and `ChildWorkVisibility` stay analyzer-local for
   the first landing,
2. exported checkpoints remain on `schema_version = "v0.4"` throughout `R3.75`,
3. sentinel replay/live compatibility and presentation remain unchanged throughout `R3.75`,
4. a separate child `rollout-*.jsonl` or child session id is an explicit opacity boundary for
   `R3.75`, not a license to stitch parent and child trajectories in this packet,
5. bounded parent/child linkage and supported delegated-session semantics remain deferred to `R7`.

## Non-Goals

This design does not:

1. join multiple child transcripts into one full causal execution graph,
2. infer child success from parent commentary,
3. make delegated sessions first-class acceptance proof by default,
4. redesign compactor bundle schema unless current rollout markers prove insufficient,
5. change sentinel posture or scheduler policy directly.

## Current Repo Truth

The analyzer already has strong evidence that delegation exists in some sessions, but current usage
is limited to exclusion and screening:

1. the `R1` packet family screened out sessions with rollout markers such as:
   - `multi_agent_v1`
   - `spawn_agent`
   - `wait_agent`
   - `close_agent`
2. the `R2` acceptance corpus explicitly excludes the known delegated sessions:
   - `019e93f8-a5e9-7490-ac1a-955b74c92ad0`
   - `019e9406-6736-79a2-946b-8a603e557422`
3. no current analyzer module exports a stable delegation context on checkpoints.

The important nuance is that "child opacity" is often concrete rather than hypothetical: a
delegated child may have its own ordinary rollout file and its own session id, while the current
analyzer path only sees the parent-visible rollout unless a later packet deliberately stitches
those artifacts together.

That means the stack can currently answer:

1. "should this session stay out of the non-subagent proof corpus?"

but it cannot yet answer:

1. "how much semantic confidence is justified for this delegated session?"
2. "did visible parent behavior actually make progress, or did it only allocate opaque child work?"

## Conceptual Output

The cleanest first seam is one analyzer-local state object conceptually equivalent to:

```text
DelegationContext
- topology: DelegationTopology
- child_work_visibility: ChildWorkVisibility
- markers: Vec<EvidenceRef>
- confidence: Confidence

DelegationTopology
- single_agent
- delegating_parent
- delegated_child
- mixed_or_ambiguous

ChildWorkVisibility
- none
- partial
- opaque
```

Interpretation rules:

1. this is not a quality judgment,
2. this does not claim whether delegation was good or bad,
3. it describes what kind of execution topology the visible checkpoint belongs to,
4. it records whether downstream semantics can honestly see the work that matters.

## Topology Rules

### 1. `single_agent`

Use when no credible delegation markers are observed in the visible session prefix.

This remains the ordinary baseline path and preserves the current bounded proof posture.

### 2. `delegating_parent`

Use when the visible trajectory clearly allocates, waits on, resumes around, or closes child work.

Typical evidence:

1. `multi_agent_v1` markers,
2. `spawn_agent`,
3. `wait_agent`,
4. `close_agent`,
5. explicit parent-side orchestration around child execution.

### 3. `delegated_child`

Use only when the visible trajectory is itself clearly the child work unit rather than the parent
controller.

This should be conservative in the first version. If current repo artifacts do not expose that
role reliably, do not guess it.

### 4. `mixed_or_ambiguous`

Use when delegation markers exist but the visible role or visibility boundary is not stable enough
to classify confidently as parent or child.

This is preferable to inventing certainty.

## Visibility Rules

The key semantic boundary is not just "was there delegation?" It is "can the analyzer actually see
the work that would justify semantic claims?"

### 1. `none`

Use when:

1. no delegation is observed, or
2. the visible trajectory is the actual work surface and no child opacity applies.

### 2. `partial`

Use when:

1. some child-facing results or summaries are visible,
2. but the decisive child trajectory is not fully available checkpoint-by-checkpoint.

### 3. `opaque`

Use when:

1. the parent clearly delegates,
2. the visible transcript mostly shows orchestration, waiting, or summary references,
3. the analyzer cannot see enough of the child trajectory to claim child intent or progress
   directly.

This often means the decisive child work lives in a separate ordinary rollout JSONL that is not
currently joined into the parent-visible analyzer bundle.

This is the most important first-pass state because it is what should force later semantic modules
to stay humble.

## Evidence Sources

The first implementation should prefer already visible rollout and analyzer surfaces:

1. rollout text markers already used during `R1` screening,
2. explicit tool or command observations tied to delegation,
3. task-frame evidence that the visible session is orchestrating work rather than executing the
   substantive implementation steps itself,
4. turn-context signals showing long wait-orchestration sequences with limited direct local edits or
   verification.

The design should avoid new upstream schema requirements unless current markers prove too weak.
It should also assume the current first-pass analyzer view is usually parent-rollout-local rather
than a stitched parent-plus-child rollout graph.

## Consumption Rules For Later Packets

### `R4` Session Archetype

`R4` may still classify delegated sessions, but it must do so with explicit humility.

Rules:

1. if `DelegationContext.topology = single_agent`, ordinary `R4` logic applies,
2. if `topology = delegating_parent` and `child_work_visibility = opaque`, archetype may still be
   classified from visible parent behavior, but confidence should be capped conservatively,
3. parent-side orchestration around child work must not be mistaken for direct implementation
   cadence,
4. if visible evidence is too thin once child opacity is accounted for, the analyzer should prefer
   lower confidence rather than pretending certainty.

### `R5` Session Progress

`R5` is where this seam becomes mandatory rather than merely nice-to-have.

Rules:

1. do not claim troubleshooting frontier advance through child work unless the frontier movement is
   visible in the parent-visible evidence,
2. do not claim implementation verification-wall progress merely because the parent delegated and
   later received a summary,
3. parent orchestration progress may still be measurable, but it must be described as parent-level
   progress, not invisible child execution progress.

### `R6` Drift Scorer Cutover

`R6` scorers must treat delegation as a semantic guardrail.

Rules:

1. waiting on opaque child work must not be treated the same as idle repeated parent thrash,
2. repeated delegation without visible child result may be suspicious, but that is a different
   hypothesis than ordinary single-agent `dead_end_thrash`,
3. if later scorer work wants delegated-session-specific failure modes, that belongs in a later
   packet rather than being smuggled into the first delegation seam.

### `R7` Full Delegated-Session Support

`R7` is where the analyzer should move from guarded delegation-awareness into bounded supported
delegated-session semantics.

Rules:

1. `R7` may add explicit parent-versus-child semantic handling when child evidence is actually
   available,
2. `R7` may define supported delegated-session progress and drift interpretation for a bounded
   subset of delegated runs,
3. `R7` is the first packet that may deliberately bridge a parent-visible rollout with one or more
   separate ordinary child rollout files / child session ids when that linkage is explicit and
   bounded,
4. `R7` should not require sentinel interpretation consolidation in the same packet,
5. `R7` should preserve the conservative `R3.75` fallback for delegated sessions whose child work
   remains opaque.

### `R8` Sentinel Interpretation

`R8` should consume the analyzer boundary rather than define it.

Replay/live interpretation may eventually surface delegation-aware presentation, but it should ride
on analyzer-exported semantics, not invent its own delegation heuristics.

## Recommended Packet Placement

The minimum honest placement is:

1. freeze this design before `R3.75` implementation starts,
2. land the first analyzer-local delegation boundary as `R3.75`, before `R4` session archetype
   work,
3. land full delegated-session semantic support as `R7` before sentinel consolidation,
4. do not delay the boundary until after `R8`.

Reason:

1. `R4` can still land narrowly with conservative delegation handling once `R3.75` is in place,
2. `R5` and `R6` become much riskier if they harden single-agent progress and drift assumptions
   first,
3. `R7` is the right place for bounded supported delegated semantics after ordinary single-session
   semantics have stabilized,
4. `R8` is too late because it only consolidates interpretation of whatever analyzer semantics
   already exist.

## Validation Expectations

The first delegation-aware packet should validate three things:

1. non-delegated sessions still behave identically on the existing bounded corpus,
2. known delegated sessions are recognized as delegated rather than silently treated as ordinary
   single-agent sessions,
3. delegated-session semantics degrade confidence or scope claims honestly instead of inventing
   child-visible progress.

The first validation corpus does not need to make delegated sessions "fully supported." It only
needs to prove that the analyzer stops over-claiming when delegation is visible but child work is
opaque.

## Open Questions

1. Are current rollout markers sufficient to distinguish `delegating_parent` from
   `mixed_or_ambiguous`, or will some sessions require a slightly richer helper?
2. Should the first bounded delegated corpus remain report-only, or should one or two delegated
   cases become explicit low-confidence regression fixtures once the boundary lands?
