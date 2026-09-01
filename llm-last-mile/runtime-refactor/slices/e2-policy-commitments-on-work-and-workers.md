**Kind:** slice row
**Stable ID:** `e2-policy-commitments-on-work-and-workers`
**Canonical for:** extracted E2 slice row plus the controlling E2 authority correction below
**Status:** eligible only for another fresh admission and explicit dispatch; currently undispatched
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
exact source-owned B2.1 `WorldWorkExecutionClaimV1` identity/hash/preimage and durable claim key.
There is no E2 `SupervisorObservationClaimV1`, no `resumable` field, and no E2 inference of
resumability. Fresh worker launch first durably stores and indexes the complete E2 policy/cap
reservation, then B3.2a admits with that reservation's exact preallocated identities, then E2
retains the reservation ref, exact-links the stable B3.2a admission identity, and publishes the
immutable commitment. Fork does not use B3.2a: frozen B3.2a explicitly excludes fork, so E2
exact-links the strict validated fork request, immutable source cap, and fixed child/bootstrap
identity without claiming admission or lifecycle ownership. Those subject-specific links preserve
current ownership and create no B1/B3.2a expansion or reverse dependency.

### Retained current-parent drift

The retained-target resolver remains the owner of retained identity and routing authentication. It
keeps every session, participant, backend, world, B3.2a admission, registration, descriptor,
resume-handle, retained-worker, ancestry, lifecycle, routability, and immutable launch-policy/cap
check. It returns the verified immutable launch cap separately from the independently resolved
current parent. Launch policy/ref/revision equality with current parent is not an E2 prerequisite.
Only the exact equality checks that currently conflate those two inputs may change.

E2 composes Continue from current parent now, immutable worker cap, and turn patch; Fork uses
current parent now, immutable source-worker cap, and fork patch. Current-parent narrowing may
further restrict future work, while broadening can never exceed the immutable launch cap. A worker
without verifiable immutable cap bytes/ref/hash returns typed `UnsupportedLegacyState` before
Continue or Fork. No route, lifecycle, registration, or StateStore persistence authority moves to
E2.

### Fresh-Spawn reservation and crash boundaries

The E2-owned durable request/subject index is the pre-B3.2a reservation. First-writer CAS stores an
immutable reservation object and indexes its ref, freezing request/idempotency/subject identity,
an E2-keyed domain-separated commitment over every canonical validated Spawn request field
(including action, mode, and complete payload/prompt), parent ref/revision, complete E1 patch
identity/hash, exact E1 snapshot bytes/ref/hash, proposed cap/commitment identity, stable
worker/bootstrap identities, reason, and world/backend bindings. Only the request commitment's key
ID/digest persists; prompt/payload preimage and raw secret-derived SHA-256 do not.
Publication and file/directory `fsync` complete before B3.2a is invoked. An
E2-registry-authenticated private constructor recomputes and compares the full-request commitment,
then carries only an opaque reservation capability/ref plus the preallocated identities into the
bounded B3.2a input seam. The request commitment and bytes never cross that seam. B3.2a uses those
identities in its unchanged fingerprint rather than generating replacements.
The reservation grants no participant admission, lifecycle, or routability; those remain
B3.2a-owned, and B3.2a's frozen fingerprint is not changed or versioned to carry E1 material.

After B3.2a reaches its first registration-bearing state, E2 projects the exact stable source fields
of `RetainedWorkerAdmissionRecordV1` (all fields except mutable `state`/`record_revision`) plus its
exact registration into a domain-separated stable-admission link. E2 CAS-retains the reservation
ref, links that identity, and publishes the immutable record. Later B3.2a lifecycle revision changes
do not invalidate the link. The reservation, committed index, stable admission link, and final
commitment equality-project only the fields each owner actually duplicates: the stable admission
link carries and compares source-owned B3.2a fields plus registration, never E2-only request
commitment, patch, snapshot, cap, or final commitment material.
Identical retry joins either reservation or commitment. Changed validated request/payload, patch,
parent, cap, subject, binding, or generated identity conflicts before B3.2a. Crash before reservation publication retries
the first-writer CAS; crash after reservation/before admission resumes from it; crash after
admission/before E2 publication exact-joins B3.2a with the reservation's stable identities and then
publishes; crash after publication joins the commitment. Once E2 activates, production B3.2a
without the exact E2 reservation is unsupported. Patch, snapshot, and cap material are never
reconstructed from current parent state.

