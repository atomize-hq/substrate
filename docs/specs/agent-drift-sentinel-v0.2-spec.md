# Spec: Agent Drift Sentinel v0.2

## Assumptions I'm Making

1. `agent-drift-sentinel` is a later-stage integration module and is not required for the first
   useful historical proof.
2. The sentinel should consume analyzer checkpoints and bounded live or replay event windows rather
   than redefining transcript parsing.
3. Model adjudication is optional and secondary to deterministic analyzer output.
4. The first live consumer is still the human operator, not an autonomous steering agent.
5. Live scheduling policies should be configurable and conservative to avoid operator fatigue.
6. The sentinel binary should stay thin so later live integration can embed library behavior inside
   broader Substrate flows.

## Objective

Define the later-stage live integration module that schedules evaluations, optionally adjudicates
ambiguous situations, and presents operator-facing drift warnings based on the analyzer contract.

Primary user:

- the operator watching a long or live run and wanting timely, evidence-backed warnings

Success means:

- the system can evaluate live or replayed runs on explicit triggers
- operator-facing warnings are grounded in analyzer checkpoints
- optional model adjudication sharpens ambiguous cases without becoming the sole explanation path

## Tech Stack

- Language: Rust 2021
- Product shape: library-first workspace crate plus thin binary or sidecar
- Crate name: `agent-drift-sentinel`
- Primary input contracts:
  - analyzer checkpoints with:
    - deterministic `schema_version`, `session_id`, and `checkpoint_id`
    - start/end `RowRef` boundaries
    - task-frame objective, evidence, and confidence
    - explicit `wrong_plan_branch`, `ignoring_repo_truth`, and `dead_end_thrash` scores
    - `expected_next_step` for operator warnings
  - bounded replay or live event windows
- Optional model posture:
  - `gpt-5.4-mini`
  - reasoning effort `medium`

Dependency posture:

- no need to own raw Codex parser seams
- no requirement to block or steer the main agent in early versions
- no requirement to integrate with the full Substrate shell/world stack in the first live cut

## Commands

Workspace validation:

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace -- --nocapture
```

Targeted crate commands:

```bash
cargo build -p agent-drift-sentinel
cargo test -p agent-drift-sentinel -- --nocapture
cargo run -p agent-drift-sentinel -- \
  --checkpoint-dir target/agent-drift-analyzer/session-<session-id> \
  --mode replay
```

Future live example:

```bash
cargo run -p agent-drift-sentinel -- \
  --mode live \
  --enable-model-adjudication \
  --model gpt-5.4-mini \
  --reasoning-effort medium
```

## Project Structure

```text
Cargo.toml
  Add the workspace member.

crates/agent-drift-sentinel/Cargo.toml
  Crate manifest.

crates/agent-drift-sentinel/src/main.rs
  Thin CLI entrypoint over library APIs.

crates/agent-drift-sentinel/src/lib.rs
  Public crate surface.

crates/agent-drift-sentinel/src/cli.rs
  Runtime options and mode selection.

crates/agent-drift-sentinel/src/input.rs
  Analyzer checkpoint loading and live-window ingestion.

crates/agent-drift-sentinel/src/scheduler.rs
  Trigger scheduling and debounce rules.

crates/agent-drift-sentinel/src/adjudication.rs
  Optional model-request shaping and fallback handling.

crates/agent-drift-sentinel/src/operator_surface.rs
  Warning summaries, logs, and local output surfaces.

crates/agent-drift-sentinel/tests/
  Replay-mode and scheduling coverage.

docs/specs/agent-drift-sentinel-v0.2-spec.md
  This spec.
```

## Code Style

Keep the sentinel thin. It should orchestrate evaluation timing and operator presentation, not
redefine analyzer semantics.

```rust
pub fn should_evaluate(trigger: Trigger, state: &SentinelState) -> bool {
    match trigger {
        Trigger::CheckpointReady => true,
        Trigger::RepeatedFailure => state.cooldown_allows_immediate_check(),
        Trigger::Heartbeat => state.heartbeat_due(),
    }
}
```

Conventions:

- scheduling policy explicit and configurable
- analyzer checkpoint semantics reused, not reinterpreted ad hoc
- model requests bounded and optional
- operator output concise and evidence-backed
- binary remains a thin wrapper around library-owned behavior

## Testing Strategy

Frameworks:

- unit tests beside implementation modules
- integration tests in `crates/agent-drift-sentinel/tests/`

Required test layers:

1. Scheduler tests
   - trigger handling is deterministic
   - debounce and cooldown rules prevent warning floods
2. Replay-mode tests
   - replayed analyzer checkpoints produce stable operator output
3. Adjudication tests
   - model path is off by default
   - request shaping pins the intended model and reasoning effort
   - adjudication failure degrades to analyzer-only output
4. Operator-surface tests
   - warning output includes checkpoint evidence and expected next step
   - non-flagged checkpoints do not overfire as warnings

## Boundaries

- Always:
  - treat analyzer checkpoints as the primary decision surface
  - keep model adjudication optional and bounded
  - keep operator messaging evidence-backed and concise
  - support replay mode before live mode is treated as complete
- Ask first:
  - automated steering or intervention
  - blocking the main agent based on sentinel output
  - deep runtime integration into shell/world flows
  - expanding model usage beyond bounded ambiguous cases
- Never:
  - make the model the sole explanation path
  - redefine transcript parsing inside the sentinel
  - require live runtime integration for the historical proof
  - treat every checkpoint as a visible warning

## Success Criteria

The spec is satisfied when:

1. The repo contains a standalone crate named `agent-drift-sentinel`.
2. The sentinel can run in replay mode over analyzer output.
3. The scheduler evaluates on explicit trigger classes rather than wall-clock spam alone.
4. Optional model adjudication is disabled by default and degrades safely.
5. Operator-facing warnings are grounded in analyzer checkpoints and evidence.

## Open Questions

1. What trigger mix is best for live runs without overwhelming the operator?
2. Which operator surfaces should ship first: local log, sidecar file, or interactive console view?
3. Should replay mode and live mode share one binary from the start or stay separate until the
   live contract stabilizes?
