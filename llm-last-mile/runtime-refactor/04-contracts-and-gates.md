# Contracts and Gates

## Normative conventions

- All V1 records are durable, schema-versioned, and reject unknown identity/binding ambiguity.
- IDs and refs are opaque. Model-visible callers may receive task/worker handles but may not construct internal participant, resume, lease, policy-ref, or UAA-session truth.
- `authority_revision` and receipt/manifest revisions increase monotonically under compare-and-swap or equivalent atomic persistence.
- Timestamps are UTC.
- Hashes are over canonical serialized content, not presentation JSON.
- A field marked `ref` identifies a durable object whose hash is verified before use.

## 1. `DurableSessionAuthorityV1`

```rust
struct DurableSessionAuthorityV1 {
    schema_version: u32,                 // exactly 1
    orchestration_session_id: String,
    authority_revision: u64,
    authoritative_participant_lineage: Vec<String>,
    active_authoritative_participant_id: Option<String>,
    workspace_binding: WorkspaceBindingV1,
    world_binding: Option<WorldBindingV1>,
    host_attach_contract: Option<HostAttachContractV1>,
    retained_worker_refs: Vec<RetainedWorkerRefV1>,
    internal_resume_handle_refs: Vec<ResumeHandleRefV1>,
    lifecycle_posture: HostSessionPostureV1,
    current_policy_ref: Option<PolicyRefV1>,
    current_policy_revision: Option<String>,
    updated_at: Timestamp,
}
```

Allowed postures:

```text
ActiveAttached
ParkedResumable
DetachedReconciled
AwaitingAttention
Terminal
StaleRecoverable
Invalid
```

Acceptance rules:

1. Only `HostSessionAuthority` may create a newer `authority_revision` or change posture, lineage, binding, attach contract, or authoritative participant.
2. `AwaitingAttention` is derived from unresolved attention-driving obligations; it is not set from a worker flag or inbox row alone.
3. `Terminal` is monotonic unless a separately versioned recovery protocol explicitly creates a successor session; it is never reversed in-place.
4. World binding is the exact `(world_id, world_generation)` pair. Partial binding is invalid.
5. PID, socket, heartbeat, and attached-client data do not belong in this contract.

## 1A. `HostSessionTransitionIntentV1`

`HostSessionTransitionIntentV1` is the durable authority request for host-session `Start`, `Attach`, and `ResumeOneTurn`. A hidden-helper launch plan is only a private transport projection of this record. Reading or deleting that plan does not consume, apply, reject, or expire the intent.

```rust
struct HostSessionTransitionIntentV1 {
    schema_version: u32,                 // exactly 1
    intent_id: String,                   // globally unique in the authority store
    issuer_request_id: String,           // idempotency key for issuance
    intent_revision: u64,                // starts at 1; monotonic under CAS
    mode: HostSessionTransitionModeV1,
    authority_precondition: HostSessionAuthorityPreconditionV1,
    orchestration_session_id: String,
    shell_trace_session_id: String,
    caller: HostSessionTransitionCallerV1,
    source_authoritative_participant_id: Option<String>,
    target_authoritative_participant_id: String,
    target_participant_lease_token_hash: String,
    run_id: String,
    resulting_authoritative_lineage: Vec<String>,
    workspace_binding: WorkspaceBindingV1,
    world_binding: Option<WorldBindingV1>,
    descriptor_ref: AgentDescriptorRefV1,
    descriptor_hash: String,
    host_attach_contract: HostAttachContractV1,
    host_attach_contract_hash: String,
    resume_handle_ref: Option<ResumeHandleRefV1>,
    resume_handle_hash: Option<String>,
    transition_input_hash: Option<String>,
    post_turn_disposition: Option<HostPostTurnDispositionV1>,
    transport_payload_ref: HostSessionTransitionTransportPayloadRefV1,
    transport_payload_hash: String,
    payload_hash: String,
    issued_at: Timestamp,
    expires_at: Timestamp,
    state: HostSessionTransitionIntentStateV1,
    input_handoff: HostSessionTransitionInputHandoffV1,
    transport_payload_state: HostSessionTransitionTransportPayloadStateV1,
    updated_at: Timestamp,
}
```

Modes and authority preconditions:

```rust
enum HostSessionTransitionModeV1 {
    Start,
    Attach,
    ResumeOneTurn,
}

struct HostSessionTransitionCallerV1 {
    kind: HostSessionTransitionCallerKindV1,
    caller_participant_id: Option<String>,
    auto_attach_obligation_id: Option<String>,
    auto_attach_claim_owner: Option<String>,
}

enum HostSessionTransitionCallerKindV1 {
    PublicCli,
    Repl,
    RouterAutoAttach,
}

enum HostSessionAuthorityPreconditionV1 {
    ExpectedAbsent,
    ExpectedRevision {
        authority_revision: u64,
        authority_record_hash: String,
        active_authoritative_participant_id: String,
        authoritative_lineage_hash: String,
        lifecycle_posture: HostSessionPostureV1,
    },
}

enum HostPostTurnDispositionV1 {
    ReconcileToAttentionParkOrTerminal,
}
```

