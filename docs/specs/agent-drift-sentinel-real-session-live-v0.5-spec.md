# Spec: Agent Drift Sentinel Real Session Live v0.5

## Assumptions I'm Making

1. The fixture-backed live seam from Packet 16 is already landed and remains the correct library
   base for real-session work.
2. The immediate next real-time requirement is single-session live monitoring over a real active
   Codex rollout JSONL file, not multi-session aggregation or general host-runtime orchestration.
3. `agent-drift-sentinel` must continue to consume analyzer checkpoints as its semantic unit. It
   should not reimplement compactor normalization or analyzer scoring over raw transport rows.
4. The correct real-session path is:
   `rollout-*.jsonl -> agent-session-compactor -> agent-drift-analyzer -> agent-drift-sentinel live runtime`
   using existing library crates, not subprocess shell-outs.
5. A real-session proof should be run against an actually active session under `~/.codex/sessions`
   while the implementation agent is working, rather than against archived/static artifacts only.
6. This slice still stops short of wiring into `shell`, `world`, `shim`, or broader Substrate
   runtime hooks. The live source is the Codex session artifact, not a Substrate execution stream.

## Objective

Deliver honest real-time sentinel support for one active Codex session.

Primary user:

- the engineer running or supervising a live Codex session and wanting sentinel warnings to update
  as the session JSONL grows

Success means:

- the sentinel can target one live session by `session_id`
- it can discover and read the active `rollout-*.jsonl` artifact under a real `CODEX_HOME`
- it incrementally reruns the compactor/analyzer pipeline for that session and feeds newly observed
  checkpoints into the existing live runtime
- `--mode live` is enabled for this bounded real-session path
- the implementation is proven on an actual active session, not only on fixtures

## Packet Boundary

This packet is **real-session live support for one Codex session**.

In scope:

- active-session discovery under `~/.codex/sessions`
- session-scoped polling over append-only rollout JSONL
- library-owned compactor/analyzer invocation for one target session
- checkpoint delta detection and delivery into the existing live runtime
- bounded `--mode live` CLI enablement
- real active-session proof commands and documentation

Out of scope:

- shell/world integration
- multi-session dashboards or fan-in
- daemonization or background service management
- changing analyzer scoring logic
- changing scheduler semantics beyond what the existing live runtime already does

## Tech Stack

- Language: Rust 2021
- Coordinating crate: `crates/agent-drift-sentinel`
- Upstream library crates:
  - `crates/agent-session-compactor`
  - `crates/agent-drift-analyzer`
- Existing live runtime modules:
  - `src/live_input.rs`
  - `src/live_runtime.rs`
  - `src/operator_sink.rs`

Likely new sentinel-owned seams:

- live session source or coordinator module
- session-scoped pipeline state for:
  - target `codex_home`
  - target `session_id`
  - working output directory
  - last delivered checkpoint cursor or checkpoint id

## Commands

Targeted crate validation:

```bash
cargo test -p agent-drift-sentinel real_session_live -- --nocapture
cargo test -p agent-drift-sentinel live_runtime -- --nocapture
cargo test -p agent-drift-sentinel -- --nocapture
```

Pipeline validation:

```bash
cargo test -p agent-session-compactor -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
```

Bounded real-session proof shape:

```bash
export SESSION_ID="<active-session-id>"
export CODEX_HOME="${CODEX_HOME:-$HOME/.codex}"
export LIVE_STATE_DIR="target/hybrid-drift-live/$SESSION_ID"

cargo run -p agent-drift-sentinel -- \
  --mode live \
  --codex-home "$CODEX_HOME" \
  --session-id "$SESSION_ID" \
  --checkpoint-dir "$LIVE_STATE_DIR"
```

That proof is only valid if the targeted rollout file is actively growing while the sentinel is
running.

## Project Structure

```text
crates/agent-session-compactor/src/lib.rs
crates/agent-session-compactor/src/discovery.rs
crates/agent-session-compactor/src/ingest/codex_rollout.rs
  Existing session discovery and rollout ingestion for real Codex session files.

crates/agent-drift-analyzer/src/lib.rs
  Existing analyzer bundle execution over a compactor output directory.

crates/agent-drift-sentinel/src/lib.rs
  Enables bounded live mode once the real-session path exists.

crates/agent-drift-sentinel/src/cli.rs
  Adds the real-session live CLI surface.

crates/agent-drift-sentinel/src/live_runtime.rs
crates/agent-drift-sentinel/src/live_input.rs
crates/agent-drift-sentinel/src/operator_sink.rs
  Existing live runtime and event/sink seams reused by the real-session coordinator.

crates/agent-drift-sentinel/tests/
  Temp-`CODEX_HOME` integration tests plus bounded real-session proof guidance.

docs/specs/agent-drift-sentinel-real-session-live-v0.5-*.md
  Spec, plan, and tasks for this slice.
```

## Code Style

Keep real-session coordination library-first and session-scoped.

```rust
pub struct LiveSessionRequest {
    pub codex_home: Utf8PathBuf,
    pub session_id: String,
    pub state_dir: Utf8PathBuf,
}
```

Conventions:

- use library APIs for compactor and analyzer rather than shelling out to the binaries
- treat analyzer checkpoints as the only semantic input to the live runtime
- only emit newly observed checkpoints into the live runtime
- keep session polling append-only and deterministic
- preserve a bounded single-session workflow before considering broader orchestration

## Testing Strategy

Required test layers:

1. Session discovery and polling tests
   - target one session id under a temp `CODEX_HOME`
   - prove append-only rollout growth is observed deterministically
2. Pipeline coordination tests
   - rerun compactor/analyzer for the target session and detect newly emitted checkpoints only
3. Sentinel live regression tests
   - feed new checkpoints into the existing live runtime and sink behavior
4. Real active-session proof
   - run the bounded live command against an actually active session while the implementation agent
     is producing messages and tool calls

## Exact Success Criteria

This slice is complete only when all of the following are true:

1. `agent-drift-sentinel --mode live` can target a real active session by `session_id`.
2. The implementation uses the compactor/analyzer library crates to derive checkpoints from the
   live rollout artifact rather than reconstructing analyzer logic locally.
3. The sentinel emits only newly observed checkpoints into the live runtime as the session grows.
4. The bounded proof is run against a real active session under `~/.codex/sessions`, and the proof
   is documented with the exact `SESSION_ID` and command used.
5. The slice still avoids shell/world integration and broader host-runtime wiring.
