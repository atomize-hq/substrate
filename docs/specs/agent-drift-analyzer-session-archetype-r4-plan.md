# Plan: Agent Drift Analyzer Session Archetype R4

## Scope

Implementation status on `2026-06-08`:

- The analyzer outcome-evidence `R1` family is landed end-to-end.
- `R2` acceptance-fixture hardening is landed.
- `R3` turn-context promotion is landed with analyzer checkpoint schema `v0.4`.
- `R3.5` replay/live trigger-headline canonicalization is landed and closes the last known
  sentinel-local checkpoint headline mismatch from the `R3` family.
- `R3.75` delegation-aware analyzer boundary is landed on this worktree and is now an input to
  `R4`, not a blocker in front of it.
- the `SPECIFY`, `PLAN`, and `TASKS` artifacts now exist for `R4`, but no `R4` implementation
  packet has started yet
- `R5`, `R6`, `R7`, and `R8` remain outside the next-packet boundary.

This plan implements:

- `docs/specs/agent-drift-analyzer-session-archetype-r4-spec.md`

Validation gate for this phase:

- this plan is the `PLAN` phase artifact for review, not implementation approval by itself
- do not start `R4-1+` code work until the `SPECIFY` and `TASKS` artifacts are reviewed and
  accepted
- `SPECIFY` / `PLAN` / `TASKS` are pre-implementation gates and should not be counted as packeted
  implementation work
- if the spec changes materially, update the spec first and then re-align this plan

`R4` is the first session-meaning packet after the landed `R3` structure family, the landed
`R3.5` presentation cleanup, and the landed `R3.75` delegation-aware analyzer boundary.

Its job is to promote session archetype from an implicit human inference into explicit
analyzer-owned checkpoint state that later packets can consume directly.

This packet should:

- define one structured `SessionArchetype` contract on analyzer checkpoints
- classify checkpoints as `troubleshooting`, `planning`, `autonomous_implementation`, or
  `verification_closeout`
- derive the classification deterministically from existing task-frame, diagnostics, turn-context,
  and command-observation evidence
- treat the exported result as session type at checkpoint scope, not merely turn-archetype labeling
- keep the first code landing behavior-first and allow kickoff prompt shape / skill markers only as
  deferred, disabled, or weak-only bounded priors
- consume the landed `R3.75` delegation boundary to cap confidence and add counter-evidence when
  child work is opaque
- tighten low-hanging command-role and file-role interpretation so broad command families do not act
  as one-step label proxies
- widen analyzer checkpoints from `v0.4` to `v0.5`
- preserve replay/live compatibility by extending sentinel support to `v0.5` while keeping
  `v0.2` through `v0.4`
- render compact archetype inspection in analyzer summary and replay/live operator surfaces

This packet should not:

- add `R5` progress semantics
- retune `dead_end_thrash` or any other drift scorer
- redesign scheduler triggers, warning thresholds, or live-runtime coordination
- introduce model-based or prompt-based archetype inference
- broaden into a full phase/failure taxonomy redesign

## Why This Packet Comes Next

`R3` already solved the structural context gap by exporting explicit turn-local state, and `R3.5`
closed the remaining replay/live headline mismatch that would have distracted later semantic work.

The current checkpoint contract still lacks one critical piece of meaning: what mode of work the
checkpoint belongs to.

Without that seam:

- later `R5` progress modeling has no explicit task-mode baseline
- later `R6` scorer retuning still risks treating troubleshooting, planning, implementation, and
  verification closeout as if they meant the same thing
- operators still need to infer session mode manually from turn context, objective wording, and
  command mix

`R4` is therefore the smallest honest next step:

- richer meaning than `R3`
- still narrower than `R5` progress or `R6` scorer cutover
- no scheduler or scorer retuning yet

## Implementation Strategy

Spec-driven-development gate, already satisfied before implementation begins:

- the `SPECIFY` artifact locks scope and success criteria
- this `PLAN` artifact defines implementation order and risks
- the `TASKS` artifact names the packeted implementation units
- implementation packet numbering begins below at the first code-bearing slice

Execution split:

1. `R4-1`: add analyzer-owned schema types and legacy-safe `v0.5` export
2. `R4-2`: add the lower intent-evidence seam plus deterministic archetype aggregation
3. `R4-3`: lock analyzer summary / fixture authority / regression walls
4. `R4-4`: extend sentinel compatibility without changing posture behavior
5. `R4-5`: surface compact replay/live archetype presentation on top of the stabilized contract

