# Plan: Agent Drift Analyzer Delegation-Aware Boundary R3.75

## Scope

Implementation status on `2026-06-08`:

- The analyzer outcome-evidence `R1` family is landed end-to-end.
- `R2` acceptance-fixture hardening is landed and explicitly excludes delegated sessions from the
  success-tail corpus.
- `R3` turn-context promotion is landed with analyzer checkpoint schema `v0.4`.
- `R3.5` replay/live trigger-headline canonicalization is landed.
- `R3.75-1` through `R3.75-3` are now landed on this worktree.
- `R3.75-4` bounded delegated regression coverage remains the only open packet before `R4`.
- `R4`, `R5`, `R6`, `R7`, and `R8` remain outside the `R3.75` packet boundary.

This plan implements:

- `docs/specs/agent-drift-analyzer-delegation-aware-boundary-r3_75-spec.md`

Validation gate for this phase:

- this plan remains the `PLAN` phase artifact for `R3.75`, now serving as the packet-progress
  reference through `R3.75-4`
- do not treat `R3.75` as complete or begin `R4` code work until the remaining `R3.75-4` packet
  is reviewed and accepted
- if the spec changes materially, update the spec first and then re-align this plan

`R3.75` is the narrow delegation guardrail packet between the landed `R3` structure family and the
later `R4` through `R6` semantic packets.

Its job is to stop the analyzer from pretending delegated and non-delegated sessions mean the same
thing before archetype, progress, and scorer logic deepen that assumption.

This packet should:

- define one analyzer-local `DelegationContext` seam
- classify visible delegation topology conservatively
- classify whether child work is visible, partially visible, or opaque
- keep the first landing descriptive and confidence-bearing
- preserve the current `v0.4` checkpoint contract
- preserve current sentinel compatibility by avoiding checkpoint-schema changes
- add a bounded delegated regression wall without reopening the `R2` success-tail corpus
- treat separate child rollout files / child session ids as a source of legitimate opacity in the
  parent-visible rollout rather than as data that `R3.75` should stitch automatically

This packet should not:

- widen checkpoint schema to `v0.5`
- change replay/live sentinel loaders or operator presentation
- claim child progress, child intent, or delegated-session-specific drift classes
- implement `R4` archetype, `R5` progress, `R6` scorer, or `R7` supported parent/child semantics

## Why This Packet Comes Next

`R3` and `R3.5` solved the ordinary checkpoint structure and replay/live compatibility problems,
but they deliberately kept delegated sessions out of scope.

Without `R3.75`:

- `R4` could classify a parent-orchestration transcript as if it were ordinary direct execution
- `R5` could claim progress when the visible parent only delegated and waited
- `R6` could treat child opacity as single-agent repeated thrash

`R3.75` is therefore the smallest honest next step:

- narrower than full delegated-session support
- earlier than any deeper semantic packet
- sufficient to cap confidence and suppress over-claims before later packets consume the seam

## Implementation Strategy

Execution split:

1. `R3.75-1`: lock the repo-doc contract and packet boundary only
2. `R3.75-2`: add analyzer-local delegation contract and marker harvesting
3. `R3.75-3`: derive checkpoint-local topology/visibility/confidence and, if needed, analyzer-only
   reporting
4. `R3.75-4`: add bounded delegated regressions while preserving the existing non-subagent
   acceptance corpus

### Workstream 1: Lock The Delegation-Boundary Contract

Lock the first-landing contract in repo docs first:

- analyzer-local `DelegationContext`
- conservative `DelegationTopology`
- conservative `ChildWorkVisibility`
- explicit decision to keep checkpoint schema at `v0.4`
- explicit decision that separate child rollout files / child session ids only justify `partial` or
  `opaque` visibility in `R3.75`; they do not authorize parent/child stitching in this packet
- explicit decision that `R7`, not `R3.75`, owns bounded parent/child linkage and supported
  delegated-session semantics

Why first:

- `R4` already assumes it owns the next explicit checkpoint widening as `v0.5`
- this is where the repo must freeze the difference between `R3.75` guardrails and `R7`
  full delegated-session semantics
- `R3.75-1` ends at doc authority; no analyzer code changes belong in this packet

### Workstream 2: Add Analyzer-Local Contract And Marker Harvesting

Implement one analyzer-local classification seam that gathers delegated evidence from:

- rollout markers already used during `R1` screening:
  - `multi_agent_v1`
  - `spawn_agent`
  - `wait_agent`
  - `close_agent`
