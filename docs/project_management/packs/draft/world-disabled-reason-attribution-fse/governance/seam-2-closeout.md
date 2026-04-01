---
seam_id: SEAM-2
status: landed
closeout_version: v1
seam_exit_gate:
  source_ref: ../threaded-seams/seam-2-replay-attribution-runtime-surfaces/slice-4-seam-exit-gate.md
  status: passed
  promotion_readiness: ready
basis:
  currentness: current
  upstream_closeouts:
    - governance/seam-1-closeout.md
  required_threads:
    - THR-01
    - THR-02
    - THR-03
    - THR-04
  stale_triggers:
    - revalidate SEAM-3 if replay fragments or recorded-host punctuation differ from plan
    - revalidate SEAM-3 if telemetry field names, enum values, or omission rules differ from plan
gates:
  post_exec:
    landing: passed
    closeout: passed
open_remediations: []
---

# Closeout - SEAM-2 Replay attribution runtime surfaces

This is a post-exec scaffold. The authoritative current state before execution remains in `seam-2-replay-attribution-runtime-surfaces.md`.

## Seam-exit gate record

- **Source artifact**: `../threaded-seams/seam-2-replay-attribution-runtime-surfaces/slice-4-seam-exit-gate.md`
- **Landed evidence**:
  - S1 contract-definition landing: `fa89eb18` tightened `C-03` / `C-04` in `threaded-seams/seam-2-replay-attribution-runtime-surfaces/slice-1-contract-definition-replay-attribution-runtime-surfaces.md`, including the replay-runtime layer/code set, the `source_unknown` -> `unknown` mapping, and the explicit `default` blocker rule.
  - S2 replay-copy landing: `4c28c166` updated `crates/shell/src/execution/routing/replay.rs` and `crates/shell/tests/replay_world.rs` so recorded-host replay emits `host (recorded; <reason>)`, effective-disable host warnings reuse the published reason fragments, and replay-local opt-out copy remains unchanged.
  - S3 telemetry landing: `05ca9bd6` updated `crates/replay/src/replay/mod.rs`, `crates/replay/src/replay/executor.rs`, `crates/replay/src/state.rs`, `crates/replay/tests/planner_executor.rs`, `crates/shell/src/execution/routing/replay.rs`, and `crates/shell/tests/replay_world.rs` so `replay_strategy` emits additive `world_disabled_*` reason codes plus tokenized `world_disable_source`, while replay-local opt-out cases omit that object.
  - Passing verification commands:
    - `cargo fmt --all -- --check`
    - `cargo test -p substrate-replay --lib -- --nocapture`
    - `cargo test -p substrate-replay --test planner_executor -- --nocapture`
    - `cargo test -p shell --lib recorded_host_summary_uses_effective_disable_source_unknown_reason -- --nocapture`
    - `cargo test -p shell --test replay_world -- --nocapture`
- **Contracts published or changed**:
  - `C-03`
  - `C-04`
- **Threads published / advanced**:
  - `THR-03 published`
  - `THR-04 published`
- **Review-surface delta**: none material; the landed replay stderr and `replay_strategy` surfaces match the review bundle's copy, telemetry, and omission-risk focus areas.
- **Planned-vs-landed delta**: none material; the landed implementation preserves replay-local opt-out fragments, uses the exact recorded-host punctuation, and publishes the additive telemetry contract without widening the enum or object-key surface.
- **Downstream stale triggers raised**:
  - revalidate `SEAM-3` if runtime fragments, telemetry fields, or omission rules differ from the source contract
- **Remediation disposition**: none
- **Promotion blockers**: none
- **Promotion readiness**: ready

## Post-exec gate disposition

- **Landing gate**: passed
- **Closeout gate**: passed
- **Unresolved remediations**: none
- **Carried-forward remediations**: none
