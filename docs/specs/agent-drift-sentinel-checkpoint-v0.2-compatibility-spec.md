# Spec: Agent Drift Sentinel Checkpoint v0.2 Compatibility

## Assumptions I'm Making

1. `agent-session-compactor` and `agent-drift-analyzer` have already completed the upstream
   contract cutover, so this packet is about restoring the downstream sentinel seam rather than
   widening analyzer or compactor again.
2. This packet should be a direct current-schema cutover to analyzer checkpoint `schema_version =
   "v0.2"`, not a long-lived mixed `v0.1`/`v0.2` bridge, unless implementation proves a
   repo-local current test path still requires both.
3. `diagnostics` is now part of the required `agent_drift_analyzer::Checkpoint` shape, but the
   current sentinel scheduler and operator surfaces still make decisions from:
   - `checkpoint.flagged`
   - `checkpoint.drift_scores`
   - `checkpoint.expected_next_step`
   - `warning_fingerprint(checkpoint)`
   so this packet should restore compatibility first and defer new behavior that consumes
   `diagnostics`.
4. Live mode remains bounded and library-first. Updating `live_input.rs`, `live_runtime.rs`, and
   live fixtures for current-schema compatibility does not authorize broader shell/world wiring or
   removing the existing live-mode CLI gate.
5. `SC1` is already landed before this packet, so replay input and bounded live compatibility
   already accept analyzer checkpoint `schema_version = "v0.2"`; `SC2` is limited to canonical
   test-support parity, fixture parity, full regression proof, and continuity-doc refresh.
6. Continuity docs and task checklists should describe the restored downstream seam after the
   green proof and continue to defer first sentinel behavior that consumes `diagnostics`.

## Objective

Restore `agent-drift-sentinel` as a current-schema consumer of analyzer checkpoints after analyzer
`v0.2` landed.

Primary user:

- the engineer or operator who wants replay and bounded live sentinel paths to keep working against
  the current analyzer checkpoint contract

Success means:

- replay input accepts analyzer checkpoint `schema_version = "v0.2"`
- live compatibility checks accept analyzer checkpoint `schema_version = "v0.2"`
- sentinel test support and checked-in live fixtures include `CheckpointDiagnostics`
- sentinel behavior stays compatibility-only in this packet: `diagnostics` is carried through the
  shared `Checkpoint` type but is not yet used to change scheduler, warning, or operator behavior
- stale docs and task checklists can be refreshed once the downstream seam is green again

## Packet Boundary

This packet is **compatibility-only**.

In scope:

- replay input compatibility with analyzer checkpoint `v0.2`
- live input compatibility with analyzer checkpoint `v0.2`
- sentinel test support updates for `CheckpointDiagnostics`
- regeneration or rewrite of sentinel live fixtures so they encode `diagnostics`
- continuity-doc refresh after the downstream seam is restored

Out of scope:

- new scheduler behavior derived from `diagnostics`
- new operator-surface rendering derived from `diagnostics`
- warning-fingerprint or debounce-policy changes
- analyzer contract widening beyond the landed `v0.2` checkpoint shape
- broader live-mode/runtime integration
- a permanent mixed `v0.1`/`v0.2` compatibility layer unless implementation proves it is required

## Tech Stack

- Language: Rust 2021
- Upstream checkpoint type: `agent_drift_analyzer::Checkpoint`
- Target crate: `crates/agent-drift-sentinel`
- Relevant analyzer contract source:
  - `crates/agent-drift-analyzer/src/checkpoint/schema.rs`
  - `crates/agent-drift-analyzer/src/checkpoint/mod.rs`
  - `crates/agent-drift-analyzer/src/checkpoint/export.rs`
- Sentinel compatibility surfaces:
  - `crates/agent-drift-sentinel/src/input.rs`
  - `crates/agent-drift-sentinel/src/live_input.rs`
  - `crates/agent-drift-sentinel/tests/support/mod.rs`
  - `crates/agent-drift-sentinel/tests/fixtures/live/*.jsonl`

Dependency posture:

- no new crate dependency is required
- no new analyzer artifact is required
- no new sentinel behavior contract is required
- the shared checkpoint type already carries `CheckpointDiagnostics`; sentinel only needs to stop
  rejecting or omitting it

## Commands

Upstream confirmation:

```bash
cargo test -p agent-drift-analyzer -- --nocapture
```

Restored downstream proof wall:

```bash
cargo test -p agent-drift-sentinel -- --nocapture
```

Targeted sentinel verification for this packet:

```bash
cargo test -p agent-drift-sentinel replay_input -- --nocapture
cargo test -p agent-drift-sentinel live_input -- --nocapture
cargo test -p agent-drift-sentinel live_input_adapter -- --nocapture
cargo test -p agent-drift-sentinel live_checkpoint_compatibility -- --nocapture
cargo test -p agent-drift-sentinel live_runtime -- --nocapture
cargo test -p agent-drift-sentinel live_end_to_end -- --nocapture
cargo test -p agent-drift-sentinel -- --nocapture
```

Fixture inspection:

```bash
sed -n '1,120p' crates/agent-drift-sentinel/tests/fixtures/live/append_only_stream.jsonl
sed -n '1,120p' crates/agent-drift-sentinel/tests/fixtures/live/cursor_regression_stream.jsonl
```

## Project Structure

