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
    secret_handoff_ref: Option<SecretHandoffRefV1>,
    mediation_posture: AdapterMediationPostureV1,
    command_broker_required: bool,
    allowed_side_effect_channels: Vec<BrokeredSideEffectChannelV1>,
}

enum AdapterMediationPostureV1 {
    BrokerRequired,
    CompatibilityUnproven { compatibility_mode_id: String },
}
```

Envelope kinds are `HostOrchestrator` and `WorldMember`. A `WorldMember` envelope requires exact world binding, guest-realizable entrypoint, immutable policy snapshot, and `command_broker_required=true` for side-effect-capable UAA runtimes.

During D1-before-D2 staging, `CompatibilityUnproven` may preserve explicitly named and logged existing behavior, but it cannot claim Substrate policy mediation, cannot satisfy UAA caging/broker gates, and cannot promote this seam. `BrokerRequired` requires `command_broker_required=true` and fails closed when any declared side-effect channel lacks broker support.

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

## 9. Cancel outcome categories

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

## 10. Supervisor idempotency and restart rules

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

## 11. Immutable `PolicySnapshotV3` acceptance rules

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

## 12. Dispatch narrowing monotonicity rules

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

## 13. Contract promotion gates

A contract is not considered landed until tests prove:

1. serialization and validation;
2. atomic persistence and revision conflict handling;
3. the real ingress/dispatch/runtime path uses it;
4. restart/replay behavior where durable;
5. fail-closed negative cases; and
6. at least one smoke/e2e path joins session, binding, policy, receipt, runtime event, and terminal/obligation truth.