Intent states:

```rust
enum HostSessionTransitionIntentStateV1 {
    Issued,
    Claimed {
        claim_id: String,
        claimant_attempt_id: String,
        claim_revision: u64,
        claimed_at: Timestamp,
        claim_expires_at: Timestamp,
    },
    Applied {
        claim_id: String,
        authority_revision_before: Option<u64>,
        authority_revision_after: u64,
        active_authoritative_participant_id: String,
        resulting_posture: HostSessionPostureV1,
        authority_record_hash: String,
        result_hash: String,
        post_turn: HostSessionPostTurnApplicationV1,
        applied_at: Timestamp,
    },
    Rejected {
        reason: HostSessionTransitionTerminalRejectionV1,
        rejected_at: Timestamp,
    },
    Expired {
        expired_at: Timestamp,
    },
}

enum HostSessionPostTurnApplicationV1 {
    NotApplicable,
    Pending {
        expected_run_id: String,
        expected_authority_revision: u64,
    },
    Applied {
        completion_ref: String,
        completion_hash: String,
        authority_revision_before: u64,
        authority_revision_after: u64,
        resulting_posture: HostSessionPostureV1,
        result_hash: String,
        applied_at: Timestamp,
    },
}

enum HostSessionTransitionInputHandoffV1 {
    NotApplicable,
    Pending {
        input_hash: String,
        run_id: String,
    },
    Accepted {
        input_hash: String,
        run_id: String,
        acceptance_ref: String,
        acceptance_hash: String,
        accepted_at: Timestamp,
    },
    TerminalWithoutAcceptance {
        input_hash: String,
        run_id: String,
        terminal_ref: String,
        terminal_hash: String,
        terminal_at: Timestamp,
    },
}

enum HostSessionTransitionTerminalRejectionV1 {
    InvalidCommittedModePrecondition,
    AuthorityPreconditionNoLongerHolds,
    StaleAuthorityRevision,
    AuthorityRecordHashMismatch,
    CommittedDescriptorUnavailableOrChanged,
    CommittedAttachContractInvalidOrChanged,
    CommittedResumeHandleUnavailableOrChanged,
    CommittedTransportPayloadUnavailableOrChanged,
}

enum HostSessionTransitionAttemptRejectionV1 {
    UnknownIntent,
    IntentOrRequestIdentityMismatch,
    IntentRevisionMismatch,
    ClaimRevisionMismatch,
    PayloadHashMismatch,
    TransportPayloadHashMismatch,
    SessionIdentityMismatch,
    CallerIdentityMismatch,
    ParticipantOrLineageMismatch,
    WorkspaceBindingMismatch,
    WorldBindingMismatch,
    DescriptorProjectionMismatch,
    AttachContractProjectionMismatch,
    ResumeHandleProjectionMismatch,
    TransitionInputMismatch,
    SupersededClaim,
    ConflictingIntent,
}

enum HostSessionTransitionTransportPayloadStateV1 {
    Retained,
    ReleaseEligible {
        terminal_handoff_ref: String,
        terminal_handoff_hash: String,
    },
    Released {
        terminal_handoff_ref: String,
        released_at: Timestamp,
    },
}
```

Attempt rejection and terminal intent rejection are separate outcomes:

| Validation outcome | Durable effect |
|---|---|
| Invalid request before a valid intent is issued | Reject the request; persist no intent. |
| Exact `intent_id`, `issuer_request_id`, and `payload_hash` presented after `Applied` | Return/join the stored transition, input-handoff, and post-turn results read-only, even if the caller retained an older intent revision; do not reapply or mutate anything. |
| Unknown/substituted/mismatched attempt, stale intent/claim revision used for a claim/application/state mutation, superseded claim, or conflicting replay against a valid stored intent | Return and audit `HostSessionTransitionAttemptRejectionV1`; do not mutate, reject, expire, release, or otherwise strand the stored intent. |
| The stored intent was validly issued but its committed authority precondition or referenced durable commitment no longer holds before application | CAS the stored intent to `Rejected` with `HostSessionTransitionTerminalRejectionV1`; no authority mutation is permitted. |
| The fixed intent expiry passes before application and no authority commit exists | CAS the stored intent to `Expired`. |
| The exact stored intent and attempt validate | Continue the `Issued`/`Claimed`/`Applied` protocol. |

A substituted request using a stored `intent_id` or `issuer_request_id` with a different `payload_hash` is therefore attempt-rejected and audit-recorded without mutating or terminalizing the valid stored intent.

### Immutable commitments and identity

