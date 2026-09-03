**Kind:** contract
**Stable ID:** `dispatch-policy-commitment-v1`
**Status:** canonical; implemented and terminally complete under the E2 closure
**Canonical for:** additive `DispatchPolicyCommitmentV1` persisted record, exact subject/linkage rules, immutable policy/cap semantics, the specified-but-unadmitted `E2-RM` accepted-work receipt-material projection prerequisite, downstream receipt/manifest consumption, and mixed-version fail-closed behavior
**Authority scope:** canonical E2 contract plus terminal status projection and the documentation-only `E2-RM` prerequisite specification; this document grants no new admission, dispatch, implementation, receipt construction, retained-manifest construction, migration, or synthetic-cap authority
**Supersedes:** only E2 statements that require E2 to construct a final receipt or complete `RetainedWorkerManifestV1`; omit the real dispatch/tool-translation carrier needed to consume E1 narrowing; require a retained worker's launch policy to equal the current parent; place fresh-Spawn E2 persistence after B3.2a admission; define a second snapshot canonicalization; or require a synthetic `SupervisorObservationClaimV1`/`resumable` field
**Superseded by:** none
**Projection consumers:** [`../slices/e2-policy-commitments-on-work-and-workers.md`](../slices/e2-policy-commitments-on-work-and-workers.md), [`../slices/b2-2-foreground-receipt-return.md`](../slices/b2-2-foreground-receipt-return.md), [`active-ephemeral-task-receipt-v1.md`](active-ephemeral-task-receipt-v1.md), [`active-retained-turn-receipt-v1.md`](active-retained-turn-receipt-v1.md), [`retained-worker-manifest-v1.md`](retained-worker-manifest-v1.md), [`../gates/final-receipt-immutable-policy-snapshot-v3-acceptance.md`](../gates/final-receipt-immutable-policy-snapshot-v3-acceptance.md)

# `DispatchPolicyCommitmentV1`

> **Authority boundary:** E2 owns one immutable, independently verifiable policy commitment/cap
> record. It persists that record and its exact link before accepted work or a worker launch is
> reported. B2.2/B3.2 receipt owners and the later retained-manifest owners consume its reference;
> E2 does not construct those records. D1 execution-envelope identity and E3 config-projection
> identity are neither fields required to validate this record nor E2 prerequisites.

## Additive V1 schema

