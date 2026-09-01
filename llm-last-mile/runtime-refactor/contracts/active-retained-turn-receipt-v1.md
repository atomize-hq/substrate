**Kind:** contract
**Status:** canonical
**Canonical for:** complete extracted `ActiveRetainedTurnReceiptV1` schema, state transitions, rules 1–3 covering the single active cancelable turn, `Parked` continuity, the host/worker posture boundary, and the current E2 composition correction
**Source provenance:** extracted byte-for-byte from [`../04-contracts-and-gates.md#4-activeretainedturnreceiptv1`](../04-contracts-and-gates.md#4-activeretainedturnreceiptv1), baseline lines 139–183; the exact 1443-byte source body is preserved between the boundary markers below
**Baseline span SHA-256:** `2cc996111d82d296f89bdd88f80df5c0b559bdb305aec546881b749ace93f67e`

## Current E2 composition correction

E2 does not construct this receipt or own `Parked`/terminal lifecycle. E2 persists the immutable
[`DispatchPolicyCommitmentV1`](dispatch-policy-commitment-v1.md) after exact B1 retained-turn
acceptance and B2.1 observation linkage and before acceptance is reported. B2.2/B3.2 remain receipt
and lifecycle owners and must add/consume `DispatchPolicyCommitmentRefV1`. The extracted worker-cap
and turn-snapshot fields equality-project the verified E2 record; they cannot be reconstructed from
the current parent or independently mutated.

<!-- exact-extracted-body:start -->
## 4. `ActiveRetainedTurnReceiptV1`

```rust
struct ActiveRetainedTurnReceiptV1 {
    schema_version: u32,
    acceptance_record_id: String,
    active_run_id: String,
    request_id: String,
    orchestration_session_id: String,
    orchestrator_participant_id: String,
    target_participant_id: String,
    target_backend_id: String,
    world_id: String,
    world_generation: u64,
    message_id: String,
    thread_id: Option<String>,
    worker_policy_cap_hash: String,
    turn_policy_snapshot_ref: PolicySnapshotRefV1,
    turn_policy_snapshot_hash: String,
    turn_policy_revision: String,
    narrowing_reason: Option<String>,
    runtime_acceptance: RuntimeAcceptanceEvidenceV1,
    observation_claim: SupervisorObservationClaimV1,
    accepted_at: Timestamp,
    state_revision: u64,
    state: ActiveRetainedTurnStateV1,
    cancel_supported: bool,
    terminal: Option<WorldWorkTerminalV1>,
}
```

States:

```text
Accepted -> Running
Running -> AttentionPending -> Running
Accepted|Running|AttentionPending -> Parked|Terminal|Failed|Cancelled|Stopped
```

Rules:

1. One retained worker may have at most one active cancelable turn unless a later version explicitly models concurrency.
2. `Parked` closes the active turn but preserves the retained worker manifest and resume handle.
3. Host posture is not copied from turn state. Obligations may cause host `AwaitingAttention`; worker `AttentionPending` remains worker truth.

<!-- exact-extracted-body:end -->
