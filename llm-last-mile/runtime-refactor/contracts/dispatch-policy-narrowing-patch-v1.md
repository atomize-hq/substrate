**Kind:** contract
**Status:** canonical
**Canonical for:** complete extracted `DispatchPolicyNarrowingPatchV1` schema, `DispatchCapabilitySubjectV1` enum, and the V1 `RestrictedPolicyPatchV1` `world_fs`-only rejection rule
**Source provenance:** extracted byte-for-byte from [`../04-contracts-and-gates.md#7-dispatchpolicynarrowingpatchv1`](../04-contracts-and-gates.md#7-dispatchpolicynarrowingpatchv1), baseline lines 141–167; the exact 810-byte source body is preserved between the boundary markers below
**Baseline span SHA-256:** `c0fd22a7c630c30e8b6fca976645d84dc7515b435af759c4ecc0bc8b8871da4b`

<!-- exact-extracted-body:start -->
## 7. `DispatchPolicyNarrowingPatchV1`

```rust
struct DispatchPolicyNarrowingPatchV1 {
    schema_version: u32,
    request_id: String,
    orchestration_session_id: String,
    caller_participant_id: String,
    target_backend_id: String,
    target_world: WorldBindingRefV1,
    applies_to: DispatchCapabilitySubjectV1,
    parent_policy_ref: PolicyRefV1,
    parent_policy_revision: String,
    restricted_policy_patch: RestrictedPolicyPatchV1,
    reason: Option<String>,
}

enum DispatchCapabilitySubjectV1 {
    EphemeralTask,
    RetainedWorkerSpawn,
    RetainedWorkerTurn { retained_participant_id: String },
    RetainedWorkerFork { source_participant_id: String },
}
```

V1 `RestrictedPolicyPatchV1` may contain only `world_fs` fields. Unknown or non-`world_fs` fields are rejected, not ignored.

<!-- exact-extracted-body:end -->