```rust
struct DispatchPolicyCommitmentV1 {
    schema_version: u32, // exactly 1
    authority_store_id: String,
    commitment_id: String,
    subject: DispatchPolicyCommitmentSubjectV1,

    request_id: String,
    idempotency_key: String,
    orchestration_session_id: String,
    caller_participant_id: String,
    caller_backend_id: String,
    target_backend_id: String,
    world_id: String,
    world_generation: u64,

    authority_link: PolicyCommitmentAuthorityLinkV1,
    execution_claim_link: Option<WorldWorkExecutionClaimLinkV1>,
    fresh_spawn_reservation_ref: Option<DispatchPolicyCommitmentReservationRefV1>,
    fresh_spawn_validated_request_commitment: Option<FreshSpawnValidatedRequestCommitmentV1>,

    parent_policy_ref: PolicyRefV1,
    parent_policy_revision: String,
    applied_patch: AppliedDispatchPolicyPatchIdentityV1,

    policy_snapshot_bytes: ImmutableBytesMaterialV1,
    policy_snapshot_ref: PolicySnapshotRefV1,
    policy_snapshot_hash: String,
    policy_snapshot_revision: String,
    reason: Option<String>,

    retained_worker_cap_link: Option<RetainedWorkerCapLinkV1>,
    source_worker_cap_ref: Option<DispatchPolicyCommitmentRefV1>,
    created_revision: u64,
    application_revision: u64,
    status: PolicyCommitmentStatusV1, // exactly Immutable
    created_at: Timestamp,
    exact_linkage_hash: String,
}

enum DispatchPolicyCommitmentSubjectV1 {
    EphemeralWork {
        task_run_id: String,
    },
    RetainedWorkerLaunch {
        retained_participant_id: String,
        bootstrap_run_id: String,
    },
    RetainedWorkerTurn {
        retained_participant_id: String,
        active_run_id: String,
        message_id: String,
    },
    RetainedWorkerFork {
        source_participant_id: String,
        child_participant_id: String,
        bootstrap_run_id: String,
    },
}

enum PolicyCommitmentAuthorityLinkV1 {
    B1 {
        acceptance_record_id: String,
        acceptance_record_revision: u64,
        runtime_acceptance: RuntimeAcceptanceEvidenceV1,
    },
    RetainedAdmission {
        stable_admission_identity: RetainedWorkerAdmissionStableIdentityLinkV1,
    },
    ForkDispatch {
        canonical_validated_dispatch_request_sha256: String,
    },
}

struct WorldWorkExecutionClaimLinkV1 {
    authority_store_id: String,
    durable_claim_key: WorldWorkExecutionClaimDurableKeyV1,
    acceptance_record_id: String,
    acceptance_record_revision: u64,
    claim_revision: u64,
    observer_instance_id: String,
    observer_epoch: u64,
    claim_preimage: ImmutableBytesMaterialV1,
    claim_linkage_hash: String,
}

struct RetainedWorkerAdmissionStableIdentityLinkV1 {
    registry_key: RetainedWorkerAdmissionRegistryKeyV1,
    stable_source_fields: RetainedWorkerAdmissionStableSourceFieldsV1,
    registration: RetainedWorkerAdmissionRegistrationV1,
    stable_identity_hash: String,
}

struct RetainedWorkerAdmissionRegistryKeyV1 {
    orchestration_session_id: String,
    retained_participant_id: String,
}

struct RetainedWorkerAdmissionStableSourceFieldsV1 {
    schema_version: u32, // exactly the source B3.2a record schema, currently 1
    authority_store_id: String,
    issuer_request_id: String,
    canonical_spawn_fingerprint: RetainedWorkerAdmissionCommitmentV1,
    orchestration_session_id: String,
    admission_authority_revision: u64,
    admission_authority_record_commitment: AuthorityObjectCommitmentV1,
    retained_participant_id: String,
    bootstrap_run_id: String,
    backend_id: String,
    protocol: String,
    world_binding: WorldBindingV1,
    current_policy_ref: PolicyRefV1,
    current_policy_revision: String,
    max_live_retained_workers: u64,
}

struct WorldWorkExecutionClaimDurableKeyV1 {
    supervisor_schema_version: u32, // exactly 1
    executions_by_acceptance_record_id_key: String,
}

enum AppliedDispatchPolicyPatchIdentityV1 {
    UnchangedParent,
    RestrictedWorldFs {
        patch_schema_version: u32, // exactly 1
        canonical_patch: ImmutableBytesMaterialV1,
        patch_hash: String,
    },
}

enum ImmutableBytesMaterialV1 {
    Inline {
        bytes_base64: String,
        byte_length: u64,
    },
    Durable {
        immutable_bytes_ref: ImmutableBytesRefV1,
    },
}

struct ImmutableBytesRefV1 {
    authority_store_id: String,
    object_ref: String,
    byte_length: u64,
    sha256: String,
}

struct DispatchPolicyCommitmentRefV1 {
    authority_store_id: String,
    commitment_id: String,
    exact_linkage_hash: String,
}

struct DispatchPolicyCommitmentLookupKeyV1 {
    authority_store_id: String,
    orchestration_session_id: String,
    request_id: String,
    subject: DispatchPolicyCommitmentSubjectKeyV1,
}

enum DispatchPolicyCommitmentSubjectKeyV1 {
    EphemeralWork {
        task_run_id: String,
    },
    RetainedWorkerLaunch,
    RetainedWorkerTurn {
        active_run_id: String,
        message_id: String,
    },
    RetainedWorkerFork {
        source_participant_id: String,
    },
}

enum RetainedWorkerCapLinkV1 {
    ThisCommitment,
    Existing {
        cap_ref: DispatchPolicyCommitmentRefV1,
    },
}

enum PolicyCommitmentStatusV1 {
    Immutable,
}

enum DispatchPolicyCommitmentIndexEntryV1 {
    FreshSpawnReserved {
        reservation_ref: DispatchPolicyCommitmentReservationRefV1,
    },
    Committed {
        reservation_ref: Option<DispatchPolicyCommitmentReservationRefV1>,
        fresh_spawn_validated_request_commitment: Option<FreshSpawnValidatedRequestCommitmentV1>,
        commitment_ref: DispatchPolicyCommitmentRefV1,
    },
}

struct DispatchPolicyCommitmentReservationRefV1 {
    authority_store_id: String,
    reservation_id: String,
    reservation_hash: String,
}

struct FreshSpawnValidatedRequestCommitmentV1 {
    schema_version: u32, // exactly 1
    algorithm: FreshSpawnRequestCommitmentAlgorithmV1, // exactly HmacSha256
    key_id: String,
    domain: String, // exactly substrate.e2.fresh-spawn-validated-request.v1
    digest_hex: String,
}

enum FreshSpawnRequestCommitmentAlgorithmV1 {
    HmacSha256,
}

struct AuthenticatedFreshSpawnReservationProofV1 {
    reservation_ref: DispatchPolicyCommitmentReservationRefV1,
    retained_participant_id: String,
    bootstrap_run_id: String,
    narrowing_attestation: Option<OpaqueFreshSpawnNarrowingAttestationV1>,
}

// Private, ephemeral E2 capability. It has no serializable fields or public constructor.
struct OpaqueFreshSpawnNarrowingAttestationV1;

struct DispatchPolicyCommitmentReservationV1 {
    schema_version: u32, // exactly 1
    authority_store_id: String,
    reservation_id: String,
    lookup_key: DispatchPolicyCommitmentLookupKeyV1,
    subject: DispatchPolicyCommitmentSubjectV1, // exactly RetainedWorkerLaunch
    validated_request_commitment: FreshSpawnValidatedRequestCommitmentV1,
    request_id: String,
    idempotency_key: String,
    orchestration_session_id: String,
    caller_participant_id: String,
    caller_backend_id: String,
    target_backend_id: String,
    world_id: String,
    world_generation: u64,
    retained_participant_id: String,
    bootstrap_run_id: String,
    parent_policy_ref: PolicyRefV1,
    parent_policy_revision: String,
    applied_patch: AppliedDispatchPolicyPatchIdentityV1,
    policy_snapshot_bytes: ImmutableBytesMaterialV1,
    policy_snapshot_ref: PolicySnapshotRefV1,
    policy_snapshot_hash: String,
    policy_snapshot_revision: String,
    reason: Option<String>,
    proposed_commitment_id: String,
    proposed_worker_cap: ProposedRetainedWorkerCapV1,
    created_revision: u64,
    created_at: Timestamp,
    reservation_hash: String,
}

struct ProposedRetainedWorkerCapV1 {
    commitment_id: String,
    policy_snapshot_ref: PolicySnapshotRefV1,
    policy_snapshot_hash: String,
    policy_snapshot_revision: String,
    retained_worker_cap_link: RetainedWorkerCapLinkV1, // exactly ThisCommitment
}
```

