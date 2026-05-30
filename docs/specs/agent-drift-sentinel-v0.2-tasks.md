# Tasks: Agent Drift Sentinel v0.2

This task list implements:

- [agent-drift-sentinel-v0.2-spec.md](/Users/spensermcconnell/.codex/worktrees/97a0/substrate/docs/specs/agent-drift-sentinel-v0.2-spec.md:1)
- [agent-drift-sentinel-v0.2-plan.md](/Users/spensermcconnell/.codex/worktrees/97a0/substrate/docs/specs/agent-drift-sentinel-v0.2-plan.md:1)

## Task List

- [ ] Task: Scaffold the library-first crate and thin binary entrypoint
  - Acceptance: `agent-drift-sentinel` exists as a workspace member with `src/lib.rs` and a minimal `src/main.rs` that delegates to library-owned behavior.
  - Verify: `cargo build -p agent-drift-sentinel`
  - Files: `Cargo.toml`, `crates/agent-drift-sentinel/Cargo.toml`, `crates/agent-drift-sentinel/src/lib.rs`, `crates/agent-drift-sentinel/src/main.rs`

- [ ] Task: Implement replay-mode checkpoint loading
  - Acceptance: the sentinel can load analyzer checkpoint bundles deterministically in replay mode, with stable ordering and cursoring.
  - Verify: `cargo test -p agent-drift-sentinel replay_input -- --nocapture`
  - Files: `crates/agent-drift-sentinel/src/input.rs`, `crates/agent-drift-sentinel/src/lib.rs`, `crates/agent-drift-sentinel/tests/replay_input.rs`

- [ ] Task: Implement scheduler state and trigger classes
  - Acceptance: the sentinel supports explicit trigger classes, cooldowns, debounce rules, and repeated-failure fast paths without relying on wall-clock spam alone.
  - Verify: `cargo test -p agent-drift-sentinel scheduler -- --nocapture`
  - Files: `crates/agent-drift-sentinel/src/scheduler.rs`, `crates/agent-drift-sentinel/src/lib.rs`, `crates/agent-drift-sentinel/tests/scheduler.rs`

- [ ] Task: Implement replay-mode operator summaries
  - Acceptance: replay-mode output renders concise checkpoint summaries, warning summaries, evidence refs or excerpts, and expected next steps.
  - Verify: `cargo test -p agent-drift-sentinel operator_surface -- --nocapture`
  - Files: `crates/agent-drift-sentinel/src/operator_surface.rs`, `crates/agent-drift-sentinel/src/lib.rs`, `crates/agent-drift-sentinel/tests/operator_surface.rs`

- [ ] Task: Separate visible warnings from silent checkpoint handling
  - Acceptance: not every checkpoint becomes a visible warning; the warning policy is explicit and test-covered to reduce operator fatigue.
  - Verify: `cargo test -p agent-drift-sentinel warning_policy -- --nocapture`
  - Files: `crates/agent-drift-sentinel/src/operator_surface.rs`, `crates/agent-drift-sentinel/src/scheduler.rs`, `crates/agent-drift-sentinel/tests/warning_policy.rs`

- [ ] Task: Implement optional model adjudication request shaping
  - Acceptance: adjudication is disabled by default, uses bounded analyzer inputs, requests `gpt-5.4-mini` with `medium` reasoning effort when enabled, and never becomes the sole explanation path.
  - Verify: `cargo test -p agent-drift-sentinel adjudication -- --nocapture`
  - Files: `crates/agent-drift-sentinel/src/adjudication.rs`, `crates/agent-drift-sentinel/src/lib.rs`, `crates/agent-drift-sentinel/tests/adjudication.rs`

- [ ] Task: Implement safe adjudication fallback behavior
  - Acceptance: adjudication failure degrades cleanly to analyzer-only output without hiding the original checkpoint evidence.
  - Verify: `cargo test -p agent-drift-sentinel adjudication_fallback -- --nocapture`
  - Files: `crates/agent-drift-sentinel/src/adjudication.rs`, `crates/agent-drift-sentinel/src/operator_surface.rs`, `crates/agent-drift-sentinel/tests/adjudication_fallback.rs`

- [ ] Task: Wire the thin CLI for replay mode
  - Acceptance: the CLI supports replay-mode execution against analyzer checkpoint directories and delegates to library-owned behavior.
  - Verify: `cargo run -p agent-drift-sentinel -- --checkpoint-dir target/agent-drift-analyzer/session-<session-id> --mode replay`
  - Files: `crates/agent-drift-sentinel/src/main.rs`, `crates/agent-drift-sentinel/src/cli.rs`, `crates/agent-drift-sentinel/src/lib.rs`

- [ ] Task: Gate replay-mode usefulness before live-mode work starts
  - Acceptance: replay-mode output is reviewed for operator usefulness, warning noise, and checkpoint clarity before any live integration begins.
  - Verify: `cargo test -p agent-drift-sentinel -- --nocapture`
  - Files: `docs/specs/agent-drift-sentinel-v0.2-plan.md`, `docs/specs/agent-drift-sentinel-v0.2-spec.md`

- [ ] Task: Gate live-mode entry and only then scope live integration work
  - Acceptance: live-mode work stays explicitly gated behind replay validation, and any follow-on runtime integration is framed as a new bounded slice rather than leaking into replay-mode implementation.
  - Verify: human review gate recorded in the plan/spec before implementation starts
  - Files: `docs/specs/agent-drift-sentinel-v0.2-plan.md`, `docs/specs/agent-drift-sentinel-v0.2-spec.md`, `docs/ideas/hybrid-drift-sentinel-architecture-overview.md`
