**Kind:** contract
**Stable ID:** `dispatch-policy-commitment-v1`
**Status:** canonical E2 authority correction; implementation undispatched
**Canonical for:** additive `DispatchPolicyCommitmentV1` persisted record, exact subject/linkage rules, immutable policy/cap semantics, downstream receipt/manifest consumption, and mixed-version fail-closed behavior
**Authority scope:** documentation-only E2 contract correction; no E2 admission, dispatch, implementation, completion, receipt construction, retained-manifest construction, migration, or synthetic-cap authority
**Supersedes:** only E2 statements that require E2 to construct a final receipt or complete `RetainedWorkerManifestV1`, or that omit the real dispatch/tool-translation carrier needed to consume E1 narrowing
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
    observation_claim_link: Option<SupervisorObservationClaimLinkV1>,

    parent_policy_ref: PolicyRefV1,
    parent_policy_revision: String,
    applied_patch: AppliedDispatchPolicyPatchIdentityV1,

    canonical_policy_snapshot: CanonicalPolicySnapshotMaterialV1,
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
        issuer_request_id: String,
        canonical_spawn_fingerprint: RetainedWorkerAdmissionCommitmentV1,
        retained_participant_id: String,
        bootstrap_run_id: String,
        admission_record_revision: u64,
    },
    ForkDispatch {
        canonical_validated_dispatch_request_sha256: String,
    },
}

struct SupervisorObservationClaimLinkV1 {
    acceptance_record_id: String,
    observation_claim: SupervisorObservationClaimV1,
}

enum AppliedDispatchPolicyPatchIdentityV1 {
    UnchangedParent,
    RestrictedWorldFs {
        patch_schema_version: u32, // exactly 1
        canonical_patch: CanonicalPolicyBytesV1,
        patch_hash: String,
    },
}

enum CanonicalPolicySnapshotMaterialV1 {
    Inline {
        canonical_snapshot_bytes_base64: String,
        byte_length: u64,
    },
    Durable {
        immutable_bytes_ref: ImmutablePolicyBytesRefV1,
    },
}

enum CanonicalPolicyBytesV1 {
    Inline {
        canonical_bytes_base64: String,
        byte_length: u64,
    },
    Durable {
        immutable_bytes_ref: ImmutablePolicyBytesRefV1,
    },
}

struct ImmutablePolicyBytesRefV1 {
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
    RetainedWorkerLaunch {
        retained_participant_id: String,
    },
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
bytes whose length and SHA-256 match the reference. `policy_snapshot_hash` is the SHA-256 of the
exact canonical `PolicySnapshotV3` bytes; the resolved `policy_snapshot_ref` commitment must
authenticate those same bytes and hash. Missing bytes, a missing immutable reference, schema
mismatch, or any hash, revision, subject, acceptance, observation, parent, patch, cap, backend, or
world mismatch fails before persistence or use.

`SupervisorObservationClaimLinkV1.observation_claim` is the exact complete B2.1
`SupervisorObservationClaimV1`, including its resumability value; the wrapper only binds that claim
to the immutable B1 acceptance ID.

Every `authority_store_id` nested in a B1/B3.2a owner record, policy/cap ref, or immutable byte ref
must equal the record's top-level `authority_store_id`. A `DispatchPolicyCommitmentRefV1` resolves
only by `(authority_store_id, commitment_id)` and is valid only when the stored record recomputes
the referenced `exact_linkage_hash`.

The durable unique retry index is `DispatchPolicyCommitmentLookupKeyV1`. The subject discriminator
allows one request to own both a retained-turn commitment and a later fork commitment, while the
fork key deliberately excludes its first-writer-generated child identity. Lookup/reservation
precedes commitment UUID, fork-child, revision, and timestamp allocation. The first writer uses one
CAS transaction to allocate those stable values and atomically publish the index plus complete
immutable record. A retry resolves the index, reuses every stored generated value, and rejoins only
when the complete caller-supplied canonical input—including idempotency key, bindings, owner links,
parent, patch, snapshot, and existing cap refs—matches. Any changed input, alternate generated
identity, occupied-key mismatch, or index/record tear fails closed without a second record or
worker launch.

## Subject and composition rules

1. **Ephemeral work:** resolve `current parent AND dispatch patch`. The subject requires the exact
   immutable B1 acceptance record and exact B2.1 observation-claim link. Worker-cap fields are
   absent.
2. **Worker launch:** resolve `parent at spawn AND spawn patch`. The subject requires the existing
   B3.2a retained-admission identity because the frozen B1 schema does not own Spawn acceptance.
   `retained_worker_cap_link` is `ThisCommitment`; `source_worker_cap_ref` and the B2.1 observation
   link are absent. After the record's exact linkage hash is computed, its cap identity is the
   resulting `DispatchPolicyCommitmentRefV1` without a self-hash cycle.
3. **Future retained turn:** resolve `current parent now AND immutable worker cap AND turn patch`.
   The exact B1 retained-turn acceptance and B2.1 observation claim are required, and
   `retained_worker_cap_link` is `Existing` with the exact already-verified launch/fork cap ref;
   `source_worker_cap_ref` is absent.
4. **Fork:** resolve `current parent now AND immutable source-worker cap AND fork patch`. The exact
   source cap and E2-owned `ForkDispatch` link are required; B3.2a is forbidden because its frozen
   authority covers only fresh Spawn. The fork link hashes the complete strict
   `ValidatedWorldDispatchRequestV1` after typed E1 carrier translation. `source_worker_cap_ref`
   carries the source cap's exact ref and `retained_worker_cap_link` is `ThisCommitment` for the new
   child cap; the B2.1 observation link is absent. Exact retry reuses the record's child participant
   and bootstrap identities; a conflicting request hash or retry-local child substitution fails
   closed.
5. A missing patch uses `UnchangedParent`; it is not represented as missing identity. Any supplied
   patch must be the exact authenticated E1 `DispatchPolicyNarrowingPatchV1` canonical JSON bytes
   or immutable byte ref. `patch_hash` is lowercase SHA-256 of those exact UTF-8 canonical JSON
   bytes, using the same recursive key-sorting and array-order rules with no additional wrapper.

The subject-specific authority link does not widen B1. `WorldWorkAcceptanceRecordV1`
continues to own only ephemeral-task and retained-turn acceptance; B3.2a continues to own Spawn
admission and explicitly does not own fork. E2's fork-dispatch link authenticates only the fork
request-to-cap commitment and creates no admission/lifecycle authority. This avoids a reverse
dependency or a fabricated B1/B3.2a identity while requiring pre-existing acceptance/admission
truth wherever the subject has one.

Validation fetches the exact linked B1 or B3.2a record revision from the named authority store. A
B1 link requires equality of request/session/caller/backend/world/subject and runtime-acceptance
identity; its `current_policy_snapshot_ref` and `current_policy_revision` must equal this record's
parent policy reference/revision, while its accepted `current_policy_snapshot_hash` must equal this
record's effective `policy_snapshot_hash`. A B3.2a link requires equality of issuer request,
session, backend, world, worker/bootstrap subject, exact spawn
fingerprint, parent policy reference/revision, and admission revision. These comparisons link
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

## Persistence, reporting, and downstream composition wall

For accepted ephemeral work and retained turns, B1 acceptance and the B2.1 claim become durable
under their existing owners, then E2 atomically persists this record with those exact links before
the foreground or tool surface reports acceptance. For fresh worker launch, E2 atomically persists
and links the cap to the exact B3.2a admission before launch/routability success is reported. For
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