- visible orchestration behavior in command/context evidence
- existing task-frame and turn-context signals that indicate parent-side coordination rather than
  direct local execution

Important constraint:

- the current first landing remains parent-rollout-local even when delegated child work exists in a
  separate ordinary rollout JSONL with its own session id

The seam should produce:

- one topology label
- one child-visibility label
- one confidence level
- supporting and counter-evidence

Why second:

- the classifier contract must exist before checkpoint analysis can consume it consistently
- this is the packet that resolves the first-landing question in favor of analyzer-local state,
  not exported checkpoint schema

### Workstream 3: Derive Checkpoint-Local Delegation State And Analyzer Proof Surface

Once the classifier seam exists, wire it into checkpoint analysis so every checkpoint analysis can
reason about:

- ordinary single-agent work
- visible parent orchestration with opaque or partial child visibility
- ambiguous delegated situations that should suppress certainty
- the fact that child opacity may come from a separate child rollout artifact that is not yet
  joined into the current analyzer input

This workstream may also add a compact analyzer-only proof surface if needed, such as summary
rendering that shows:

- topology
- child visibility
- confidence
- the small set of evidence rows that justified the classification

Why third:

- proof output should ride on top of the stabilized analyzer-local state, not shape the state
  contract itself

### Workstream 4: Add Bounded Delegated Regression Coverage

Lock a small delegated regression wall that proves the seam is honest without broadening support:

- keep the `R2` non-subagent acceptance wall unchanged
- add one or two known delegated cases as bounded regressions outside that corpus
- assert conservative classification for those delegated cases
- keep delegated regressions analyzer-only in the first landing

Why fourth:

- this packet needs deterministic proof that delegated sessions are no longer silently treated as
  ordinary
- the proof should stay bounded and should not imply that delegated sessions are now fully
  supported

## Sequencing

Sequential work:

1. `R3.75-1`: lock the `R3.75` contract in repo docs
2. `R3.75-2`: add analyzer-local delegation types/helper and marker harvesting
3. `R3.75-3`: populate delegation topology/visibility/confidence during checkpoint analysis
4. `R3.75-3`: add compact analyzer proof output if needed
5. `R3.75-4`: add delegated regression coverage and preserve the `R2` corpus boundary
6. `R3.75-4`: run the focused analyzer wall

Human review checkpoints:

1. review the `SPECIFY` artifact before starting `R3.75-2`
2. review this `PLAN` artifact before treating the packet order as implementation-ready
3. review the `TASKS` artifact before starting code changes under `R3.75-2+`

Parallel-safe work after the contract is locked:

- fixture drafting for the known delegated sessions
- summary/rendering test drafting if analyzer-only proof output is desired
- evidence-selection discussion for `delegated_child` versus `mixed_or_ambiguous`

Not parallel-safe:

- widening checkpoint schema before the analyzer-local seam is validated
- changing sentinel compatibility or replay/live presentation during the same packet
- introducing `R4`/`R5`/`R6` semantics before the delegation boundary is stable

## Major Risks And Mitigations

### Risk 1: `R3.75` Quietly Becomes `R7`

Mitigation:

- keep the packet descriptive only
- reject any field or rule that claims parent/child progress semantics
- keep success criteria focused on topology, visibility, confidence, and corpus boundaries

### Risk 2: Marker Ambiguity Produces False Certainty

Mitigation:

- treat `mixed_or_ambiguous` as a first-class conservative outcome
- prefer `opaque` or `partial` visibility over inventing child visibility
- require evidence and counter-evidence to be surfaced for auditability

### Risk 3: Checkpoint-Schema Growth Collides With `R4`

Mitigation:

- keep `DelegationContext` analyzer-local in the first landing
- preserve exported checkpoints as `v0.4`
- defer any checkpoint export decision to later doc authority if still needed

### Risk 4: Delegated Regressions Contaminate The `R2` Acceptance Wall

Mitigation:

- keep the `R2` success-tail corpus unchanged
- place delegated proof in a separate bounded regression surface
- continue documenting delegated sessions as excluded from the non-subagent acceptance corpus

### Risk 5: Analyzer Proof Output Starts Acting Like A Scoring Surface

Mitigation:

- keep any `R3.75` reporting descriptive and compact
- do not render progress or severity judgments from the delegation boundary alone
- defer delegated-session operator semantics to later analyzer packets
