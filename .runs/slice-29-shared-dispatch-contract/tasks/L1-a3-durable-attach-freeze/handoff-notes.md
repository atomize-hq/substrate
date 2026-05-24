Worker branch: `codex/feat-gateway-mediated-llm-fulfillment-s29-a3-durable-attach-freeze`

Worker tip: `aa656181aa53c92cefbec1264145c5b071e5e803`

Parent integration:

- Cherry-picked onto the authoritative branch as `42636f8eb68fe2e817d973a4896f5c35cfc5b12b`.
- No merge conflicts.
- `dispatch_contract.rs` changed only to update the co-located `persisted_attach_contract_is_explicit_baseline_domain` unit-test fixture for the new persisted `HostAttachContract` fields; the shared contract owner and dispatch semantics remained frozen after `G1`.
- The durable-state changes stayed centered on `orchestration_session.rs` and `state_store.rs`, with successor attach truth and detached-posture checks now keyed off persisted contract truth.