`commitment_id` is one `dpc_<lowercase UUIDv7>` allocated by the named authority store and reused
on exact retry. `exact_linkage_hash` is lowercase SHA-256 over UTF-8 canonical JSON of
`{"domain":"substrate.e2.dispatch-policy-commitment.v1","record":<record>}`, where `<record>` is
the complete `DispatchPolicyCommitmentV1` with only `exact_linkage_hash` omitted. Canonical JSON
recursively sorts object keys, preserves array order, emits the shown enum variant tags, and emits
absent optional values as `null`. Enums use standard externally tagged Serde JSON with the exact
case-sensitive identifiers shown (unit variants are strings); timestamps use the existing V1 JSON
timestamp encoding. Inline base64 must be canonical padded RFC 4648 with no whitespace. A durable
byte reference is valid only when it resolves through the bound authority store to immutable
bytes whose length and SHA-256 match the reference.

The record/index/linkage canonical JSON above does **not** define policy-snapshot bytes.
`policy_snapshot_bytes` are byte-for-byte the output of the landed E1
`serde_json::to_vec(PolicySnapshotV3)` path. `policy_snapshot_hash` is the exact existing E1
lowercase SHA-256 over those bytes. E2 must resolve the inline/ref bytes, decode them as the expected
`PolicySnapshotV3`, require `serde_json::to_vec(decoded) == policy_snapshot_bytes`, and reproduce
the exact E1 hash. B1's stored `current_policy_snapshot_hash`, E2's bytes/hash, and the linked
`policy_snapshot_ref` therefore identify one byte sequence. E2 must not recursively key-sort,
re-canonicalize, or otherwise replace that sequence, and it does not change `PolicySnapshotV3`, E1
serialization, schema 3, or existing B1 hashes. Missing bytes, a missing immutable reference, schema
mismatch, or any hash, revision, subject, acceptance, observation, parent, patch, cap, backend, or
world mismatch fails before persistence or use.

`WorldWorkExecutionClaimLinkV1` links the exact source-owned B2.1
`WorldWorkExecutionClaimV1`, not a six-field projection. `claim_preimage` is the exact output of the
landed B2.1 `host_session_authority::canonical_json::to_vec(WorldWorkExecutionClaimV1)` path
returned by the read-only supervisor projection at link time. `claim_linkage_hash` is lowercase
SHA-256 over UTF-8 canonical JSON of
`{"domain":"substrate.e2.world-work-execution-claim-link.v1","claim":<exact-claim>}`. Its durable
key is the exact `WorldWorkExecutionSupervisorStateV1.executions_by_acceptance_record_id` key and
must equal both the B1 acceptance ID and the claim's `acceptance_record_id`; the listed claim and
observer revisions must equal the preimage, and the hash must reproduce. This preserves the
exact historical preimage when B2.1 later advances its independently owned observer epoch/claim
revision. E2 neither requires nor infers `resumable`. No E2 subject currently requires an
interruption or cursor reference; if a later subject does, it must link that separately source-owned
B2.1 durable identity rather than add fields to or reinterpret `WorldWorkExecutionClaimV1`.

Every `authority_store_id` nested in a B1/B3.2a owner record, policy/cap ref, or immutable byte ref
must equal the record's top-level `authority_store_id`. A `DispatchPolicyCommitmentRefV1` resolves
only by `(authority_store_id, commitment_id)` and is valid only when the stored record recomputes
the referenced `exact_linkage_hash`.

`reservation_hash` is lowercase SHA-256 over UTF-8 canonical JSON of
`{"domain":"substrate.e2.dispatch-policy-commitment-reservation.v1","reservation":<reservation>}`,
with only `reservation_hash` omitted. It is an E2 registry hash; it does not redefine or wrap
`policy_snapshot_hash`. The same distinction applies to `exact_linkage_hash`, patch hashes, claim
linkage hashes, and fork-request hashes: each authenticates its stated deterministic preimage, while
only the unwrapped E1 `serde_json::to_vec(PolicySnapshotV3)` bytes define the policy snapshot hash.

The reservation is an independently stored immutable E2 object. Its ref resolves only by
`(authority_store_id, reservation_id)` and only when the stored object reproduces
`reservation_hash`; moving the request/subject index from `FreshSpawnReserved` to `Committed` never
deletes or replaces it. A launch commitment requires the same non-optional reservation ref in the
committed index entry and `fresh_spawn_reservation_ref`, plus the same non-optional
`fresh_spawn_validated_request_commitment` in the committed index entry, reservation, and final
record; every other subject requires those index and record fields absent.

`FreshSpawnValidatedRequestCommitmentV1` prevents request-material drift before B3.2a exists. Its
digest is `HMAC-SHA-256(K, u64_be(len(D)) || D || u64_be(len(R)) || R)`, where `K` is the E2
registry's durable request-commitment key, `D` is exactly the UTF-8 byte sequence
`substrate.e2.fresh-spawn-validated-request.v1`, `R` is exactly the landed B3.2a
`host_session_authority::canonical_json::to_vec(CanonicalValidatedSpawnRequestV1)` byte sequence,
both lengths are unsigned 64-bit big-endian byte lengths, and `digest_hex` is exactly 64 lowercase
hexadecimal characters. `R` covers every validated field, including action, mode, backend/world,
idempotency, and complete payload/prompt. Only the key ID and digest persist; E2 stores no
prompt/payload preimage or raw secret-derived SHA-256. The private
`AuthenticatedFreshSpawnReservationProofV1` constructor resolves and authenticates the immutable
reservation and privately recomputes this commitment from the exact request being handed to
B3.2a. Only after that comparison succeeds does it return an opaque authenticated reservation
capability/ref plus the stored identities. The HMAC/key ID/digest and request bytes do not cross
into B3.2a. A changed request field therefore conflicts before B3.2a after any reservation-era
crash.