1. `intent_id` identifies exactly one logical transition. `issuer_request_id` makes issuance replay-safe: an exact retry with the same canonical payload returns the existing intent; reuse with different content fails closed.
2. `payload_hash` covers the canonical serialization of every immutable semantic field from `mode` through `post_turn_disposition` plus `transport_payload_ref` and `transport_payload_hash`. This includes the complete authority precondition, caller, shell-trace/session/run identities, source/target/lease-token-hash and lineage commitments, bindings, descriptor ref/hash, canonical inline attach contract/hash, resume ref/hash, and `transition_input_hash`. It excludes revisions, timestamps, claims, mutable input-handoff/post-turn/transport-retention state, and terminal results.
3. The referenced descriptor and resume handle are loaded from durable authority-owned locations and hash-verified before claim and again before application. `host_attach_contract` is the canonical inline contract, not a ref; canonical serialization must equal `host_attach_contract_hash` and the matching durable session contract where one exists. Plan-file JSON may not replace or supplement any commitment.
4. `transport_payload_ref` identifies an access-controlled, durable, hash-verified payload in the same authority namespace. It retains the exact raw participant lease token, transition input bytes or secure local ref, and non-authoritative launch projection material required to reproduce the helper plan. The lease token and input must match their committed hashes. Credentials are prohibited; prompt/input bytes are never emitted to traces, logs, diagnostics, or rejection text.
5. `expires_at` is fixed at issuance, is later than `issued_at`, and is bounded by the configured V1 maximum. Ambient retries, plan rewrites, helper restarts, and claim renewal do not extend it.
6. `intent_revision` increments on every durable transition state, claim, input-handoff, post-turn, or transport-payload-state change. A stale intent or claim revision cannot mutate the record; this does not block the read-only exact-result join defined above.
7. At most one nonterminal intent may reserve a given `(orchestration_session_id, authority_precondition)`. A different mode, target, or payload against that reservation is a conflict rather than a second candidate transition.
8. `PublicCli` and `Repl` callers have no auto-attach fields. `RouterAutoAttach` is valid only for `Attach` and commits the already-existing obligation ID and claim owner; A1 neither decides eligibility nor changes claim/settlement semantics. When `caller_participant_id` is present, it must match the exact authoritative caller permitted by the precondition. For a new `Start`, it is absent because no durable session participant exists yet.
9. For all three A1 modes, legacy `source_orchestration_session_id` plan data is absent. Startup prompt stream paths, helper PIDs, sockets, and plan paths are transport-only and cannot enter the authority record; existing path builders may reproject them from the retained payload without an endpoint/path redesign, and delivered content is accepted only when its committed hash matches.

### Exact mode preconditions

The posture domain is closed for V1:

| Mode | Required source posture | Initial resulting posture |
|---|---|---|
| `Start` | No authority record: `ExpectedAbsent` only. | `ActiveAttached`, committed only when the validated owner-helper/REPL application is ready to own the exact target. |
| `Attach` | Exactly one of `ParkedResumable`, `DetachedReconciled`, `AwaitingAttention`, or an explicitly persisted `StaleRecoverable`; never a posture inferred from process loss during the attempt. | `ActiveAttached`. Existing obligations remain unchanged and canonical. |
| `ResumeOneTurn` | Exactly one of `ParkedResumable`, `DetachedReconciled`, `AwaitingAttention`, or an explicitly persisted `StaleRecoverable`, plus the exact resume handle. | `ActiveAttached` with `post_turn=Pending`. Exact accepted terminal evidence later yields `Terminal`; otherwise unresolved attention obligations yield `AwaitingAttention`, and a resumable clean result yields `ParkedResumable`. |

Every unlisted source posture fails closed. In particular, a new independent `Attach`/`ResumeOneTurn` against `ActiveAttached`, `Terminal`, or `Invalid` is rejected; only an exact retry of the already-`Applied` intent may join its existing result. A1 does not create, resolve, or otherwise change obligations when selecting the deterministic post-turn posture.

`Start` requires all of the following:

1. `authority_precondition` is `ExpectedAbsent`. Absence means the session ID has never been allocated in the authority namespace, including retained tombstones; delete-and-recreate or a compatibility read that hides a prior record does not satisfy it.
2. `source_authoritative_participant_id` and `resume_handle_ref`/hash are absent.
3. The target participant, lease-token hash, run ID, and shell trace session ID are new and exact; the resulting lineage is exactly the valid initial lineage ending in that target. The caller is `PublicCli` or `Repl`, has no prior participant ID, and is bound to the unique `issuer_request_id`.
4. The workspace binding is normalized and absolute; a world-scoped start commits the complete `(world_id, world_generation)` pair, while a host-only start commits no world binding. Partial or ambient reconstruction fails closed.
5. The descriptor ref/hash and canonical inline attach contract/hash resolve exactly. Optional start input is bound by `transition_input_hash`; `post_turn_disposition` is absent.
6. `input_handoff` is `Pending` with the exact input hash/run ID when start input is present and `NotApplicable` otherwise.

`Attach` requires all of the following:

1. `authority_precondition` is `ExpectedRevision` and exactly matches the current authority revision, canonical record hash, active authoritative participant, lineage hash, and one of the four `Attach` source postures enumerated above. PID, socket, helper, heartbeat, or attached-client observations cannot make a posture eligible.
2. The source participant equals the current active authoritative participant and occurs at the expected lineage tip. The target participant, lease-token hash, and run ID are new and exact; the target is a distinct valid successor, and the resulting lineage is exactly the authority-approved append/replacement result. The shell trace session ID exactly preserves the session's committed trace identity.
3. Workspace/world bindings, descriptor, and attach contract exactly match the current durable session and requested successor. A world binding must match both world ID and generation.
4. A resume handle is present exactly when the committed attach contract requires it and is ref/hash verified. Attach has no transition input and no post-turn disposition.
5. `input_handoff` is `NotApplicable`.