### Snapshot and B2.1 identity

`DispatchPolicyCommitmentV1.policy_snapshot_bytes` are exactly the bytes from the landed E1
`serde_json::to_vec(PolicySnapshotV3)` path, and `policy_snapshot_hash` is exactly E1's SHA-256 over
those bytes. E2 decodes the bytes to the expected `PolicySnapshotV3`, requires reserialization to be
byte-identical, and reproduces the existing E1/B1 hash. Recursive object-key sorting remains
available only for separately and uniquely domain-separated E2 record, reservation, patch,
admission, claim, and fork-link hashes; it does not define the policy snapshot hash.
`PolicySnapshotV3`, E1 serialization, schema 3, and B1 stored hashes remain unchanged.

The B2.1 link uses the exact canonical `WorldWorkExecutionClaimV1` preimage and an E2 linkage hash,
plus the exact source-owned durable key in
`WorldWorkExecutionSupervisorStateV1.executions_by_acceptance_record_id`. Interruption or cursor
references remain separate B2.1-owned evidence and are absent because no current E2 subject needs
them. E2 does not mutate supervisor state, reinterpret lifecycle, create acceptance, infer
resumability, or mutate receipts/observations.

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
   capability/export wiring and focused serialization, reservation/publication `fsync`, CAS,
   exact-join, conflict, retry, restart, and compatibility tests is permitted. Fresh-Spawn callers
   must reserve before `RetainedWorkerRuntime::reserve_admission_slot` and publish only after exact
   B3.2a admission. Because the landed function generates worker/bootstrap IDs internally, future
   E2 may change only `RetainedWorkerRuntime::reserve_admission_slot`, its private
   `reserve_admission_slot_with` helper, their direct production callers, and focused tests to accept
   an E2-registry-authenticated, privately constructed reservation proof and use its two preallocated
   identities. The E2 registry recomputes and equality-checks the complete validated-request
   commitment before constructing that opaque capability; only the capability/ref and IDs cross
   into B3.2a. Existing non-E2 callers retain their current generation behavior before activation;
   once E2 activates, production unreserved admission is unsupported. The proof is equality-checked
   before slot allocation and is not added to or used to version the B3.2a fingerprint.

   The landed `validate_admission_plan` separately rejects
   `policy.allow_capability_narrowing == true`, while authenticated nonempty E1 narrowing requires
   that parent capability. The private E2 proof constructor emits a distinct, opaque narrowing
   attestation only after authenticating a nonempty `RestrictedWorldFs` patch, parent permission,
   exact E1 effective snapshot bytes/ref/hash, complete validated-request commitment, and both
   preallocated identities; `UnchangedParent` and empty patches cannot obtain it. Future E2 may
   change only that exact rejection, the minimum opaque-proof
   parameter threading through all seven direct B3.2a call sites
   (`canonical_plan_for_existing_admission`, `validate_admitted_record_graph`,
   `prepare_registration_head_in_registry`, `advance_registration_head_in_registry`,
   `claim_transport_in_registry`, `advance_admission_runtime_truth_in_registry`, and
   `reserve_slot_in_registry`), their minimum proof-forwarding wrapper chain, and focused tests.
   That wrapper fence is limited to `reserve_admission_slot`/`reserve_admission_slot_with`,
   `register_admitted_worker`/`register_admitted_worker_with`,
   `prepare_admission_registration_head`/`prepare_admission_registration_head_with`,
   `advance_registration_head_after_r0`/`advance_registration_head_after_r0_with`,
   `claim_admission_transport`/`claim_admission_transport_at`/`claim_admission_transport_with`,
   `launch_authority_proof_for_claim`, `publish_admission_runtime_truth`,
   `mark_admission_routable`, `mark_admission_terminal`, `mark_admission_interrupted`, and their
   direct E2 production callers. Each wrapper may only carry the opaque authenticated reservation capability;
   it may not change control flow, durable state, or lifecycle meaning. `true` is accepted only for an exact-joined,
   authenticated E2 reservation whose distinct narrowing attestation proves that its parent,
   nonempty `RestrictedWorldFs` patch, effective snapshot, full validated request commitment, and
   preallocated identities match; B3.2a does not parse or persist E1 patch
   material. The B3.2a plan and fingerprint already bind the policy bit and remain byte/schema/
   algorithm unchanged. B3.2a's other validation, registry schema, admission record/state, cap
   accounting, registration-head ordering, registration, transport, routability, lifecycle, and
   unrelated tests remain frozen. StateStore does not parse, compose, synthesize, or repair policy.