The private constructor emits `narrowing_attestation: Some` only after it additionally authenticates
all of the following from that same reservation: `applied_patch` is a nonempty
`RestrictedWorldFs` E1 patch, the referenced parent has `allow_capability_narrowing == true`, the
effective snapshot bytes/ref/hash are the exact E1 result, the full validated-request commitment
matches, and both preallocated identities match. `UnchangedParent` and every empty patch produce
`None`. This distinct opaque, ephemeral capability is the only condition under which B3.2a may
accept `allow_capability_narrowing == true`; B3.2a cannot construct it and receives no patch,
request-commitment, or snapshot bytes through it.

The durable unique retry index is `DispatchPolicyCommitmentLookupKeyV1`. The subject discriminator
allows one request to own both a retained-turn commitment and a later fork commitment. The
fresh-Spawn key excludes its first-writer-generated worker/bootstrap identities, and the fork key
excludes its first-writer-generated child identity. Non-Spawn subjects may still atomically publish
their complete immutable record. Fresh Spawn uses the two-state reservation/commit protocol below.
Every retry resolves the index, reuses every stored generated value, and rejoins only when the
complete caller-supplied canonical input matches. Any changed input, alternate generated identity,
occupied-key mismatch, or index/record tear fails closed without a second record or worker launch.

## Subject and composition rules

1. **Ephemeral work:** resolve `current parent AND dispatch patch`. The subject requires the exact
   immutable B1 acceptance record and exact B2.1 execution-claim link. Worker-cap fields are
   absent.
2. **Worker launch:** resolve `parent at spawn AND spawn patch`. Before B3.2a, the subject requires
   an exact authenticated E2 reservation containing the proposed cap plus stable worker/bootstrap
   identities. After B3.2a succeeds, the committed record requires that exact stable B3.2a
   retained-admission identity and the immutable reservation ref because the frozen B1 schema does
   not own Spawn acceptance.
   `retained_worker_cap_link` is `ThisCommitment`; `source_worker_cap_ref` and the B2.1
   execution-claim link are absent. After the record's exact linkage hash is computed, its cap identity is the
   resulting `DispatchPolicyCommitmentRefV1` without a self-hash cycle.
3. **Future retained turn:** resolve `current parent now AND immutable worker cap AND turn patch`.
   The exact B1 retained-turn acceptance and B2.1 execution claim are required, and
   `retained_worker_cap_link` is `Existing` with the exact already-verified launch/fork cap ref;
   `source_worker_cap_ref` is absent.
4. **Fork:** resolve `current parent now AND immutable source-worker cap AND fork patch`. The exact
   source cap and E2-owned `ForkDispatch` link are required; B3.2a is forbidden because its frozen
   authority covers only fresh Spawn. The fork link hashes the complete strict
   `ValidatedWorldDispatchRequestV1` after typed E1 carrier translation. `source_worker_cap_ref`
   carries the source cap's exact ref and `retained_worker_cap_link` is `ThisCommitment` for the new
   child cap; the B2.1 execution-claim link is absent. Exact retry reuses the record's child participant
   and bootstrap identities; a conflicting request hash or retry-local child substitution fails
   closed.
5. A missing patch uses `UnchangedParent`; it is not represented as missing identity. Any supplied
   patch must be the exact authenticated E1 `DispatchPolicyNarrowingPatchV1` canonical JSON bytes
   or immutable byte ref. `patch_hash` is lowercase SHA-256 over UTF-8 canonical JSON of
   `{"domain":"substrate.e2.dispatch-policy-patch.v1","patch":<exact-patch>}`, using the same
   recursive key-sorting and array-order rules. This E2 hash does not replace or alter any E1
   identity.

The subject-specific authority link does not widen B1. `WorldWorkAcceptanceRecordV1`
continues to own only ephemeral-task and retained-turn acceptance; B3.2a continues to own Spawn
admission and explicitly does not own fork. E2's fork-dispatch link authenticates only the fork
request-to-cap commitment and creates no admission/lifecycle authority. This avoids a reverse
dependency or a fabricated B1/B3.2a identity while requiring pre-existing acceptance/admission
truth wherever the subject has one.

Validation fetches the exact linked B1 record revision or the B3.2a record at its stable registry
key from the named authority store. A B1 link requires equality of
request/session/caller/backend/world/subject and runtime-acceptance
identity; its `current_policy_snapshot_ref` and `current_policy_revision` must equal this record's
parent policy reference/revision, while its accepted `current_policy_snapshot_hash` must equal this
record's effective `policy_snapshot_hash`. A B3.2a link projects every
`RetainedWorkerAdmissionRecordV1` field except mutable `state` and `record_revision`, requires the
source record's existing fingerprint verification, and exact-links the registration carried by
`PreTransportNonterminal` or every later registration-bearing state. `stable_identity_hash` is
lowercase SHA-256 over UTF-8 canonical JSON of
`{"domain":"substrate.e2.b3-2a-stable-admission-link.v1","stable_source_fields":<fields>,"registration":<registration>}`.
The live source record must reproduce that preimage/hash and equality of issuer request, session,
backend, world, worker/bootstrap subject, fingerprint, launch-time parent policy ref/revision, and
registration. `SlotReserved`, `AuthorityRegistrationHead`, and `RejectedBeforeRegistration` cannot
publish the E2 commitment. Later B3.2a lifecycle transitions do not invalidate the link because the
mutable state/revision is deliberately excluded while the stable source fields and registration
remain exact. These comparisons link
immutable owner truth without rewriting either owner schema. A `ForkDispatch` link is valid only
for a fork subject and binds `request_id`, `idempotency_key`, source/child/bootstrap identity,
backend/world, parent/source cap, and patch through the record's exact linkage hash. Its nested
request hash is lowercase SHA-256 over canonical JSON of
`{"domain":"substrate.e2.fork-dispatch-request.v1","validated_dispatch_request":<request>}` using
the same recursive object-key and array-order rules.

