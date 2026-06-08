# Spec: Agent Drift Analyzer Delegation-Aware Boundary R3.75

## Assumptions I'm Making

1. Live repo truth on `2026-06-08` is the authority: the analyzer outcome-evidence `R1` family,
   acceptance-fixture `R2`, turn-context `R3`, trigger-headline `R3.5`, and the bounded
   delegation-aware analyzer boundary `R3.75` are landed on this worktree; `R4` is now the next
   semantic packet family, while `R5` through `R8` remain queued behind it.
2. The first `R3.75` landing should keep `DelegationContext` analyzer-local rather than widening
   checkpoint schema. That preserves the already-frozen `R4` intent to own the next explicit
   checkpoint widening as `v0.5`.
3. Existing delegated-session evidence from prior screening is sufficient for a first bounded seam:
   `multi_agent_v1`, `spawn_agent`, `wait_agent`, and `close_agent` markers plus visible
   orchestration behavior should be enough to recognize delegated topology conservatively.
4. In real subagent/delegated runs, the child work may be recorded in a separate ordinary
   `rollout-*.jsonl` with its own session id, while the current first landing still reasons only
   over the visible parent rollout.
5. The `R2` success-tail acceptance corpus should remain non-subagent. Delegated coverage in
   `R3.75` should be a separate bounded regression wall, not a reopening of the acceptance corpus.
6. `R3.75` is descriptive and guardrail-oriented. It should classify visible delegation topology
   and child-work visibility, but it should not claim child intent, child progress, or
   delegated-session-specific drift semantics.
7. Sentinel replay/live compatibility should remain unchanged in this packet. If the first landing
   needs an operator-facing proof surface, it should come from analyzer-owned reporting rather than
   a new checkpoint field consumed by sentinel.
8. If current evidence cannot honestly distinguish `delegated_child` from
   `mixed_or_ambiguous`, the first landing should degrade conservatively instead of guessing.

If any of these assumptions are wrong, correct them before reusing this spec as the delegation
authority for `R4+`.

## Objective

Add one analyzer-owned delegation boundary so the hybrid-drift stack can tell whether a checkpoint
belongs to an ordinary single-agent session or to a visible delegated topology with limited child
visibility before `R4`, `R5`, and `R6` harden deeper semantic claims.

Primary users:

- the maintainer reading analyzer artifacts and needing an explicit statement that a session is
  parent-orchestration or delegation-ambiguous rather than ordinary single-agent work
- the later `R4`, `R5`, and `R6` packet author who needs a stable delegation guardrail before
  adding archetype, progress, or scorer semantics

Success means:

- analyzer checkpoint analysis derives a structured delegation boundary for each checkpoint
- the first landing keeps that boundary analyzer-local and does not widen checkpoint schema beyond
  the existing `v0.4`
- known delegated sessions are recognized conservatively as non-ordinary rather than silently
  treated as `single_agent`
- non-delegated sessions remain unchanged on the existing bounded non-subagent corpus
- child-opaque delegation lowers confidence or scope claims honestly instead of inventing
  child-visible meaning

## Packet Boundary

This packet is **delegation topology and child-visibility classification only**.

`R3.75-1` decision lock for the first landing:

- repo docs freeze `DelegationContext`, `DelegationTopology`, and `ChildWorkVisibility` as
  analyzer-local concepts for the first landing
- exported checkpoints stay on `schema_version = "v0.4"` throughout `R3.75`
- sentinel replay/live loaders and presentation stay unchanged throughout `R3.75`
- separate child rollout files / child session ids are an explicit opacity boundary in `R3.75`,
  not input that this packet is allowed to stitch or join
- bounded parent/child linkage and supported delegated-session semantics remain deferred to `R7`

In scope:

- define one analyzer-local `DelegationContext` concept for every checkpoint analysis
- define one conservative `DelegationTopology` with:
  - `single_agent`
  - `delegating_parent`
  - `delegated_child`
  - `mixed_or_ambiguous`
- define one conservative `ChildWorkVisibility` with:
  - `none`
  - `partial`
  - `opaque`
- derive topology, visibility, confidence, and evidence from existing rollout/context surfaces
- treat separate child rollout files / child session ids as the canonical reason that parent-only
  evidence may be `partial` or `opaque`
- keep delegated-session validation separate from the `R2` non-subagent acceptance corpus
- optionally surface compact analyzer-owned delegation inspection in summary/report artifacts if
  needed for operator proof

Out of scope:

- checkpoint schema widening beyond `v0.4`
- sentinel replay/live compatibility changes
- public replay/live operator presentation changes
- `R4` session-archetype classification
- `R5` progress semantics
- `R6` scorer cutover or delegated-session-specific drift classes
- `R7` supported parent/child semantic handling
- joining child transcripts into a multi-trajectory execution graph

`R3.75` interpretation rule for separate rollout files:

- if delegation markers indicate child work but the child trajectory lives in a separate ordinary
  rollout file / session id that is not joined into the current analyzer input, the first landing
  should classify the visible parent evidence conservatively as `partial` or `opaque`

## Tech Stack

- Language: Rust 2021
- Target crate:
  - `crates/agent-drift-analyzer`
- Existing analyzer checkpoint contract:
  - `v0.4`
- Checkpoint contract after `R3.75` first landing:
  - still `v0.4`
- Existing sentinel compatibility:
  - `v0.2 | v0.3 | v0.4`
- Required sentinel compatibility after `R3.75`:
  - unchanged, because `R3.75` should not widen the checkpoint contract

No new crate or external dependency is required by default.