`ResumeOneTurn` requires all of the following:

1. The same exact `ExpectedRevision`, source/target lineage, binding, descriptor, and attach-contract checks as `Attach`, against one of the four `ResumeOneTurn` source postures enumerated above.
2. The exact durable resume handle ref/hash is present and valid for the current source participant and session.
3. `transition_input_hash` is present and matches the one-turn input delivered to the runtime.
4. `input_handoff` is `Pending` with that exact input hash and run ID.
5. `post_turn_disposition` is `ReconcileToAttentionParkOrTerminal`. Accepted completion plus the claimed intent, exact unresolved-obligation read, and current authority revision—not queue delivery, helper liveness, or timeout—determine the revision-checked post-turn transition enumerated above.

### Lifecycle, retry, and fail-closed rules

1. Issuance atomically writes `Issued` and the hash-verified retained transport payload before any plan is written or helper is launched. Failure to project or launch leaves a fully reprojectable durable intent until it is rejected or expires.
2. Claim is a CAS transition from `Issued` to `Claimed` after revalidating the payload hash, expiry, authority precondition, identities, bindings, descriptor, attach contract, and resume/input commitments. Claim does not mutate session authority.
3. An exact retry of the current claim returns the same claim. A different claimant fails while the claim lease is current. After `claim_expires_at`, a new attempt may replace the claim only by CAS against the current intent revision and only after all preconditions still validate; PID or socket liveness is not claim authority.
4. Application commits the initial authority mutation and the `Applied` transition result atomically in one state-root transaction or an equivalent durable journal keyed by `intent_id`. A split commit without deterministic reconciliation is invalid. `Start` and `Attach` record `post_turn=NotApplicable`; `ResumeOneTurn` records `post_turn=Pending` for the exact run and resulting authority revision.
5. The core `Applied` transition result is immutable. An exact retry with the same `intent_id`, `issuer_request_id`, and `payload_hash` returns or joins that stored result regardless of the caller's older intent revision and must not allocate another participant, append lineage again, rewrite binding, or repeat the initial authority revision. The current revision remains mandatory for every claim/application/input/post-turn/payload-state mutation.
6. A stored valid intent whose authority precondition becomes stale before application transitions to `Rejected` with the exact reason. In the same revision, any `Pending` input becomes `TerminalWithoutAcceptance` using the canonical rejection record ref/hash. `Rejected` is terminal; retry requires a newly issued intent against current authority.
7. An `Issued` intent may become `Expired` at `expires_at`. A `Claimed` intent may expire only after proving no authority mutation committed. Expiry atomically moves any `Pending` input to `TerminalWithoutAcceptance` using the canonical expiry record ref/hash. If an authority commit marker or exact resulting authority exists, reconciliation must complete `Applied`; it may not mark the intent expired or rejected.
8. Stale intent/claim revisions used to request a mutation, substituted plan contents, mismatched payload/ref hashes, wrong session/caller/source/target, wrong lineage, wrong workspace/world generation, descriptor/attach/resume mismatch, superseded claim, and a conflicting replay all fail closed before authority mutation. A read-only exact `Applied` join follows rule 5 instead.
9. Removing a helper plan, losing a helper, or observing EOF/timeout changes transport evidence only. None destroys the intent, releases the retained payload, or rolls back an already-applied authority transition.
10. Input acceptance is a CAS transition from `input_handoff=Pending` to `Accepted` bound to the exact intent, run, input hash, and acceptance ref/hash. Exact duplicate acceptance joins the stored result. Exact terminal failure/abort before acceptance records `TerminalWithoutAcceptance`; missing, stale, reordered, or mismatched evidence cannot change the substate.
11. If the plan is loaded and removed before claim/application/input acceptance, a retry reprojects the exact same committed material from `transport_payload_ref` through the existing plan/path builders. If the initial transition is already `Applied`, retry joins that result and may reproject only the remaining handoff; it never repeats authority application. `Accepted` or `TerminalWithoutAcceptance` input joins its durable result and is never redelivered.
12. `transport_payload_state` remains `Retained` until an exact terminal handoff is durable. Every input-bearing mode, including optional-input `Start`, requires `input_handoff=Accepted` or `TerminalWithoutAcceptance` before release; input-free `Start`/`Attach` requires `NotApplicable`. `Start`/`Attach` additionally require the `Applied` result plus exact target/run startup-ownership acceptance or an exact terminal reconciliation. `ResumeOneTurn` additionally requires the `Applied` result plus `post_turn=Applied` or an exact terminal failure/abort. `Rejected`/`Expired` requires proof that no authority application committed. The authority first CAS-records `ReleaseEligible` with the terminal handoff ref/hash, then idempotently removes the payload and records `Released`. A missing plan file is never release evidence.
13. For `ResumeOneTurn`, accepted completion is applied exactly once by atomically advancing `post_turn=Pending` to `post_turn=Applied` with the revision-checked park/terminal authority transition. The completion ref/hash, run ID, intent ID/hash, and expected authority revision must all match. Duplicate exact completion joins the stored post-turn result; stale, reordered, mismatched, or conflicting completion fails closed and cannot reapply it. Intent expiry after the initial application does not erase a pending input/post-turn reconciliation or release its payload.

