# Contracts and Gates

## Normative conventions

Canonical content: [`foundations/normative-conventions.md#normative-conventions`](foundations/normative-conventions.md#normative-conventions).

## Development review and remediation contract

Canonical content: [`contracts/development-review-and-remediation-contract.md#development-review-and-remediation-contract`](contracts/development-review-and-remediation-contract.md#development-review-and-remediation-contract).

### Required workflow skills and selected outcome

Canonical content: [`contracts/development-review-and-remediation-contract.md#required-workflow-skills-and-selected-outcome`](contracts/development-review-and-remediation-contract.md#required-workflow-skills-and-selected-outcome).

### Review priority

Canonical content: [`contracts/development-review-and-remediation-contract.md#review-priority`](contracts/development-review-and-remediation-contract.md#review-priority).

### Bounded review cycles

Canonical content: [`contracts/development-review-and-remediation-contract.md#bounded-review-cycles`](contracts/development-review-and-remediation-contract.md#bounded-review-cycles).

### Machine-auditable cycle record

Canonical content: [`contracts/development-review-and-remediation-contract.md#machine-auditable-cycle-record`](contracts/development-review-and-remediation-contract.md#machine-auditable-cycle-record).

### Mechanical changes and completion

Canonical content: [`contracts/development-review-and-remediation-contract.md#mechanical-changes-and-completion`](contracts/development-review-and-remediation-contract.md#mechanical-changes-and-completion).

### A1 canonical encoding, path identity, supporting types, and persistence

Canonical content: [`a1-2-earlier-histories/contracts-and-gates.md#a1-canonical-encoding-path-identity-supporting-types-and-persistence`](a1-2-earlier-histories/contracts-and-gates.md#a1-canonical-encoding-path-identity-supporting-types-and-persistence).

##### Host-path normalization and validation

Compatibility anchor only; canonical content: [`a1-2-earlier-histories/contracts-and-gates.md#host-path-normalization-and-validation`](a1-2-earlier-histories/contracts-and-gates.md#host-path-normalization-and-validation).

## 1. `DurableSessionAuthorityV1`

Canonical content: [`a1-2-earlier-histories/contracts-and-gates.md#1-durablesessionauthorityv1`](a1-2-earlier-histories/contracts-and-gates.md#1-durablesessionauthorityv1).

## 1A. strict `HostSessionTransitionIntentV1`/`HostSessionTransitionIntentV2`

Canonical content: [`a1-2-earlier-histories/contracts-and-gates.md#1a-strict-hostsessiontransitionintentv1hostsessiontransitionintentv2`](a1-2-earlier-histories/contracts-and-gates.md#1a-strict-hostsessiontransitionintentv1hostsessiontransitionintentv2).

### Lifecycle, retry, and fail-closed rules

Compatibility anchor only; canonical content: [`a1-2-earlier-histories/contracts-and-gates.md#lifecycle-retry-and-fail-closed-rules`](a1-2-earlier-histories/contracts-and-gates.md#lifecycle-retry-and-fail-closed-rules).

## 2. `HostExecutionEpisodeV1`

Canonical content: [`contracts/host-execution-episode-v1.md#2-hostexecutionepisodev1`](contracts/host-execution-episode-v1.md#2-hostexecutionepisodev1).

## 2A. Runtime event identity and ordering carrier

Canonical content: [`b1-b2-1/contracts-and-gates.md#2a-runtime-event-identity-and-ordering-carrier`](b1-b2-1/contracts-and-gates.md#2a-runtime-event-identity-and-ordering-carrier).

### B2.1-3 exact producer replay transport

Canonical content: [`b1-b2-1/contracts-and-gates.md#b21-3-exact-producer-replay-transport`](b1-b2-1/contracts-and-gates.md#b21-3-exact-producer-replay-transport).

## 2B. Bounded retained worker event envelope

Canonical content: [`b3-1-c1/contracts-and-gates.md#2b-bounded-retained-worker-event-envelope`](b3-1-c1/contracts-and-gates.md#2b-bounded-retained-worker-event-envelope).

### B1/B2.1 bounded read-only dispatch-authority adapter

Canonical content: [`b1-b2-1/contracts-and-gates.md#b1b21-bounded-read-only-dispatch-authority-adapter`](b1-b2-1/contracts-and-gates.md#b1b21-bounded-read-only-dispatch-authority-adapter).

### Deferred retained-spawn admission recovery contract

Canonical content: [`contracts/deferred-retained-spawn-admission-recovery-contract.md#deferred-retained-spawn-admission-recovery-contract`](contracts/deferred-retained-spawn-admission-recovery-contract.md#deferred-retained-spawn-admission-recovery-contract).

### B3.2a-WA `ExactBoundWorldOwnershipAdoptionV1`

Canonical content: [`b1-b2-1/contracts-and-gates.md#b32a-wa-exactboundworldownershipadoptionv1`](b1-b2-1/contracts-and-gates.md#b32a-wa-exactboundworldownershipadoptionv1).

## 2C. `WorldWorkAcceptanceRecordV1`

Canonical content: [`b1-b2-1/contracts-and-gates.md#2c-worldworkacceptancerecordv1`](b1-b2-1/contracts-and-gates.md#2c-worldworkacceptancerecordv1).

### B1 receipt-core review versus production completion

Canonical content: [`b1-b2-1/contracts-and-gates.md#b1-receipt-core-review-versus-production-completion`](b1-b2-1/contracts-and-gates.md#b1-receipt-core-review-versus-production-completion).

### B1 frozen proposal, persistence, and revision semantics

Canonical content: [`b1-b2-1/contracts-and-gates.md#b1-frozen-proposal-persistence-and-revision-semantics`](b1-b2-1/contracts-and-gates.md#b1-frozen-proposal-persistence-and-revision-semantics).

## 3. `ActiveEphemeralTaskReceiptV1`

Canonical content: [`contracts/active-ephemeral-task-receipt-v1.md#3-activeephemeraltaskreceiptv1`](contracts/active-ephemeral-task-receipt-v1.md#3-activeephemeraltaskreceiptv1).

## 4. `ActiveRetainedTurnReceiptV1`

Canonical content: [`contracts/active-retained-turn-receipt-v1.md#4-activeretainedturnreceiptv1`](contracts/active-retained-turn-receipt-v1.md#4-activeretainedturnreceiptv1).

## 5. Receipt acceptance source

Canonical content: [`b1-b2-1/contracts-and-gates.md#5-receipt-acceptance-source`](b1-b2-1/contracts-and-gates.md#5-receipt-acceptance-source).

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

Canonical content: [`b1-b2-1/contracts-and-gates.md#11-supervisor-idempotency-and-restart-rules`](b1-b2-1/contracts-and-gates.md#11-supervisor-idempotency-and-restart-rules).

## 11A. Differential-baseline transition gate

Canonical content: [`b1-b2-1/contracts-and-gates.md#11a-differential-baseline-transition-gate`](b1-b2-1/contracts-and-gates.md#11a-differential-baseline-transition-gate).

## 12. Final-receipt immutable `PolicySnapshotV3` acceptance rules

B1's pre-E2 acceptance anchor records the exact current policy identity used by the runtime but is
not a final receipt and is not model-facing. E2 owns the immutable active-run snapshot and retained
worker cap below; B2.2 may expose a receipt only after those commitments and the B2.1 observation
claim are durable and linked to the B1 record.

A final active task/turn receipt may be exposed only when all are true:

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
9. no compatibility copy or `CompatibilityUnproven` evidence is used for contract promotion; and
10. B1/B2.1 production proof shows both accepted work families enter the durable supervisor without
    a legacy-writer attempt, caller/foreground drop does not erase truth, and B3.1 begins only after
    the joint closeout.

##### Remaining R2-2 same-process carrier closure

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/contracts-and-gates.md#remaining-r2-2-same-process-carrier-closure`](a1.1d-5r2-2f/contracts-and-gates.md#remaining-r2-2-same-process-carrier-closure).

##### `PlatformBootstrapMappingV1` construction and verification

Compatibility anchor only; canonical content: [`a1.1d-5r2-3/contracts-and-gates.md#platformbootstrapmappingv1-construction-and-verification`](a1.1d-5r2-3/contracts-and-gates.md#platformbootstrapmappingv1-construction-and-verification).

## A1.1d-5R2-2F0-HC corrected complete process-resource ledger

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/contracts-and-gates.md#a11d-5r2-2f0-hc-corrected-complete-process-resource-ledger`](a1.1d-5r2-2f/contracts-and-gates.md#a11d-5r2-2f0-hc-corrected-complete-process-resource-ledger).

### Identity and use closure

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/contracts-and-gates.md#identity-and-use-closure`](a1.1d-5r2-2f/contracts-and-gates.md#identity-and-use-closure).

### Semantics, ownership, and primary dispositions

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/contracts-and-gates.md#semantics-ownership-and-primary-dispositions`](a1.1d-5r2-2f/contracts-and-gates.md#semantics-ownership-and-primary-dispositions).

### Frozen combined implementation gate

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/contracts-and-gates.md#frozen-combined-implementation-gate`](a1.1d-5r2-2f/contracts-and-gates.md#frozen-combined-implementation-gate).

### Corrected historical differential evidence-authority contract

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/contracts-and-gates.md#corrected-historical-differential-evidence-authority-contract`](a1.1d-5r2-2f/contracts-and-gates.md#corrected-historical-differential-evidence-authority-contract).

### Canonical harness gate result

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/contracts-and-gates.md#canonical-harness-gate-result`](a1.1d-5r2-2f/contracts-and-gates.md#canonical-harness-gate-result).

## A1.1d-5R2-2F readiness and outbound-environment correction

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/contracts-and-gates.md#a11d-5r2-2f-readiness-and-outbound-environment-correction`](a1.1d-5r2-2f/contracts-and-gates.md#a11d-5r2-2f-readiness-and-outbound-environment-correction).

### Exact outbound command environment

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/contracts-and-gates.md#exact-outbound-command-environment`](a1.1d-5r2-2f/contracts-and-gates.md#exact-outbound-command-environment).

### Forbidden input and forwarding table

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/contracts-and-gates.md#forbidden-input-and-forwarding-table`](a1.1d-5r2-2f/contracts-and-gates.md#forbidden-input-and-forwarding-table).

### Explicit readiness contract

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/contracts-and-gates.md#explicit-readiness-contract`](a1.1d-5r2-2f/contracts-and-gates.md#explicit-readiness-contract).

### Required future F3/F4 proof

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/contracts-and-gates.md#required-future-f3f4-proof`](a1.1d-5r2-2f/contracts-and-gates.md#required-future-f3f4-proof).

## Historical F5-PD non-mutating nested Doctor contract

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/contracts-and-gates.md#historical-f5-pd-non-mutating-nested-doctor-contract`](a1.1d-5r2-2f/contracts-and-gates.md#historical-f5-pd-non-mutating-nested-doctor-contract).

### Compatibility invariant

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/contracts-and-gates.md#compatibility-invariant`](a1.1d-5r2-2f/contracts-and-gates.md#compatibility-invariant).

### Authenticated internal mode

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/contracts-and-gates.md#authenticated-internal-mode`](a1.1d-5r2-2f/contracts-and-gates.md#authenticated-internal-mode).

### Evidence and truthful classification

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/contracts-and-gates.md#evidence-and-truthful-classification`](a1.1d-5r2-2f/contracts-and-gates.md#evidence-and-truthful-classification).

### Exact implementation allowlist

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/contracts-and-gates.md#exact-implementation-allowlist`](a1.1d-5r2-2f/contracts-and-gates.md#exact-implementation-allowlist).

### Exact impact authorization

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/contracts-and-gates.md#exact-impact-authorization`](a1.1d-5r2-2f/contracts-and-gates.md#exact-impact-authorization).

### Required proof

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/contracts-and-gates.md#required-proof`](a1.1d-5r2-2f/contracts-and-gates.md#required-proof).

### Mandatory stops and exit sequence

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/contracts-and-gates.md#mandatory-stops-and-exit-sequence`](a1.1d-5r2-2f/contracts-and-gates.md#mandatory-stops-and-exit-sequence).

## A1.1d-5R2-2F completed gate record

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/contracts-and-gates.md#a11d-5r2-2f-completed-gate-record`](a1.1d-5r2-2f/contracts-and-gates.md#a11d-5r2-2f-completed-gate-record).

### Final composition contract

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/contracts-and-gates.md#final-composition-contract`](a1.1d-5r2-2f/contracts-and-gates.md#final-composition-contract).

### Final regression authority

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/contracts-and-gates.md#final-regression-authority`](a1.1d-5r2-2f/contracts-and-gates.md#final-regression-authority).
## Canonical shell-library broad-wall invocation contract

Compatibility anchor only; canonical content: [`a1.1d-5r2-2-renewed-closeout/contracts-and-gates.md#canonical-shell-library-broad-wall-invocation-contract`](a1.1d-5r2-2-renewed-closeout/contracts-and-gates.md#canonical-shell-library-broad-wall-invocation-contract).

### Fresh private root and preflight

Compatibility anchor only; canonical content: [`a1.1d-5r2-2-renewed-closeout/contracts-and-gates.md#fresh-private-root-and-preflight`](a1.1d-5r2-2-renewed-closeout/contracts-and-gates.md#fresh-private-root-and-preflight).

### Canonical command forms

Compatibility anchor only; canonical content: [`a1.1d-5r2-2-renewed-closeout/contracts-and-gates.md#canonical-command-forms`](a1.1d-5r2-2-renewed-closeout/contracts-and-gates.md#canonical-command-forms).

### Reproducible creation and cleanup template

Compatibility anchor only; canonical content: [`a1.1d-5r2-2-renewed-closeout/contracts-and-gates.md#reproducible-creation-and-cleanup-template`](a1.1d-5r2-2-renewed-closeout/contracts-and-gates.md#reproducible-creation-and-cleanup-template).

### Required provenance and eligibility

Compatibility anchor only; canonical content: [`a1.1d-5r2-2-renewed-closeout/contracts-and-gates.md#required-provenance-and-eligibility`](a1.1d-5r2-2-renewed-closeout/contracts-and-gates.md#required-provenance-and-eligibility).

### Frozen baseline and command-mismatch evidence

Compatibility anchor only; canonical content: [`a1.1d-5r2-2-renewed-closeout/contracts-and-gates.md#frozen-baseline-and-command-mismatch-evidence`](a1.1d-5r2-2-renewed-closeout/contracts-and-gates.md#frozen-baseline-and-command-mismatch-evidence).
## R2-3 closeout status

Compatibility anchor only; canonical content: [`a1.1d-5r2-3/contracts-and-gates.md#r2-3-closeout-status`](a1.1d-5r2-3/contracts-and-gates.md#r2-3-closeout-status).
## Normative renewed R2-2 publication contract

Compatibility anchor only; canonical content: [`a1.1d-5r2-2-renewed-closeout/contracts-and-gates.md#normative-renewed-r2-2-publication-contract`](a1.1d-5r2-2-renewed-closeout/contracts-and-gates.md#normative-renewed-r2-2-publication-contract).

### Mandatory sequence

Compatibility anchor only; canonical content: [`a1.1d-5r2-2-renewed-closeout/contracts-and-gates.md#mandatory-sequence`](a1.1d-5r2-2-renewed-closeout/contracts-and-gates.md#mandatory-sequence).

### Required topology

Compatibility anchor only; canonical content: [`a1.1d-5r2-2-renewed-closeout/contracts-and-gates.md#required-topology`](a1.1d-5r2-2-renewed-closeout/contracts-and-gates.md#required-topology).

### Failure and no-push posture

Compatibility anchor only; canonical content: [`a1.1d-5r2-2-renewed-closeout/contracts-and-gates.md#failure-and-no-push-posture`](a1.1d-5r2-2-renewed-closeout/contracts-and-gates.md#failure-and-no-push-posture).

## A1.1d-5R2-2 closeout-remediation contracts (historical RP1/RP2 record)

Compatibility anchor only; canonical content: [`a1.1d-5r2-2-renewed-closeout/contracts-and-gates.md#a11d-5r2-2-closeout-remediation-contracts-historical-rp1rp2-record`](a1.1d-5r2-2-renewed-closeout/contracts-and-gates.md#a11d-5r2-2-closeout-remediation-contracts-historical-rp1rp2-record).

### R1 — release dry-run authenticated-carrier non-disclosure

Compatibility anchor only; canonical content: [`a1.1d-5r2-2-renewed-closeout/contracts-and-gates.md#r1--release-dry-run-authenticated-carrier-non-disclosure`](a1.1d-5r2-2-renewed-closeout/contracts-and-gates.md#r1--release-dry-run-authenticated-carrier-non-disclosure).

#### Closed source facts

Compatibility anchor only; canonical content: [`a1.1d-5r2-2-renewed-closeout/contracts-and-gates.md#closed-source-facts`](a1.1d-5r2-2-renewed-closeout/contracts-and-gates.md#closed-source-facts).

#### Exact implementation allowlist

Compatibility anchor only; canonical content: [`a1.1d-5r2-2-renewed-closeout/contracts-and-gates.md#exact-implementation-allowlist`](a1.1d-5r2-2-renewed-closeout/contracts-and-gates.md#exact-implementation-allowlist).

#### Two-channel command contract

Compatibility anchor only; canonical content: [`a1.1d-5r2-2-renewed-closeout/contracts-and-gates.md#two-channel-command-contract`](a1.1d-5r2-2-renewed-closeout/contracts-and-gates.md#two-channel-command-contract).

#### R1 proof gate

Compatibility anchor only; canonical content: [`a1.1d-5r2-2-renewed-closeout/contracts-and-gates.md#r1-proof-gate`](a1.1d-5r2-2-renewed-closeout/contracts-and-gates.md#r1-proof-gate).

### P1 — canonical broad-wall provenance runner

Compatibility anchor only; canonical content: [`a1.1d-5r2-2-renewed-closeout/contracts-and-gates.md#p1--canonical-broad-wall-provenance-runner`](a1.1d-5r2-2-renewed-closeout/contracts-and-gates.md#p1--canonical-broad-wall-provenance-runner).

#### Source-closure decision

Compatibility anchor only; canonical content: [`a1.1d-5r2-2-renewed-closeout/contracts-and-gates.md#source-closure-decision`](a1.1d-5r2-2-renewed-closeout/contracts-and-gates.md#source-closure-decision).

#### Historical runner allowlist (retired diagnostic-only)

Compatibility anchor only; canonical content: [`a1.1d-5r2-2-renewed-closeout/contracts-and-gates.md#historical-runner-allowlist-retired-diagnostic-only`](a1.1d-5r2-2-renewed-closeout/contracts-and-gates.md#historical-runner-allowlist-retired-diagnostic-only).

#### Historical runner interface (retired diagnostic-only)

Compatibility anchor only; canonical content: [`a1.1d-5r2-2-renewed-closeout/contracts-and-gates.md#historical-runner-interface-retired-diagnostic-only`](a1.1d-5r2-2-renewed-closeout/contracts-and-gates.md#historical-runner-interface-retired-diagnostic-only).

#### Descendant-containment contract

Compatibility anchor only; canonical content: [`a1.1d-5r2-2-renewed-closeout/contracts-and-gates.md#descendant-containment-contract`](a1.1d-5r2-2-renewed-closeout/contracts-and-gates.md#descendant-containment-contract).

#### Exact namespace and mount lifecycle

Compatibility anchor only; canonical content: [`a1.1d-5r2-2-renewed-closeout/contracts-and-gates.md#exact-namespace-and-mount-lifecycle`](a1.1d-5r2-2-renewed-closeout/contracts-and-gates.md#exact-namespace-and-mount-lifecycle).

#### Root, descriptor, and deletion contract

Compatibility anchor only; canonical content: [`a1.1d-5r2-2-renewed-closeout/contracts-and-gates.md#root-descriptor-and-deletion-contract`](a1.1d-5r2-2-renewed-closeout/contracts-and-gates.md#root-descriptor-and-deletion-contract).

#### Evidence layout, bounds, and atomic finalization

Compatibility anchor only; canonical content: [`a1.1d-5r2-2-renewed-closeout/contracts-and-gates.md#evidence-layout-bounds-and-atomic-finalization`](a1.1d-5r2-2-renewed-closeout/contracts-and-gates.md#evidence-layout-bounds-and-atomic-finalization).

#### Exact provenance schema

Compatibility anchor only; canonical content: [`a1.1d-5r2-2-renewed-closeout/contracts-and-gates.md#exact-provenance-schema`](a1.1d-5r2-2-renewed-closeout/contracts-and-gates.md#exact-provenance-schema).

#### Exact log-summary compatibility contract

Compatibility anchor only; canonical content: [`a1.1d-5r2-2-renewed-closeout/contracts-and-gates.md#exact-log-summary-compatibility-contract`](a1.1d-5r2-2-renewed-closeout/contracts-and-gates.md#exact-log-summary-compatibility-contract).

#### P1 self-test gate

Compatibility anchor only; canonical content: [`a1.1d-5r2-2-renewed-closeout/contracts-and-gates.md#p1-self-test-gate`](a1.1d-5r2-2-renewed-closeout/contracts-and-gates.md#p1-self-test-gate).

### RP5 publication and sequencing gate

Compatibility anchor only; canonical content: [`a1.1d-5r2-2-renewed-closeout/contracts-and-gates.md#rp5-publication-and-sequencing-gate`](a1.1d-5r2-2-renewed-closeout/contracts-and-gates.md#rp5-publication-and-sequencing-gate).
## A1.1d-5R2-4 terminal gate disposition

Canonical content: [`a1.1d-5r2-4/terminal-gate-disposition.md#a11d-5r2-4-terminal-gate-disposition`](a1.1d-5r2-4/terminal-gate-disposition.md#a11d-5r2-4-terminal-gate-disposition).

## A1.1d-5R3 lifecycle contracts and gates

Canonical content: [`a1.1d-5r3/contracts-and-gates.md#a11d-5r3-lifecycle-contracts-and-gates`](a1.1d-5r3/contracts-and-gates.md#a11d-5r3-lifecycle-contracts-and-gates).

### `R3-CANDIDATE-01` — exact synchronous candidate rollback

Canonical content: [`a1.1d-5r3/contracts-and-gates.md#r3-candidate-01--exact-synchronous-candidate-rollback`](a1.1d-5r3/contracts-and-gates.md#r3-candidate-01--exact-synchronous-candidate-rollback).

### `R3-MANIFEST-01` — canonical managed-artifact authority

Canonical content: [`a1.1d-5r3/contracts-and-gates.md#r3-manifest-01--canonical-managed-artifact-authority`](a1.1d-5r3/contracts-and-gates.md#r3-manifest-01--canonical-managed-artifact-authority).

### `R3-TEMP-ROLLBACK-01` — pre-manifest current-attempt temporary trees

Canonical content: [`a1.1d-5r3/contracts-and-gates.md#r3-temp-rollback-01--pre-manifest-current-attempt-temporary-trees`](a1.1d-5r3/contracts-and-gates.md#r3-temp-rollback-01--pre-manifest-current-attempt-temporary-trees).

### `R3-CLASS-01` — mutually explicit lifecycle classes

Canonical content: [`a1.1d-5r3/contracts-and-gates.md#r3-class-01--mutually-explicit-lifecycle-classes`](a1.1d-5r3/contracts-and-gates.md#r3-class-01--mutually-explicit-lifecycle-classes).

### `R3-ACTION-01` — mutation and crash/retry protocol

Canonical content: [`a1.1d-5r3/contracts-and-gates.md#r3-action-01--mutation-and-crashretry-protocol`](a1.1d-5r3/contracts-and-gates.md#r3-action-01--mutation-and-crashretry-protocol).

### `R3-PRESERVE-01` — exact removal and restoration

Canonical content: [`a1.1d-5r3/contracts-and-gates.md#r3-preserve-01--exact-removal-and-restoration`](a1.1d-5r3/contracts-and-gates.md#r3-preserve-01--exact-removal-and-restoration).

### Platform clauses

Canonical content: [`a1.1d-5r3/contracts-and-gates.md#platform-clauses`](a1.1d-5r3/contracts-and-gates.md#platform-clauses).

### Review, proof, and publication wall

Canonical content: [`a1.1d-5r3/contracts-and-gates.md#review-proof-and-publication-wall`](a1.1d-5r3/contracts-and-gates.md#review-proof-and-publication-wall).
## R3 implementation status append

Canonical content: [`a1.1d-5r3/contracts-status.md#r3-implementation-status-append`](a1.1d-5r3/contracts-status.md#r3-implementation-status-append).

### `A1.1d-5R3-MANIFEST`

Canonical content: [`a1.1d-5r3/contracts-status.md#a11d-5r3-manifest`](a1.1d-5r3/contracts-status.md#a11d-5r3-manifest).

### `A1.1d-5R3-LINUX`

Canonical content: [`a1.1d-5r3/contracts-status.md#a11d-5r3-linux`](a1.1d-5r3/contracts-status.md#a11d-5r3-linux).

### `A1.1d-5R3-LINUX-CLOSEOUT`

Canonical content: [`a1.1d-5r3/contracts-status.md#a11d-5r3-linux-closeout`](a1.1d-5r3/contracts-status.md#a11d-5r3-linux-closeout).

### `A1.1d-5R3-MAC`

Canonical content: [`a1.1d-5r3/contracts-status.md#a11d-5r3-mac`](a1.1d-5r3/contracts-status.md#a11d-5r3-mac).
## A1.1d-5R3-MAC attempt-4 remediation status (2026-08-06)

Canonical content: [`r3-mac-evidence-recovery/contracts-and-gates-status.md#a11d-5r3-mac-attempt-4-remediation-status-2026-08-06`](r3-mac-evidence-recovery/contracts-and-gates-status.md#a11d-5r3-mac-attempt-4-remediation-status-2026-08-06).
## AUX-R3-MAC-EVIDENCE-RECOVERY-PLAN recovery decision (2026-08-07)

Canonical content: [`r3-mac-evidence-recovery/contracts-and-gates-status.md#aux-r3-mac-evidence-recovery-plan-recovery-decision-2026-08-07`](r3-mac-evidence-recovery/contracts-and-gates-status.md#aux-r3-mac-evidence-recovery-plan-recovery-decision-2026-08-07).
## AUX-R3-MAC-EVIDENCE-RECOVERY-PLAN current contract correction (2026-08-07)

Canonical content: [`r3-mac-evidence-recovery/contracts-and-gates-status.md#aux-r3-mac-evidence-recovery-plan-current-contract-correction-2026-08-07`](r3-mac-evidence-recovery/contracts-and-gates-status.md#aux-r3-mac-evidence-recovery-plan-current-contract-correction-2026-08-07).
## AUX-R3-MAC-EVIDENCE-RECOVERY-R3 recovery-current validator invocation (2026-08-07)

Canonical content: [`r3-mac-evidence-recovery/contracts-and-gates-status.md#aux-r3-mac-evidence-recovery-r3-recovery-current-validator-invocation-2026-08-07`](r3-mac-evidence-recovery/contracts-and-gates-status.md#aux-r3-mac-evidence-recovery-r3-recovery-current-validator-invocation-2026-08-07).
## AUX-R3-MAC-SYSTEM-KEYCHAIN-SOFTWARE-SIGNER-CORRECTION (2026-08-10)

Canonical content: [`r3-mac-evidence-recovery/contracts-and-gates-status.md#aux-r3-mac-system-keychain-software-signer-correction-2026-08-10`](r3-mac-evidence-recovery/contracts-and-gates-status.md#aux-r3-mac-system-keychain-software-signer-correction-2026-08-10).
## AUX-R3-MAC-EVIDENCE-RETIREMENT-V2 planning contract amendment (2026-08-13; docs-only)

Canonical content: [`r3-mac-evidence-recovery/contracts-and-gates-status.md#aux-r3-mac-evidence-retirement-v2-planning-contract-amendment-2026-08-13-docs-only`](r3-mac-evidence-recovery/contracts-and-gates-status.md#aux-r3-mac-evidence-retirement-v2-planning-contract-amendment-2026-08-13-docs-only).

### Normative authority and chronology

Canonical content: [`r3-mac-evidence-recovery/contracts-and-gates-status.md#normative-authority-and-chronology`](r3-mac-evidence-recovery/contracts-and-gates-status.md#normative-authority-and-chronology).

### Closed prospective transition order

Canonical content: [`r3-mac-evidence-recovery/contracts-and-gates-status.md#closed-prospective-transition-order`](r3-mac-evidence-recovery/contracts-and-gates-status.md#closed-prospective-transition-order).

### Receipt and parity artifact wall

Canonical content: [`r3-mac-evidence-recovery/contracts-and-gates-status.md#receipt-and-parity-artifact-wall`](r3-mac-evidence-recovery/contracts-and-gates-status.md#receipt-and-parity-artifact-wall).

### Fixed external-finalizer boundary

Canonical content: [`r3-mac-evidence-recovery/contracts-and-gates-status.md#fixed-external-finalizer-boundary`](r3-mac-evidence-recovery/contracts-and-gates-status.md#fixed-external-finalizer-boundary).

### Keychain capability and experiment gate

Canonical content: [`r3-mac-evidence-recovery/contracts-and-gates-status.md#keychain-capability-and-experiment-gate`](r3-mac-evidence-recovery/contracts-and-gates-status.md#keychain-capability-and-experiment-gate).

### Closed stops, review matrix, and authority posture

Canonical content: [`r3-mac-evidence-recovery/contracts-and-gates-status.md#closed-stops-review-matrix-and-authority-posture`](r3-mac-evidence-recovery/contracts-and-gates-status.md#closed-stops-review-matrix-and-authority-posture).
## `AUTHORITY_REQUIRED:MACOS_DEV_PARITY` contract (2026-08-19; macOS lane)

Canonical content: [`gates/authority-required-macos-dev-parity.md#authority_requiredmacos_dev_parity-contract-2026-08-19-macos-lane`](gates/authority-required-macos-dev-parity.md#authority_requiredmacos_dev_parity-contract-2026-08-19-macos-lane).

### Admission

Canonical content: [`gates/authority-required-macos-dev-parity.md#admission`](gates/authority-required-macos-dev-parity.md#admission).

### Allowed completion claim

Canonical content: [`gates/authority-required-macos-dev-parity.md#allowed-completion-claim`](gates/authority-required-macos-dev-parity.md#allowed-completion-claim).

### Prohibited ownership and actions

Canonical content: [`gates/authority-required-macos-dev-parity.md#prohibited-ownership-and-actions`](gates/authority-required-macos-dev-parity.md#prohibited-ownership-and-actions).

### Exit and continuation

Canonical content: [`gates/authority-required-macos-dev-parity.md#exit-and-continuation`](gates/authority-required-macos-dev-parity.md#exit-and-continuation).

## `AUTHORITY_REQUIRED:RUNTIME_REFACTOR_REENTRY` contract (2026-08-20; closed selection record)

Canonical content: [`gates/authority-required-runtime-refactor-reentry.md#authority_requiredruntime_refactor_reentry-contract-2026-08-20-closed-selection-record`](gates/authority-required-runtime-refactor-reentry.md#authority_requiredruntime_refactor_reentry-contract-2026-08-20-closed-selection-record).

### Admission

Canonical content: [`gates/authority-required-runtime-refactor-reentry.md#admission`](gates/authority-required-runtime-refactor-reentry.md#admission).

### Allowed completion claim

Canonical content: [`gates/authority-required-runtime-refactor-reentry.md#allowed-completion-claim`](gates/authority-required-runtime-refactor-reentry.md#allowed-completion-claim).

### Prohibited ownership and actions

Canonical content: [`gates/authority-required-runtime-refactor-reentry.md#prohibited-ownership-and-actions`](gates/authority-required-runtime-refactor-reentry.md#prohibited-ownership-and-actions).