## Commands

Formatting gate:

```bash
cargo fmt --all -- --check
```

Analyzer-focused validation:

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo test -p agent-drift-analyzer export_bundle -- --nocapture
cargo test -p agent-drift-analyzer acceptance_fixtures -- --nocapture
cargo test -p agent-drift-analyzer end_to_end -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
```

Optional focused wall while iterating on delegated cases:

```bash
cargo test -p agent-drift-analyzer checkpoints_are_deterministic_and_session_scoped -- --nocapture
cargo test -p agent-drift-analyzer acceptance_fixtures -- --nocapture
```

## Project Structure

```text
crates/agent-drift-analyzer/src/inference/mod.rs
  Existing analyzer-local inference seam. R3.75 can add delegation classification helpers here or
  split them into a narrow adjacent analyzer-local module.

crates/agent-drift-analyzer/src/checkpoint/mod.rs
  Owns checkpoint analysis assembly. R3.75 should attach analyzer-local delegation context here
  without widening the exported checkpoint DTO.

crates/agent-drift-analyzer/src/checkpoint/export.rs
  Owns `summary.md`. If R3.75 needs an operator-facing proof surface, it should render compact
  delegation inspection here while staying analyzer-owned.

crates/agent-drift-analyzer/tests/checkpoints.rs
  Deterministic checkpoint-analysis coverage. R3.75 should add topology/visibility assertions
  here.

crates/agent-drift-analyzer/tests/export_bundle.rs
  Summary/report rendering coverage. R3.75 should lock any compact delegation inspection here.

crates/agent-drift-analyzer/tests/acceptance_fixtures.rs
  Guards the existing bounded non-subagent corpus. R3.75 should prove those cases stay unchanged.

crates/agent-drift-analyzer/tests/support/mod.rs
  Shared fixture helpers. R3.75 can add bounded delegated fixture utilities here.

crates/agent-drift-analyzer/tests/fixtures/acceptance/README.md
  Documents why delegated sessions stay outside the `R2` success-tail corpus and can point to the
  separate `R3.75` delegated regression surface.
```

## Code Style

Planned Rust style for this packet:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DelegationTopology {
    SingleAgent,
    DelegatingParent,
    DelegatedChild,
    MixedOrAmbiguous,
}

fn classify_delegation_topology(markers: &[String]) -> DelegationTopology {
    if markers.iter().any(|marker| marker == "spawn_agent") {
        return DelegationTopology::DelegatingParent;
    }

    DelegationTopology::SingleAgent
}
```

Conventions:

- keep first-landing delegation state analyzer-local rather than serializing it into the checkpoint
  schema
- use conservative rule ordering; ambiguous evidence should lower confidence or choose
  `mixed_or_ambiguous`
- keep any operator-facing wording descriptive: topology, visibility, confidence, evidence
- keep delegated-session progress or drift judgments out of this packet entirely

## Testing Strategy

- Framework: existing Rust unit and integration tests via `cargo test`
- Analyzer checkpoint-analysis tests:
  - `crates/agent-drift-analyzer/tests/checkpoints.rs`
- Analyzer summary/report tests:
  - `crates/agent-drift-analyzer/tests/export_bundle.rs`
- Corpus-boundary tests:
  - `crates/agent-drift-analyzer/tests/acceptance_fixtures.rs`
- End-to-end artifact stability:
  - `crates/agent-drift-analyzer/tests/end_to_end.rs`

Coverage expectation for this packet:

- one ordinary single-agent case stays `single_agent` with `child_work_visibility = none`
- one known delegated-parent case proves conservative non-single-agent classification
- one second delegated case proves conservative non-single-agent or ambiguous classification
- the `R2` acceptance corpus remains unchanged and explicitly non-subagent

## Boundaries

- Always:
  - keep `R3.75` limited to analyzer-owned delegation topology and child-visibility classification
  - preserve checkpoint schema `v0.4` in the first landing
  - keep delegated coverage separate from the `R2` success-tail acceptance corpus
  - degrade ambiguous delegated cases conservatively instead of guessing child semantics
  - run the focused analyzer validation commands before claiming the packet landed
- Ask first:
  - exporting `DelegationContext` on checkpoints before `R4`
  - any sentinel replay/live compatibility or operator-surface changes
  - adding new upstream compactor schema solely for delegation detection
  - broadening the taxonomy or semantics into `R4`, `R5`, `R6`, or `R7`
- Never:
  - silently repurpose `R4`'s planned `v0.5` checkpoint widening
  - treat delegated-parent waiting/orchestration as child execution progress
  - fold delegated cases into the frozen `R2` success-tail corpus
  - let sentinel invent delegation heuristics in this packet

## Success Criteria

- Analyzer checkpoint analysis has a stable internal delegation boundary with topology, child-work
  visibility, confidence, and evidence.
- Checkpoint JSON artifacts remain on `schema_version = "v0.4"` after the first `R3.75` landing.
- Known delegated sessions `019e93f8-a5e9-7490-ac1a-955b74c92ad0` and
  `019e9406-6736-79a2-946b-8a603e557422` are recognized conservatively as non-ordinary rather than
  silently treated as `single_agent`.
- Existing bounded non-subagent acceptance cases remain unchanged in outcome and corpus boundary.
- Any analyzer-facing delegation reporting stays descriptive and confidence-bearing, not
  progress-bearing or scorer-bearing.

## Open Questions

- If the current evidence surface never cleanly proves `delegated_child`, is it acceptable for the
  first landing to leave that label effectively dormant until later delegated corpus work?
