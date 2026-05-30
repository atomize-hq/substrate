# Spec: Agent Drift Sentinel Live Integration v0.3

## Assumptions I'm Making

1. `S10` remains a hard implementation gate; this slice approves only the next bounded live-design
   target, not broader runtime wiring in this session.
2. The replay-mode contract in
   [agent-drift-sentinel-v0.2-spec.md](/Users/spensermcconnell/.codex/worktrees/97a0/substrate/docs/specs/agent-drift-sentinel-v0.2-spec.md:1)
   stays fixed unless a concrete defect is found.
3. `agent-drift-analyzer` continues to own checkpoint generation, drift scoring, and task-frame
   inference; live sentinel work must consume those outputs rather than recreate them.
4. The first live proof should be library-first and fixture-driven inside `agent-drift-sentinel`,
   not coupled to `shell`, `world`, `shim`, or any host-orchestrator runtime path.
5. The first operator surface for live mode is still local and reviewable: console text, JSONL
   event sink, or similar thin outputs, not a full interactive UI.
6. If the existing analyzer checkpoint contract is insufficient for incremental live consumption,
   the correct response is to stop and plan the analyzer seam explicitly rather than hide new logic
   inside the sentinel.

## Objective

Define the next bounded live-integration slice after `S10`.

In this slice, "live integration" means:

- the sentinel can consume incremental analyzer checkpoint events as they arrive
- the sentinel can evaluate those checkpoints on explicit live triggers instead of replaying a
  closed bundle
- the sentinel can emit operator-facing live warning events through a thin sink surface
- the replay path remains intact and is reused where possible

This slice does **not** mean:

- wiring the sentinel into `substrate` shell/world execution
- re-owning analyzer logic
- attaching to raw Codex transcripts directly
- shipping autonomous steering, blocking, or approval behavior

Primary user:

- the operator watching a long-running agent session and wanting bounded, evidence-backed warnings

Success means:

- the sentinel has a defined library seam for live checkpoint intake and live warning emission
- live triggering reuses the existing scheduler and warning policy rather than inventing a second
  scoring system
- a bounded live proof can be verified with synthetic or file-backed checkpoint streams
- broader runtime integration stays explicitly deferred after this slice

## Tech Stack

- Language: Rust 2021
- Product shape: library-first workspace crate plus thin binary-friendly surfaces
- Crate: `agent-drift-sentinel`
- Upstream dependency: `agent-drift-analyzer::Checkpoint`
- Existing reusable modules:
  - `src/scheduler.rs`
  - `src/operator_surface.rs`
  - `src/adjudication.rs`
- Proposed new library seams:
  - `src/live_input.rs`
    - append-only checkpoint-event loading or adapter traits
  - `src/live_runtime.rs`
    - live event loop over scheduler and operator sink
  - `src/operator_sink.rs`
    - structured live warning/event emission surface

Proposed live-mode input contract:

- `CheckpointReady`
  - carries one analyzer `Checkpoint`
  - includes a stable `CheckpointCursor`
  - may include source metadata such as bundle path or emission ordinal
- `Heartbeat`
  - synthetic trigger when no visible warning has fired within the configured interval
- `RepeatedFailure`
  - synthetic trigger when consecutive flagged checkpoints cross the existing scheduler threshold
- `ManualReview`
  - operator- or harness-forced evaluation trigger

Critical rule:

- live mode consumes analyzer checkpoints and trigger events; it does not reconstruct checkpoints
  from raw transcript data

## Commands

Workspace validation:

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace -- --nocapture
```

Targeted crate validation for this slice:

```bash
cargo build -p agent-drift-sentinel
cargo test -p agent-drift-sentinel live_input -- --nocapture
cargo test -p agent-drift-sentinel live_runtime -- --nocapture
cargo test -p agent-drift-sentinel operator_sink -- --nocapture
cargo test -p agent-drift-sentinel live_end_to_end -- --nocapture
```

Bounded future proof command for the thin binary, if the slice elects to expose one:

```bash
cargo run -p agent-drift-sentinel -- \
  --mode live \
  --checkpoint-stream fixtures/live/checkpoints.jsonl \
  --event-stream fixtures/live/events.jsonl
```

That command is a bounded sentinel-local proof only. It is not shell/world integration.

## Project Structure

```text
crates/agent-drift-sentinel/src/lib.rs
  Keep the replay entrypoint stable and expose the new live runtime seam.