### Workstream 1: Add Analyzer Schema Types And Legacy-Safe `v0.5` Export

Implement the analyzer-owned checkpoint contract surface:

- `SessionArchetype`
- `SessionArchetypeLabel`
- explicit `schema_version = "v0.5"` export
- exported `session_archetype` checkpoint field
- shared DTO serde that stays compatible with `v0.2` through `v0.4` artifacts while letting
  `v0.5` validation require the field explicitly

Why first:

- `R4-1` should lock the analyzer-owned schema surface and explicit version cutover before
  deterministic classification is wired into checkpoint construction

### Workstream 2: Add Lower Intent Evidence And Archetype Aggregation

Once the schema surface is fixed, implement one analyzer-local lower layer plus one aggregation
helper that derives the current checkpoint archetype using:

- task-frame objective and truth-artifact signals
- `R3` turn-context execution mode and activity mix
- `R3.75` delegation topology / child-visibility guardrails
- diagnostics such as verification density, recovery, and task-frame transitions
- command-observation cadence and working-set concentration interpreted through command-role and
  file-role heuristics
- kickoff-prompt prior signals only if they survive the behavior-first guardrails

The helper should produce:

- one primary archetype label
- one confidence level
- evidence and counter-evidence lists that explain why that label won

Interpretation rule:

- the exported label is session type at the current checkpoint, derived from cumulative evidence up
  to that checkpoint
- turn-local evidence is an input, not the final scope of the label

It should also:

- attach `session_archetype` to every checkpoint
- cap or degrade confidence when delegated-parent evidence is child-opaque
- prefer low-confidence `planning` over overclaiming when evidence stays sparse

### Workstream 3: Lock Summary, Fixture Authority, And Regression Walls

Once the derivation seam is stable, lock the packet proof surface:

- update summary rendering with compact archetype inspection
- add the fixture-manifest authority doc for the first label matrix
- lock deterministic regressions for the four initial archetypes plus the required ambiguous,
  transition, delegated, and compatibility cases

Why third:

- summary output and regression walls should ride on top of the stabilized derivation boundary

### Workstream 4: Extend Sentinel Compatibility

Update replay/live checkpoint loaders to:

- accept `v0.5`
- preserve `v0.2` through `v0.4` fallback behavior
- treat missing `session_archetype` as acceptable for legacy schemas only
- update any explicit-analyzer-state helper that still hard-codes only `v0.3 | v0.4`

Why fourth:

- once analyzer emits `v0.5`, replay/live compatibility becomes the primary downstream risk

### Workstream 5: Surface Archetype In Replay/Live Presentation

Add a compact presentation seam so replay/live consumers can inspect checkpoint archetype directly
without opening raw JSON artifacts.

This should:

- render a compact summary of checkpoint archetype and confidence
- stay presentation-only
- preserve current posture/severity behavior
- keep replay/live parity tests tight

Why fifth:

- consumer visibility is part of the packet objective, but it should ride on top of the stabilized
  analyzer contract rather than shape it

## Sequencing

Sequential work:

1. review and accept the `SPECIFY`, `PLAN`, and `TASKS` artifacts before code starts
2. `R4-1`: add analyzer-local session-archetype types and explicit legacy-safe `v0.5` export seam
3. `R4-2`: populate deterministic `session_archetype` through an intent-evidence layer plus
   aggregation helper
4. `R4-3`: update analyzer summary rendering, fixture authority, and analyzer tests
5. `R4-4`: extend sentinel replay/live compatibility to `v0.5`
6. `R4-5`: expose compact archetype output in replay/live presentation
7. after `R4-1` through `R4-5`, run focused analyzer and sentinel walls

Human review checkpoints:

1. review the `SPECIFY` artifact before starting `R4-1`
2. review this `PLAN` artifact before treating the packet order as implementation-ready
3. review the `TASKS` artifact before starting code changes under `R4-1+`

Parallel-safe work after the contract is locked:

- analyzer fixture/test drafting for the four initial archetype shapes
- summary rendering test drafting while archetype derivation is being coded
- sentinel presentation wording once the structured fields are stable

Not parallel-safe:

- finalizing sentinel compatibility before the `v0.5` analyzer contract is stable
- adding `R5` progress fields while archetype definitions are still moving
- folding archetype into drift scorers during the same packet

