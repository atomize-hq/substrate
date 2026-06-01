# Tasks: Agent Drift Sentinel Diagnostics Output v0.4

This task list implements:

- `docs/specs/agent-drift-sentinel-diagnostics-output-v0.4-spec.md`
- `docs/specs/agent-drift-sentinel-diagnostics-output-v0.4-plan.md`

## Task List

- [ ] Task: Lock the shared diagnostics-output contract for replay and live mode
  - Acceptance: the repo docs define one compact diagnostics summary that includes:
    - `task_frame_transitioned`
    - `working_set_changed`
    - interval verification density and raw counts
    - `evidence_item_count`
    and it explicitly states that this slice does not change scheduler behavior.
  - Verify: doc review against the live analyzer checkpoint schema and current sentinel output code
  - Files:
    - `docs/specs/agent-drift-sentinel-diagnostics-output-v0.4-spec.md`
    - `docs/specs/agent-drift-sentinel-diagnostics-output-v0.4-plan.md`
    - `docs/specs/agent-drift-sentinel-diagnostics-output-v0.4-tasks.md`

- [ ] Task: Add compact diagnostics rendering to replay checkpoint presentation
  - Acceptance: replay-visible and replay-silent checkpoint output surfaces a concise diagnostics
    summary without changing visible-warning selection or ordering.
  - Verify: `cargo test -p agent-drift-sentinel operator_surface -- --nocapture`
  - Files:
    - `crates/agent-drift-sentinel/src/operator_surface.rs`
    - `crates/agent-drift-sentinel/tests/operator_surface.rs`

- [ ] Task: Carry the same diagnostics summary through live operator events
  - Acceptance: structured live events expose the same diagnostics facts through the sink payloads,
    preserving the existing visible/silent/status event split.
  - Verify: `cargo test -p agent-drift-sentinel operator_sink -- --nocapture`
  - Files:
    - `crates/agent-drift-sentinel/src/operator_sink.rs`
    - `crates/agent-drift-sentinel/tests/operator_sink.rs`

- [ ] Task: Prove replay and live diagnostics output stay aligned
  - Acceptance: shared checkpoint fixtures prove replay and live outputs report the same
    task-frame, working-set, verification, and evidence-count facts.
  - Verify: `cargo test -p agent-drift-sentinel live_end_to_end -- --nocapture`
  - Files:
    - `crates/agent-drift-sentinel/tests/live_end_to_end.rs`
    - `crates/agent-drift-sentinel/tests/fixtures/live/`

- [ ] Task: Re-run the full sentinel regression wall without scheduler drift
  - Acceptance: the full crate passes and no test proves a change in cooldown, debounce,
    repeated-failure, or heartbeat semantics.
  - Verify: `cargo test -p agent-drift-sentinel -- --nocapture`
  - Files:
    - `crates/agent-drift-sentinel/src/operator_surface.rs`
    - `crates/agent-drift-sentinel/src/operator_sink.rs`
    - `crates/agent-drift-sentinel/tests/`

Packet exit condition:

- replay and live outputs expose one shared diagnostics contract
- the output explains checkpoint cadence/context better without changing scheduler behavior
- the full sentinel crate remains green