## Immutability and temporal rules

- Parent narrowing affects future work, turns, and forks.
- Parent broadening never widens an existing worker because every future composition intersects
  the current parent with the immutable worker cap.
- An accepted active-run commitment is immutable. Parent changes do not rewrite it.
- Emergency invalidation is an explicit audited cancel/revoke record that references the
  commitment; it never edits or replaces commitment bytes.
- Missing, incompatible, or hash-invalid cap material fails closed before continue or fork.
- `created_revision`, `application_revision`, and `exact_linkage_hash` are equality-only facts.
  They cannot be inferred from timestamps, process state, the current parent, or a later record.

## Retained-target resolution under parent drift

The existing retained-target resolver continues to own and authenticate retained identity;
session, participant, backend, and world binding; B3.2a admission and retained-worker identity; and
the immutable launch-time policy/cap reference. It must return two distinct authenticated inputs:

1. the immutable worker launch cap bytes/ref/hash exact-linked to the launch/fork commitment; and
2. the independently resolved current parent policy/ref/revision.

The launch-time policy reference/revision is not required to equal the current parent. The existing
checks that require B3.2a admission or the retained-worker object graph's launch policy to equal
`ResolvedCurrentAuthorityV1.current_policy` are replaced only by exact launch-policy/cap linkage to
the E2 committed record. All non-policy identity, admission, ancestry, routability, backend, world,
descriptor, resume-handle, registration, and object-graph checks remain unchanged. E2 then computes
Continue as `current parent now AND immutable worker cap AND turn patch`, and Fork as
`current parent now AND immutable source-worker cap AND fork patch`. A current-parent narrowing may
further restrict future work; a broadening cannot widen the immutable cap. A worker without
verifiable immutable cap bytes/ref/hash returns typed `UnsupportedLegacyState` before Continue or
Fork.

This is not authority for unrelated StateStore persistence, compatibility, routing, registration,
lifecycle, or retained-worker changes. The resolver still owns routing authentication; E2 only
consumes its two separately authenticated policy inputs.

## Fresh-Spawn reservation, admission, and publication

For `RetainedWorkerLaunch`, the E2 request/subject index is a pre-admission authority reservation:

1. A first-writer CAS allocates and durably publishes one complete
   `DispatchPolicyCommitmentReservationV1`, including a keyed commitment over the complete validated
   request plus request/idempotency/subject identity;
   parent ref/revision; complete E1 patch bytes/ref/hash; exact E1 snapshot bytes/ref/hash; proposed
   worker cap and commitment identity; stable worker/bootstrap identities; reason; and
   store/session/caller/backend/world bindings.
2. Reservation object, index publication, file, and directory `fsync` complete before any B3.2a
   admission call. E2 completes the full-request commitment comparison privately, then passes only
   an opaque authenticated reservation capability/ref plus the exact preallocated
   worker/bootstrap identities into B3.2a. The E2 path through
   `RetainedWorkerRuntime::reserve_admission_slot` consumes those identities instead of generating
   replacements and accepts only the E2-constructed capability before slot allocation. B3.2a does
   not receive, recompute, or compare the E2 request commitment. The unchanged B3.2a
   fingerprint continues to bind its existing plan and those exact identities; it does not carry
   patch, snapshot, cap, or reservation material.
3. B3.2a retains participant admission, registration, lifecycle, and routability authority. The E2
   reservation grants none of them and does not modify or version B3.2a's frozen fingerprint to
   carry patch material.
4. After B3.2a succeeds, one E2 CAS exact-links its stable Fresh-Spawn admission identity, retains
   the immutable reservation ref, and changes the index entry from `FreshSpawnReserved` to
   `Committed`, publishing the immutable record. The reservation, committed index, and final record
   must carry and equality-project the same non-optional full validated-request commitment. The
   reservation and final record equality-project their E2-owned request/idempotency, subject,
   generated identities, parent, patch, snapshot, proposed commitment/cap, and backend/world
   fields; the committed index carries only its defined reservation, request-commitment, and final
   commitment references. The stable B3.2a admission projection equality-projects only its
   overlapping source-owned request/session/backend/world/worker/bootstrap/launch-parent fields
   plus exact registration. It never carries E2 request-commitment, patch, snapshot, cap, or final
   commitment material. The reservation's proposed commitment ID, the final record's commitment
   ID, and the committed index's commitment ref must identify the same record. Any mismatch fails
   closed.
5. Identical retry exact-joins either state. Changed validated request/payload, patch, parent, cap,
   subject, bindings, or generated worker/bootstrap identity conflicts before B3.2a.
6. Crash before reservation publication leaves no admissible E2 identity; retry performs the same
   first-writer CAS. Crash after durable reservation but before B3.2a resumes from that reservation.
   Crash after B3.2a but before E2 publication reuses the reservation's exact identities,
   exact-joins B3.2a, and then publishes the commitment. Crash after commitment publication joins
   the committed record.
7. Once E2 is activated, a production B3.2a admission without the required exact E2 reservation is
   unsupported. No retry reconstructs patch, snapshot, or cap material from current parent state.

The reservation and commitment are E2 policy authority only. They cannot create a participant,
grant admission, make a worker routable, settle lifecycle, or substitute for B3.2a truth.

