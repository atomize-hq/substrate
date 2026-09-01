**Kind:** slice row
**Stable ID:** `e2-policy-commitments-on-work-and-workers`
**Canonical for:** extracted E2 slice row plus the controlling E2 authority correction below
**Status:** eligible for fresh admission and explicit dispatch; currently undispatched
**Authority scope:** exact extracted source table header and row plus the documentation-only correction below; no E2 admission, dispatch, implementation, or completion authority
**Source span:** [`../03-phase-slice-map.md`](../03-phase-slice-map.md) line 221
**Supersedes:** canonical ownership of the extracted `E2 — Policy commitments on work and workers` row and only the stale E2 clauses identified in the correction below
**Superseded by:** none
**Projection consumers:** [`track-e-dispatch-policy-and-config-projection.md`](track-e-dispatch-policy-and-config-projection.md), [`../03-phase-slice-map.md`](../03-phase-slice-map.md)

# E2 — Policy commitments on work and workers

> **Authority boundary:** This file preserves the extracted E2 Track E row as chronology and owns
> the controlling correction below. It does not reopen B1/B2.1, B3.1/C1, D1, or E3 ownership and
> does not admit, dispatch, implement, or complete E2.

## Current E2 authority correction (2026-09-01; controlling)

E2 owns the immutable, independently valid
[`DispatchPolicyCommitmentV1`](../contracts/dispatch-policy-commitment-v1.md) record. The record is
persisted and exact-linked before accepted work or worker-launch success is reported. It is not a
receipt and is not a partially populated `RetainedWorkerManifestV1`. Later receipt and retained-
manifest owners consume `DispatchPolicyCommitmentRefV1` and equality-project its snapshot/cap
fields.

The corrected formulas are:

```text
ephemeral work = current parent AND dispatch patch
worker launch  = parent at spawn AND spawn patch
future turn    = current parent now AND immutable worker cap AND turn patch
fork cap       = current parent now AND immutable source-worker cap AND fork patch
```

Parent narrowing therefore affects future work. Parent broadening cannot widen an existing worker.
An accepted active-run commitment is immutable; emergency invalidation is an explicit audited
cancel/revoke reference, never commitment mutation. Missing, incompatible, or hash-invalid cap
bytes/ref fail closed.

For ephemeral work and retained turns, the record exact-links the immutable B1 acceptance and the
B2.1 observation claim. Fresh worker launch exact-links the existing B3.2a retained-admission
identity because the frozen B1 V1 schema does not own Spawn acceptance. Fork does not: frozen
B3.2a explicitly excludes fork, so E2 exact-links the strict validated fork request, immutable
source cap, and fixed child/bootstrap identity without claiming admission or lifecycle ownership.
Those subject-specific links preserve current ownership and create no B1/B3.2a expansion or
reverse dependency.

### Receipt and manifest completion wall

- E2 persists only the immutable policy component and its exact B1/B2.1, fresh-Spawn B3.2a, or
  E2-owned fork-dispatch link.
- B2.2/B3.2 retain receipt construction, foreground-return, lifecycle, park, and remaining
  `RG-RECEIPT-02` ownership; they must consume the exact E2 ref.
- The later full retained manifest remains incomplete until its existing D1 owner supplies the
  execution-envelope identity and its existing E3 owner supplies the config-projection identity.
  D1 and E3 are not E2 prerequisites.
- A record omitting either later-owned identity cannot validate or be reported as a complete
  `RetainedWorkerManifestV1`.

### E2 implementation fence for a future fresh dispatch

This is a symbol fence, not file-wide authority:

1. In `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`, only
   `WorldDispatchRequestV1`, `ValidatedWorldDispatchRequestV1`,
   `WorldDispatchRequestV1::validate`, the minimum strict typed narrowing carrier, and focused
   tests may change.
2. In `crates/shell/src/execution/agent_runtime/tool_invocation_contract.rs`, only
   `RunWorldTaskToolCallV1`, `SpawnWorldWorkerToolCallV1`,
   `HostToolFollowUpArgumentsV1<P>`,
   `translate_host_tool_invocation_request_to_internal_dispatch_request_v1`,
   `translate_run_world_task_to_internal_dispatch_request_v1`,
   `translate_spawn_world_worker_to_internal_dispatch_request_v1`,
   `translate_follow_up_arguments_v1`,
   `translate_follow_up_tool_to_internal_dispatch_request_v1`, `BuildDispatchRequestArgsV1`,
   `build_dispatch_request_v1`, and focused tests may carry the authenticated optional E1 patch
   into `WorldDispatchRequestV1`. Unknown, duplicated, runtime-owned, or subject-mismatched
   material is rejected, not ignored.