6. In `crates/shell/src/execution/agent_runtime/state_store.rs`, only
   `ResolvedWorldWorkRegistryAuthorityV1`, `ResolvedCanonicalRetainedWorldDispatchTargetV1`,
   `resolve_canonical_retained_world_dispatch_target`,
   `BoundAgentRuntimeStateStore::resolve_world_work_registry_authority`, their directly necessary
   E2 callers, and focused tests may change. In
   `agent_runtime/retained_worker_runtime.rs`, only `ResolvedRetainedTargetV1`,
   `RetainedWorkerRuntime::resolve_retained_target`, its directly necessary E2 callers, and focused
   tests may change. The edit is limited to the existing comparisons of
   `admission.current_policy_ref`/`current_policy_revision` with current parent,
   `resolved.retained_worker.policy_ref`/`resolved.current_policy` with current parent, and
   `current.authority.current_policy_ref` with the retained worker's launch policy. Those checks are
   replaced only by authenticated immutable launch-cap linkage and a result that returns that cap
   separately from current parent. Every non-policy identity/routing/admission/routability check and
   all persistence, lifecycle, compatibility, registration, and reconciliation behavior remain
   frozen.
7. In `crates/shell/src/execution/agent_runtime/world_work_execution_supervisor.rs`, E2 may add one
   read-only, behavior-neutral accessor/projection that returns the exact source-owned
   `WorldWorkExecutionClaimV1` identity, canonical preimage/hash inputs, and durable claim key, plus
   focused tests. It may not alter supervisor state, reinterpret lifecycle, create acceptance,
   infer resumability, expose unrelated state, or mutate receipts/observations. Cursor/interruption
   references are excluded unless a separately authorized E2 subject demonstrably requires them.

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
work, D1/E3-owned manifest material, and non-Linux platforms remain outside E2. This second
correction makes E2 eligible only for another fresh admission; it does not admit, dispatch,
implement, or complete it.

## Preserved pre-correction row (chronology only)

| Slice | Goal | Must-read docs | Sibling context | Allowed code areas | Explicit non-goals | Exit gate | Regression gates |
|---|---|---|---|---|---|---|---|
| **E2 — Policy commitments on work and workers** | Build immutable active-run snapshots and retained-worker caps linked to B1 acceptance records; recompute future turns as parent ∧ cap ∧ turn patch before B2.2/B3.2 expose final receipts. | `04` acceptance/receipt/manifest and immutable snapshot rules; `02` ReceiptRegistry/RetainedRuntime rows | B1 acceptance records; B3.1 event truth; B2.2/B3.2 consumers; E1 resolver; fork lifecycle | receipt/manifest persistence; `orchestrator_world_dispatch.rs`; retained lifecycle code; policy tests | No mutation of accepted snapshots; no automatic worker broadening; no config rendering. | Final receipts/manifests reference their B1 acceptance record and record immutable hash/ref/revision/reason; parent narrowing affects future turns; parent broadening does not widen worker; fork inherits cap. | `RG-POLICY-03`, `RG-RECEIPT-02`, `RG-CANCEL-01`; E2 policy clause of `RG-OBS-01` |