```text
crates/agent-drift-analyzer/src/checkpoint/schema.rs
  Defines `Checkpoint` and `CheckpointDiagnostics`, including `schema_version = "v0.2"`.

crates/agent-drift-analyzer/src/checkpoint/mod.rs
  Builds checkpoint rows and sets the exported schema version.

crates/agent-drift-analyzer/src/checkpoint/export.rs
  Writes `checkpoints.jsonl` and `summary.md` in the current analyzer shape.

crates/agent-drift-sentinel/src/input.rs
  Replay bundle loading, sorting, schema-version validation, and replay cursor handling.

crates/agent-drift-sentinel/src/live_input.rs
  Live fixture loading, live checkpoint compatibility validation, and append-only event checks.

crates/agent-drift-sentinel/src/live_runtime.rs
  Reuses compatibility-checked checkpoints in the bounded live runtime.

crates/agent-drift-sentinel/src/operator_surface.rs
crates/agent-drift-sentinel/src/scheduler.rs
  Existing behavior surfaces that remain intentionally unchanged in this packet.

crates/agent-drift-sentinel/tests/support/mod.rs
  Canonical test-only checkpoint builder that must move to current schema.

crates/agent-drift-sentinel/tests/fixtures/live/append_only_stream.jsonl
crates/agent-drift-sentinel/tests/fixtures/live/cursor_regression_stream.jsonl
  Checked-in live fixture streams that must encode `diagnostics`.

docs/specs/agent-drift-sentinel-checkpoint-v0.2-compatibility-*.md
  Spec, plan, and tasks for this compatibility-restoration packet.
```

## Code Style

Treat current-schema compatibility as an explicit contract gate, not as an implicit side effect of
deserialization.

```rust
const SUPPORTED_ANALYZER_CHECKPOINT_SCHEMA: &str = "v0.2";

if checkpoint.schema_version != SUPPORTED_ANALYZER_CHECKPOINT_SCHEMA {
    return Err(compatibility_gap(
        checkpoint,
        "schema_version",
        format!(
            "expected {} but found {}",
            SUPPORTED_ANALYZER_CHECKPOINT_SCHEMA,
            checkpoint.schema_version
        ),
    ));
}
```

Conventions:

- keep schema-version checks explicit and identical across replay and live surfaces where possible
- prefer direct current-schema cutover over compatibility scaffolding
- treat `diagnostics` as required data that must deserialize and round-trip cleanly
- do not add new warning or scheduling semantics in this packet
- keep fixture payloads minimal but structurally honest to the live checkpoint type

## Testing Strategy

Frameworks:

- unit tests beside replay/live compatibility modules
- integration tests in `crates/agent-drift-sentinel/tests/`

Required test layers:

1. Replay input contract tests
   - replay bundle loads and sorts current-schema checkpoints
   - bundle metadata reports `schema_version = "v0.2"`
   - unsupported or mixed schema versions still fail clearly
2. Live compatibility tests
   - live checkpoint compatibility accepts current-schema checkpoints with `diagnostics`
   - cursor, objective, and expected-next-step validation stays intact
3. Fixture-backed live stream tests
   - append-only fixture stream still validates and drains in order
   - cursor-regression fixture still fails for ordering reasons, not schema drift
4. Full sentinel regression
   - the full crate test suite passes once the downstream seam is restored
5. Continuity checks
   - docs/task checklists are refreshed only after the green proof, so repo guidance no longer
     describes sentinel as lagging the analyzer checkpoint contract

## Boundaries

- Always:
  - cut sentinel to the current analyzer checkpoint schema
  - keep replay and bounded live compatibility aligned on the same checkpoint version
  - update test support and checked-in fixtures together
  - verify with targeted sentinel suites and a full sentinel test run
- Ask first:
  - any attempt to keep both `v0.1` and `v0.2` supported long-term
  - any new operator summary or scheduler behavior based on `diagnostics`
  - any change that widens analyzer output again
  - any removal of the live-mode CLI gate
- Never:
  - silently ignore missing `diagnostics`
  - treat fixture-only edits as sufficient proof without running sentinel tests
  - broaden this packet into live runtime integration or autonomous behavior changes

## Downstream Seam Restored: Exact Success Criteria

The downstream seam is restored only when all of the following are true:

1. `crates/agent-drift-sentinel/src/input.rs` accepts replay bundles whose checkpoint rows all use
   `schema_version = "v0.2"` and reports that version on the loaded bundle.
2. `crates/agent-drift-sentinel/src/live_input.rs` accepts live checkpoint payloads whose
   checkpoints use `schema_version = "v0.2"` and include `diagnostics`.
3. `crates/agent-drift-sentinel/tests/support/mod.rs` builds `Checkpoint` values with an explicit
   `CheckpointDiagnostics` payload and `schema_version = "v0.2"`.
4. Both checked-in live fixtures include `diagnostics` on every `checkpoint_ready` payload while
   preserving the ordering and cursor behavior their tests assert.
5. `cargo test -p agent-drift-sentinel -- --nocapture` passes.
6. Scheduler, warning, operator-surface, and adjudication behavior remain compatibility-only in
   this packet: no new user-visible logic depends on `diagnostics`.
7. Stale continuity docs or packet checklists can be updated to say the sentinel downstream seam is
   restored against the analyzer checkpoint contract.

## Open Questions

1. Which sentinel surface should consume `diagnostics` first in a later packet: scheduler policy,
   warning fingerprinting, or operator presentation?
2. Does the repo want a single follow-up packet for first `diagnostics` consumption, or separate
   replay-surface and live-surface follow-ups after this compatibility cutover lands?