3. The landed E1 `AuthenticatedDispatchPolicyNarrowingContextV1`,
   `ResolvedDispatchPolicyNarrowingAuthorityV1`, `ResolvedDispatchPolicySnapshotV1`,
   `resolve_dispatch_narrowed_policy_snapshot`, `DispatchPolicyNarrowingPatchV1`, and existing
   `ExecuteRequest.policy_snapshot` are unchanged consumed dependencies, not E2 edit authority.
   The directly necessary carrier/plumbing fence is limited to the minimum retained-turn policy
   carrier and validation in `MemberTurnSubmitRequestV1` in
   `crates/transport-api-types/src/lib.rs`;
   `MemberDispatchTransportRequest`,
   `build_agent_client_and_member_dispatch_request_for_cwd` and
   `build_agent_client_and_member_dispatch_request_impl` in
   `execution/routing/dispatch/world_ops.rs`; and these exact
   `orchestrator_world_dispatch.rs` integration symbols:
   `PreparedSpawnWorldWorkerBootstrap`, `PreparedForkWorldWorkerBootstrap`,
   `prepare_orchestrator_world_dispatch`, `prepare_authority_bound_spawn_world_worker`,
   `prepare_task_acceptance_submission`, `prepare_retained_acceptance_submission`,
   `build_run_world_task_transport_request`, `build_spawn_world_worker_transport_request`,
   `build_fork_world_worker_transport_request`, `build_continue_world_worker_submit_request`,
   `build_continue_world_worker_submit_request_with_acceptance_context`,
   `observe_run_world_task_stream`, `execute_continue_world_worker_stream_for_turn_kind_impl`,
   `fork_world_worker`, `prepare_fork_world_worker_bootstrap`,
   `continue_world_worker_fork_command_bootstrap_after_delivery`, and
   `spawn_prepared_world_worker`, `execute_spawn_world_worker_stream`, plus focused tests. The only permitted observation-path edit is
   the E2 persistence/link call immediately after the existing durable B1/B2.1 join; the only
   permitted fork-path edits bind/persist the E2 policy component before child launch is reported.
4. The live internal-toolbox Linux carrier fence in `crates/shell/src/repl/async_repl.rs` is limited
   to `PreparedAgentRuntime`, only the Spawn/Fork arms of
   `handle_internal_toolbox_world_dispatch_request`,
   `prepare_member_runtime_startup_from_authority_registration`,
   `prepare_fork_child_runtime_startup_for_descriptor`,
   `build_member_dispatch_transport_request`, `start_internal_dispatch_member_runtime`, and
   `start_remote_member_runtime_with_prepared`, plus focused tests. These symbols may only carry,
   persist, and equality-check the E1 result/E2 ref before reporting launch; provider, credential,
   lifecycle, routing, and unrelated runtime-start behavior remain frozen.
5. One bounded E2 commitment-registry module plus the minimum opaque physical StateStore
   capability/export wiring and focused serialization, CAS, retry, restart, and compatibility tests
   is permitted. StateStore does not parse, compose, synthesize, or repair policy.

The narrowed `PolicySnapshotV3` must enter the real `ExecuteRequest.policy_snapshot` or retained-
turn carrier through this strict translation path. E2 grants no file-wide authority and no change
to unrelated orchestration lifecycle, supervisor semantics, cancellation behavior, routers,
providers, obligations, or public surfaces.

### Mixed-version boundary

Retained workers without recoverable canonical cap bytes, or an immutable byte reference resolving
to bytes with the matching hash, are unsupported for E2 continue/fork. The cap is never inferred
from the current parent and is never reconstructed from a newer or broader parent. Continue/fork
returns the typed `PolicyCommitmentCompatibilityResultV1::UnsupportedLegacyState` result before
dispatch. Existing work proceeds only where current authority already defines safe completion
without the missing cap. No migration, backfill, or synthetic cap is authorized.

### Gate and dependency disposition

E2 retains `RG-POLICY-03` and only the remaining receipt/manifest linkage clause of
`RG-POLICY-01`. `RG-RECEIPT-02` remains with B2.2/B3.2 except for E2's immutable policy component.
Only E2-specific immutable-policy clauses flow into `RG-CANCEL-01` and `RG-OBS-01`.
`RG-BASE-03` remains under C2 and `RG-DIFF-01` remains mandatory. E3/E4, B2.2/B3.2, broader C/D
work, D1/E3-owned manifest material, and non-Linux platforms remain outside E2. This correction
makes E2 eligible for fresh admission; it does not admit or dispatch it.

## Preserved pre-correction row (chronology only)

| Slice | Goal | Must-read docs | Sibling context | Allowed code areas | Explicit non-goals | Exit gate | Regression gates |
|---|---|---|---|---|---|---|---|
| **E2 — Policy commitments on work and workers** | Build immutable active-run snapshots and retained-worker caps linked to B1 acceptance records; recompute future turns as parent ∧ cap ∧ turn patch before B2.2/B3.2 expose final receipts. | `04` acceptance/receipt/manifest and immutable snapshot rules; `02` ReceiptRegistry/RetainedRuntime rows | B1 acceptance records; B3.1 event truth; B2.2/B3.2 consumers; E1 resolver; fork lifecycle | receipt/manifest persistence; `orchestrator_world_dispatch.rs`; retained lifecycle code; policy tests | No mutation of accepted snapshots; no automatic worker broadening; no config rendering. | Final receipts/manifests reference their B1 acceptance record and record immutable hash/ref/revision/reason; parent narrowing affects future turns; parent broadening does not widen worker; fork inherits cap. | `RG-POLICY-03`, `RG-RECEIPT-02`, `RG-CANCEL-01`; E2 policy clause of `RG-OBS-01` |
