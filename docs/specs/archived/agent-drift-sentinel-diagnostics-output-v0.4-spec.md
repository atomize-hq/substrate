# Spec: Agent Drift Sentinel Diagnostics Output v0.4

## Assumptions I'm Making

1. `agent-drift-analyzer` already exports compact checkpoint diagnostics in the shared
   `Checkpoint.diagnostics` payload and the analyzer-side summary/reporting slice is no longer the
   blocking seam.
2. The downstream sentinel compatibility repair is already complete: replay input, bounded live
   input, test support, and checked-in fixtures all accept analyzer checkpoint `schema_version =
   "v0.2"` with `diagnostics`.
3. This slice should remain presentation-only. The sentinel scheduler must keep its current
   cooldown, heartbeat, repeated-failure, and warning-debounce semantics unless a later packet
   explicitly changes them.
4. The immediate product gap is operator legibility, not missing analyzer signal. Today the
   sentinel carries `diagnostics` through the checkpoint type but does not expose it clearly in the
   replay report or live sink outputs.
5. Replay and live mode should share one diagnostics-presentation contract so the same checkpoint
   explains itself consistently in console output and structured live events.

## Objective

Extend `agent-drift-sentinel` so checkpoint diagnostics are visible and structured in downstream
sentinel outputs.

Primary user:

- the operator or engineer trying to understand why a warning fired, why a checkpoint stayed
  silent, or how the task frame changed between incremental checkpoints

Success means:

- replay output surfaces compact, human-readable checkpoint diagnostics
- live sink events carry a machine-readable diagnostics summary alongside the existing presentation
- no scheduler behavior changes in this slice
- replay and live outputs stay aligned on one stable diagnostics contract

## Packet Boundary

This packet is **diagnostics-output only**.

In scope:

- replay warning/checkpoint presentation updates
- live sink event payload updates
- compact diagnostics formatting rules
- regression coverage that proves the same checkpoint yields aligned replay/live diagnostics output

Out of scope:

- changing scheduler thresholds, cooldowns, or repeated-failure logic
- changing warning fingerprint semantics
- changing analyzer checkpoint generation
- enabling full real-session live mode over active Codex session files
- shell/world or broader Substrate runtime integration

## Tech Stack

- Language: Rust 2021
- Target crate: `crates/agent-drift-sentinel`
- Upstream shared type: `agent_drift_analyzer::Checkpoint`
- Diagnostics source fields:
  - `checkpoint.diagnostics.task_frame_transitioned`
  - `checkpoint.diagnostics.working_set_changed`
  - `checkpoint.diagnostics.interval_command_count`
  - `checkpoint.diagnostics.interval_verification_command_count`
  - `checkpoint.diagnostics.evidence_item_count`

Relevant sentinel surfaces:

- `crates/agent-drift-sentinel/src/operator_surface.rs`
- `crates/agent-drift-sentinel/src/operator_sink.rs`
- `crates/agent-drift-sentinel/src/live_runtime.rs`
- `crates/agent-drift-sentinel/tests/`

## Commands

Primary validation:

```bash
cargo test -p agent-drift-sentinel operator_surface -- --nocapture
cargo test -p agent-drift-sentinel operator_sink -- --nocapture
cargo test -p agent-drift-sentinel live_end_to_end -- --nocapture
cargo test -p agent-drift-sentinel -- --nocapture
```

Upstream confirmation:

```bash
cargo test -p agent-drift-analyzer export_bundle -- --nocapture
```

Bounded replay proof:

```bash
cargo run -p agent-drift-sentinel -- \
  --checkpoint-dir "$ANALYZER_OUT" \
  --cursor-session-id "$SESSION_ID" \
  --cursor-ordinal 1
```

## Project Structure

```text
crates/agent-drift-sentinel/src/operator_surface.rs
  Replay-facing checkpoint presentation and console report rendering.

crates/agent-drift-sentinel/src/operator_sink.rs
  Structured live operator events that should expose the same diagnostics contract.

crates/agent-drift-sentinel/src/live_runtime.rs
  Reuses the shared presentation on incremental live events.

crates/agent-drift-sentinel/tests/operator_surface.rs
crates/agent-drift-sentinel/tests/operator_sink.rs
crates/agent-drift-sentinel/tests/live_end_to_end.rs
  Regression coverage for diagnostics output across replay and live paths.

docs/specs/agent-drift-sentinel-diagnostics-output-v0.4-*.md
  Spec, plan, and tasks for this slice.
```

## Code Style

Keep diagnostics compact, explicit, and shared.

```rust
pub struct CheckpointDiagnosticsSummary {
    pub task_frame_transitioned: bool,
    pub working_set_changed: bool,
    pub interval_command_count: usize,
    pub interval_verification_command_count: usize,
    pub evidence_item_count: usize,
}
```

Conventions:

- use one shared diagnostics summary shape for replay and live outputs
- compute verification density from the existing interval counts instead of re-deriving it from
  unrelated fields
- render zero-denominator verification density as `unavailable`, not fake `0.00`
- keep the operator output concise; diagnostics should explain the checkpoint, not drown it
- preserve current warning/silent classification rules exactly

## Testing Strategy

Required test layers:

1. Replay presentation tests
   - visible warnings render the new diagnostics line
   - silent checkpoints render the new diagnostics line when applicable
   - diagnostics formatting stays stable for zero-command checkpoints
2. Live sink tests
   - visible warning events include structured diagnostics payloads
   - silent/status/heartbeat events can reference the same checkpoint diagnostics summary
3. Shared alignment tests
   - the same checkpoint fixture yields matching replay/live diagnostics facts
4. Full sentinel regression
   - the crate stays green with no scheduler behavior drift

## Exact Success Criteria

This slice is complete only when all of the following are true:

1. Replay output in `operator_surface.rs` shows a compact diagnostics summary for surfaced
   checkpoints.
2. Live operator events in `operator_sink.rs` carry the same diagnostics facts in structured form.
3. The slice does not change `SchedulerPolicy`, warning debounce, or replay/live trigger behavior.
4. Sentinel regression coverage proves replay and live outputs stay aligned on the diagnostics
   contract.