## Persistence, reporting, and downstream composition wall

For accepted ephemeral work and retained turns, B1 acceptance and the B2.1 claim become durable
under their existing owners, then E2 atomically persists this record with those exact links before
the foreground or tool surface reports acceptance. For fresh worker launch, the complete E2
reservation is durable before B3.2a; after exact admission, E2 atomically publishes the admission
link and immutable cap before launch/routability success is reported. For
fork, E2 atomically persists the E2-owned fork request/cap link before child-launch success is
reported, without claiming B3.2a admission. An exact retry rejoins the existing identical record;
conflicting bytes or linkage fail closed.

Later receipt owners must store `DispatchPolicyCommitmentRefV1` and equality-project the existing
snapshot/cap fields from this record. Later retained-manifest owners must consume the same ref for
their worker-cap fields. The E2 record is independently complete without D1 or E3 material and is
not a partial `RetainedWorkerManifestV1`. A full retained manifest is complete only after its
existing owners also supply the D1 execution-envelope and E3 config-projection identities; a
record omitting either must not validate or be reported as a complete manifest.

## `E2-RM` — authenticated accepted-work receipt-material projection prerequisite

**Stable ID:** `E2-RM`

**Status:** specified prerequisite; not admitted, not dispatched, and not implemented

The B2.2 admission review found that the completed E2 store preserves the immutable material needed
to reproduce the original foreground receipt, but its existing authenticated lookup accepts only an
already-known `DispatchPolicyCommitmentRefV1`. After response loss or restart, a B2.2 caller that
holds exact B1 acceptance and request/subject identity cannot recover that ref or the complete
historic receipt material. Looking at the current B2.1 claim or current parent policy would be an
invalid reconstruction because either may have advanced since E2 committed the accepted work.

`E2-RM` is the smallest separately bounded prerequisite that may close that gap. It remains owned
by the E2 commitment boundary and is not B2.2 behavior. A later fresh admission may authorize only
the following crate-internal, read-only responsibility in
`crates/shell/src/execution/agent_runtime/dispatch_policy_commitment.rs`:

```rust
pub(crate) fn resolve_accepted_work_receipt_material(
    authority: &HostSessionAuthority,
    exact_request_subject_key: &DispatchPolicyCommitmentLookupKeyV1,
    expected_b1_acceptance: &WorldWorkAcceptanceRecordV1,
) -> Result<AcceptedWorkReceiptMaterialResolutionV1, DispatchPolicyCommitmentError>;

pub(crate) enum AcceptedWorkReceiptMaterialResolutionV1 {
    Resolved(AuthenticatedAcceptedWorkReceiptMaterialV1),
    UnsupportedLegacyState {
        exact_request_subject_key: DispatchPolicyCommitmentLookupKeyV1,
        reason: AcceptedWorkReceiptMaterialLegacyReasonV1,
    },
}

pub(crate) enum AcceptedWorkReceiptMaterialLegacyReasonV1 {
    MissingExactHistoricE2Commitment,
    UnsupportedHistoricE2CommitmentSchema,
}

pub(crate) struct AuthenticatedAcceptedWorkReceiptMaterialV1 {
    commitment_ref: DispatchPolicyCommitmentRefV1,
    exact_request_subject_key: DispatchPolicyCommitmentLookupKeyV1,
    subject: DispatchPolicyCommitmentSubjectV1,
    idempotency_key: String,

    authority_store_id: String,
    authority_revision_observed: u64,
    orchestration_session_id: String,
    caller_participant_id: String,
    caller_backend_id: String,
    target_backend_id: String,
    world_id: String,
    world_generation: u64,

    acceptance_record_id: String,
    acceptance_record_revision: u64,
    b1_authority_link: PolicyCommitmentAuthorityLinkV1,
    accepted_work_identity: AcceptedWorldWorkIdentityV1,
    runtime_acceptance: RuntimeAcceptanceEvidenceV1,
    host_transition_correlation: Option<HostTransitionWorkCorrelationV1>,
    accepted_at: DateTime<Utc>,

    execution_claim: AuthenticatedAcceptedWorkExecutionClaimV1,

    policy_snapshot_bytes: Vec<u8>,
    policy_snapshot_ref: AuthorityObjectRefV1,
    policy_snapshot_hash: String,
    policy_snapshot_revision: String,
    policy_reason: Option<String>,

    retained_worker_cap: Option<AuthenticatedAcceptedWorkRetainedCapV1>,
}

pub(crate) struct AuthenticatedAcceptedWorkExecutionClaimV1 {
    durable_key: WorldWorkExecutionClaimDurableKeyV1,
    acceptance_record_id: String,
    acceptance_record_revision: u64,
    claim_revision: u64,
    observer_instance_id: String,
    observer_epoch: u64,
    canonical_preimage: Vec<u8>,
    linkage_hash: String,
}

pub(crate) struct AuthenticatedAcceptedWorkRetainedCapV1 {
    retained_participant_id: String,
    cap_ref: DispatchPolicyCommitmentRefV1,
    cap_exact_linkage_hash: String,
    authority_store_id: String,
    orchestration_session_id: String,
    caller_participant_id: String,
    caller_backend_id: String,
    target_backend_id: String,
    world_id: String,
    world_generation: u64,
    cap_policy_snapshot_bytes: Vec<u8>,
    cap_policy_snapshot_ref: AuthorityObjectRefV1,
    cap_policy_snapshot_hash: String,
    cap_policy_snapshot_revision: String,
}
```

