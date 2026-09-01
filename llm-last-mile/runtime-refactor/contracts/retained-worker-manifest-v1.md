**Kind:** contract
**Status:** canonical
**Canonical for:** complete extracted `RetainedWorkerManifestV1` schema and rules 1–5 covering spawn-time cap narrowing, fork narrowing, parked identity continuity, world-generation invalidation, the immutable B1 accepted-record boundary, and the current manifest-completion correction
**Source provenance:** extracted byte-for-byte from [`../04-contracts-and-gates.md#6-retainedworkermanifestv1`](../04-contracts-and-gates.md#6-retainedworkermanifestv1), baseline lines 103–140; the exact 1576-byte source body is preserved between the boundary markers below
**Baseline span SHA-256:** `cd25ec1c19548b8442e01f753517c117cab9a350cb828de8214a351bdc09ee07`

## Current manifest-completion correction

E2 does not construct this manifest and must not serialize a partial instance that appears
complete. E2 persists only the independently valid
[`DispatchPolicyCommitmentV1`](dispatch-policy-commitment-v1.md) cap and its exact admission/work
link. The later manifest owner consumes `DispatchPolicyCommitmentRefV1`; the extracted
`worker_policy_cap_ref`/hash/snapshot fields must equality-project its verified canonical bytes.

The full manifest remains incomplete until D1 supplies `execution_envelope_ref` and E3 supplies
`config_projection_identity` and `config_projection_ref`. D1/E3 later compose their owned material
with the immutable E2 cap; neither is an E2 prerequisite. Missing D1 or E3 material is a validation
failure, not an incomplete-but-valid manifest state.

<!-- exact-extracted-body:start -->
## 6. `RetainedWorkerManifestV1`

```rust
struct RetainedWorkerManifestV1 {
    schema_version: u32,
    retained_participant_id: String,
    orchestration_session_id: String,
    orchestrator_participant_id: String,
    parent_retained_participant_id: Option<String>,
    target_backend_id: String,
    world_id: String,
    world_generation: u64,
    runtime_family: String,
    resume_handle_ref: Option<ResumeHandleRefV1>,
    worker_policy_cap_ref: PolicySnapshotRefV1,
    worker_policy_cap_hash: String,
    worker_policy_cap_snapshot: PolicySnapshotV3,
    config_projection_identity: ConfigProjectionIdentityV1,
    config_projection_ref: ConfigProjectionRefV1,
    execution_envelope_ref: ExecutionEnvelopeRefV1,
    lifecycle_state: RetainedWorkerLifecycleStateV1,
    active_turn_ref: Option<ActiveRunRefV1>,
    manifest_revision: u64,
    created_at: Timestamp,
    updated_at: Timestamp,
}
```

Rules:

1. Spawn-time effective policy, including spawn narrowing, becomes the maximum worker cap.
2. Fork inherits the source cap by default and may narrow further; it cannot broaden.
3. Clean turn exit may park the worker. It must not delete retained identity or resume continuity.
4. World generation mismatch makes the worker unroutable/invalidated; it does not silently rebind.
5. `active_turn_ref` is a later RetainedWorkerRuntime lifecycle reference to the immutable B1
   accepted record. B1 does not create, update, or close it; the later retained-lifecycle packet
   owns that separate atomic lifecycle transition without mutating accepted-turn identity.

<!-- exact-extracted-body:end -->
