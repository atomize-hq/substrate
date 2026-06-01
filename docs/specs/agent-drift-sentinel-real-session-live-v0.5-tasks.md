# Tasks: Agent Drift Sentinel Real Session Live v0.5

This task list implements:

- `docs/specs/agent-drift-sentinel-real-session-live-v0.5-spec.md`
- `docs/specs/agent-drift-sentinel-real-session-live-v0.5-plan.md`

## Task List

- [ ] Task: Lock the real-session live contract and proof requirements in repo docs
  - Acceptance: the docs explicitly define the bounded real-session path as
    `rollout-*.jsonl -> compactor -> analyzer -> sentinel live runtime`, scoped to one active
    session and one real `CODEX_HOME`.
  - Verify: doc review against the current compactor/analyzer/sentinel crates
  - Files:
    - `docs/specs/agent-drift-sentinel-real-session-live-v0.5-spec.md`
    - `docs/specs/agent-drift-sentinel-real-session-live-v0.5-plan.md`
    - `docs/specs/agent-drift-sentinel-real-session-live-v0.5-tasks.md`

- [ ] Task: Add a sentinel-owned real-session coordinator that targets one active session
  - Acceptance: the sentinel can accept `codex_home`, `session_id`, and a working directory, poll
    the matching rollout artifact, and decide when upstream pipeline reruns are needed.
  - Verify: `cargo test -p agent-drift-sentinel real_session_live -- --nocapture`
  - Files:
    - `crates/agent-drift-sentinel/src/lib.rs`
    - `crates/agent-drift-sentinel/src/cli.rs`
    - sentinel live-session coordinator module(s)

- [ ] Task: Invoke compactor and analyzer libraries directly for the target session
  - Acceptance: the coordinator derives live analyzer checkpoints through the existing library APIs
    for `agent-session-compactor` and `agent-drift-analyzer`, without subprocess shell-outs.
  - Verify:
    - `cargo test -p agent-session-compactor -- --nocapture`
    - `cargo test -p agent-drift-analyzer -- --nocapture`
  - Files:
    - `crates/agent-drift-sentinel/Cargo.toml`
    - sentinel live-session coordinator module(s)

- [ ] Task: Emit only newly observed checkpoints into the existing live runtime
  - Acceptance: repeated polling does not replay already-delivered checkpoints; the sentinel only
    forwards checkpoint deltas to the live runtime and sink.
  - Verify: `cargo test -p agent-drift-sentinel real_session_live -- --nocapture`
  - Files:
    - sentinel live-session coordinator module(s)
    - `crates/agent-drift-sentinel/src/live_runtime.rs`
    - `crates/agent-drift-sentinel/tests/`

- [ ] Task: Enable the bounded `--mode live` CLI for real-session monitoring
  - Acceptance: the live CLI path requires a target session and bounded state directory, and it no
    longer bails behind the old `S10` replay-only gate once the real-session path is ready.
  - Verify: `cargo test -p agent-drift-sentinel -- --nocapture`
  - Files:
    - `crates/agent-drift-sentinel/src/cli.rs`
    - `crates/agent-drift-sentinel/src/lib.rs`

- [ ] Task: Prove the implementation on an actually active live session
  - Acceptance: while the implementation agent is running a real Codex session, the sentinel is run
    against that session’s live rollout JSONL path via the bounded live command and documented with:
    - exact `SESSION_ID`
    - exact command
    - observed checkpoint/warning progression
  - Verify:
    - `cargo run -p agent-drift-sentinel -- --mode live --codex-home "$CODEX_HOME" --session-id "$SESSION_ID" --checkpoint-dir "$LIVE_STATE_DIR"`
  - Files:
    - `docs/specs/hybrid-drift-sentinel-implementation-order.md`
    - any proof note or testing guide updated during landing

Packet exit condition:

- sentinel live mode works over one actual active Codex session
- the real-session path uses compactor/analyzer libraries rather than raw sentinel parsing
- proof is recorded against a live session, not just archived/static fixtures