crates/agent-drift-sentinel/src/cli.rs
  Stay thin; only grow enough surface to drive a bounded sentinel-local live proof if needed.

crates/agent-drift-sentinel/src/input.rs
  Preserve closed-bundle replay loading without folding live concerns into it.

crates/agent-drift-sentinel/src/live_input.rs
  New incremental checkpoint event contract and adapter-loading helpers.

crates/agent-drift-sentinel/src/live_runtime.rs
  New library-owned live loop that consumes checkpoint events and emits operator events.

crates/agent-drift-sentinel/src/scheduler.rs
  Reused trigger/cooldown logic shared by replay and live flows.

crates/agent-drift-sentinel/src/operator_surface.rs
  Reused checkpoint presentation and warning policy logic.

crates/agent-drift-sentinel/src/operator_sink.rs
  New sink trait or structured output module for live warning delivery.

crates/agent-drift-sentinel/tests/
  Fixture-driven live input, runtime, and sink tests.

docs/specs/agent-drift-sentinel-live-integration-v0.3-*.md
  Spec, plan, and task chain for this bounded slice.
```

## Code Style

Keep the new live path library-owned and adapter-driven.

```rust
pub trait LiveCheckpointSource {
    fn next_event(&mut self) -> anyhow::Result<Option<LiveCheckpointEvent>>;
}

pub fn handle_live_event(
    runtime: &mut LiveRuntime,
    event: LiveCheckpointEvent,
) -> anyhow::Result<Vec<OperatorEvent>> {
    runtime.observe(event)
}
```

Conventions:

- live mode reuses `SchedulerPolicy`, `WarningPolicy`, and checkpoint presentation logic
- replay-only loading stays in `input.rs`
- live input surfaces are incremental and append-only
- operator delivery is explicit through a sink interface, not hidden in `println!` calls
- binary code remains a wrapper around library-owned live and replay behavior

## Testing Strategy

Frameworks:

- unit tests beside live input/runtime/sink modules
- integration tests in `crates/agent-drift-sentinel/tests/`

Required test layers:

1. Live input contract tests
   - incremental checkpoint events load in stable order
   - duplicate or out-of-order cursors are rejected or surfaced explicitly
2. Shared scheduler tests
   - replay and live paths use the same trigger semantics
   - heartbeat and repeated-failure behavior stay deterministic
3. Operator sink tests
   - visible warnings, silent checkpoints, and heartbeat/status events render or serialize
     consistently
4. Live runtime end-to-end tests
   - a fixture stream of checkpoint-ready and heartbeat events produces the expected warning stream
   - adjudication remains optional and bounded in live mode just as it is in replay mode
5. Regression tests for boundary preservation
   - replay mode still works unchanged
   - no test requires shell/world integration

## Boundaries

- Always:
  - consume analyzer checkpoints rather than raw transcript rows
  - keep replay mode behavior stable while adding live seams
  - reuse scheduler and warning-policy logic across replay and live paths
  - verify live behavior with fixture-driven streams before considering runtime wiring
  - keep binary changes thin and subordinate to library APIs
- Ask first:
  - any change to the analyzer checkpoint contract
  - any public CLI surface that implies shell/world or host-orchestrator integration
  - persistent background processes, daemons, or thread ownership
  - operator actions that block, kill, or steer the main agent
- Never:
  - re-score drift classes inside the sentinel
  - silently widen into `crates/shell`, `crates/world*`, `crates/broker`, or shim wiring
  - treat file-backed fixture streaming as equivalent to full runtime integration
  - mark the live runtime as production-integrated after this slice

## Success Criteria

This slice is complete when:

1. `agent-drift-sentinel` has a documented live input contract that accepts incremental analyzer
   checkpoints plus explicit trigger events.
2. The library exposes a live runtime seam that reuses the existing scheduler and operator
   presentation logic instead of forking them.
3. A bounded sentinel-local live proof can run against synthetic or file-backed checkpoint streams
   with deterministic output.
4. The replay path remains valid and unchanged from the operator's perspective.
5. The slice ends with an explicit gate that keeps shell/world and broader runtime integration out
   of scope pending a later approval.

## Open Questions

1. Should the first live proof read append-only JSONL fixtures directly, or should it require an
   adapter trait plus a test harness that feeds in-memory events?
2. Does the operator need structured JSONL sink output in the first live slice, or is console plus
   snapshot-tested text enough?
3. Is a bounded `--mode live` fixture driver worth exposing in the binary, or should the first
   live proof stay library-only until the host integration contract is approved?