### Crash reconciliation

On process restart or before retrying a nonterminal intent, `HostSessionAuthority` reconciles in this order:

1. `Issued` with no authority mutation remains claimable until expiry.
2. `Claimed` with an unchanged precondition may resume under the exact claim or be CAS-reclaimed after the claim lease expires.
3. If the exact authority mutation and journal/application marker committed but the caller did not observe success, record or recover the identical `Applied` result and join it.
4. At any of those points, missing plan transport is reprojected from the retained payload only after checking the revisioned `input_handoff` plus durable application/post-turn state, so restart cannot duplicate a transition or prompt.
5. An `Applied` `ResumeOneTurn` with pending post-turn work reconciles only from exact durable completion/failure evidence for the committed run. Exact evidence applies or joins one post-turn result; missing or ambiguous evidence remains diagnosable and fails closed rather than inferring success from helper, queue, PID, socket, or plan state.
6. If neither the old precondition nor the exact committed result can be proven, fail closed and retain diagnosable state plus its payload; never guess from helper, PID, socket, or plan presence.
7. Reconciliation and terminal payload cleanup are idempotent across repeated crashes and cannot increment the initial or post-turn authority transition more than once for one intent.

### Compatibility and migration

Legacy helper plans without `intent_id` and `payload_hash` cannot drive a contract-correct A1 transition. Once a `Start`, `Attach`, or `ResumeOneTurn` producer is switched, a mixed-version consumer fails closed and requires reissuance; it does not reconstruct authority from the plan. Existing durable sessions may be compatibility-read or migrated, but every new transition still requires an exact intent. This contract changes neither helper endpoints/paths nor auto-attach policy, eligibility, claim, or settlement semantics.

## 2. `HostExecutionEpisodeV1`

```rust
struct HostExecutionEpisodeV1 {
    schema_version: u32,                 // exactly 1
    episode_id: String,
    kind: HostExecutionEpisodeKindV1,
    orchestration_session_id: String,
    observed_authority_revision: u64,
    backend_id: Option<String>,
    process_ref: Option<ProcessRefV1>,
    transport_status: HostExecutionEpisodeTransportStatusV1,
    started_at: Timestamp,
    last_heartbeat_at: Option<Timestamp>,
    ended_at: Option<Timestamp>,
    exit_observation: Option<EpisodeExitObservationV1>,
}
```

Episode kinds:

```text
ReplAttachedEpisode
HiddenOwnerHelperStartEpisode
HiddenOwnerHelperAttachEpisode
HiddenOwnerHelperResumeOneTurnEpisode
RuntimeToolboxEpisode
SyntheticOrRecoveredEpisode
```

Transport status:

```text
Available
UnavailableButDurableAuthorityExists
UnavailableAndNoAuthoritativeRoute
StaleOrOrphaned
```

Rules:

1. Episodes submit observations and requested transitions to `HostSessionAuthority`; they never write durable posture directly.
2. An observation whose `observed_authority_revision` is stale cannot mutate authority.
3. Episode exit does not delete session, worker, binding, receipt, or obligation truth.
4. Private transport success may accelerate delivery; it does not define durable success.

## 3. `ActiveEphemeralTaskReceiptV1`

