# Spec: Agent Drift Sentinel Trigger Headline Canonicalization R3.5

## Assumptions I'm Making

1. Live repo truth on `2026-06-07` is the authority: the full `R3` turn-context family is landed,
   and the next honest seam is the replay/live trigger-headline mismatch documented in
   `HYBRID_DRIFT_REMAINING_GAPS_AND_LANDING_ORDER.md`.
2. This packet is sentinel-local. Analyzer schema, analyzer scoring, checkpoint posture, and
   turn-context semantics are already the truth source and should remain unchanged here.
3. The current mismatch is narrow and concrete:
   - replay checkpoint presentation derives `TriggerClass::RepeatedFailure` from
     `checkpoint.flagged`
   - live checkpoint presentation keeps the ingress trigger `checkpoint_ready`
   - real scheduler fast-path events already exist separately on the live path
4. The operator-facing bug is not that sentinel cannot represent scheduler fast paths. The bug is
   that replay overstates ordinary flagged checkpoints as if they were scheduler-trigger events.
5. File naming for this packet should stay distinct from the already-landed `R3-5` turn-context
   packet. Repo files for this packet therefore use `r3_5` rather than `r3-5`.

## Objective

Make replay and live use one honest operator-facing trigger headline for the same checkpoint
without reopening analyzer semantics or broadening into sentinel interpretation consolidation.

Primary users:

- the operator comparing replay and live output for the same checkpoint and needing the headline to
  mean the same thing in both places
- the maintainer landing `R4+` packets and needing replay/live presentation parity before adding
  deeper analyzer semantics

Success means:

- matched replay/live checkpoint presentations headline the checkpoint the same way
- ordinary checkpoint presentations do not claim
  `scheduler_repeated_failure_trigger` solely because a checkpoint is flagged
- real scheduler fast-path events still render distinctly when sentinel actually emits them
- checkpoint posture, drift summaries, diagnostics, and turn-context output remain unchanged

## Packet Boundary

This packet is **trigger-headline canonicalization only**.

In scope:

- define one canonical operator-facing headline rule for checkpoint presentations
- make replay checkpoint presentation follow that canonical rule
- preserve live checkpoint-ready presentation under the same rule
- keep scheduler fast-path rendering distinct when sentinel actually handles a synthetic
  repeated-failure event
- add focused sentinel tests that prove replay/live parity for matched checkpoints and preserve the
  distinct scheduler fast path

Out of scope:

- analyzer checkpoint contract changes
- analyzer scorer or posture changes
- scheduler policy or warning-threshold changes
- replay/live interpretation consolidation beyond the minimum needed for this headline cutover
- `R4` archetype, `R5` progress, `R6` scorer cutover, or `R7` consolidation work

## Canonical Trigger Rule

Use these rules for operator-facing presentation:

1. If sentinel is presenting an ordinary checkpoint payload, headline it as `checkpoint_ready`.
2. If sentinel is presenting a true synthetic scheduler fast-path event, render
   `scheduler_repeated_failure_trigger`.
3. Do not derive the checkpoint headline from `checkpoint.flagged`.
4. Preserve analyzer-owned posture (`active`, `recovered`, `historical-only`) as the truth source
   for the checkpoint's semantic status.

This packet intentionally separates:

- how a checkpoint arrived at the operator surface
- what semantic posture the analyzer assigned to that checkpoint

## Tech Stack

- Language: Rust 2021
- Target crate:
  - `crates/agent-drift-sentinel`
- Existing analyzer checkpoint contracts:
  - `v0.2 | v0.3 | v0.4`
- No new crate or external dependency is required

## Commands

Formatting gate:

```bash
cargo fmt --all -- --check
```

Sentinel-focused verification wall:

```bash
cargo test -p agent-drift-sentinel operator_surface -- --nocapture
cargo test -p agent-drift-sentinel operator_sink -- --nocapture
cargo test -p agent-drift-sentinel live_runtime -- --nocapture
cargo test -p agent-drift-sentinel live_end_to_end -- --nocapture
```

Optional broader sentinel confidence wall:

```bash
cargo test -p agent-drift-sentinel replay_input -- --nocapture
cargo test -p agent-drift-sentinel live_checkpoint_compatibility -- --nocapture
```

## Project Structure

```text
crates/agent-drift-sentinel/src/operator_surface.rs
  Owns replay checkpoint presentation and the rendered headline. This packet should stop deriving
  replay checkpoint headlines from `checkpoint.flagged`.

crates/agent-drift-sentinel/src/live_input.rs
  Owns live event typing. This packet should preserve `checkpoint_ready` for ordinary checkpoint
  arrivals and keep synthetic repeated-failure events distinct.

crates/agent-drift-sentinel/src/live_runtime.rs
  Owns live event observation and should continue to surface synthetic repeated-failure fast paths
  distinctly after the checkpoint-headline cutover.

crates/agent-drift-sentinel/tests/operator_surface.rs
  Owns checkpoint-presentation coverage. This packet should add or adjust replay assertions here.

crates/agent-drift-sentinel/tests/operator_sink.rs
  Owns operator-event labeling coverage. This packet should keep scheduler fast-path labels honest.

crates/agent-drift-sentinel/tests/live_end_to_end.rs
  Owns replay/live parity coverage and should lock the distinction between checkpoint headlines and
  true scheduler fast-path events.
```

## Acceptance

- replay and live render the same headline for the same matched checkpoint
- replay no longer labels an ordinary flagged checkpoint as
  `scheduler_repeated_failure_trigger`
- live ordinary checkpoint-ready events continue to present as `checkpoint_ready`
- live synthetic repeated-failure events still render
  `scheduler_repeated_failure_trigger`
- recovered and historical-only checkpoints do not lose their analyzer-owned posture reporting
- the fix stays sentinel-local and does not widen into broader interpretation refactors