These are responsibility-level shapes, not authorization to add serialized records. The resolver
and result types are visible only inside the `shell` crate via `pub(crate)`; all fields remain
private and B2.2 can later read them only through `pub(crate)` immutable accessors. The result is an
opaque, internally constructed in-memory projection. It is never persisted, never accepted from
transport, and has no externally public or mutable constructor. Names may change during a later
fresh admission only if the same ownership, inputs, outputs, visibility, and validation remain
exact.

### Exact authentication and validation order

One resolver transaction must perform all of the following before returning `Resolved`:

1. authenticate `authority`, open its E2 store, and require the supplied key's
   `authority_store_id` to equal the opened store;
2. validate `expected_b1_acceptance` under the existing B1 schema, then require its store,
   request, session, accepted-work subject, caller, caller backend, target backend, world, and
   generation to equal the supplied exact request/subject key and the indexed E2 record;
3. recompute the durable request/subject index digest, require exactly one byte-identical stored
   key, require its `Committed` variant with no Fresh-Spawn reservation fields, and require its
   exact `DispatchPolicyCommitmentRefV1` to name the indexed immutable record;
4. validate the complete E2 registry graph, record schema/status, record/reference identity, and
   domain-separated `exact_linkage_hash`; an unindexed, duplicated, torn, ambiguous, or conflicting
   record is invalid rather than legacy-compatible;
5. require an `EphemeralWork` or `RetainedWorkerTurn` subject and an exact stored B1 authority link;
   compare acceptance record ID/revision and the complete stored `RuntimeAcceptanceEvidenceV1` to
   `expected_b1_acceptance`, and compare every overlapping authority, request, subject, session,
   caller, backend, world, generation, work-identity, correlation, and snapshot field;
6. resolve the E2-preserved `WorldWorkExecutionClaimV1` preimage, require byte-identical canonical
   re-encoding, recompute its domain-separated linkage hash, verify its immutable identity and
   durable `executions_by_acceptance_record_id` key, exact-join every overlapping B1 field, and
   return the exact preserved acceptance ID/revision, claim revision, observer instance/epoch,
   canonical preimage, durable key, and linkage hash; never consult or compare the current B2.1
   claim revision, cursor, observer, journal, or terminal state;
7. resolve the historic E1 `PolicySnapshotV3` bytes, require byte-identical E1 reserialization,
   reproduce the unwrapped E1/B1 SHA-256, and require exact ref/hash/revision equality with both the
   E2 record and expected B1 acceptance. The stored reason is authenticated by the complete E2
   linkage hash and must equal the reason in the stored authenticated patch; `UnchangedParent`
   requires no reason. Current-parent lookup or policy recomputation is forbidden; and
8. for `RetainedWorkerTurn`, require the exact `Existing { cap_ref }`, resolve that ref through the
   same store and participant/session indexes, authenticate the complete launch/fork cap record and
   its `exact_linkage_hash`, require the cap record's authority store, retained participant,
   orchestration session, target backend, world ID, and world generation to equal the accepted-turn
   record/result, require its caller participant/backend to exact-join the same authority lineage,
   validate its canonical cap snapshot bytes/ref/hash/revision, and expose those bindings plus the
   exact ref and hash. `EphemeralWork` requires all cap fields absent.

`MissingExactHistoricE2Commitment` is permitted only after one read transaction proves both that
the exact request/subject index is absent and that an exhaustive E2 registry scan contains no V1
index occupancy, digest collision, matching record, or orphaned record for the supplied key,
expected B1 acceptance ID/revision, or accepted-work identity. A conclusively recognized older
schema that cannot contain the exact record may return `UnsupportedHistoricE2CommitmentSchema`
only under the same no-V1-footprint proof. Any partial V1 footprint—including an index without its
record, a record without its exact index, conflicting index bytes at the digest, or a matching or
orphaned V1 record—is corruption, not legacy absence. Missing material inside a claimed V1 record,
malformed or unknown bytes, an index/record tear, ambiguous occupancy, or any corrupt, cross-store,
cross-request, cross-subject, cross-session, cross-caller, cross-backend, cross-world,
cross-generation, conflicting, or hash-invalid material is a fail-closed error and must never be
downgraded to `UnsupportedLegacyState`.

The projection is behavior-neutral and equality-only. It performs no persistence, mutation,
reconciliation, cleanup, backfill, migration, current-parent inference, policy recomputation,
receipt construction, B1/B2.1 mutation, or caller return. Identical retries after crash, response
loss, shell restart, B2.1 observer/claim advancement, or parent-policy drift return the same
historic projection. B2.2 may later consume that result unchanged, but cannot construct, mutate,
repair, reinterpret, or supplement it.

### Later implementation fence

A later explicit `E2-RM` dispatch may edit only
`crates/shell/src/execution/agent_runtime/dispatch_policy_commitment.rs`: the crate-internal
result/error types, the resolver, strictly necessary private helpers and immutable accessors, and
colocated focused tests. No module/export wiring change is expected or authorized because
`agent_runtime::dispatch_policy_commitment` is already declared `pub(crate)`; a later discovered
wiring need requires a new authority correction before editing another path. No caller integration
is in this fence. In particular, no StateStore, supervisor, B1/B2.1,
orchestrator, REPL, transport, receipt, manifest, policy, retained-runtime, world-service, product
surface, or non-colocated test file may change. The prerequisite may not alter an existing record,
registry, index, canonical byte sequence, hash domain, persistence transaction, recovery path, or
runtime branch.

