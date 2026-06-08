# Tasks: Agent Drift Analyzer Delegation-Aware Boundary R3.75

This task list implements:

- `docs/specs/agent-drift-analyzer-delegation-aware-boundary-r3_75-spec.md`
- `docs/specs/agent-drift-analyzer-delegation-aware-boundary-r3_75-plan.md`

## Task List

Completion status on `2026-06-08`:

- The analyzer outcome-evidence `R1` family, `R2` acceptance-fixture hardening, `R3` turn
  context, and `R3.5` trigger-headline cleanup are landed on this worktree.
- `R3.75-1` repo-doc lock, `R3.75-2` analyzer-local delegation contract / marker harvesting, and
  `R3.75-3` checkpoint-local derivation plus analyzer-owned proof surface are landed on this
  worktree.
- `R3.75-4` bounded delegated regression coverage is also landed on this worktree, so `R3.75` is
  complete and `R4` is now the next packet family.
- `R7` remains distinct from this boundary.

Validation gate for this phase:

- this task list is now the landed `R3.75` packet ledger and history artifact
- `R4` may proceed without treating `R3.75` as an open blocker
- if scope or packet boundaries change, update the spec first, then the plan, then this task list

## Packet R3.75-1: Repo Doc Contract Lock

- [x] Task: Lock the `R3.75` packet boundary and first-landing contract in repo docs
  - Acceptance:
    - docs define `R3.75` as analyzer-owned delegation topology and child-visibility
      classification only
    - docs define analyzer-local `DelegationContext`, `DelegationTopology`, and
      `ChildWorkVisibility`
    - docs explicitly preserve checkpoint schema `v0.4` in the first landing
    - docs explicitly keep sentinel compatibility/presentation unchanged in this packet
    - docs explicitly treat separate child rollout files / child session ids as an opacity
      boundary in `R3.75`, not as permission to stitch parent and child trajectories here
    - docs explicitly keep `R4`, `R5`, `R6`, `R7`, and `R8` out of scope
    - docs explicitly keep delegated proof separate from the `R2` non-subagent acceptance corpus
  - Verify:
    - doc review against `HYBRID_DRIFT_REMAINING_GAPS_AND_LANDING_ORDER.md`
    - doc review against `docs/specs/hybrid-drift-sentinel-implementation-order.md`
    - doc review against `docs/specs/DESIGN-r4-delegation-aware-analyzer-boundary.md`
  - Files:
    - `docs/specs/agent-drift-analyzer-delegation-aware-boundary-r3_75-spec.md`
    - `docs/specs/agent-drift-analyzer-delegation-aware-boundary-r3_75-plan.md`
    - `docs/specs/agent-drift-analyzer-delegation-aware-boundary-r3_75-tasks.md`
    - `docs/specs/DESIGN-r4-delegation-aware-analyzer-boundary.md`
    - `docs/specs/hybrid-drift-sentinel-implementation-order.md`
    - `HYBRID_DRIFT_REMAINING_GAPS_AND_LANDING_ORDER.md`

Packet `R3.75-1` historical exit condition before `R3.75-2+` lands:

- repo docs lock `R3.75` as analyzer-local delegation topology and visibility only
- repo docs lock the first-landing decision to preserve `v0.4` rather than widening checkpoint
  schema
- repo docs lock separate child rollout files / child session ids as a conservative visibility
  boundary rather than a stitched semantic surface
- repo docs lock the separation between `R3.75` guardrails and `R7` full delegated-session
  semantics
- `R3.75-2`, `R3.75-3`, `R3.75-4`, `R4`, `R5`, `R6`, `R7`, and `R8` remain unstarted here

## Packet R3.75-2: Analyzer-Local Delegation Contract And Marker Harvesting

- [x] Task: Add analyzer-local delegation types and conservative marker harvesting
  - Acceptance:
    - analyzer-local code defines `DelegationContext`, `DelegationTopology`, and
      `ChildWorkVisibility` or an equivalent internal seam
    - checkpoint analysis carries delegation state internally without serializing it on exported
      checkpoint DTOs
    - marker harvesting reuses existing `multi_agent_v1`, `spawn_agent`, `wait_agent`, and
      `close_agent` evidence plus conservative context/command signals
    - no checkpoint `schema_version` bump occurs
  - Verify:
    - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
    - `cargo test -p agent-drift-analyzer -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/src/inference/mod.rs`
    - `crates/agent-drift-analyzer/src/checkpoint/mod.rs`
    - `crates/agent-drift-analyzer/src/context/mod.rs`

## Packet R3.75-3: Analyzer Derivation And Proof Surface

- [x] Task: Derive checkpoint-local delegation topology and child-work visibility
  - Acceptance:
    - ordinary non-delegated sessions classify as `single_agent` with `child_work_visibility =
      none`
    - known delegated parent-orchestration cases classify conservatively as non-single-agent
    - ambiguous delegated cases degrade to `mixed_or_ambiguous`, `partial`, or `opaque` instead of
      inventing child certainty
    - supporting and counter-evidence are populated deterministically
    - no `R4`, `R5`, `R6`, or `R7` semantics are introduced
  - Verify:
    - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/src/checkpoint/mod.rs`
    - `crates/agent-drift-analyzer/tests/checkpoints.rs`

- [x] Task: Surface compact analyzer-owned delegation inspection if needed for packet proof
  - Acceptance:
    - any new operator-facing artifact renders delegation topology, child visibility, confidence,
      and compact evidence references
    - the wording stays descriptive rather than progress-bearing or severity-bearing
    - output remains analyzer-owned and does not require sentinel changes
  - Verify:
    - `cargo test -p agent-drift-analyzer export_bundle -- --nocapture`
    - `cargo test -p agent-drift-analyzer end_to_end -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/src/checkpoint/export.rs`
    - `crates/agent-drift-analyzer/tests/export_bundle.rs`
    - `crates/agent-drift-analyzer/tests/end_to_end.rs`

## Packet R3.75-4: Bounded Delegated Regression Wall

- [x] Task: Add bounded delegated regression coverage without reopening the `R2` acceptance corpus
  - Acceptance:
    - one known delegated case,
      `019e93f8-a5e9-7490-ac1a-955b74c92ad0`, proves conservative non-single-agent classification
    - one second delegated case,
      `019e9406-6736-79a2-946b-8a603e557422`, proves conservative non-single-agent or
      `mixed_or_ambiguous` classification
    - the existing `R2` acceptance cases remain unchanged in outcome and corpus boundary
    - delegated sessions remain explicitly excluded from the non-subagent success-tail corpus
  - Verify:
    - `cargo test -p agent-drift-analyzer acceptance_fixtures -- --nocapture`
    - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
    - `cargo test -p agent-drift-analyzer end_to_end -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/tests/checkpoints.rs`
    - `crates/agent-drift-analyzer/tests/acceptance_fixtures.rs`
    - `crates/agent-drift-analyzer/tests/support/mod.rs`
    - `crates/agent-drift-analyzer/tests/fixtures/acceptance/README.md`

Packet `R3.75` exit condition:

- analyzer has a stable internal delegation boundary with topology, visibility, confidence, and
  evidence
- exported checkpoint JSON remains on `schema_version = "v0.4"`
- known delegated sessions are no longer silently treated as ordinary single-agent sessions
- the `R2` non-subagent acceptance corpus remains unchanged
- no sentinel compatibility, operator-surface, progress, scorer, or full delegated-session
  semantics land in this packet

Packet `R3.75` completion note on `2026-06-08`:

- the bounded delegated regression wall is landed
- the delegation-aware analyzer boundary is now complete and reusable by `R4`
- `R4` session archetype is now the next packet family
