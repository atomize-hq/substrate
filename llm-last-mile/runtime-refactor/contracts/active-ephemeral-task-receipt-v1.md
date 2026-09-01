**Kind:** contract
**Status:** canonical
**Canonical for:** complete extracted `ActiveEphemeralTaskReceiptV1` schema, state transitions, terminal monotonicity, explicit result-class literal order, `NeedsRetainedFollowup` negative requirements, and the current E2 composition correction
**Source provenance:** extracted byte-for-byte from [`../04-contracts-and-gates.md#3-activeephemeraltaskreceiptv1`](../04-contracts-and-gates.md#3-activeephemeraltaskreceiptv1), baseline lines 91–138; the exact 1557-byte source body is preserved between the boundary markers below
**Baseline span SHA-256:** `c997752654f041de820210f03c86706551b44389994df5e8e0341aa41a34fa3b`

## Current E2 composition correction

E2 does not construct this receipt. E2 persists an independently valid
[`DispatchPolicyCommitmentV1`](dispatch-policy-commitment-v1.md) after exact B1 acceptance and
B2.1 observation linkage and before acceptance is reported. B2.2 remains the receipt owner and must
add/consume the exact `DispatchPolicyCommitmentRefV1`; the extracted
`policy_snapshot_ref`/hash/revision/reason fields equality-project that immutable record and cannot
be independently recomputed. The receipt must not be exposed before that ref is durable and
exact-linked.

<!-- exact-extracted-body:start -->
## 3. `ActiveEphemeralTaskReceiptV1`

```rust
struct ActiveEphemeralTaskReceiptV1 {
    schema_version: u32,
    acceptance_record_id: String,
    task_run_id: String,
    request_id: String,
    orchestration_session_id: String,
    caller_participant_id: String,
    target_backend_id: String,
    world_id: String,
    world_generation: u64,
    policy_snapshot_ref: PolicySnapshotRefV1,
    policy_snapshot_hash: String,
    policy_revision: String,
    narrowing_reason: Option<String>,
    runtime_acceptance: RuntimeAcceptanceEvidenceV1,
    observation_claim: SupervisorObservationClaimV1,
    accepted_at: Timestamp,
    state_revision: u64,
    state: ActiveTaskStateV1,
    cancel_supported: bool,
    terminal: Option<WorldWorkTerminalV1>,
}
```

States:

```text
Accepted -> Running -> AttentionPending -> Running
Accepted|Running|AttentionPending -> Terminal|Failed|Cancelled|Invalidated
```

`Terminal`, `Failed`, `Cancelled`, and `Invalidated` are terminal and monotonic. `Parked` is not valid for an ephemeral task.

`WorldWorkTerminalV1` for an ephemeral task carries one explicit result class:

```text
Completed
Failed
Cancelled
NeedsRetainedFollowup
Invalidated
```

`NeedsRetainedFollowup` is a terminal ephemeral result, not a retained worker state. It promises no durable participant identity, creates no `continue_world_worker` route, and does not silently create a retained worker or durable conversational obligation. The host must make a new, explicit, policy-checked `spawn_world_worker` decision if ongoing work is warranted.

<!-- exact-extracted-body:end -->