Focused tests must prove ephemeral and retained-turn success; exact retry after reopen; byte-equal
results after response loss, current B2.1 observer/claim advancement, and current-parent narrowing
or broadening; and retained-cap authentication. The negative matrix must cover missing index,
wrong index variant, duplicated/ambiguous occupancy, missing record, altered ref/linkage hash,
changed expected B1 acceptance/runtime evidence, claim-key/preimage/hash substitution, snapshot
bytes/ref/hash/revision/reason mismatch, missing or substituted retained cap, cap-to-turn store,
participant, session, caller, caller-backend, target-backend, world, and generation substitution,
and every accepted-work store, request, subject, session, caller, backend, world, and generation
substitution. Separate
tests must prove that complete absence of both the exact index and every related V1 footprint, and
a recognized unsupported historic schema with no V1 footprint, return `UnsupportedLegacyState`;
an index-only footprint, record-only footprint, digest collision, orphan V1 record, or other corrupt
or partially present V1 material must fail closed.

The Linux acceptance wall for that later implementation uses the exact admission baseline recorded
as `e2_rm_base` and requires these commands and outputs, in order:

```bash
test -n "${E2_RM_ADMISSION_BASELINE:?set the recorded 40-hex admission baseline}"
e2_rm_base=$(git rev-parse --verify "$E2_RM_ADMISSION_BASELINE^{commit}")
test "$e2_rm_base" = "$E2_RM_ADMISSION_BASELINE"
test "$(git rev-parse HEAD)" = "$e2_rm_base"
set -o pipefail
uname -a
rustc -vV
cargo fmt --all -- --check
cargo test -p shell --lib dispatch_policy_commitment::tests -- --nocapture
cargo clippy -p shell --all-targets -- -D warnings
cargo test -p shell --lib -- --nocapture
cargo build -p substrate --bin substrate --bin substrate-shim
cargo build --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace -- --nocapture
target/debug/substrate world doctor --json | tee /tmp/e2-rm-world-doctor.json
target/debug/substrate shim doctor --json | tee /tmp/e2-rm-shim-doctor.json
target/debug/substrate health --json | tee /tmp/e2-rm-health.json
jq -e . /tmp/e2-rm-world-doctor.json
jq -e . /tmp/e2-rm-shim-doctor.json
jq -e . /tmp/e2-rm-health.json
git diff --check "$e2_rm_base" --
git diff --name-only "$e2_rm_base" --
git diff --name-only --diff-filter=A "$e2_rm_base" --
git diff --name-only "$e2_rm_base" -- '*.md'
rg -n 'resolve_accepted_work_receipt_material\(' crates/shell/src --glob '*.rs'
git status --short
```

The changed-path output must contain exactly
`crates/shell/src/execution/agent_runtime/dispatch_policy_commitment.rs`; the added-path and changed-
Markdown outputs must both be empty, making new-file whitespace/EOF and changed-document
link/fragment validation exact zero-input checks. The `rg` output must contain only the definition
and colocated `#[cfg(test)]` call sites in that file—zero production callers—and `git status
--short` must name only that tracked file before commit. The recorded focused-test output must name
every positive, crash/retry, legacy, corruption, and substitution case above. Formatting, both
builds, both clippy commands, the focused tests, the full shell library, and the full workspace test
must all exit zero; this prerequisite grants no inherited-failure or baseline-differential waiver.
Each source-built doctor must exit zero, emit parseable JSON, and have its complete captured output
attached to the completion evidence. Any doctor failure or non-pass/needs-attention diagnostic is
completion-blocking unless a later fresh authority record explicitly accepts an exact-baseline,
same-host/toolchain differential; `E2-RM` itself pre-authorizes no such exception. No live smoke can
substitute for the unit-level historical-material matrix, and no live behavior change is expected
because the unit remains unintegrated.

The review sequence is one fresh independent gpt-5.4 Extra High full-candidate review, remediation
of every finding inside the unchanged fence, and a different fresh final review of the remediated
complete candidate. Completion evidence must record the exact baseline/commit/tree/parent,
changed paths, patch and file SHA-256 values, command outputs, Linux identity, named test matrix,
proof of zero production call sites, review prompts/answers or durable review records, and a final
`CLEAN` verdict. Landing `E2-RM` does not admit B2.2.

B2.2 requires a fresh admission and explicit dispatch after `E2-RM` is separately admitted,
implemented, review-clean, committed, and identified by exact commit/tree in the B2.2 admission
record. That fresh review must re-evaluate B2.2 against the then-live E2-RM contract, consume only
`AuthenticatedAcceptedWorkReceiptMaterialV1`, keep receipt construction/return inside B2.2, and
retain B3.2, B4, C2, C3, D1, E3, and all unrelated authority outside its fence.

## Mixed-version fail-closed contract

```rust
enum PolicyCommitmentCompatibilityResultV1 {
    Compatible {
        commitment_ref: DispatchPolicyCommitmentRefV1,
    },
    UnsupportedLegacyState {
        retained_participant_id: String,
        reason: PolicyCommitmentCompatibilityReasonV1,
    },
}

enum PolicyCommitmentCompatibilityReasonV1 {
    MissingCanonicalCapBytes,
    MissingImmutableCapBytesRef,
    CapHashMismatch,
    UnsupportedCapSchema,
    CapSubjectOrLinkageMismatch,
}
```

Workers created without recoverable canonical cap bytes, or an immutable byte reference resolving
to bytes with the matching hash, are unsupported for E2 continue and fork. No cap may be inferred
from the current parent, and no newer or broader parent may reconstruct historical worker
authority. Continue and fork return the typed `UnsupportedLegacyState` result before dispatch.
Existing work may proceed only along an already-authoritative safe-completion path that does not
need the missing cap, such as consuming existing exact terminal or audited cancellation truth.
This contract authorizes no migration, backfill, synthetic cap, or parent-derived repair.
