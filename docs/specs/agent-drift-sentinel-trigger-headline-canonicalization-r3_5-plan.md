# Plan: Agent Drift Sentinel Trigger Headline Canonicalization R3.5

## Scope

Implementation status on `2026-06-07`:

- `R3` is landed end-to-end.
- Real-rollout proof on `019ea027-9bb6-7aa2-9d05-94f3ba997948` validated the new turn-context
  surface.
- That same proof exposed one remaining sentinel-local mismatch: replay headlines a flagged
  checkpoint as `scheduler_repeated_failure_trigger`, while live headlines the same checkpoint as
  `checkpoint_ready`.

This plan implements:

- `docs/specs/agent-drift-sentinel-trigger-headline-canonicalization-r3_5-spec.md`

`R3.5` is a narrow sentinel packet between the landed `R3` turn-context family and the later
`R4+` analyzer-deepening packets.

It should:

- canonicalize checkpoint headlines across replay and live
- keep analyzer-owned posture as the semantic truth source
- keep ordinary checkpoint arrivals canonicalized as `checkpoint_ready`
- preserve distinct scheduler fast-path labeling when sentinel actually emits that path
- reserve `scheduler_repeated_failure_trigger` for true synthetic scheduler fast-path events
- stay explicitly distinct from the already-landed `R3-5` turn-context packet by keeping
  `R3.5` packet naming and `r3_5` repo-file naming
- prove replay/live parity without widening into broader interpretation cleanup

It should not:

- change analyzer checkpoints, posture rules, or scorer behavior
- change scheduler policy or warning thresholds
- collapse replay and live into one larger interpretation module
- start `R4`, `R5`, `R6`, or `R7`

## Why This Packet Comes Next

`R3` already solved the structural checkpoint-context gap. The remaining inconsistency is smaller
but still operator-facing: the same checkpoint can be described as
`scheduler_repeated_failure_trigger` in replay and `checkpoint_ready` in live even though posture,
drift, diagnostics, and turn context all match.

That seam should close before `R4` because:

- later packets should not stack new semantics on top of a known presentation mismatch
- the fix is narrow, reviewable, and local to sentinel
- the fix clarifies the distinction between checkpoint posture and scheduler fast paths

## Implementation Strategy

Execution split:

1. `R3.5-1`: lock the repo-doc contract and packet boundary
2. `R3.5-2`: canonicalize replay/live checkpoint headlines
3. `R3.5-3`: preserve scheduler fast-path distinction and prove parity with focused regressions

### Workstream 1: Lock The R3.5 Contract

Write packet authority that defines:

- the canonical checkpoint headline rule
- the distinction between checkpoint arrival and scheduler fast path
- the naming distinction between the new `R3.5` packet and the already-landed `R3-5` packet
- the explicit non-goals for analyzer semantics and broader sentinel consolidation

Why first:

- the code change is small, but the naming is easy to confuse with the already-landed `R3-5`
  turn-context packet
- later review needs a crisp contract for what counts as ordinary checkpoint presentation versus a
  true fast-path event

### Workstream 2: Canonicalize Checkpoint Headlines

Update sentinel presentation so that ordinary checkpoint payloads headline consistently across
replay and live.

Expected direction:

- replay checkpoint presentation should headline `checkpoint_ready`
- live checkpoint-ready presentation should remain `checkpoint_ready`
- analyzer-owned posture remains unchanged and still carries the semantic warning meaning

Why second:

- this is the actual operator-facing bug
- it is the smallest implementation step that resolves the mismatch without refactoring the whole
  sentinel interpretation stack

### Workstream 3: Preserve Fast-Path Distinction And Prove It

Keep synthetic repeated-failure events visibly distinct after the checkpoint-headline cutover.

This should prove:

- ordinary flagged checkpoints do not overclaim scheduler-trigger status
- real repeated-failure fast-path events still render distinctly
- recovered and historical-only follow-on events still show honest posture

Why third:

- the main regression risk is accidentally erasing the scheduler fast-path label entirely
- parity proof belongs in the same packet as the presentation cutover

## Sequencing

Sequential work:

1. `R3.5-1`: land spec/plan/tasks authority and naming distinction
2. `R3.5-2`: change replay checkpoint headline derivation to follow the canonical rule
3. `R3.5-2`: keep live checkpoint-ready presentation aligned with the same rule
4. `R3.5-3`: update replay/live parity tests for matched checkpoints
5. `R3.5-3`: keep scheduler fast-path tests green

Not parallel-safe:

- starting `R4` archetype work before this operator-facing mismatch is closed
- widening the change into replay/live interpretation consolidation
- tying checkpoint headline rendering to analyzer posture or score retuning

## Major Risks And Mitigations

### Risk 1: The Packet Quietly Becomes R7

Mitigation:

- keep the change local to headline derivation and focused tests
- do not extract a broad shared interpretation module in this packet

### Risk 2: Scheduler Fast-Path Visibility Gets Lost

Mitigation:

- preserve distinct repeated-failure rendering for synthetic scheduler events
- add focused tests that prove the label still appears when the fast path is actually emitted

### Risk 3: Posture And Trigger Semantics Get Re-coupled

Mitigation:

- keep posture derived from analyzer checkpoint state
- keep trigger labeling derived from how sentinel is presenting the event
- prove recovered and historical-only checkpoints retain their existing posture lines

## Verification Checkpoints

### Checkpoint 1: Contract Boundary Locked

Confirm the spec, plan, and tasks all agree that:

- `R3.5` is sentinel-local trigger-headline canonicalization only
- analyzer semantics remain out of scope
- file naming and packet naming stay distinct from `R3-5`
- ordinary checkpoint arrivals stay `checkpoint_ready`
- `scheduler_repeated_failure_trigger` remains reserved for true synthetic scheduler fast paths

### Checkpoint 2: Replay/Live Checkpoint Parity Holds

Run:

```bash
cargo test -p agent-drift-sentinel operator_surface -- --nocapture
cargo test -p agent-drift-sentinel live_end_to_end -- --nocapture
```

Pass means:

- matched replay/live checkpoints headline `checkpoint_ready`
- posture, drift, diagnostics, and turn-context rendering remain aligned

### Checkpoint 3: Scheduler Fast Path Stays Distinct

Run:

```bash
cargo test -p agent-drift-sentinel operator_sink -- --nocapture
cargo test -p agent-drift-sentinel live_runtime -- --nocapture
```

Pass means:

- synthetic repeated-failure events still render `scheduler_repeated_failure_trigger`
- the packet did not flatten all trigger labels into one path