```rust
struct ActiveEphemeralTaskReceiptV1 {
    schema_version: u32,
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

## 4. `ActiveRetainedTurnReceiptV1`

```rust
struct ActiveRetainedTurnReceiptV1 {
    schema_version: u32,
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

## 5. Receipt acceptance source

A foreground call may return an accepted receipt only after all of the following are durable:

1. exact session, caller, backend, world, and task/worker target identity;
2. the accepted immutable `PolicySnapshotV3` ref/hash;
3. exact `task_run_id` or `active_run_id` plus retained `message_id` where applicable;
4. a supervisor observation cursor or resumable observation claim; and
5. runtime submission acknowledgement that is joined to the same identity.

The persisted acceptance evidence has this minimum shape:

```rust
struct RuntimeAcceptanceEvidenceV1 {
    acknowledgement_kind: RuntimeAcceptanceAcknowledgementKindV1,
    stream_or_submission_id: String,
    frame_sequence: Option<u64>,
    task_run_id: Option<String>,
    active_run_id: Option<String>,
    message_id: Option<String>,
    retained_participant_id: Option<String>,
    observed_at: Timestamp,
}

enum RuntimeAcceptanceAcknowledgementKindV1 {
    SubmissionAccepted,
    StartFrame,
    RegisteredFrame,
}

struct SupervisorObservationClaimV1 {
    stream_or_submission_id: String,
    last_durable_sequence: Option<u64>,
    claim_revision: u64,
    lease_epoch: u64,
    resumable: bool,
}
```

V1 acceptance boundaries:

- For `run_world_task`, acceptance may use the first non-terminal `Start` or `Registered` frame only when the protocol defines that frame as runtime acceptance and it includes or unambiguously joins to `task_run_id`.
- For `continue_world_worker`, acceptance requires an explicit retained-turn submission acknowledgement or first non-terminal frame that includes or unambiguously joins to `active_run_id`, `message_id`, and the exact retained target.
- A socket write, HTTP request submission, process spawn attempt, or locally allocated ID is not runtime acceptance by itself.
- A terminal-only identity observation cannot be relabeled as pre-terminal acceptance.
- If the current runtime protocol cannot expose accepted identity before terminal exit, extend that protocol before changing the foreground tool to receipt-oriented early return.

B1 may persist and inspect the acceptance record while the existing foreground call still waits. Foreground early return is not allowed until B2 can atomically hand the observation claim to the supervisor.

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
5. `active_turn_ref` is updated atomically with active-turn acceptance/closeout.

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

## 8. `WorldRuntimeAdapterExecutionEnvelopeV1`

```rust
struct WorldRuntimeAdapterExecutionEnvelopeV1 {
    schema_version: u32,
    envelope_id: String,
    kind: AdapterExecutionEnvelopeKindV1,
    runtime_family: String,
    guest_entrypoint: PathBuf,
    runtime_dependency_ref: RuntimeDependencyRefV1,
    projected_home: Option<PathBuf>,
    workspace_root: PathBuf,
    world_id: String,
    world_generation: u64,
    retained_participant_id: Option<String>,
    active_run_id: Option<String>,
    policy_snapshot_ref: PolicySnapshotRefV1,
    policy_snapshot_hash: String,
    env_projection_ref: EnvProjectionRefV1,
    config_projection_identity: ConfigProjectionIdentityV1,
    config_projection_ref: ConfigProjectionRefV1,
    in_world_gateway_ref: Option<InWorldGatewayRefV1>,
    credential_posture: AdapterCredentialPostureV1,
    mediation_posture: AdapterMediationPostureV1,
    command_broker_required: bool,
    allowed_side_effect_channels: Vec<BrokeredSideEffectChannelV1>,
}

enum AdapterMediationPostureV1 {
    BrokerRequired,
    CompatibilityUnproven { compatibility_mode_id: String },
}

enum AdapterCredentialPostureV1 {
    NoCredentialsRequired,
    SecureGatewayHandoff { secret_handoff_ref: SecretHandoffRefV1 },
    CompatibilityCopyBridge { compatibility_mode_id: String },
}
```

Envelope kinds are `HostOrchestrator` and `WorldMember`. A `WorldMember` envelope requires exact world binding, guest-realizable entrypoint, immutable policy snapshot, explicit credential posture, and `command_broker_required=true` for side-effect-capable UAA runtimes.

During D1-before-D2 staging, `CompatibilityUnproven` may preserve explicitly named and logged existing behavior, but it cannot claim Substrate policy mediation, cannot satisfy UAA caging/broker gates, and cannot promote this seam. `BrokerRequired` requires `command_broker_required=true` and fails closed when any declared side-effect channel lacks broker support.

Credential-posture invariants:

1. `SecureGatewayHandoff` requires an exact `in_world_gateway_ref` in the same world generation and a valid `LaunchTimeSecretHandoffV1` ref.
2. `NoCredentialsRequired` requires no secret-handoff ref and cannot later discover ambient host credentials.
3. `CompatibilityCopyBridge` requires a named/logged compatibility mode and cannot satisfy credential, projection, or UAA contract-promotion gates.
4. The UAA child receives the gateway endpoint/session contract, never the raw secret FD or host credential payload.

Allowed channel values describe broker support, not permission to bypass:

```text
ShellCommand
PatchOrApplyEdit
DirectFileWrite
McpOrToolCall
ProcessSpawn
NetworkOperation
ProviderNativeSideEffect
```

Any side-effect channel absent from the envelope is disabled in world scope.

## 9. `LaunchTimeSecretHandoffV1`

### Existing carrier versus remaining adoption work

The current repo already implements the secure carrier mechanics for the managed in-world gateway: `GatewayAuthBundleV1`, an inherited pipe prepared by `world-service`, the pointer environment variable `SUBSTRATE_LLM_AUTH_BUNDLE_FD`, raw-secret env scrubbing, and gateway-side one-time read plus validation. Focused launcher and consumer integration tests make this a positive landed primitive that later slices must reuse and preserve.

`LaunchTimeSecretHandoffV1` adds the orchestration-facing identity, lifecycle, and non-secret evidence needed to join that carrier to an exact world generation, envelope, retained participant, and gateway receiver. The absence of this complete durable record does not mean the FD carrier itself is absent.

Remaining adoption work is to:

1. expose or persist the non-secret handoff reference/state required by the envelope without persisting secret payloads;
2. point direct world Codex/UAA provider traffic at the exact managed gateway that consumed the handoff;
3. construct per-worker runtime-native config from Substrate logical config plus accepted policy instead of copied host config/auth; and
4. prove the complete joined path with production-path smoke/e2e.

Do not replace the existing carrier merely to make its implementation names resemble this control-plane contract. Extend or adapt it only where one of the identity, evidence, fail-closed, or adoption requirements is genuinely missing.

```rust
struct LaunchTimeSecretHandoffV1 {
    schema_version: u32,                 // exactly 1
    handoff_id: String,
    orchestration_session_id: String,
    world_id: String,
    world_generation: u64,
    retained_participant_id: Option<String>,
    runtime_family: String,

    // Non-secret authority references only.
    credential_source_ref: CredentialSourceRefV1,
    receiving_gateway_ref: InWorldGatewayRefV1,
    delivery: SecretDeliveryMechanismV1,

    created_at: Timestamp,
    delivered_at: Option<Timestamp>,
    consumed_at: Option<Timestamp>,
    expires_at: Timestamp,
    state_revision: u64,
    state: SecretHandoffStateV1,
    failure_diagnostic_ref: Option<RedactedDiagnosticRefV1>,
}

enum SecretDeliveryMechanismV1 {
    SecureFd {
        fd_name: String,
        one_time: bool,
        gateway_receiver_only: bool,
        deny_child_inheritance: bool,
        close_after_consume: bool,
    },
}

enum SecretHandoffStateV1 {
    Prepared,
    Delivered,
    Consumed,
    Failed,
    Expired,
}
```

Allowed transitions:

```text
Prepared -> Delivered -> Consumed
Prepared|Delivered -> Failed|Expired
```

`Consumed`, `Failed`, and `Expired` are terminal. Reuse requires a new `handoff_id` and new descriptor.

Launch-time secret handoff rules:

1. Secret material is resolved by host credential authority and must not be persisted in Substrate records, runtime-native config, workspace overlays, manifests, traces, or logs.
2. `credential_source_ref` is an opaque host-authority reference, not a host filesystem path, credential-store locator exposed to the world, or digest of the secret payload. `fd_name` is a non-secret logical descriptor label.
3. Contract-correct world execution must not copy host credential files or secret-bearing host config into world-visible `CODEX_HOME`, `.codex`, `config.toml`, auth files, or equivalent runtime homes.
4. A bounded non-secret runtime config may be rendered from Substrate-owned logical inventory. Copying a host `config.toml` as authority is compatibility bridging, not projection authority.
5. V1 validation accepts `SecureFd` only when `one_time`, `gateway_receiver_only`, `deny_child_inheritance`, and `close_after_consume` are all `true`.
6. The secure FD is scoped to the exact `receiving_gateway_ref`, consumed by the in-world Substrate gateway at world launch, closed after consumption, and never inherited by the UAA adapter or its children.
7. The gateway—not Codex/UAA—owns credential application, gateway session material, and upstream provider forwarding. The UAA talks to the gateway through the envelope's endpoint/session contract.
8. Logs, receipts, traces, and manifests may contain handoff ID, non-secret refs, state, timestamps, and redacted diagnostics. They must not contain secret payloads, secret-bearing file paths, or reusable hashes/fingerprints derived from the secret payload.
9. Failure, expiry, receiver mismatch, world-generation mismatch, duplicate consumption, or descriptor inheritance risk fails closed for credential-requiring world adapters.
10. A compatibility copied-credential mode is temporary, explicitly named and logged, has retirement criteria, and cannot satisfy `ContractCorrectAndProven` or any secure-handoff acceptance gate.
11. Failed/expired handoffs close the descriptor and clear transient buffers before retry; retry creates a new handoff rather than reopening or replaying the old payload.

## 10. Cancel outcome categories

```rust
enum CancelWorldWorkOutcomeV1 {
    CancelledViaLiveTransport { active_run_id, terminal_ref },
    CancelAcceptedPendingCloseout { active_run_id, cancel_request_id },
    AlreadyTerminal { active_run_id, terminal_ref },
    NoActiveCancelableWork { worker_or_task_ref },
    OwnerUnreachable { active_run_id, durable_state, retryability },
    InvalidTarget { reason },
    WorldBindingMismatch { expected, actual },
    AmbiguousTarget { candidates },
    PolicyDenied { denial_code, explanation },
}
```

Rules:

1. Exact identity and world binding resolve before transport use.
2. `NoActiveCancelableWork` means valid routing context but no accepted non-terminal cancelable receipt.
3. `OwnerUnreachable` means a valid active receipt exists but the live cancellation route is unavailable and durable policy cannot yet declare closeout.
4. `CancelAcceptedPendingCloseout` is not terminal success; inspect remains able to observe eventual closeout.
5. Repeated cancel after terminal returns `AlreadyTerminal`; it never regresses the receipt.
6. Stop targets worker lifecycle. Cancel targets one active task/turn. They are not aliases.

## 11. Supervisor idempotency and restart rules

1. **Persist before return:** accepted receipt and immutable policy ref/hash are durable before the foreground caller receives success.
2. **Single logical observer:** supervisors claim a lease with `(active_run_id, receipt_revision, lease_epoch)`. A stale lease cannot write a newer revision.
3. **Restart discovery:** startup scans non-terminal accepted/running receipts and resumes observation or performs exact runtime reconciliation.
4. **Frame dedupe:** each frame is keyed by `(active_run_id, stream_id, sequence)` or an equivalent stable key. Duplicate frames are no-ops.
5. **Event dedupe:** durable worker events and obligations use stable event/causation IDs. Reprocessing cannot duplicate obligations.
6. **Monotonic states:** terminal states never revert; stale observers cannot overwrite newer state; equal-revision conflicting writes fail closed.
7. **Interrupted observation:** EOF/observer loss without terminal proof records an observation interruption and retry metadata. It does not fabricate terminal success.
8. **Reconciliation:** if runtime truth proves the process/run ended without a valid terminal frame, close as `Failed` with diagnostics; if truth is ambiguous, remain non-terminal and retry/fail closed.
9. **Atomic closeout:** terminal receipt state, worker active-turn clearing, terminal event, and obligation materialization commit atomically or through replay-safe idempotent steps.
10. **Cancellation:** one durable cancel request ID is reused across retries; repeated transport delivery is safe.
11. **Obligation timing:** attention events are persisted/materialized when observed, not deferred until terminal exit.
12. **Diagnostics:** non-zero exit, stream error, reconciliation failure, and cancel failure retain exact active-run/session/world/policy joins.

## 12. Immutable `PolicySnapshotV3` acceptance rules

An active task/turn may be accepted only when all are true:

1. exact session, caller, backend, and world binding are resolved;
2. steering policy allows the verb/mode/target;
3. current parent policy is resolved at a known revision;
4. retained worker cap is loaded and hash-verified when applicable;
5. optional narrowing is validated as monotonic;
6. the resulting `PolicySnapshotV3` canonicalizes and passes existing schema/enforcement validation;
7. snapshot bytes/ref/hash/revision are durable;
8. the execution envelope/world-service request carries the same verified snapshot;
9. runtime acceptance evidence joins the acknowledgement to the exact work identity;
10. the observation claim is durable and resumable; and
11. the receipt references the snapshot, acceptance evidence, and observation claim before the foreground caller is told the work was accepted.

After acceptance:

- the receipt's policy ref/hash is immutable;
- parent broadening or narrowing does not rewrite the active receipt;
- parent changes apply to future task acceptance, continue, fork, or worker turns;
- emergency revocation is an explicit audited cancel/revoke path, never silent snapshot mutation; and
- snapshot mismatch at broker/world-service fails closed.

## 13. Dispatch narrowing monotonicity rules

The resolver computes:

```text
ephemeral = current_parent AND dispatch_patch
worker_cap = current_parent_at_spawn AND spawn_patch
turn = current_parent_now AND worker_cap AND turn_patch
fork_cap = current_parent_now AND source_worker_cap AND fork_patch
```

V1 field rules:

| Field | Allowed narrowing | Rejected broadening |
|---|---|---|
| `world_fs.host_visible` | `true -> false` | `false -> true` |
| `world_fs.fail_closed.routing` | `false -> true` | `true -> false` |
| `world_fs.caged_required` | `false -> true` | `true -> false` |
| `world_fs.write.enabled` | `true -> false` | `false -> true` |
| `world_fs.deny_enforcement` | same or stronger rank | weaker rank or removal |
| discover/read/write `allow_list` | each requested path is contained by at least one parent path | root/directory/file widening |
| discover/read/write `deny_list` | add denials or retain parent denials | remove parent denial |

Path-containment rules:

1. Normalize relative to the authoritative world/project root.
2. `.` contains `src` and `src/parser.rs`; `src` contains `src/parser.rs`; a file contains only itself.
3. Reject absolute host paths, `..` escape, unsupported glob semantics, NUL, symlink escape, and paths outside the root.
4. Compare canonical policy paths without requiring the target file to already exist; runtime resolution must re-check symlink/ancestor escape at enforcement time.
5. `host_visible=true` with deny-list or unprovable isolation semantics fails closed unless the patch legally narrows to supported full isolation.
6. A patch supplied while `agents.world_dispatch.allow_capability_narrowing=false` is rejected, not ignored.
7. Narrowing may not enable a dispatch action, backend, mode, capability, network route, or side-effect channel forbidden by the parent.
8. Adapter config may receive policy hints, but only broker/world-service enforcement counts.

## 14. Contract promotion gates

A contract is not considered landed until tests prove:

1. serialization and validation;
2. atomic persistence and revision conflict handling;
3. the real ingress/dispatch/runtime path uses it;
4. restart/replay behavior where durable;
5. fail-closed negative cases;
6. every revision-bound host transition joins intent issuance, claim, authority application, and exact result on the real CLI and REPL path, including crash reconciliation and no-reapply exact retry;
7. at least one smoke/e2e path joins session, binding, policy, receipt, runtime event, and terminal/obligation truth;
8. credential-requiring world UAA proof joins the envelope to a consumed one-time in-world gateway handoff without copied secret files or inherited descriptors; and
9. no compatibility copy or `CompatibilityUnproven` evidence is used for contract promotion.
