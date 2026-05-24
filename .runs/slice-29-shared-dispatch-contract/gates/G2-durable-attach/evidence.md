# G2 Evidence

`G2` passes because the integrated authoritative tree satisfies the durable attach freeze acceptance criteria in `ORCH_PLAN.md`.

1. Session birth persists generalized host attach truth derived from the resolved host launch contract.
   - `HostAttachContract` now persists launch descriptor, capabilities, attach knobs, and continuity selector with additive serde defaults: [`orchestration_session.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs:75), [`orchestration_session.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs:88), [`orchestration_session.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs:127)
   - Session creation derives that persisted truth from the resolved host launch manifest via `from_manifest(...)`: [`orchestration_session.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs:145), [`orchestration_session.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs:159)
   - Birth invariants are covered by `new_session_starts_active_attached`: [`orchestration_session.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs:595)
2. Successor copy preserves launch truth while clearing only continuity-specific state.
   - `fork_successor_attach_contract(...)` clones the persisted attach contract and clears only `continuity_uaa_session_id`: [`orchestration_session.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs:335)
   - The exact behavior is proven by `successor_attach_contract_clears_continuity_and_preserves_launch_truth`: [`orchestration_session.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs:664)
3. `state_store.rs` uses persisted attach truth rather than ambient participant state for detached attach planning.
   - Public control targeting carries the persisted contract, gates resume/fork/stop off persisted capabilities, and requires continuity from the durable selector rather than the live participant: [`state_store.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:779)
   - Public turn targeting now returns the persisted host attach contract on the resolved target: [`state_store.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:901), [`state_store.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:968)
   - Detached posture reconciliation and stale-attachment recovery both depend on persisted capability and continuity truth: [`state_store.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:2360), [`state_store.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:2399)
   - Coverage exists for persisted continuity control/turn routing and legacy-json backfill: [`state_store.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:3554), [`state_store.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:3609), [`state_store.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:3807)
4. Persisted-state validation remains additive and fail-closed.
   - The generalized fields are migration-safe through serde defaults, and `validate_persisted_invariants(...)` fail-closes on descriptor, scope, protocol, and attach-knob drift: [`orchestration_session.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs:80), [`orchestration_session.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs:128), [`orchestration_session.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs:396)
   - The drift guard is covered by `host_attach_contract_knob_drift_fails_closed`: [`orchestration_session.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs:691)
   - Legacy session JSON without the new fields backfills defaults and still loads cleanly: [`state_store.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:3807)
5. The durable-state schema freeze finalized without reopening `L0` semantics.
   - The accepted authoritative diff from `50a450a05f374e46c736f79f9b5c7fec5c0e54d9` to `42636f8eb68fe2e817d973a4896f5c35cfc5b12b` is limited to `orchestration_session.rs`, `state_store.rs`, and one compile-only fixture update in `dispatch_contract.rs`.
   - The only `dispatch_contract.rs` change is the test fixture expansion needed to instantiate the generalized `HostAttachContract`; persisted attach baseline semantics stay frozen: [`dispatch_contract.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/dispatch_contract.rs:828)
   - Focused authoritative validation passed on the merged tree:
     - `cargo fmt --all -- --check`
     - `cargo clippy -p shell --lib --tests -- -D warnings`
     - `cargo test -p shell resolve_public_control_target -- --nocapture`
     - `cargo test -p shell new_session_starts_active_attached -- --nocapture`
     - `cargo test -p shell detached_postures_enforce_pending_inbox_truth -- --nocapture`
     - `cargo test -p shell successor_attach_contract_clears_continuity_and_preserves_launch_truth -- --nocapture`
     - `cargo test -p shell host_attach_contract_knob_drift_fails_closed -- --nocapture`
     - `cargo test -p shell resolve_public_turn_target_uses_persisted_continuity_truth -- --nocapture`
     - `cargo test -p shell load_session_backfills_generalized_attach_contract_defaults_for_legacy_json -- --nocapture`
     - `cargo test -p shell persisted_attach_contract_is_explicit_baseline_domain -- --nocapture`

Ancillary evidence:

- `accepted_tip_after_G2 = 42636f8eb68fe2e817d973a4896f5c35cfc5b12b`
- `L2` and `L3` were both created from that exact accepted authoritative SHA.
- GitNexus `detect-changes` compare remains stale and medium-risk because the index refresh crashes, but the output was preserved in the task artifacts rather than ignored.
