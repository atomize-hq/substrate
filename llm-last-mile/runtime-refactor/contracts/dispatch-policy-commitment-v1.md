**Kind:** contract
**Stable ID:** `dispatch-policy-commitment-v1`
**Status:** canonical E2 authority correction; implementation undispatched
**Canonical for:** additive `DispatchPolicyCommitmentV1` persisted record, exact subject/linkage rules, immutable policy/cap semantics, downstream receipt/manifest consumption, and mixed-version fail-closed behavior
**Authority scope:** documentation-only E2 contract correction; no E2 admission, dispatch, implementation, completion, receipt construction, retained-manifest construction, migration, or synthetic-cap authority
**Supersedes:** only E2 statements that require E2 to construct a final receipt or complete `RetainedWorkerManifestV1`; omit the real dispatch/tool-translation carrier needed to consume E1 narrowing; require a retained worker's launch policy to equal the current parent; place fresh-Spawn E2 persistence after B3.2a admission; define a second snapshot canonicalization; or require a synthetic `SupervisorObservationClaimV1`/`resumable` field
**Superseded by:** none
**Projection consumers:** [`../slices/e2-policy-commitments-on-work-and-workers.md`](../slices/e2-policy-commitments-on-work-and-workers.md), [`active-ephemeral-task-receipt-v1.md`](active-ephemeral-task-receipt-v1.md), [`active-retained-turn-receipt-v1.md`](active-retained-turn-receipt-v1.md), [`retained-worker-manifest-v1.md`](retained-worker-manifest-v1.md), [`../gates/final-receipt-immutable-policy-snapshot-v3-acceptance.md`](../gates/final-receipt-immutable-policy-snapshot-v3-acceptance.md)

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