## Major Risks And Mitigations

### Risk 1: `R4` Ignores The Landed Delegation Boundary

Mitigation:

- consume the existing analyzer-local `DelegationContext` instead of re-deriving delegation ad hoc
- cap confidence and add counter-evidence for delegated-parent plus opaque-child cases
- keep `R7` as the first packet allowed to claim supported parent/child semantics

### Risk 2: `R4` Quietly Becomes `R5`

Mitigation:

- keep the packet focused on classification only
- reject any field that claims frontier advance, convergence, or review completeness
- keep success criteria focused on explicit session-mode exposure

### Risk 3: Prompt Wording Overpowers Observable Evidence

Mitigation:

- require classification to combine multiple observable signals
- bias single-signal or keyword-only classifications to lower confidence
- expose evidence and counter-evidence so misclassifications are auditable

### Risk 4: Additive `v0.4` Growth Creates Contract Ambiguity

Mitigation:

- widen to explicit schema `v0.5`
- keep the shared DTO serde backward-compatible while making `v0.5` requiredness explicit in
  schema-aware validation
- test legacy fallback and `v0.5` preferred behavior side by side

### Risk 5: Broad Command Families Produce Bad Archetype Evidence

Mitigation:

- add low-hanging command-role parsing for common verification, format, build, dependency, and
  inspect-vcs commands
- combine command role with file-role context before strengthening a label
- keep the first pass deterministic and conservative rather than building a shell parser

### Risk 6: Replay/Live Surfaces Drift Apart

Mitigation:

- keep one compact presentation shape for archetype output
- prove replay/live parity with existing `live_end_to_end` style tests
- avoid custom formatting branches that exist only in one path

### Risk 7: Archetype Labels Start Acting Like Quality Judgments

Mitigation:

- treat labels as descriptive session mode, not success/failure verdicts
- keep confidence separate from severity
- defer progress and drift judgments to `R5` and `R6`

### Risk 8: Prompt Priors Overfit Repo-Specific Wording

Mitigation:

- treat kickoff prompt features as bounded priors only
- require behavioral confirmation before emitting high confidence
- keep explicit evidence lines so prompt-derived influence remains auditable

## Verification Checkpoints

### Checkpoint 1: Contract Boundary Locked

Confirm the spec, plan, and tasks all agree that:

- `R4` is checkpoint-local session-archetype classification only
- schema widening is explicit and versioned
- `R5` progress, `R6` scorer changes, `R7` full delegated-session support, and `R8` sentinel
  consolidation stay out of scope

### Checkpoint 2: Analyzer `v0.5` Contract Passes

Run:

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo test -p agent-drift-analyzer export_bundle -- --nocapture
cargo test -p agent-drift-analyzer end_to_end -- --nocapture
```

Pass means:

- checkpoints emit `schema_version = "v0.5"`
- each `v0.5` checkpoint carries explicit `session_archetype`
- legacy `v0.2` through `v0.4` artifacts still deserialize cleanly
- summary output exposes compact archetype inspection

### Checkpoint 3: Archetype Regressions Hold

Run:

```bash
cargo test -p agent-drift-analyzer -- --nocapture
```

Pass means:

- deterministic cases cover troubleshooting, planning, autonomous implementation, and
  verification-closeout shapes
- ambiguous mixed cases stay capped at `low` or `medium` confidence
- delegated-parent plus opaque-child cases cap confidence conservatively
- transition / hysteresis cases avoid one-checkpoint flapping
- PR-response and proof-closeout loops keep their intended boundary
- ambiguous cases degrade confidence instead of inventing new archetypes
- reruns on the same bundle preserve identical archetype output

### Checkpoint 4: Sentinel Compatibility And Presentation Hold

Run:

```bash
cargo test -p agent-drift-sentinel replay_input -- --nocapture
cargo test -p agent-drift-sentinel live_checkpoint_compatibility -- --nocapture
cargo test -p agent-drift-sentinel operator_surface -- --nocapture
cargo test -p agent-drift-sentinel live_end_to_end -- --nocapture
```

Pass means:

- replay/live loaders accept `v0.5` while preserving legacy schemas
- replay/live surfaces expose the same compact archetype view for matching checkpoints
- posture, diagnostics, turn-context rendering, and checkpoint headlines remain unchanged
