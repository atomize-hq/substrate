# Contracts and Gates

## Normative conventions

- All V1 records are durable, schema-versioned, and reject unknown fields and identity/binding
  ambiguity.
- IDs and refs are opaque. Model-visible callers may receive task/worker handles but may not
  construct internal participant, resume, lease, policy-ref, or UAA-session truth.
- `root_revision`, `authority_revision`, and intent/receipt/manifest revisions increase
  monotonically under the exact atomic persistence rules owned by their contracts.
- Every A1 timestamp is `TimestampV1`; every A1 structured commitment is over the named immutable
  `CanonicalJsonV1` input, never a presentation record.
- A field marked `ref` is a complete `AuthorityObjectRefV1`. Its parent record owns the expected
  kind, schema version, and commitment and verifies all three before use.

### A1 canonical encoding, path identity, supporting types, and persistence

#### `CanonicalJsonV1` and timestamp encoding

`CanonicalJsonV1` is a repository-owned encoder/decoder with this closed algorithm:

1. Input and output are UTF-8 only; invalid UTF-8 and unpaired surrogate values are rejected.
2. Object keys are sorted lexicographically by their exact UTF-8 byte sequence.
3. No insignificant whitespace is emitted.
4. Arrays preserve their exact input/value order.
5. Strings are not Unicode-normalized.
6. Non-control Unicode scalar values are emitted directly as UTF-8.
7. Double quote and backslash are escaped as `\"` and `\\`.
8. U+0000 through U+001F use lowercase `\u00xx`; alternate short escapes such as `\n`, `\t`, or
   `\b` are forbidden.
9. Boolean and null literals are exactly `true`, `false`, and `null`.
10. A1 canonical objects permit only explicitly typed signed or unsigned integers within the
    range of the field declared by the V1 schema. Integers use minimal base-ten form: no leading
    plus, no leading zero except `0`, and no negative zero.
11. Floating-point, decimal, non-finite, and arbitrary JSON numeric values are rejected unless a
    later schema introduces a distinct canonical decimal type.
12. Unknown and duplicate fields are rejected for every hashed V1 object. Every declared field is
    present; an absent Rust `Option` is encoded as JSON `null` rather than by omitting the field.
13. Struct field names are the exact snake_case names shown by this contract. A unit enum variant
    is `{"kind":"VariantName"}`; a variant with fields is
    `{"kind":"VariantName","value":{...}}`. Maps have string keys only. Raw bytes are never
    coerced into JSON strings.
14. Encoding is implemented by the repository-owned `CanonicalJsonV1` path, not delegated to
    `serde_json`, display/debug formatting, or any presentation serializer.

```rust
struct TimestampV1(String);
```

`TimestampV1` encodes as one JSON string containing a valid UTC RFC3339 timestamp with exactly
nine fractional digits and uppercase `Z`, for example
`2026-07-11T12:49:11.000000000Z`. Offsets, omitted or shorter fractions, lowercase `z`, leap-second
`:60`, and non-calendar values are rejected. This is the only A1 timestamp serialization.

#### Exact directory identity

```rust
struct CanonicalDirectoryV1 {
    physical_path: String,
    physical_identity: DirectoryPhysicalIdentityV1,
}

enum DirectoryPhysicalIdentityV1 {
    Linux { device_id: u64, inode: u64 },
    MacOs {
        volume_uuid: String,
        file_id: u64,
        case_sensitive: bool,
    },
}

struct WorkspaceBindingV1 {
    workspace_root: CanonicalDirectoryV1,
    authority_store_root: CanonicalDirectoryV1,
    authority_store_id: String,
}

struct WorldBindingV1 {
    world_id: String,
    world_generation: u64,
}
```

Both input roots must be absolute UTF-8 paths naming existing directories. Raw input containing a
`.` or `..` component, a relative path, an empty path, a nonexistent directory, a file, an ambient
CWD fallback, or a failed-home fallback is rejected before issuance. Workspace-root bootstrap may
resolve symlinks under its existing canonical-identity rules, then opens the resulting directory,
derives the physical path and identity from that trusted handle, and persists them once.
`authority_store_root` is stricter because it is `SUBSTRATE_HOME`: bootstrap opens every component
from the filesystem root with no-follow semantics and rejects any symlink component or final
symlink rather than canonicalizing through it. Authority-store descendant access then remains
directory-relative with no-follow semantics; it never reinterprets the persisted string through
ambient CWD. Workspace content traversal remains governed by its existing execution/policy
contracts; this private-home rule does not narrow or expand it.

#### `PrivateSubstrateHomeV1` acceptance rules

`SUBSTRATE_HOME` is one user's private configuration, policy, dependency-inventory, runtime, and
authority root. On supported Unix hosts, the creator first opens and retains the intended physical
parent and validates from that descriptor that it has the expected type and owner, is not writable
by another principal, has no disallowed ACL grant, and retains the expected physical identity. The
creator then invokes `mkdirat` relative to that trusted parent or joins `AlreadyExists`. Successful
`mkdirat` establishes only a candidate name; it does not establish the accepted child identity.
Portable Linux/macOS APIs do not atomically create a directory and return an inode-bound directory
handle.

The accepted `PrivateSubstrateHomeV1` physical identity begins at the first successful no-follow
directory open of the candidate relative to that same parent descriptor. Descriptor-based
validation then proves all of the following before any descendant bootstrap:

1. the selected absolute UTF-8 physical path is the path represented by the opened handle;
2. the handle names a directory owned by the intended per-user owner;
3. permission and special bits are exactly `0700`;
4. the ACL has only the base owner/group/other access entries represented by mode `0700`, with no
   named user, named group, inherited, default, or other extended ACL entry, even when an ACL mask
   would make an unexpected entry ineffective;
5. no path component or final entry was followed through a symlink or substituted with another
   type, and the accepted platform physical identity is sourced only from the opened descriptor;
   and
6. the candidate resides on the expected filesystem where the trusted-filesystem contract requires
   that constraint.

Creation uses a directory-relative sequence that distinguishes successful creation from
`AlreadyExists`. Legitimate concurrent Substrate creators are supported: one may create while
another observes `AlreadyExists`; each then no-follow opens and exactly validates the candidate,
and only the identity captured from its accepted descriptor proceeds. A newly created root is
never accepted from its requested mode alone. The creator must `fsync` the new directory and its
affected parent where the platform supports the A1 durable-filesystem contract. Ambient umask may
remove bits during creation, but the creator may set the newly opened candidate to exact `0700`
before acceptance; it may never broaden or otherwise repair a root that existed before the
attempt.

After the first accepted open, every access remains descriptor-relative. At required publication
and acceptance boundaries, a no-follow lookup beneath the retained parent must still join the
child name to that accepted descriptor identity and descriptor validation must still prove type,
owner, exact mode, ACL, and required filesystem posture. Rename, replacement, owner/mode/ACL drift,
or validation uncertainty fails closed. No path-based descendant operation is permitted after
binding.

The trusted-parent owner, mode, and ACL rules prevent a different-principal attacker from replacing
the child. A1 V1 does not establish a boundary against malicious root or malicious code already
executing under the same UID, so substitution by either before the first child descriptor is
acquired is outside the threat model. A1.1d-5 makes no atomic create-and-bind claim and requires no
privileged creation broker, protected staging root, or root-owned publication protocol. Such a
broker is an explicit non-goal unless a later separately approved architecture expands the threat
model.

An existing root, including a custom `SUBSTRATE_HOME`, is accepted only if it already passes the
same rules. Wrong type, symlink, owner mismatch, `0755`, `0750`, any group/world permission,
setuid/setgid/sticky or other special bits, foreign or inherited ACL grants, changed identity, or
an indeterminate check returns exactly this diagnostic shape before any descendant write:

```text
substrate: unsupported SUBSTRATE_HOME '<path>': expected a private directory owned by intended uid <uid> with exact mode 0700 and no foreign ACL grants; found <reason>. Existing roots are never repaired; reset it manually and retry.
```

When privileged execution cannot resolve an intended non-root account, no intended UID exists to
render in that shape. It fails before inspecting or creating the root with this separate exact
diagnostic:

```text
substrate: unsupported SUBSTRATE_HOME '<path>': cannot determine the intended per-user owner while running with effective uid 0; found owner-ambiguous. Set the supported explicit user input or run as the intended user; no home was created or modified.
```

The generic diagnostic's `<reason>` token is one of `missing-parent`, `wrong-type`, `symlink`,
`wrong-owner`, `wrong-mode`, `foreign-acl`, `replaced`, or `validation-unavailable`. For an
ordinary non-root process, the intended owner is its effective UID. An installer or provisioner
running as root resolves the intended account from its explicit supported user input first
(`SUBSTRATE_INSTALL_PRIMARY_USER` where applicable), then its verified invoking-user signal such
as `SUDO_USER`; macOS/Lima guest provisioning uses the discovered Lima VM user. It resolves that
account through the platform account database and uses the resulting UID. Root execution with no
unambiguous intended non-root user fails as `owner-ambiguous`; it does not silently create a
root-owned user state home. Descriptor-bound owner and exact-mode initialization is permitted only
for the candidate created by the current attempt; a candidate joined through `AlreadyExists` is a
pre-existing root. No product path chmods, chowns, removes ACLs from, deletes contents from,
migrates, adopts, converts, or repairs a pre-existing root or falls back to a shared home. Failure
occurs before config/runtime scaffolding and before any authority marker, key, root, or legacy state
mutation. Repeating creation against an unchanged valid root is idempotent.

Exact `0700` applies to the `SUBSTRATE_HOME` root. Existing stricter authority-store descendant
contracts remain unchanged: authority directories remain owner-only `0700` and authority files
remain owner-only `0600`. Multiple operating-system users directly sharing/traversing one home are
unsupported in A1 V1. A future separate installation root does not weaken this state-root contract;
`SUBSTRATE_ROOT` separation is not implemented here.

Acceptance changes no policy or world semantics. Effective-policy snapshots, world requests,
network routing, filesystem enforcement plans, allow/deny lists, isolation and host-visibility
flags, and policy hashes must remain identical for identical inputs. World members receive
config/policy/dependency/credential material through existing Substrate-owned projection or
mediation. If an unprivileged world process requires direct home traversal, A1.1d-5 stops as a
capability-boundary change instead of broadening permissions.

`authority_store_root` is also the one normalized bootstrap home. Host bootstrap resolves that
home once, before StateStore construction or config, policy, and inventory resolution, and passes
the same opened `CanonicalDirectoryV1` identity to all four consumers. The value persisted in
`WorkspaceBindingV1.authority_store_root` must equal that bootstrap-home identity byte-for-byte and
by physical identity, and must equal `StateRootV1.bootstrap_home` before any authority record,
intent, or object is accepted. No issuer, helper, retry, restart, reconciliation, config resolver,
policy resolver, or inventory resolver may later reread `SUBSTRATE_HOME`, fall back to `$HOME`, or
resolve the home relative to ambient CWD. A different home, even if valid and content-equivalent,
is a binding mismatch.

For effective-policy resolution, host bootstrap derives the global policy source from that accepted
home without rereading ambient environment state. The shell passes an explicit broker-appropriate
source input, not a shell authority type, to one bounded broker entry point. The broker remains the
sole owner and applies the same canonical defaults, global patch parsing, workspace discovery and
patch parsing, patch precedence, explanation provenance, validation, and finalization used by its
ambient API. The explicit-source and ambient APIs must return semantically identical finalized
policy, source paths/layers, explanation output, derived legacy and V3 filesystem fields,
network/backend/dispatch values, and validation errors for semantically identical inputs.

Conflicting ambient and explicit homes must prove that only the accepted explicit source affects the
explicit result; global environment mutation may not simulate this API. Malformed or unsafe global
or workspace input fails consistently with the ambient canonical path. Config validation preserves
its existing conditional policy-parsing behavior and must not read or require policy when the
canonical ambient path would not. A missing descriptor entry never bypasses exact store, workspace,
session, reference, policy, or snapshot identity validation. Shell projection may record or validate
the canonical broker result, but it may not duplicate policy parsing, layering, explanation,
validation, or finalization. This A1.1e rule does not authorize dispatch-scoped narrowing, worker-cap
composition, immutable active-work acceptance, policy schema/precedence/default changes, or policy
enforcement changes.

Issuer, helper validation/application, retry, restart, and reconciliation all use this
same resolver and comparison rule. None may substitute lexical normalization, ambient CWD, a
different case rule, or path-only equality.

On Linux, comparison requires byte-for-byte equality of the selected UTF-8 physical path and exact
`(device_id, inode)`. On macOS, the path comes from the opened directory's physical path with the
filesystem's actual component spelling; no case-folding or Unicode normalization is performed.
The exact volume UUID, file ID, and case-sensitivity flag must also match. Thus case aliases on a
case-insensitive volume converge only through the same opened physical directory, case-distinct
entries on a case-sensitive volume remain distinct, and volume replacement or rebinding fails
closed. Windows A1 authority initialization and transitions are unsupported and fail closed until
a later contract defines and proves drive-relative versus absolute paths, drive/UNC identity,
separator normalization, reparse-point/junction resolution, case and volume identity, owner-only
ACLs, locking, and atomic durable replacement. No Windows path is silently mapped to the Unix or
macOS model.

#### Typed immutable object references

```rust
enum AuthorityObjectKindV1 {
    AgentDescriptor,
    RetainedWorker,
    ResumeHandle,
    Policy,
    HostAttachContract,
    TransitionTransportPayload,
    TransitionInput,
    LeaseToken,
    ApplicationResult,
    InputAcceptance,
    StartupOwnershipResult,
    ObligationSnapshot,
    PostTurnProtocolEvent,
    PostTurnCompletion,
    TerminalHandoff,
}

enum AuthorityObjectCommitmentV1 {
    CanonicalSha256 {
        digest_hex: String,
    },
    StoreHmacSha256 {
        key_id: String,
        domain: String,
        digest_hex: String,
    },
}

struct AuthorityObjectRefV1 {
    ref_id: String,
    object_kind: AuthorityObjectKindV1,
    schema_version: u32,
    commitment: AuthorityObjectCommitmentV1,
}

type ResumeHandleRefV1 = AuthorityObjectRefV1;
type PolicyRefV1 = AuthorityObjectRefV1;
```

An A1 `ref_id` is exactly `ao_` plus 32 lowercase hexadecimal characters encoding 128 OS-CSPRNG
bits, generated and collision-checked under the root lock. It is never supplied as a path or
derived from caller text. The final location is
`objects/<kind-slug>/v<schema_version>/<ref_id>.obj`, with minimal-decimal schema version and this
closed kind-slug mapping:

```text
AgentDescriptor=agent-descriptor
RetainedWorker=retained-worker
ResumeHandle=resume-handle
Policy=policy
HostAttachContract=host-attach-contract
TransitionTransportPayload=transition-transport-payload
TransitionInput=transition-input
LeaseToken=lease-token
ApplicationResult=application-result
InputAcceptance=input-acceptance
StartupOwnershipResult=startup-ownership-result
ObligationSnapshot=obligation-snapshot
PostTurnProtocolEvent=post-turn-protocol-event
PostTurnCompletion=post-turn-completion
TerminalHandoff=terminal-handoff
```

Publication has no-replace semantics. Exact retry may join an existing final object only after
verifying its complete ref and bytes.

The parent record carries the full `AuthorityObjectRefV1`; `object_index` metadata is routing and
lifecycle data only. Wrong kind, wrong schema version, unexpected commitment variant/domain/key,
or commitment mismatch fails closed. Replacing object bytes together with object-index metadata
cannot change the parent-owned commitment. Sensitive/raw objects use `StoreHmacSha256`.
Non-sensitive, closed structured objects may use `CanonicalSha256`. All digests are exactly 64
lowercase hexadecimal characters. A canonical object's file bytes are exactly its named
`CanonicalJsonV1` hash-input bytes; a sensitive object's file bytes are exactly the HMAC input's
`raw` field. Raw provider credentials are prohibited from every A1 object.

#### Named immutable hash inputs

```rust
struct DurableSessionAuthorityHashInputV1 {
    schema_version: u32,
    orchestration_session_id: String,
    shell_trace_session_id: String,
    authority_revision: u64,
    origin: DurableSessionAuthorityOriginV1,
    authoritative_participant_lineage: Vec<String>,
    active_authoritative_participant_id: Option<String>,
    workspace_binding: WorkspaceBindingV1,
    world_binding: Option<WorldBindingV1>,
    host_attach_contract_ref: Option<AuthorityObjectRefV1>,
    retained_worker_refs: Vec<AuthorityObjectRefV1>,
    internal_resume_handle_refs: Vec<AuthorityObjectRefV1>,
    lifecycle_posture: HostSessionPostureV1,
    current_policy_ref: Option<AuthorityObjectRefV1>,
    current_policy_revision: Option<String>,
}

struct AuthoritativeLineageHashInputV1 {
    schema_version: u32,
    orchestration_session_id: String,
    participant_ids: Vec<String>,
}

enum AgentExecutionScopeV1 {
    Host,
    World,
}

enum RuntimeBackendKindV1 {
    Codex,
    ClaudeCode,
}

struct AgentDescriptorV1 {
    schema_version: u32,
    agent_id: String,
    backend_id: String,
    backend_kind: RuntimeBackendKindV1,
    protocol: String,
    execution_scope: AgentExecutionScopeV1,
    binary_path: String,
}

struct HostAttachCapabilitiesV1 {
    session_resume: bool,
    session_fork: bool,
    session_stop: bool,
    status_snapshot: bool,
    event_stream: bool,
}

enum HostAttachExecutionClientStartV1 {
    StartNow,
    Defer,
}

enum HostAttachModePreferenceV1 {
    ContinuityRequired,
    ContinuityPreferred,
    FreshAllowed,
}

struct HostAttachLaunchKnobsV1 {
    requested_execution_scope: AgentExecutionScopeV1,
    host_execution_client_start: HostAttachExecutionClientStartV1,
    attach_mode_preference: HostAttachModePreferenceV1,
}

struct HostAttachContractV1 {
    schema_version: u32,
    backend_id: String,
    execution_scope: AgentExecutionScopeV1,
    protocol: String,
    descriptor_ref: AuthorityObjectRefV1,
    capabilities: HostAttachCapabilitiesV1,
    attach_launch_knobs: HostAttachLaunchKnobsV1,
    policy_ref: AuthorityObjectRefV1,
    continuity_resume_handle_ref: Option<AuthorityObjectRefV1>,
}

struct HostSessionTransitionPayloadHashInputV1 {
    schema_version: u32,
    intent_id: String,
    issuer_request_id: String,
    mode: HostSessionTransitionModeV1,
    authority_precondition: HostSessionAuthorityPreconditionV1,
    orchestration_session_id: String,
    shell_trace_session_id: String,
    caller: HostSessionTransitionCallerV1,
    source_authoritative_participant_id: Option<String>,
    target_authoritative_participant_id: String,
    target_participant_lease_token_ref: AuthorityObjectRefV1,
    run_id: String,
    resulting_authoritative_lineage: Vec<String>,
    workspace_binding: WorkspaceBindingV1,
    world_binding: Option<WorldBindingV1>,
    descriptor_ref: AuthorityObjectRefV1,
    host_attach_contract_ref: AuthorityObjectRefV1,
    resume_handle_ref: Option<AuthorityObjectRefV1>,
    transition_input_ref: Option<AuthorityObjectRefV1>,
    post_turn_disposition: Option<HostPostTurnDispositionV1>,
    transport_payload_ref: AuthorityObjectRefV1,
    issued_at: TimestampV1,
    expires_at: TimestampV1,
}

struct TransitionTransportPayloadObjectV1 {
    schema_version: u32,
    intent_id: String,
    mode: HostSessionTransitionModeV1,
    orchestration_session_id: String,
    shell_trace_session_id: String,
    caller: HostSessionTransitionCallerV1,
    source_authoritative_participant_id: Option<String>,
    target_authoritative_participant_id: String,
    target_participant_lease_token_ref: AuthorityObjectRefV1,
    run_id: String,
    resulting_authoritative_lineage: Vec<String>,
    workspace_binding: WorkspaceBindingV1,
    world_binding: Option<WorldBindingV1>,
    descriptor_ref: AuthorityObjectRefV1,
    host_attach_contract_ref: AuthorityObjectRefV1,
    resume_handle_ref: Option<AuthorityObjectRefV1>,
    transition_input_ref: Option<AuthorityObjectRefV1>,
    post_turn_disposition: Option<HostPostTurnDispositionV1>,
}

struct AgentDescriptorHashInputV1 {
    schema_version: u32,
    descriptor: AgentDescriptorV1,
}

struct HostAttachContractHashInputV1 {
    schema_version: u32,
    contract: HostAttachContractV1,
}

struct ResumeHandleHashInputV1 {
    schema_version: u32,
    orchestration_session_id: String,
    participant_id: String,
    backend_id: String,
    protocol: String,
    internal_uaa_session_id: String,
}

struct PolicyObjectHashInputV1 {
    schema_version: u32,
    policy_revision: String,
    canonical_policy_snapshot_sha256: String,
}

struct RetainedWorkerObjectHashInputV1 {
    schema_version: u32,
    orchestration_session_id: String,
    participant_id: String,
    world_binding: WorldBindingV1,
    descriptor_ref: AuthorityObjectRefV1,
    resume_handle_ref: AuthorityObjectRefV1,
    policy_ref: AuthorityObjectRefV1,
}

enum ApplicationResultPhaseV1 {
    InitialTransition {
        authority_revision_before: Option<u64>,
        authority_revision_after: u64,
        active_authoritative_participant_id: String,
        resulting_posture: HostSessionPostureV1,
        authority_record_commitment: AuthorityObjectCommitmentV1,
        post_turn_pending_run_id: Option<String>,
    },
    PostTurn {
        completion_ref: AuthorityObjectRefV1,
        obligation_snapshot_ref: Option<AuthorityObjectRefV1>,
        authority_revision_before: u64,
        authority_revision_after: u64,
        active_authoritative_participant_id: String,
        resulting_posture: HostSessionPostureV1,
        authority_record_commitment: AuthorityObjectCommitmentV1,
    },
}

struct ApplicationResultHashInputV1 {
    schema_version: u32,
    intent_id: String,
    mode: HostSessionTransitionModeV1,
    run_id: String,
    phase: ApplicationResultPhaseV1,
    applied_at: TimestampV1,
}

struct InputAcceptanceHashInputV1 {
    schema_version: u32,
    intent_id: String,
    run_id: String,
    input_ref: AuthorityObjectRefV1,
    accepting_participant_id: String,
    accepted_at: TimestampV1,
}

enum HostStartupTerminalReasonV1 {
    RuntimeCreationRejected,
    StartupFailedBeforeOwnership,
}

enum HostStartupOwnershipProtocolEventV1 {
    OwnershipAccepted {
        ownership_acknowledgement_id: String,
    },
    RuntimeCreationRejected {
        rejection_id: String,
    },
    StartupFailedBeforeOwnership {
        failure_id: String,
    },
}

enum HostStartupOwnershipProtocolActorV1 {
    TargetAuthoritativeParticipant {
        participant_id: String,
    },
    LaunchApplicationClaimant {
        claim_id: String,
        claimant_attempt_id: String,
    },
}

struct HostStartupOwnershipEvidenceV1 {
    schema_version: u32,
    evidence_id: String,
    authority_store_id: String,
    orchestration_session_id: String,
    intent_id: String,
    claim_id: String,
    claimant_attempt_id: String,
    run_id: String,
    application_result_ref: AuthorityObjectRefV1,
    expected_authority_revision: u64,
    active_authoritative_participant_id: String,
    protocol_actor: HostStartupOwnershipProtocolActorV1,
    protocol_event: HostStartupOwnershipProtocolEventV1,
    observed_at: TimestampV1,
}

enum StartupOwnershipOutcomeV1 {
    Accepted,
    TerminalReconciled {
        reason: HostStartupTerminalReasonV1,
        authority_revision_after: u64,
        resulting_posture: HostSessionPostureV1,
        authority_record_commitment: AuthorityObjectCommitmentV1,
    },
}

struct StartupOwnershipResultHashInputV1 {
    schema_version: u32,
    evidence: HostStartupOwnershipEvidenceV1,
    outcome: StartupOwnershipOutcomeV1,
    resolved_at: TimestampV1,
}

enum ObligationSnapshotRecordStateV1 {
    UnresolvedAttention,
}

struct ObligationSnapshotRecordHashInputV1 {
    schema_version: u32,
    authority_store_id: String,
    orchestration_session_id: String,
    authoritative_participant_id: String,
    source_journal_event: SupervisorJournalEventRefV1,
    obligation_id: String,
    obligation_revision: u64,
    state: ObligationSnapshotRecordStateV1,
}

struct SupervisorJournalEventRefV1 {
    schema_version: u32,          // exactly 1
    journal_entry_id: String,
    acceptance_record_id: String,
    acceptance_record_revision: u64,
    accepted_work_identity: AcceptedWorldWorkIdentityV1,
    stream_id: String,
    frame_sequence: u64,
    event_id: String,
    event_sequence: u64,
    transport_event_commitment: AuthorityObjectCommitmentV1,
}

struct UnresolvedAttentionObligationSnapshotEntryV1 {
    obligation_id: String,
    obligation_revision: u64,
    canonical_record_commitment: AuthorityObjectCommitmentV1,
}

enum ObligationAttentionDispositionV1 {
    NoUnresolvedAttention,
    HasUnresolvedAttention,
}

struct ObligationMaterializationCutV1 {
    acceptance_record_id: String,
    acceptance_record_revision: u64,
    stream_id: String,
    session_ledger_revision: u64,
    terminal_event_id: String,
    terminal_event_sequence: u64,
    materialized_through_event_sequence: u64,
}

struct ObligationSnapshotHashInputV1 {
    schema_version: u32,
    authority_store_id: String,
    orchestration_session_id: String,
    authoritative_participant_id: String,
    acceptance_record_id: String,
    acceptance_record_revision: u64,
    stream_id: String,
    accepted_work_identity: AcceptedWorldWorkIdentityV1,
    host_transition_correlation: HostTransitionWorkCorrelationV1,
    transition_intent_id: String,
    transition_run_id: String,
    authority_revision_observed: u64,
    materialization_cut: ObligationMaterializationCutV1,
    materialized_journal_events: Vec<SupervisorJournalEventRefV1>,
    attention_disposition: ObligationAttentionDispositionV1,
    unresolved_attention_obligations: Vec<UnresolvedAttentionObligationSnapshotEntryV1>,
    captured_at: TimestampV1,
}

enum ObligationLedgerSnapshotReadV1 {
    Pending {
        authority_store_id: String,
        orchestration_session_id: String,
        authoritative_participant_id: String,
        acceptance_record_id: String,
        acceptance_record_revision: u64,
        stream_id: String,
        accepted_work_identity: AcceptedWorldWorkIdentityV1,
        host_transition_correlation: HostTransitionWorkCorrelationV1,
        transition_intent_id: String,
        transition_run_id: String,
        authority_revision_observed: u64,
        observed_session_ledger_revision: u64,
        required_terminal_event_id: String,
        required_terminal_event_sequence: u64,
    },
    Complete {
        snapshot: ObligationSnapshotHashInputV1,
    },
}

enum PostTurnCompletionOutcomeV1 {
    ResumableClean,
    TerminalClean,
    TerminalFailure,
}

enum HostPostTurnTerminalReasonV1 {
    ResumeRuntimeCreationRejected,
    TargetFailedBeforeInputAcceptance,
    TargetFailedAfterInputAcceptance,
}

enum HostPostTurnProtocolEventKindV1 {
    ResumableClean,
    TerminalClean,
    TerminalFailure {
        reason: HostPostTurnTerminalReasonV1,
    },
}

enum HostPostTurnProtocolActorV1 {
    TargetAuthoritativeParticipant {
        participant_id: String,
    },
    LaunchApplicationClaimant {
        claim_id: String,
        claimant_attempt_id: String,
    },
}

struct PostTurnProtocolEventHashInputV1 {
    schema_version: u32,
    authority_store_id: String,
    orchestration_session_id: String,
    intent_id: String,
    claim_id: String,
    claimant_attempt_id: String,
    run_id: String,
    authority_revision_observed: u64,
    active_authoritative_participant_id: String,
    acceptance_record_id: String,
    acceptance_record_revision: u64,
    stream_id: String,
    accepted_work_identity: AcceptedWorldWorkIdentityV1,
    host_transition_correlation: HostTransitionWorkCorrelationV1,
    protocol_actor: HostPostTurnProtocolActorV1,
    event_id: String,
    event_sequence: u64,
    kind: HostPostTurnProtocolEventKindV1,
    emitted_at: TimestampV1,
}

struct PostTurnCompletionHashInputV1 {
    schema_version: u32,
    intent_id: String,
    run_id: String,
    authority_revision_observed: u64,
    acceptance_record_id: String,
    acceptance_record_revision: u64,
    stream_id: String,
    accepted_work_identity: AcceptedWorldWorkIdentityV1,
    host_transition_correlation: HostTransitionWorkCorrelationV1,
    terminal_event_id: String,
    terminal_event_sequence: u64,
    protocol_event_ref: AuthorityObjectRefV1,
    outcome: PostTurnCompletionOutcomeV1,
    completed_at: TimestampV1,
}

enum TerminalHandoffStateV1 {
    Applied,
    Rejected { reason: HostSessionTransitionTerminalRejectionV1 },
    Expired,
}

struct TerminalHandoffHashInputV1 {
    schema_version: u32,
    intent_id: String,
    run_id: String,
    payload_commitment: AuthorityObjectCommitmentV1,
    terminal_state: TerminalHandoffStateV1,
    application_result_ref: Option<AuthorityObjectRefV1>,
    input_acceptance_ref: Option<AuthorityObjectRefV1>,
    startup_ownership_result_ref: Option<AuthorityObjectRefV1>,
    post_turn_completion_ref: Option<AuthorityObjectRefV1>,
    post_turn_application_result_ref: Option<AuthorityObjectRefV1>,
    recorded_at: TimestampV1,
}
```

`SupervisorJournalEventRefV1.transport_event_commitment` is the B2.1-owned `CanonicalSha256` over
the complete canonical runtime event bytes. The supervisor treats those bytes as opaque transport
truth: it verifies only B0 identity/order, B1 scope, canonical-byte equality, and journal replay.
That generic contract lets B2.1 close without a B3.1 semantic dependency. Once B3.1 adopts an
accepted retained stream, every post-acknowledgement `ExecuteStreamFrame::Event` before the exact
terminal frame must decode exactly as the shared `WorldWorkerEventV1`; there is no producer-chosen
"C1-eligible" subset. B3.1 validates each event's source/target participants and backends, world,
accepted active run, thread, event class, attention bit, request/message causation, owner-supplied
host-transition correlation when present, payload, and emission time against the B0/B1 join. A
missing or invalid typed member makes the stream protocol-invalid and keeps the C1 cut
non-Complete; neither B3.1 nor C1 may drop it. C1 revalidates that each generic journal commitment
is the canonical commitment of its full envelope before classification, materialization, or
snapshot capture. The canonical obligation record commits this typed ref, so a Complete snapshot
cannot drop or substitute any retained event field.

`ObligationSnapshotHashInputV1.materialized_journal_events` contains every post-acknowledgement
retained `Event` journal ref in the exact acceptance/stream scope at or below the
materialized-through watermark, sorted by `event_sequence`, with unique journal entry, frame, and
event identities. It may be empty only when that closed retained scope contains no `Event` frame.
The vector must be an exhaustive join against B2.1's journal, not a set chosen by event class or
attention value. Every unresolved obligation entry's canonical source ref names an exact member of
this vector. The vector is part of the snapshot hash, so `NoUnresolvedAttention` still commits the
complete classified retained-event set and cannot prove completeness from a terminal watermark
alone.

For a transition-scoped obligation read, `accepted_work_identity` and
`host_transition_correlation` are copied from the exact B1 acceptance record, and
`acceptance_record_id`/revision plus `stream_id` match the B2.1 journal and materialization cut.
The top-level `transition_intent_id`, `transition_run_id`, and
`authority_revision_observed` exactly equal the same fields in `host_transition_correlation`; the
accepted task/active-run identity remains distinct. Missing or mismatched correlation cannot return
Complete and is never repaired from request IDs, active-run IDs, payloads, foreground state, or
A1.2 guesses.

Terminal handoff refs have this closed V1 matrix; `Some` refs must have the exact named kind and
equal the corresponding intent substate/journal ref, and every unlisted ref is `None`:

| Terminal state and mode | Initial application | Input acceptance | Startup ownership | Post-turn completion/result |
|---|---|---|---|---|
| `Rejected` or `Expired`, every mode | none | none | none | none |
| `Applied Start` or `Applied Attach` | exact initial `ApplicationResult` | exact `InputAcceptance` iff a Start input is Accepted; none only for input-free mode or a terminally reconciled Start whose input is exactly `TerminalWithoutAcceptance` | exact `StartupOwnershipResult` in Accepted or TerminalReconciled | none |
| `Applied ResumeOneTurn`, every post-turn Applied outcome | exact initial `ApplicationResult` | exact `InputAcceptance` for Accepted input; none only for `TerminalFailure` with exact `TerminalWithoutAcceptance` | none | exact `PostTurnCompletion` and post-turn `ApplicationResult` pair; `ResumableClean` additionally carries the exact Complete obligation snapshot in the post-turn application, while `TerminalClean`/`TerminalFailure` carry none |

An applied Resume cannot bypass post-turn application. `ResumableClean` and `TerminalClean`
require exact Accepted input. `TerminalFailure` requires either Accepted input or, only for an
exact pre-acceptance terminal event, atomically changes Pending input to
`TerminalWithoutAcceptance` using that same terminal handoff. A terminally reconciled applied
Start likewise atomically terminalizes any Pending input; an ownership-Accepted Start with Pending
input remains nonterminal and retained. `AwaitingObligationCut` is nonterminal and cannot release
transport or have a terminal handoff. A startup result protocol-event/outcome mismatch, a
half-present post-turn pair, a forbidden cross-mode ref, or any ref not equal to parent/journal
truth is corruption and fails closed.

`CanonicalSha256` is exactly lowercase-hex
`SHA-256(CanonicalJsonV1(named_hash_input))`. The authority-record commitment uses
`DurableSessionAuthorityHashInputV1`; lineage uses `AuthoritativeLineageHashInputV1`; immutable
intent payload uses `HostSessionTransitionPayloadHashInputV1`; descriptor and attach-contract
objects use their named wrappers; resume handles, policy objects, and retained-worker reference
objects use `ResumeHandleHashInputV1`, `PolicyObjectHashInputV1`, and
`RetainedWorkerObjectHashInputV1`; both initial and post-turn application objects use
`ApplicationResultHashInputV1`; input acceptance, startup-ownership resolution, obligation
snapshot records/snapshots, post-turn protocol events/completions, and terminal-handoff objects use their corresponding named
wrappers. `HostStartupOwnershipEvidenceV1` is embedded in and committed by the startup result;
each obligation snapshot entry carries the exact `CanonicalSha256` commitment of its
`ObligationSnapshotRecordHashInputV1`.
`updated_at`, intent/claim/root revisions not explicitly present in a
wrapper, mutable state, presentation-only fields, plan/socket paths, and adjacent object metadata
are not accidentally swept into a hash. Authority-record, lineage, payload, descriptor,
attach-contract, resume-handle, policy, retained-worker, application, input acceptance,
startup-ownership resolution, obligation snapshot, post-turn protocol event/completion, and terminal-handoff commitments
must use `CanonicalSha256`; a `StoreHmacSha256` variant in those
fields fails closed.

`AgentDescriptorV1` and `HostAttachContractV1` are the closed typed projections above, not
arbitrary `serde_json::Value`. The policy object commits the exact already-resolved policy revision
and canonical snapshot digest; A1 preserves those accepted values without rereading or
reinterpreting inventory or policy. The resume handle binds the exact internal UAA continuity
identity to session, participant, backend, and protocol. Unknown projection fields fail closed.
Descriptor IDs/protocol/binary path and resume-handle identities are non-empty; the policy snapshot
digest is exactly 64 lowercase hexadecimal characters. A HostAttachContract descriptor ref must be
kind `AgentDescriptor`, its policy ref kind `Policy`, and its optional continuity ref kind
`ResumeHandle`; their backend, protocol, scope, session, and participant identities must agree with
the parent contract/intent. A retained-worker object must use kinds `AgentDescriptor`,
`ResumeHandle`, and `Policy` for its three refs and match the same session/world identity. Any
disagreement or cross-kind substitution fails before use.

Every `AuthorityObjectKindV1` has exactly one V1 bytes and commitment rule:

| Object kind | `schema_version` | Exact file bytes | Required commitment |
|---|---:|---|---|
| `AgentDescriptor` | 1 | `CanonicalJsonV1(AgentDescriptorHashInputV1)` | `CanonicalSha256` |
| `RetainedWorker` | 1 | `CanonicalJsonV1(RetainedWorkerObjectHashInputV1)` | `CanonicalSha256` |
| `ResumeHandle` | 1 | `CanonicalJsonV1(ResumeHandleHashInputV1)` | `CanonicalSha256` |
| `Policy` | 1 | `CanonicalJsonV1(PolicyObjectHashInputV1)` | `CanonicalSha256` |
| `HostAttachContract` | 1 | `CanonicalJsonV1(HostAttachContractHashInputV1)` | `CanonicalSha256` |
| `TransitionTransportPayload` | 1 | `CanonicalJsonV1(TransitionTransportPayloadObjectV1)`; those canonical bytes are the HMAC `raw` member | `StoreHmacSha256` with `substrate.a1.raw-transport-payload.v1` |
| `TransitionInput` | 1 | the exact input byte string, with no newline, JSON, or text coercion | `StoreHmacSha256` with `substrate.a1.transition-input.v1` |
| `LeaseToken` | 1 | the exact UTF-8 lease-token bytes, with no normalization | `StoreHmacSha256` with `substrate.a1.participant-lease-token.v1` |
| `ApplicationResult` | 1 | `CanonicalJsonV1(ApplicationResultHashInputV1)` | `CanonicalSha256` |
| `InputAcceptance` | 1 | `CanonicalJsonV1(InputAcceptanceHashInputV1)` | `CanonicalSha256` |
| `StartupOwnershipResult` | 1 | `CanonicalJsonV1(StartupOwnershipResultHashInputV1)` | `CanonicalSha256` |
| `ObligationSnapshot` | 1 | `CanonicalJsonV1(ObligationSnapshotHashInputV1)` | `CanonicalSha256` |
| `PostTurnProtocolEvent` | 1 | `CanonicalJsonV1(PostTurnProtocolEventHashInputV1)` | `CanonicalSha256` |
| `PostTurnCompletion` | 1 | `CanonicalJsonV1(PostTurnCompletionHashInputV1)` | `CanonicalSha256` |
| `TerminalHandoff` | 1 | `CanonicalJsonV1(TerminalHandoffHashInputV1)` | `CanonicalSha256` |

`TransitionTransportPayloadObjectV1` is the closed replay/reprojection schema; it contains typed
refs to raw lease/input objects rather than copying their bytes. For all three sensitive kinds, the
named HMAC input below is the exact commitment wrapper and its `raw` member is exactly the file
bytes. Transition input is a length-preserved arbitrary byte string; lease token is a non-empty
UTF-8 byte string without normalization; neither is parsed as JSON. Kind, schema version, typed
path, parent-held ref, and the required commitment variant/domain are all checked before bytes are
read or used. A ref cannot be accepted under another kind even when file bytes happen to match, and
no adjacent index row can override the parent's kind, schema, or commitment.

Golden fixtures must commit exact `CanonicalJsonV1` bytes and SHA-256 digest, plus fixed test-key
HMAC values where sensitive refs occur. Start, Attach, and `ResumeOneTurn`-with-input fixtures use
`HostSessionTransitionPayloadHashInputV1`; authority, lineage, attach-contract, application-result,
input-acceptance, startup-ownership-result, obligation-snapshot-record, obligation-snapshot,
post-turn-protocol-event, post-turn-completion, terminal-handoff, descriptor, resume-handle, policy, and retained-worker
fixtures use their named wrappers; the typed-ref fixture uses `AuthorityObjectRefV1` itself. The
fixture set covers every allowed `HostStartupOwnershipProtocolEventV1` actor/variant pairing and
terminal reason plus rejected cross-actor pairings, both
`ObligationAttentionDispositionV1` variants, every `HostPostTurnProtocolEventKindV1` variant and
terminal reason/actor combination, a wrong-claimant-attempt rejection, and every terminal-handoff
matrix row. Raw input,
lease-token, and transport fixtures commit both exact bytes and fixed-key HMAC outputs. Each sensitive-domain
fixture also rejects `run_present=0x00`, an empty run, and a different run from the parent intent.
Issuer, helper,
restart/reconciliation, and release tests consume the same fixture files rather than regenerating
expected bytes through the code under test. Test HMAC keys are fixture-only and can never be
selected by a production store.

#### Keyed commitments for sensitive payloads

```rust
struct AuthorityStoreCommitmentKeyV1 {
    schema_version: u32,
    authority_store_id: String,
    key_id: String,
    algorithm: AuthorityStoreCommitmentAlgorithmV1,
    created_at: TimestampV1,
    state: AuthorityStoreCommitmentKeyStateV1,
}

struct AuthorityStoreInitializationV1 {
    schema_version: u32,
    authority_store_id: String,
    bootstrap_home: CanonicalDirectoryV1,
    initial_key_id: String,
    created_at: TimestampV1,
}

enum AuthorityStoreCommitmentAlgorithmV1 {
    HmacSha256,
}

enum AuthorityStoreCommitmentKeyStateV1 {
    Active,
    VerificationOnly,
    Retired,
}
```

Store initialization generates a 256-bit HMAC key from the OS CSPRNG while holding the root lock.
`key_id` is exactly `ak_` plus 32 lowercase hexadecimal characters encoding a separate 128-bit
OS-CSPRNG identifier.
The lifecycle record is stored only in `StateRootV1.commitment_key_registry`; it contains no key
bytes. Secret key bytes live outside the root at `authority-v1/keys/<key_id>.key`. Each key file is
an immutable binary `AuthorityStoreCommitmentKeyFileV1` envelope with this exact byte encoding:

```text
"substrate.a1.commitment-key-file.v1\0"
 || u32be(1)
 || len64(authority_store_id) || authority_store_id
 || len64(key_id) || key_id
 || 0x01
 || len64(created_at) || created_at
 || u32be(32) || 32 secret key bytes
```

Lengths are unsigned 64-bit big-endian; strings are exact UTF-8; `created_at` is the exact
`TimestampV1` string; `0x01` is the sole `HmacSha256` algorithm tag; the secret-key member is exactly
32 bytes; and the complete envelope has no trailing bytes. The file is written through
`tmp/key--<key_id>--<nonce>.tmp`, `fsync`ed, atomically renamed with no-replace semantics, and
followed by `fsync` of `keys/`. The opened complete envelope must be regular, owner-only, and
no-follow. With a valid root its filename, store ID, key ID, timestamp, and algorithm must match the
registry entry exactly. Lifecycle state exists only in the root registry: `Active` may create and
verify commitments, `VerificationOnly` may verify only, and `Retired` may do neither. During
`InitializationPending`, where no root/registry exists yet, it instead must match the strictly
decoded init marker's `authority_store_id`, `initial_key_id`, and `created_at`, use algorithm tag
`0x01`/`HmacSha256`, and is provisionally the sole `Active` key for the root that marker may create.
No other metadata source is accepted. Keys and raw values are never logged, traced, exported,
placed in helper plans, or included in diagnostics. Only local authority processes already
authorized for the store may open the key through the trusted store handle.

Exactly one registry entry is `Active`, and its key ID equals
`StateRootV1.active_commitment_key_id`. New sensitive records use only that key. Rotation is one
serialized protocol under the root lock:

1. Generate a new key ID, metadata record, and 32-byte key from the OS CSPRNG.
2. Publish and `fsync` the immutable key file and `keys/` directory.
3. Revalidate the still-locked root and commit one new root revision that inserts the new entry as
   `Active`, changes the former active entry to `VerificationOnly`, and changes
   `active_commitment_key_id` to the new key.
4. `fsync` the root parent before reporting rotation success.

A crash before step 3 leaves an unregistered key-file orphan. After a complete valid root and all
reachable refs are verified under the lock, an unregistered key ID has no commitment authority and
is deleted with a following `fsync(keys/)`; a ref naming an unregistered key is instead corruption.
A crash after step 3 is a committed rotation: both key files must exist, the new key is active, and
the previous key is verification-only. There is no mutable state field in a key file and therefore
no file/root disagreement to arbitrate.

The prior key remains `VerificationOnly` while any reachable live, terminal, retained, tombstoned,
journal, or application record references it. Retirement first performs a complete locked
reachability scan proving zero references, then commits a root revision changing that registry
entry to permanent `Retired`. Only after that root and parent directory are
`fsync`ed may the key file be deleted and `keys/` be `fsync`ed. A crash with `Retired` metadata and
the file still present resumes deletion; a retired file is never used. A missing active or
verification-only file, or an unreadable, wrong-store, wrong-key-ID, wrong-algorithm, malformed, or
mismatched envelope, is store corruption and fails closed. Retired registry entries are never
deleted or reused.

The required domains are exactly:

```text
substrate.a1.transition-input.v1
substrate.a1.participant-lease-token.v1
substrate.a1.raw-transport-payload.v1
```

For these three intent-bound domains the HMAC input is this byte sequence, where `len64` is an
unsigned 64-bit big-endian byte length, all strings are their exact UTF-8 bytes, and `raw` is not
JSON encoded:

```text
"substrate.a1.hmac-input.v1\0"
 || len64(domain) || domain
 || len64(authority_store_id) || authority_store_id
 || len64(intent_id) || intent_id
 || run_present
 || if run_present == 0x01 { len64(run_id) || run_id }
 || len64(raw) || raw
```

Transition input/prompt bytes, participant lease-token bytes, and raw transport-payload bytes use
their respective domain. All three V1 object kinds are run-bound: `run_present` is exactly `0x01`
and `run_id` is the non-empty exact run ID committed by the parent intent. `0x00`, an absent/empty
run ID, or a different run ID is invalid for every V1 domain and fails before object acceptance.
Delimiter concatenation is forbidden. The `StoreHmacSha256.digest_hex` is lowercase-hex
`HMAC-SHA-256(key, bytes_above)`. Sensitive bytes and these reusable commitments are authority
store data only; failed verification reports a redacted typed error without logging either.

#### `StateRootV1` and the session namespace

```rust
struct StateRootV1 {
    schema_version: u32,
    authority_store_id: String,
    bootstrap_home: CanonicalDirectoryV1,
    root_revision: u64,
    active_commitment_key_id: String,
    commitment_key_registry: BTreeMap<String, AuthorityStoreCommitmentKeyV1>,
    greenfield_namespace_certificate: GreenfieldNamespaceCertificateV1,
    session_namespace_map: BTreeMap<String, SessionNamespaceRecordV1>,
    transition_intent_map: BTreeMap<String, HostSessionTransitionIntentV1>,
    issuer_request_index: BTreeMap<String, IssuerRequestIndexEntryV1>,
    application_journal: BTreeMap<String, HostSessionTransitionApplicationJournalV1>,
    object_index: BTreeMap<String, AuthorityObjectIndexEntryV1>,
}

struct GreenfieldNamespaceCertificateV1 {
    schema_version: u32,
    authority_store_id: String,
    bootstrap_home: CanonicalDirectoryV1,
    certified_at: TimestampV1,
}

enum SessionNamespaceRecordV1 {
    Authority(DurableSessionAuthorityV1),
    StartReservation(SessionIdReservationV1),
    StartTombstone(SessionIdTombstoneV1),
}

struct SessionIdReservationV1 {
    schema_version: u32,
    orchestration_session_id: String,
    intent_id: String,
    issuer_request_id: String,
    payload_commitment: AuthorityObjectCommitmentV1,
    reserved_at: TimestampV1,
}

enum StartTombstoneStateV1 {
    Rejected { reason: HostSessionTransitionTerminalRejectionV1 },
    Expired,
}

struct SessionIdTombstoneV1 {
    schema_version: u32,
    orchestration_session_id: String,
    intent_id: String,
    issuer_request_id: String,
    payload_commitment: AuthorityObjectCommitmentV1,
    terminal_state: StartTombstoneStateV1,
    terminal_handoff_ref: AuthorityObjectRefV1,
    tombstoned_at: TimestampV1,
}

struct IssuerRequestIndexEntryV1 {
    schema_version: u32,
    issuer_request_id: String,
    orchestration_session_id: String,
    intent_id: String,
    payload_commitment: AuthorityObjectCommitmentV1,
}

struct InitialTransitionApplicationJournalV1 {
    authority_revision_before: Option<u64>,
    authority_revision_after: u64,
    authority_record_commitment: AuthorityObjectCommitmentV1,
    application_result_ref: AuthorityObjectRefV1,
    applied_at: TimestampV1,
}

struct PostTurnApplicationJournalV1 {
    completion_ref: AuthorityObjectRefV1,
    obligation_snapshot_ref: Option<AuthorityObjectRefV1>,
    authority_revision_before: u64,
    authority_revision_after: u64,
    authority_record_commitment: AuthorityObjectCommitmentV1,
    application_result_ref: AuthorityObjectRefV1,
    applied_at: TimestampV1,
}

struct HostSessionTransitionApplicationJournalV1 {
    schema_version: u32,
    intent_id: String,
    initial_application: InitialTransitionApplicationJournalV1,
    post_turn_application: Option<PostTurnApplicationJournalV1>,
}

enum AuthorityObjectStorageStateV1 {
    Present,
    ReleaseEligible { terminal_handoff_ref: AuthorityObjectRefV1 },
    Released {
        terminal_handoff_ref: AuthorityObjectRefV1,
        released_at: TimestampV1,
    },
}

struct AuthorityObjectIndexEntryV1 {
    schema_version: u32,
    ref_id: String,
    object_kind: AuthorityObjectKindV1,
    object_schema_version: u32,
    byte_length: u64,
    storage_state: AuthorityObjectStorageStateV1,
}
```

Map ownership and keys are exact: `session_namespace_map` is keyed by
`orchestration_session_id`; `transition_intent_map` and `application_journal` by `intent_id`;
`issuer_request_index` by store-global `issuer_request_id`; `commitment_key_registry` by `key_id`;
and `object_index` by `ref_id`. Each value repeats and must match its map key. Every non-released
object-index entry is reachable from an exact parent record in the same root; an index row alone
cannot grant semantic authority. Only a
`TransitionTransportPayload` entry may be `ReleaseEligible` or `Released`; every other committed
object kind remains immutable `Present`. A session namespace key is never deleted in V1 after
reservation, authority creation, or tombstoning; lifecycle changes replace only the value variants
explicitly authorized above.

The greenfield certificate is not a migration barrier. Its `schema_version` is exactly 1, its store
ID and bootstrap-home identity exactly equal the enclosing root, and its timestamp is the exact
timestamp fixed by `AuthorityStoreInitializationV1`. It can be created only by the `FreshAbsent`
initialization protocol below and is immutable for the life of the store. A missing, changed,
copied, synthesized, or independently created certificate is root corruption.

#### External-object write protocol and filesystem safety

The fixed layout below is entirely beneath the physically bound `authority_store_root`:

```text
authority-v1/state-root-v1.json
authority-v1/init-v1.json
authority-v1/lock/root.lock
authority-v1/tmp/
authority-v1/objects/<closed-kind>/v<schema_version>/<ref_id>.obj
authority-v1/keys/<key_id>.key
```

Every temp filename has one closed operation-bound grammar, where `nonce` is 32 lowercase
hexadecimal characters encoding 128 OS-CSPRNG bits:

```text
init--<authority_store_id>--<nonce>.tmp
key--<key_id>--<nonce>.tmp
object--<ref_id>--<nonce>.tmp
root--r<minimal-decimal-new-root_revision>--<nonce>.tmp
```

Temps exist only in `authority-v1/tmp/`, are regular owner-only `0600` files opened no-follow with
exclusive create, and never carry semantic authority. Before bootstrap classification and before
any transaction/reconciliation, the root lock holder enumerates `tmp/` and applies this exact rule:

1. A recognized safe temp is never promoted or treated as proof, whether empty, partial, complete,
   or `fsync`ed. Delete it directory-relatively and `fsync(tmp/)`.
2. If a valid committed root requires a key/object whose final file is absent, the store is corrupt;
   a matching temp cannot repair it. If a valid final file exists, its temp is still deleted.
3. With a valid pending init marker, deletion is followed by normal exact marker/key/root recovery.
   With no root or marker, deletion completes before evaluating `FreshAbsent`.
4. An unrecognized name, invalid embedded ID/revision/nonce, symlink, non-regular entry, wrong owner
   or mode, directory-enumeration error, or deletion/`fsync` failure is `CorruptOrUnsupported`.

Thus a crash before temp `fsync`, after temp `fsync`, or before/after rename has one result: an
unrenamed temp is safely removed; a completed rename is evaluated only at its final typed location;
and neither state can fabricate root, key, object, marker, or application success. Key retirement
uses final-file deletion and no temp.

Initialization creates/opens `authority-v1`, `lock`, `tmp`, `objects`, and `keys` relative to the
trusted root handle with no-follow/exclusive-create checks, then serializes competing initializers
on the exact `authority-v1/lock/root.lock` file. The lock file contains no PID authority and is
never broken by liveness heuristics. Linux and macOS use a whole-file exclusive `flock(LOCK_EX)`
held on the no-follow-opened lock-file descriptor for the entire transaction; a filesystem whose
`flock` is not cross-process and crash-safe is unsupported. Pre-lock directory creation is
idempotent only when post-open ownership, type, and mode validation succeeds; no root/key/object
publication occurs before the lock is held. The initializer `fsync`s every newly created directory,
the lock file, and their affected parent directories before classifying or publishing anything.

##### `LegacyStateStoreTransactionV1` retained-root contract

Every pre-A1 StateStore transaction that reads or mutates either guarded session/participant
authority collection is admitted through one bounded `LegacyStateStoreTransactionV1`-equivalent
capability. Admission retains all of the following as authority-bearing resources:

- the opened `TrustedAuthorityRoot` handle selected from the normalized bootstrap home;
- its exact `CanonicalDirectoryV1` physical path and platform physical identity;
- no-follow-opened collection and descendant directory handles, or the deepest safe opened parent
  plus exact missing suffix for a permitted create;
- exclusive ownership of the cross-process `authority-v1/lock/root.lock` guard; and
- the activation/classification observation made under that same root and lock.

The capability exposes only directory-relative/no-follow operations rooted in those retained
handles: bounded reads, exclusive temp creation and writes, file `fsync`, no-replace publication or
atomic rename, removals, and affected-directory `fsync`. It does not expose descendant `PathBuf`s,
absolute-path reconstruction, ambient-CWD lookup, or any API that rereads `SUBSTRATE_HOME`, `$HOME`,
or another lexical bootstrap path after admission. Every read used to decide a mutation, every
flat/canonical snapshot or lease write, every removal/rename, and every compatibility/read-repair or
parent-session persistence step that mutates a guarded collection remains inside that one retained
transaction.

Transaction completion occurs only after the final changed file and every affected directory have
been `fsync`ed while the root lock is still owned. The lock may be released only by consuming the
completed transaction or by fail-closed unwind; no successful outcome is returned after early lock
release. Rebind, rename, replacement, physical-identity mismatch or uncertainty, unsafe ACL or
ownership, unsupported no-follow/locking/atomic-publication/`fsync` semantics, or use of a handle
from another physical root fails closed. A lock retained on the former root never authorizes a
write to the lexical replacement: the replacement root must receive zero reads used for mutation,
writes, temps, renames, removals, or publication. The originally opened root may be reconciled only
through its retained handles and exact contract; otherwise the transaction returns failure without
fabricating success.

No production authority root or initialization marker may become reachable while any applicable
in-repository legacy writer can bypass this capability. A1 activation remains unavailable until
complete guarded-writer adoption and its cross-process proof pass.

A1 has no legacy-authority migration mode. Under the same trusted bootstrap-home handle and root
lock, classification safely enumerates both pre-A1 authority collections:
`run/agent-hub/sessions/` recursively and `run/agent-hub/participants/` non-recursively. A missing
collection is empty only on exact `ENOENT`; a present collection must be an owner-controlled,
no-follow-opened directory whose complete enumeration succeeds. An empty validated directory is
allowed. Any regular record, nested record/directory, symlink, hard link, device, socket, unknown
entry, or other artifact is pre-A1 authority state. Config, policy, and inventory paths outside
these two collections are not legacy authority artifacts and remain allowed.

Traversal is component-by-component and directory-relative from the already-open trusted
bootstrap-home descriptor; concatenated absolute paths and path re-resolution are forbidden. The
bootstrap home and every existing `run`, `agent-hub`, collection, and recursively visited sessions
component must be a no-follow-opened directory owned by the effective owner, with no group/world
write bit and no ACL granting another principal write, rename, delete, or traversal authority.
Each opened component's physical identity is captured using the platform-specific fields of
`DirectoryPhysicalIdentityV1`. A symlink/reparse point, non-directory ancestor, owner mismatch,
unsafe mode/ACL, cross-device rebinding, or identity-query failure is `CorruptOrUnsupported`.

The classifier retains the opened component descriptors and their physical identities for the
whole locked initialization attempt. If a suffix is absent, it retains the deepest existing safe
parent descriptor and the exact missing components. Immediately before root-temp creation and
again before root rename, it reopens each child directory-relative, verifies every existing
identity/owner/mode/ACL against the retained observation, verifies every missing suffix is still
exact `ENOENT`, and repeats the complete empty-collection enumeration. Any replacement, rename,
new entry, identity change, or validation uncertainty aborts without certificate/root publication.
Every later semantic transaction performs the same safe traversal and empty enumeration. A
same-owner unmodified or mixed-version process is outside the supported concurrency model and must
be quiesced; other filesystem principals are excluded by the required ownership/mode/ACL checks.

Classification precedence is exact: any authority-layout/root/marker/key/temp validation failure or
collection-enumeration uncertainty is `CorruptOrUnsupported`; otherwise any safely observed pre-A1
artifact is `UnsupportedLegacyState`; otherwise exactly one of `FreshAbsent`,
`InitializationPending`, or `ValidExisting` applies. Under that lock, bootstrap has exactly five
classifications:

1. `FreshAbsent`: after successful temp reconciliation and complete legacy-collection enumeration,
   `state-root-v1.json` and `init-v1.json` are both proven absent; `keys/`, `objects/`, `tmp/`, and
   both legacy authority collections contain no entries; and only the validated empty authority
   directory skeleton and `root.lock` exist. Only this state may begin initialization and mint the
   greenfield certificate.
2. `InitializationPending`: after successful temp reconciliation and complete empty
   legacy-collection enumeration, `state-root-v1.json` is proven absent and `init-v1.json` is a
   complete, strictly decoded `CanonicalJsonV1(AuthorityStoreInitializationV1)`. No objects or
   unrelated keys/temps exist. The one matching initial key file may be absent or complete.
   Recovery resumes only when the marker's persisted `bootstrap_home` equals the trusted handle's
   physical identity, then reuses the exact stored store ID, key ID, home, and timestamp; it never
   generates replacement identities.
3. `ValidExisting`: `state-root-v1.json` is a complete strictly decoded root whose store ID, bound
   `bootstrap_home`, greenfield certificate, key registry/files, maps, refs, permissions, and
   filesystem semantics all validate against the trusted handle and both legacy authority
   collections still enumerate empty. The object tree contains only closed kind slugs, minimal
   `v1` directories, registered immutable objects, safely empty known kind/version directories, and
   safely named unindexed orphan object files. An orphan must be a no-follow regular owner-only
   `0600` file at the exact typed path for a syntactically valid ref ID; it has no authority or
   reachability meaning. Unknown, unsafe, or malformed directory/object entries are corruption. A
   matching leftover init marker is removed only after its store ID, key ID, home, and timestamp
   match the root, certificate, and registry.
4. `UnsupportedLegacyState`: a complete safe enumeration finds any pre-A1 authority artifact.
   A1 does not parse, migrate, quarantine, rank, merge, delete, or select a winner from it; it
   creates no root/certificate and accepts no transition. The only recovery is an explicit
   operator/developer reset outside the A1 runtime contract.
5. `CorruptOrUnsupported`: every other state, including an unreadable or incompletely enumerable
   legacy collection, a present invalid/unreadable root, a root
   symlink or non-regular file, unsafe ownership/permissions, unsupported locking or fsync,
   root-without-required-key, key/object/temp artifacts without a valid root or init marker,
   mismatched init/key/root identities, or any malformed/partial file. This fails closed and can
   never trigger fresh-store creation or namespace replacement.

Missing/unreadable is never collapsed to absent: absence is accepted only from the no-follow,
directory-relative lookup result that distinguishes `ENOENT` from every other error. In
particular, decode failure, access denial, I/O failure, wrong type, symlink refusal, and unsupported
filesystem behavior are `CorruptOrUnsupported`.

A `FreshAbsent` initializer fixes the trusted handle's `CanonicalDirectoryV1` as `bootstrap_home`,
generates one immutable `authority_store_id` as `as_` plus 32 lowercase
hexadecimal characters encoding 128 OS-CSPRNG bits, one key ID, one 256-bit key, and one timestamp,
then performs this exact protocol:

1. Write `CanonicalJsonV1(AuthorityStoreInitializationV1)` to
   `tmp/init--<authority_store_id>--<nonce>.tmp`, `fsync` it, rename it
   with no-replace semantics to `init-v1.json`, and `fsync(authority-v1/)`.
2. Publish and `fsync` the matching immutable key envelope and `keys/` as specified above.
3. Re-enumerate both legacy authority collections under the still-held lock, then write the
   revision-1 root with that store ID, the identical persisted bootstrap home, a
   `GreenfieldNamespaceCertificateV1` carrying the same store ID/home/timestamp, one `Active` key
   registry entry, and empty maps to `tmp/root--r1--<nonce>.tmp`; `fsync` it, publish with
   no-replace semantics, and `fsync(authority-v1/)`. Any artifact or enumeration uncertainty aborts
   without publishing the root.
4. Delete `init-v1.json`, `fsync(authority-v1/)`, and only then report initialization success.

`InitializationPending` with no key repeats step 2 using a newly generated 256-bit secret under the
already-fixed key ID; with a complete key it validates that envelope only against the init marker
and fixed algorithm rule above, then reuses that exact file. Whether newly published or reused,
recovery reopens and revalidates the key file, `fsync`s the key file, and `fsync`s `keys/` before
performing steps 3–4. A crash after root publication is a committed initialization even if the
marker remains; a crash before root publication is never inferred from key presence alone. A
competitor always re-reads under the lock and joins the same pending or committed initialization.

Every object-creating or semantic root transaction uses this order:

1. Open `authority_store_root` through its trusted directory handle and revalidate its committed
   physical identity.
2. Reject unsafe/symlinked authority paths, wrong ownership, unsafe permissions, or an unsupported
   filesystem.
3. Acquire the cross-process exclusive lock on `authority-v1/lock/root.lock`.
4. Re-read and strictly decode the current `StateRootV1`.
5. Re-enumerate both legacy authority collections as empty and validate the greenfield certificate,
   `root_revision`, object state, namespace reservation, intent/application revisions, and all
   semantic preconditions. A pre-A1 artifact fails closed and cannot be imported or ignored.
6. Traverse `objects/<closed-kind-slug>/v<minimal-decimal-schema-version>` component-by-component
   from the no-follow-opened `objects/` descriptor. Existing components must be owner-only `0700`
   directories with the expected names. A missing closed kind or version directory is created
   under the lock with `mkdirat`-equivalent no-follow semantics and mode `0700`; each created
   directory and each affected parent is `fsync`ed before continuing. Unknown kind/version names,
   unsafe existing entries, or create/validation/`fsync` uncertainty fail closed. A crash may leave
   only safely empty known kind/version directories; `ValidExisting` accepts those as routing
   structure without granting object authority.
7. Write each new immutable object's bytes to
   `authority-v1/tmp/object--<ref_id>--<nonce>.tmp` on the same filesystem.
8. `fsync` each object temp file.
9. Atomically rename it with no-replace semantics into its final typed object location; an existing
   location is an exact-retry candidate only after complete verification.
10. `fsync` every affected object directory.
11. Re-read or otherwise revalidate the still-locked root revision before publication.
12. Write the new canonical root to
    `authority-v1/tmp/root--r<new-root-revision>--<nonce>.tmp`.
13. `fsync` the root temp file.
14. Atomically replace `authority-v1/state-root-v1.json`.
15. `fsync` the `authority-v1` parent directory.
16. Release the lock only after the transaction has a verified outcome.

Exact retry occupancy is semantic, not variant-name-based. A root is not authority-free merely
because `session_namespace_map` contains no `Authority` variant. Any `StartReservation`,
`StartTombstone`, transition-intent entry, issuer-request-index entry, application-journal entry, or
object-index entry is semantic authority state and participates in exact retry and conflict
validation. A retry may join only when every applicable store/session/request/intent identity,
payload commitment, root and authority revision, namespace record, object/index state, journal,
and immutable application field matches the already committed result exactly. Empty or partial
matching of one map can never authorize a join.

Root publication uses a post-reconciliation publication candidate. After temp, orphan, released
object, and key-file reconciliation, and immediately before publishing a replacement root under the
same retained trusted root and lock, rotation, retirement, CAS, and any other root publisher must
validate the complete candidate against:

- the current locked root and exact expected `root_revision` plus any expected authority revision;
- exact bootstrap-home and authority-store identity;
- the complete key registry and key files, including exactly one active key and all
  active/verification/retired constraints;
- every reachable typed object reference, object-index entry, and required object file/state;
- all namespace, reservation, tombstone, transition-intent, issuer-index, application-journal, and
  object-index invariants;
- complete safe enumeration proving both pre-A1 authority collections are empty; and
- the unchanged trusted physical-root identity and retained descendant identities.

Rotation and retirement may not rely solely on validation performed before reconciliation. A
stale, conflicting, missing, substituted, malformed, or post-reconciliation-invalid candidate
fails before root-temp creation/publication and without unauthorized cleanup or semantic mutation.

On Unix every file in this layout, including root, object, temp, key, and lock files, is `0600`;
every directory at or below `authority-v1`, including kind/version object directories, is `0700`.
The effective owner must own them, and group/world-writable components or ACLs granting another
principal access are rejected. Access is directory-relative/openat-style with `O_NOFOLLOW`
or the platform equivalent where available. A process-local mutex is insufficient. Kernel release
of the lock after process death is only an observation that permits the next process to acquire,
re-read, and reconcile; it does not prove, roll back, or complete a transaction.
Filesystems/platforms lacking trustworthy cross-process locks, no-follow access, same-filesystem
atomic no-replace/object publication, atomic root replacement, file and directory `fsync`, or
equivalent owner-only semantics fail closed. Windows remains fail-closed under the path rule above
until equivalent ACL, lock, and atomic-replace semantics are specified and proven.

Crash meaning is exact:

- Object bytes with no committed root/object-index parent are orphans. They have no authority
  meaning. A1's safe grace rule is retention: A1 authorizes no orphan deletion. A later
  schema/version may add garbage collection only with locked reconciliation plus an explicit
  time/evidence grace rule, so A1.1 does not invent one.
- A retained orphan may be adopted only during an exact retry that presents the same complete
  `AuthorityObjectRefV1`—ref ID, kind, schema, commitment variant/domain/key/digest—the exact parent
  intent/run/store context required by any HMAC wrapper, and exact final file bytes that verify the
  resulting commitment input. The locked retry then supplies the missing semantic root parent and
  object-index entry in its normal root transaction. Without that complete exact retry, the orphan
  remains retained and non-authoritative; its filename or bytes alone never reconstruct an intent,
  request, authority, or parent.
- `ObligationSnapshot` is stricter than the generic orphan rule. After a crash between snapshot
  publication and root commit, it may be adopted only if `ObligationLedger` reissues/revalidates
  the same Complete snapshot bytes under the still-current per-session ledger revision and
  materialization cut and every store/session/participant/intent/run/authority binding still
  matches. Otherwise the orphan remains non-authoritative and a new current snapshot/ref is
  published; an old empty or non-empty cut is never adopted merely because its bytes verify.
- A committed parent ref in a required-present state whose object is missing, wrong-kind,
  wrong-version, wrong-domain/key, or commitment-mismatched is store corruption. Fail closed and
  never infer success.
- `ReleaseEligible` with the transport object present may retry deletion idempotently. Deletion is
  directory-relative and the object directory is fsynced before the root records `Released`.
- `ReleaseEligible` with the transport object absent may become `Released` only after verifying the
  exact committed terminal-handoff ref and commitment in both intent and object index. Verified
  `ENOENT` is followed by `fsync` of that exact transport object directory before the root may
  record `Released`, matching the durability barrier used after successful deletion.
- `Released` is the sole intentional missing-object state. It requires the same exact terminal
  handoff and no remaining transport bytes. An unexpected surviving copy is removed
  directory-relative under the lock and the exact transport object directory is `fsync`ed before
  retry may join/report success; removal or durability uncertainty fails closed. The bytes never
  regain authority meaning.
- `Retained`/`Present` with a required object absent is store corruption and fails closed.
- If root replacement completed but the parent-directory `fsync` failed, durability is unproven:
  the operation must not report durable success. Recovery acquires the lock, accepts only a
  complete strictly decoded root and matching objects, and reconciles idempotently from the last
  verifiable root/application state. It never treats an orphan object as proof that the new root
  committed.

Production A1 timing is fixed:

```text
HOST_SESSION_TRANSITION_INTENT_DEFAULT_TTL = 300 seconds
HOST_SESSION_TRANSITION_INTENT_MAX_TTL     = 900 seconds
HOST_SESSION_TRANSITION_CLAIM_LEASE        = 30 seconds
HOST_SESSION_TRANSITION_CLAIM_LEASE_MAX    = 60 seconds
```

CLI, REPL, helper, and auto-attach producers use the default intent TTL and claim lease. Tests may
use a shorter injected clock/bound only; production environment variables or plan contents cannot
change these values in V1. Issuance rejects a non-positive TTL, a TTL above the maximum, or a claim
lease outside the positive maximum. Neither retry nor reclaim extends `expires_at`.

## 1. `DurableSessionAuthorityV1`

```rust
enum DurableSessionAuthorityOriginV1 {
    StartIntent {
        intent_id: String,
        issuer_request_id: String,
        payload_commitment: AuthorityObjectCommitmentV1,
    },
}

struct DurableSessionAuthorityV1 {
    schema_version: u32,                 // exactly 1
    orchestration_session_id: String,
    shell_trace_session_id: String,
    authority_revision: u64,
    origin: DurableSessionAuthorityOriginV1,
    authoritative_participant_lineage: Vec<String>,
    active_authoritative_participant_id: Option<String>,
    workspace_binding: WorkspaceBindingV1,
    world_binding: Option<WorldBindingV1>,
    host_attach_contract_ref: Option<AuthorityObjectRefV1>,
    retained_worker_refs: Vec<AuthorityObjectRefV1>,
    internal_resume_handle_refs: Vec<AuthorityObjectRefV1>,
    lifecycle_posture: HostSessionPostureV1,
    current_policy_ref: Option<AuthorityObjectRefV1>,
    current_policy_revision: Option<String>,
    updated_at: TimestampV1,
}

enum HostSessionPostureV1 {
    ActiveAttached,
    ParkedResumable,
    DetachedReconciled,
    AwaitingAttention,
    Terminal,
    StaleRecoverable,
    Invalid,
}
```

Acceptance rules:

1. Only `HostSessionAuthority` may create a newer `authority_revision` or change posture, lineage,
   binding, attach contract, refs, or authoritative participant.
2. Every authority is Start-created and retains the exact root-map `intent_id`, issuer request, and
   payload commitment that owned its former reservation. `StartIntent` is the only V1 origin and is
   immutable.
3. Every ref validates its closed kind: attach contract is `HostAttachContract`, retained workers
   are `RetainedWorker`, resume handles are `ResumeHandle`, and policy is `Policy`. Kind, schema,
   key/domain where applicable, and parent-owned commitment are checked before use.
4. `AwaitingAttention` is derived from unresolved attention-driving obligations; it is not set
   from a worker flag or inbox row alone.
5. `Terminal` is monotonic unless a separately versioned recovery protocol explicitly creates a
   successor session; it is never reversed in-place.
6. `HostSessionPostureV1::Invalid` is permitted only for an otherwise fully representable
   `DurableSessionAuthorityV1` that later becomes invalid or unroutable. It cannot supply missing
   identity, lineage, binding, contract, or ref fields.
7. World binding is the exact complete `(world_id, world_generation)` pair. Partial binding is
   unrepresentable, not an `Invalid` authority.
8. PID, socket, heartbeat, attached-client, helper, and plan data do not belong in this contract.
9. `shell_trace_session_id`, `workspace_binding.authority_store_root`,
   `workspace_binding.authority_store_id`, and the initial `workspace_binding.workspace_root` are
   immutable for one authority record. Attach and `ResumeOneTurn` match their complete physical
   identities; a different store or workspace requires a separately versioned explicit rebinding
   protocol outside A1.

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
    target_participant_lease_token_ref: AuthorityObjectRefV1,
    run_id: String,
    resulting_authoritative_lineage: Vec<String>,
    workspace_binding: WorkspaceBindingV1,
    world_binding: Option<WorldBindingV1>,
    descriptor_ref: AuthorityObjectRefV1,
    host_attach_contract_ref: AuthorityObjectRefV1,
    resume_handle_ref: Option<AuthorityObjectRefV1>,
    transition_input_ref: Option<AuthorityObjectRefV1>,
    post_turn_disposition: Option<HostPostTurnDispositionV1>,
    transport_payload_ref: AuthorityObjectRefV1,
    payload_commitment: AuthorityObjectCommitmentV1,
    issued_at: TimestampV1,
    expires_at: TimestampV1,
    state: HostSessionTransitionIntentStateV1,
    input_handoff: HostSessionTransitionInputHandoffV1,
    transport_payload_state: HostSessionTransitionTransportPayloadStateV1,
    updated_at: TimestampV1,
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
        authority_record_commitment: AuthorityObjectCommitmentV1,
        active_authoritative_participant_id: String,
        authoritative_lineage_commitment: AuthorityObjectCommitmentV1,
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
        claimed_at: TimestampV1,
        claim_expires_at: TimestampV1,
    },
    Applied {
        claim_id: String,
        claimant_attempt_id: String,
        authority_revision_before: Option<u64>,
        authority_revision_after: u64,
        active_authoritative_participant_id: String,
        resulting_posture: HostSessionPostureV1,
        authority_record_commitment: AuthorityObjectCommitmentV1,
        application_result_ref: AuthorityObjectRefV1,
        startup_ownership: HostSessionStartupOwnershipApplicationV1,
        post_turn: HostSessionPostTurnApplicationV1,
        applied_at: TimestampV1,
    },
    Rejected {
        reason: HostSessionTransitionTerminalRejectionV1,
        terminal_handoff_ref: AuthorityObjectRefV1,
        rejected_at: TimestampV1,
    },
    Expired {
        terminal_handoff_ref: AuthorityObjectRefV1,
        expired_at: TimestampV1,
    },
}

enum HostSessionPostTurnApplicationV1 {
    NotApplicable,
    Pending {
        expected_run_id: String,
        expected_authority_revision: u64,
    },
    AwaitingObligationCut {
        completion_ref: AuthorityObjectRefV1,
        expected_run_id: String,
        expected_authority_revision: u64,
        acceptance_record_id: String,
        acceptance_record_revision: u64,
        stream_id: String,
        accepted_work_identity: AcceptedWorldWorkIdentityV1,
        host_transition_correlation: HostTransitionWorkCorrelationV1,
        required_terminal_event_id: String,
        required_terminal_event_sequence: u64,
        recorded_at: TimestampV1,
    },
    Applied {
        completion_ref: AuthorityObjectRefV1,
        obligation_snapshot_ref: Option<AuthorityObjectRefV1>,
        authority_revision_before: u64,
        authority_revision_after: u64,
        resulting_posture: HostSessionPostureV1,
        application_result_ref: AuthorityObjectRefV1,
        applied_at: TimestampV1,
    },
}

enum HostSessionStartupOwnershipApplicationV1 {
    NotApplicable,
    Pending {
        expected_run_id: String,
        expected_authority_revision: u64,
        expected_active_authoritative_participant_id: String,
    },
    Accepted {
        evidence_id: String,
        result_ref: AuthorityObjectRefV1,
        authority_revision: u64,
        accepted_at: TimestampV1,
    },
    TerminalReconciled {
        evidence_id: String,
        result_ref: AuthorityObjectRefV1,
        authority_revision_before: u64,
        authority_revision_after: u64,
        resulting_posture: HostSessionPostureV1,
        reconciled_at: TimestampV1,
    },
}

enum HostSessionTransitionInputHandoffV1 {
    NotApplicable,
    Pending {
        input_ref: AuthorityObjectRefV1,
        run_id: String,
    },
    Accepted {
        input_ref: AuthorityObjectRefV1,
        run_id: String,
        acceptance_ref: AuthorityObjectRefV1,
        accepted_at: TimestampV1,
    },
    TerminalWithoutAcceptance {
        input_ref: AuthorityObjectRefV1,
        run_id: String,
        terminal_handoff_ref: AuthorityObjectRefV1,
        terminal_at: TimestampV1,
    },
}

enum HostSessionTransitionTerminalRejectionV1 {
    InvalidCommittedModePrecondition,
    AuthorityPreconditionNoLongerHolds,
    StaleAuthorityRevision,
    AuthorityRecordCommitmentMismatch,
}

enum HostSessionTransitionAttemptRejectionV1 {
    UnknownIntent,
    IntentOrRequestIdentityMismatch,
    IntentRevisionMismatch,
    ClaimRevisionMismatch,
    PayloadCommitmentMismatch,
    ObjectRefOrCommitmentMismatch,
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
        terminal_handoff_ref: AuthorityObjectRefV1,
    },
    Released {
        terminal_handoff_ref: AuthorityObjectRefV1,
        released_at: TimestampV1,
    },
}
```

Attempt rejection and terminal intent rejection are separate outcomes:

| Validation outcome | Durable effect |
|---|---|
| Invalid request before a valid intent is issued | Reject the request; persist no intent, reservation, tombstone, object-index entry, or issuer-index entry. An unrenamed temp is removed by temp reconciliation; a final object published before the failed root commit is only an orphan. |
| Exact `intent_id`, `issuer_request_id`, and `payload_commitment` presented for an existing intent | Return/join its current claim or stored transition/input/post-turn/terminal result as allowed by state, even if the caller retained an older intent revision; do not reissue or reapply. |
| Unknown/substituted/mismatched attempt, stale intent/claim revision used for a claim/application/state mutation, superseded claim, or conflicting replay against a valid stored intent | Return and audit `HostSessionTransitionAttemptRejectionV1`; do not mutate, reject, expire, release, or otherwise strand the stored intent. |
| The stored intent was validly issued, the root/object/key set remains valid, but its committed authority precondition no longer holds before application | Commit `Rejected` plus one exact `TerminalHandoff` ref. For Start, atomically replace its exact reservation with an intent-linked permanent tombstone; no authority mutation is permitted. |
| The fixed intent expiry passes before application and no authority commit exists | Commit `Expired` plus one exact `TerminalHandoff` ref. For Start, atomically replace its exact reservation with an intent-linked permanent tombstone. |
| The exact stored intent and attempt validate | Continue the `Issued`/`Claimed`/`Applied` protocol. |

A substituted request using a stored `intent_id` or `issuer_request_id` with a different
`payload_commitment` is therefore attempt-rejected and audit-recorded without mutating or
terminalizing the valid stored intent. A missing/mismatched committed object or required HMAC key
is store corruption, not a terminal intent rejection; it fails closed without mutating the intent,
reservation, namespace, or release state.

### Immutable commitments and identity

1. `intent_id` identifies exactly one logical transition. `issuer_request_id` is store-global and
   indexes that intent. Exact retry requires the same session ID, intent ID, issuer request, and
   payload commitment; reuse with different content or identity fails closed.
2. `payload_commitment` must be `CanonicalSha256` over exactly
   `HostSessionTransitionPayloadHashInputV1`, including fixed issuance/expiry timestamps and every
   complete typed ref. It excludes only `intent_revision`, mutable state/claim/handoff/retention
   fields, `updated_at`, and terminal results. No presentation struct or adjacent metadata is
   hashed. The issuer fixes intent ID and both timestamps once before the first root commit. Retry
   first resolves the issuer index and reconstructs this wrapper with those stored timestamps;
   it never samples a new clock value to decide equality.
3. Descriptor, attach contract, resume handle, participant lease token, transition input, and
   transport payload are immutable objects. Their parent-held refs are verified before issuance
   commit where newly published, and again before claim, application, reprojection, or use. Plan
   JSON may not replace a ref or commitment.
4. The lease-token ref is kind `LeaseToken` with the participant-lease HMAC domain. The optional
   input ref is kind `TransitionInput` with the transition-input HMAC domain. The transport ref is
   kind `TransitionTransportPayload` with the raw-transport HMAC domain. Descriptor and attach
   contract are canonical structured refs; resume handle uses its closed kind-specific schema.
   Credentials are prohibited, and prompt/input/lease/transport bytes and reusable commitments
   never enter logs, traces, diagnostics, rejection text, compatibility snapshots, or object IDs.
5. `expires_at` is fixed at issuance, later than `issued_at`, and bounded by the V1 maximum.
   Ambient retries, plan rewrites, helper restarts, and reclaim do not extend it.
6. `intent_revision` increments on every durable intent-state, claim, input-handoff,
   startup-ownership, post-turn (including `AwaitingObligationCut`), or transport-payload-state
   change. A stale intent or claim revision cannot mutate the record; this does not block an exact
   read-only join.
7. The namespace and authority precondition permit at most one transition candidate for the exact
   reserved state. A different mode, target, issuer, intent, or commitment is a conflict, never a
   second candidate.
8. `PublicCli` and `Repl` callers have no auto-attach fields. `RouterAutoAttach` is valid only for
   `Attach` and commits the already-existing obligation ID and claim owner; A1 neither decides
   eligibility nor changes claim/settlement semantics. A present caller participant matches the
   exact authoritative caller. A new Start has none.
9. Pre-A1 `source_orchestration_session_id` plan data is rejected. Startup-prompt stream paths,
   helper PIDs, sockets, and plan paths are transport-only and cannot enter reservation or
   authority truth. Existing builders may reproject them only from verified retained objects.
10. `shell_trace_session_id` exactly matches the authority record for Attach/Resume and is committed
    by the Start payload and resulting authority. It cannot be recovered from ambient tracing
    state.

### A1 packet ownership boundary

A1.1 owns only the authority-store substrate: bootstrap classification and recovery, the normalized
bootstrap-home binding, cross-process locking, exact object publication/verification, key lifecycle,
greenfield namespace certification plus fail-closed pre-A1-state detection, exact authority
resolution, and generic root/authority revision-CAS. A1.1 does not accept production
`ExpectedAbsent`, create a `StartReservation`, issue or apply a transition intent, allocate a Start
participant, or create a Start-origin authority.

To make the same-home invariant implementable, A1.1 may add explicit-bootstrap-home parameters or
sibling entry points only to the existing common path, effective config, effective policy/policy
snapshot, and agent-inventory resolvers named by the A1 packet. Those entry points consume the
already-open `CanonicalDirectoryV1`/trusted handle and preserve all existing workspace/global
precedence, merge, parsing, and policy interpretation. A1.1 does not switch real callers. A1.3 may
thread that same handle only through its named CLI/REPL Start/Attach/Resume adoption paths; it may
not broaden resolver semantics or convert unrelated callers.

A1.2 is the first packet allowed to perform production Start semantics. It owns greenfield
certificate validation, `ExpectedAbsent` acceptance, Start reservation plus intent issuance,
claim/application, initial Start-origin authority birth, startup-ownership resolution, pending
post-turn reconciliation, ledger-snapshot consumption, and release as one intent protocol. It does
not own runtime event identity, receipt acceptance, durable observation, retained-event semantics,
canonical obligations, or their event/materialization cut. B0, B1, B2.1, B3.1, and C1 own those
prerequisites respectively. Until that corridor exists, A1.2 retains `AwaitingObligationCut` and
cannot claim full packet closure. A1.3 adopts the protocol on real CLI/REPL consumers; A1.1
primitive tests are not evidence that a production Start path is adopted.

### Start namespace reservation

Start uses the namespace map and issuer index as one protocol:

1. **Pre-issuance:** A1.2 may evaluate a new `ExpectedAbsent` request only while
   the complete `StateRootV1` and immutable `GreenfieldNamespaceCertificateV1` verify against the
   trusted bootstrap home and both pre-A1 authority collections re-enumerate empty. It then requires
   no `Authority`, `StartTombstone`, or foreign `StartReservation` for the session ID. A missing or
   unreadable individual record, absent compatibility projection, unverified/copy-created
   certificate, pre-A1 artifact, incomplete enumeration, or root error is never absence. A
   previously committed exact self reservation is handled only as the exact issuance retry below;
   it is not general absence.
2. **Issuance:** after preparing/verifying required immutable objects, one A1.2 root replacement creates
   the `Issued` intent, `issuer_request_index` entry, `object_index` entries, and
   `StartReservation` owned by the exact intent ID, issuer request ID, and payload commitment.
   External bytes published before this root have no issuance meaning.
3. **Exact issuance retry:** the same session ID, intent ID, issuer request ID, and payload
   commitment joins the indexed intent and its exact self reservation or terminal result. Any
   mismatch is an attempt rejection and cannot alter the stored intent/reservation/tombstone.
4. **Claim/application:** only the reservation whose three ownership fields exactly match the
   intent satisfies that intent's `ExpectedAbsent`. A foreign reservation, `Authority`,
   `StartTombstone`, missing reservation, or mismatched commitment fails closed. PID, socket,
   helper, plan, claim-owner liveness, and process-local state are irrelevant.
5. **Applied Start:** the same root transaction that records application journal and intent
   `Applied` atomically replaces `StartReservation` with `Authority`. The authority's
   `StartIntent` origin retains the exact intent, issuer, and payload commitment.
6. **Rejected or Expired Start:** the same root transaction that records the terminal intent,
   terminal handoff, and pending-input terminalization atomically replaces the reservation with a
   permanent `StartTombstone`. Its root-map `intent_id` remains the exact intent reference. The
   session ID is never reusable in A1; an exact retry joins the stored terminal result, while a
   different Start fails closed.
7. **Invalid before issuance:** invalid identity, path, schema, object, key, TTL, or mode input
   persists no intent, reservation, tombstone, issuer entry, or semantic root change.

### Exact mode preconditions

The posture domain is closed for V1:

| Mode | Required source posture | Initial resulting posture |
|---|---|---|
| `Start` | `ExpectedAbsent`, followed after issuance by the exact intent-owned self reservation only. | `ActiveAttached`, committed only when validated application is ready to own the exact target. |
| `Attach` | Exactly one of `ParkedResumable`, `DetachedReconciled`, `AwaitingAttention`, or an explicitly persisted `StaleRecoverable`; never a posture inferred from process loss during the attempt. | `ActiveAttached`. Existing obligations remain unchanged and canonical. |
| `ResumeOneTurn` | Exactly one of `ParkedResumable`, `DetachedReconciled`, `AwaitingAttention`, or an explicitly persisted `StaleRecoverable`, plus the exact resume handle. | `ActiveAttached` with `post_turn=Pending`. Exact accepted terminal evidence later yields `Terminal`; otherwise unresolved attention obligations yield `AwaitingAttention`, and a resumable clean result yields `ParkedResumable`. |

Every unlisted source posture or namespace variant fails closed. In particular, a new independent
Attach/Resume against `ActiveAttached`, `Terminal`, or `Invalid` is rejected; only an exact retry
of the already-Applied intent may join. No `StartTombstone` can satisfy any A1 transition. A1 does
not create, resolve, or change obligations when selecting the deterministic post-turn posture.

Parked-successor application sharpens these existing mode/precondition/application rules without
introducing another intent contract:

1. `Active` orchestration-session lifecycle with `ParkedResumable` posture is valid current durable
   authority, not session absence or a request to allocate a replacement session.
2. A successor cannot replace that authority with a caller-constructed `Allocating` snapshot.
3. `Attach` or `ResumeOneTurn` uses the exact expected authority revision and authority-record
   commitment/hash already required by `ExpectedRevision`.
4. Application atomically advances posture and authoritative lineage while preserving session
   identity, nonterminal lifecycle identity, and the exact world binding unless a separately
   authorized protocol changes that binding.
5. The target participant becomes the authority-approved successor only through transition
   application; StateStore persists that selected result and does not choose the transition.
6. Host execution episode construction and launch occur only after durable application.
7. `PID=0`, no active handle, helper/readiness state, a completed prior prompt, or a pending new
   prompt cannot satisfy, deny, or replace the authority precondition.
8. Prompt bytes may be committed through the immutable `TransitionInput` ref. Prompt delivery,
   stream, acceptance, pending, or completion status remains transport/application evidence, not
   authority.
9. Exact retry joins the committed intent, application, input-handoff, and post-turn result under
   the retry rules below; it never constructs another successor snapshot.
10. Failure after application reconciles from durable intent/application truth and must not restore
    a stale pre-transition snapshot or regress lifecycle to `Allocating`.

`Start` requires all of the following:

1. `authority_precondition` is `ExpectedAbsent`; A1.2 issuance first verifies the exact greenfield
   certificate, empty pre-A1 authority collections, and no namespace record. Later
   claim/application observes only the exact self reservation. Delete/recreate, compatibility
   hiding, a missing/copied certificate, an unreadable collection, any pre-A1 artifact, or any
   permanent namespace record does not satisfy it.
2. `source_authoritative_participant_id` and `resume_handle_ref` are absent.
3. The target participant, HMAC-bound `LeaseToken` ref, run ID, and shell-trace ID are new and
   exact. Resulting lineage is the valid initial lineage ending in that target. Caller is
   `PublicCli` or `Repl`, has no prior participant ID, and is bound to the unique issuer request.
4. The complete physical workspace/store binding and immutable store ID validate. World-scoped
   Start commits the full world pair; host-only Start commits none. Partial or ambient
   reconstruction fails closed.
5. Descriptor and attach-contract refs have the exact kinds/schemas/canonical commitments.
   Optional Start input is a `TransitionInput` HMAC ref; post-turn disposition is absent.
6. `input_handoff` is `Pending` with that exact input ref/run when input exists and
   `NotApplicable` otherwise.

`Attach` requires all of the following:

1. `ExpectedRevision` exactly matches current authority revision,
   `DurableSessionAuthorityHashInputV1` commitment, active participant,
   `AuthoritativeLineageHashInputV1` commitment, and one enumerated posture. PID, socket, helper,
   heartbeat, or attached-client observations cannot make it eligible.
2. Source is the current lineage tip. Target, HMAC-bound lease-token ref, and run are new and
   exact; target is a distinct valid successor and resulting lineage is the authority-approved
   append/replacement. Shell-trace identity is preserved exactly.
3. Physical workspace/store identity, world pair, descriptor ref, and attach-contract ref match
   the current authority and requested successor.
4. A `ResumeHandle` ref is present exactly when the committed attach contract requires it. Attach
   has no transition input and no post-turn disposition.
5. `input_handoff` is `NotApplicable`.

`ResumeOneTurn` requires all of the following:

1. The exact Attach revision/commitment, lineage, binding, descriptor, and attach-contract checks
   hold against one enumerated Resume posture.
2. The exact `ResumeHandle` ref is valid for current source participant and session.
3. A `TransitionInput` ref with the required HMAC key/domain/store/intent/run commitment is present
   and verifies over the exact one-turn input bytes.
4. `input_handoff` is `Pending` with that same input ref and run ID.
5. `post_turn_disposition` is `ReconcileToAttentionParkOrTerminal`. A verified
   `PostTurnCompletion` ref, claimed intent, current authority revision, and—only for a resumable
   outcome—a Complete `ObligationLedger` snapshot for the exact terminal event cut determine the
   revision-checked result. Pending materialization keeps reconciliation Pending; queue delivery,
   helper liveness, timeout, inbox rows/counts, and worker flags never determine posture.

### Lifecycle, retry, and fail-closed rules

1. Issuance verifies/publishes the immutable refs first, then commits `Issued`, issuer index,
   object index, and—only for Start—the exact reservation in one root revision before plan write or
   helper launch. Projection/launch failure leaves a fully reprojectable intent.
2. Claim CASes `Issued` to `Claimed` after revalidating payload commitment, key availability,
   expiry, namespace/authority precondition, identities, physical bindings, and every ref. Claim
   does not mutate authority or namespace.
3. Exact current-claim retry joins. A different claimant fails while the lease is current. After
   claim expiry, reclaim requires the current intent/root revisions and all preconditions; PID or
   socket liveness is never claim authority.
4. Application creates the immutable `ApplicationResult` object, then one root transaction commits
   authority mutation, application journal, intent `Applied`, result ref, and object-index entry.
   Start also replaces its reservation with Authority in that root; Attach/Resume replace the
   existing Authority value at the next authority revision. No split embedded/external or
   multi-root success model is permitted. Start/Attach record `startup_ownership=Pending` and
   `post_turn=NotApplicable`; Resume records `startup_ownership=NotApplicable` and exact post-turn
   run/revision `Pending`.
5. Core Applied result is immutable. Exact intent/issuer/payload retry joins it despite an older
   read revision and never reallocates target, appends lineage, rewrites binding, or repeats the
   authority revision. Current revisions remain mandatory for mutations.
6. A valid stored intent whose authority precondition becomes invalid before application, while
   the root/object/key set still verifies, commits one `TerminalHandoff`, `Rejected`, and any
   pending-input `TerminalWithoutAcceptance` in one root revision. Start additionally becomes its
   permanent tombstone. Rejected Attach/Resume may be followed only by a new intent against current
   authority; rejected Start permits only exact terminal join because its ID is burned. Missing or
   mismatched committed objects/keys instead take the non-mutating corruption path above.
7. Issued may expire at fixed `expires_at`; Claimed may expire only after journal, namespace, and
   exact resulting authority prove no application. Expiry atomically records terminal handoff and
   pending-input terminalization; Start also becomes its permanent tombstone. Any application
   marker/result requires reconciliation to Applied instead.
8. Stale revisions, substituted plan, wrong kind/schema/key/domain/commitment, wrong
   session/caller/source/target/lineage/binding, superseded claim, and conflicting replay fail
   closed before authority mutation. Exact read-only join is the only old-revision exception.
9. Plan removal, helper loss, EOF, timeout, or process death changes transport evidence only. It
   cannot alter reservation, intent, authority, retention, or an applied result.
10. Input acceptance CASes Pending to Accepted with an `InputAcceptanceHashInputV1` object bound to
    exact intent/run/input ref/participant. Exact duplicate joins. For an applied Resume, definitive
    terminal failure before acceptance must first publish and apply the exact `TerminalFailure`
    post-turn completion/result pair; only its resulting terminal handoff may set
    `TerminalWithoutAcceptance`. Missing/stale/reordered/mismatched evidence cannot change the
    substate.
11. Load/remove-before-use retries reproject from verified lease/input/transport refs through the
    existing builders. Applied retry may reproject only remaining handoff and never reapplies
    authority. Accepted or terminalized input is never redelivered.
12. Applied Start/Attach resolve startup ownership through one revision-CAS operation owned by
    `HostSessionAuthority`. The target authoritative participant's startup protocol produces
    `OwnershipAccepted` only after accepting ownership of the exact applied session/intent/run,
    application result, participant, and authority revision. The launch/application protocol
    produces `RuntimeCreationRejected` only as an exact typed rejection before participant
    ownership, and the target startup protocol produces `StartupFailedBeforeOwnership` only as an
    exact typed failure causally before any ownership acknowledgement. The protocol actor is
    closed: `OwnershipAccepted` and `StartupFailedBeforeOwnership` require
    `TargetAuthoritativeParticipant` with participant ID exactly equal to the evidence's active
    participant; `RuntimeCreationRejected` requires `LaunchApplicationClaimant` whose claim and
    claimant-attempt IDs exactly equal both the evidence and the persisted Applied intent. Every
    other actor/event pairing fails closed.
    `HostExecutionEpisode` owns observation reporting and A1.3's bounded real helper/REPL adapter
    transports that exact protocol event into one closed `HostStartupOwnershipEvidenceV1`; neither
    may relabel local observations into a protocol event. `HostSessionAuthority`, not the episode
    record, durably captures the submitted actor/event inside the committed
    `StartupOwnershipResultHashInputV1`. The plan and `SurfaceAdapter` only transport it. The evidence binds exact
    store, session, intent, claim, claimant attempt, run, application ref, expected authority
    revision, active participant, event identifier, and observation time. Its `evidence_id` must
    equal the identifier carried by its protocol-event variant and the stored startup substate's
    `evidence_id` must equal the result evidence ID. `OwnershipAccepted` publishes
    `StartupOwnershipResultHashInputV1` with unchanged current authority revision and mutates only
    the intent revision/substate. `RuntimeCreationRejected` and `StartupFailedBeforeOwnership`
    publish `TerminalReconciled` with the exactly corresponding reason; an event/outcome/reason
    mismatch fails closed. Running state, readiness, endpoint publication, PID/process/helper/socket/
    handle posture, timeout, EOF, local adapter error, plan loss, or ambiguity cannot construct any
    of these protocol events and leaves startup ownership Pending.
    A definitive terminal result atomically advances authority once without restoring a
    pre-application snapshot: Start deterministically becomes `Terminal`; Attach deterministically
    becomes `DetachedReconciled`. Both preserve session identity, workspace/store/world binding,
    lineage, policy, descriptor, and active successor identity. The same root revision terminalizes
    any Pending Start input to `TerminalWithoutAcceptance` and commits the exact terminal handoff;
    crash/retry joins all three results or none. Exact duplicate evidence joins; a
    conflicting evidence ID/protocol event, claimant, stale revision, substituted application, or
    second authority advance fails closed.
    This CAS increments `intent_revision` exactly once. A1.2 owns this internal decision protocol;
    A1.3 owns invocation by the real helper/REPL consumer, and A2 later generalizes episode
    observations without weakening these A1 commitments.
13. Transport stays `Retained`/object-index `Present` until one exact committed
    `TerminalHandoffHashInputV1` proves release. Input-bearing modes require Accepted or
    TerminalWithoutAcceptance; input-free Start/Attach require NotApplicable. Applied Start/Attach
    require `startup_ownership=Accepted` or `TerminalReconciled`, and the terminal handoff carries
    that exact startup-ownership result ref. Every applied Resume terminal handoff requires
    post-turn Applied with the exact completion/result pair; `TerminalClean` and `TerminalFailure`
    carry no obligation snapshot, while `ResumableClean` carries the exact Complete ledger snapshot.
    Rejected/Expired require proof of no application.
    One root revision records `ReleaseEligible` in both parent and object index, deletion retries
    idempotently, and a later root records `Released`. A missing plan is never evidence. Exact
    issue/application/expiry retry verifies transport bytes only for Retained/Present; for
    ReleaseEligible or Released it verifies the exact matching parent/index retention state and
    terminal handoff without recursively requiring deleted bytes. A mismatched handoff/index or
    surviving Released bytes not durably removed as specified by the crash rules fails closed.
14. Resume completion first publishes/verifies one immutable
    `PostTurnProtocolEventHashInputV1` object and then a `PostTurnCompletionHashInputV1` whose exact
    `PostTurnProtocolEvent` ref, intent/claim/claimant-attempt/run/revision, B1 acceptance
    ID/revision and accepted work identity, B0 stream, the exact owner-supplied host-transition correlation,
    terminal event ID, and monotonically ordered run-local terminal event sequence all match that
    object and the persisted Applied intent. The protocol-event/completion `run_id` equals the
    correlation's `transition_run_id` and remains distinct from the accepted active-run ID.
    `ResumableClean` and `TerminalClean`
    require a `TargetAuthoritativeParticipant` actor equal to the active participant, the exactly
    matching event kind, and Accepted input. `TerminalFailure` with reason
    `ResumeRuntimeCreationRejected` requires the `LaunchApplicationClaimant` actor to equal the
    applied claim/claimant attempt and may occur only before input acceptance. `TerminalFailure`
    with reason `TargetFailedBeforeInputAcceptance` or `TargetFailedAfterInputAcceptance` requires
    the exact active participant actor and respectively Pending or Accepted input. The completion
    outcome must equal the protocol-event kind; actor,
    reason, ID, sequence, scope, or ref mismatch fails closed. PID/helper/socket/handle/process
    posture, readiness, timeout, EOF, stream loss, or a local adapter error is not a protocol event
    and cannot construct completion. Terminal clean or failure atomically advances authority to
    `Terminal`, applies the post-turn result, terminalizes Pending input only for the two exact
    pre-acceptance failure reasons, and commits terminal handoff without an obligation snapshot.
    `ResumableClean` requires Accepted input and instead commits `AwaitingObligationCut` with the
    completion ref, exact acceptance/stream/transition correlation, and required event cut but does
    not mutate authority. Exact retry joins the same event/completion/pending or terminal result,
    initial intent expiry cannot erase it, and transport remains Retained until its exact release
    gate.
15. `ObligationLedger` alone owns the semantic query and produces
    `ObligationLedgerSnapshotReadV1`; `HostSessionAuthority` is a consume-only client. A Complete
    snapshot binds exact authority store, session, active participant, B1 acceptance record
    ID/revision and accepted task/active-run identity, B0 stream ID, the unchanged verified
    transition intent/revision/payload commitment and distinct transition run, observed authority
    revision, per-session ledger revision, terminal event ID/sequence, and a
    `materialized_through_event_sequence` at least that terminal sequence. The materialization cut
    repeats the acceptance record and stream identity; all repeated fields must match exactly. The
    snapshot also carries every post-acknowledgement retained `Event` journal ref through that
    watermark as a typed B3.1 member, ordered by event sequence with unique journal/frame/event
    identities and full canonical event commitments. It is exhaustively joined to B2.1's journal;
    any untyped, omitted, substituted, or scope-mismatched retained event keeps the cut
    non-Complete even when the attention disposition would otherwise be empty. Revisions/sequences
    and all identities are nonzero/non-empty as
    applicable. Each unresolved entry binds obligation ID/revision and the exact `CanonicalSha256` commitment of
    `ObligationSnapshotRecordHashInputV1`; entries are non-empty unique IDs sorted by raw UTF-8 byte
    order. `ObligationLedger` sets the closed attention disposition and guarantees it matches the
    complete canonical record set: NoUnresolvedAttention requires an empty entry vector and
    HasUnresolvedAttention requires a non-empty vector. It derives only from canonical obligation
    records, never inbox rows, counts, worker flags, helper state, or compatibility projections.
16. A1.2 may publish the Complete snapshot bytes unchanged as `ObligationSnapshot` and validate
    only their closed schema, exact acceptance/stream/transition scope, cut/ref commitments, and
    equality with the current pending completion and authority. It cannot invent or repair the
    correlation, equate transition run with active run, or enumerate, classify, create, resolve,
    reinterpret, repair, or overwrite obligations. The ledger revalidates the same per-session
    revision and event cut immediately before the authority commit under the retained
    transaction/lock; the lock supplies physical serialization but not semantic ownership. A
    Pending ledger read leaves
    `AwaitingObligationCut` unchanged. A Complete `HasUnresolvedAttention` result selects
    `AwaitingAttention`; Complete `NoUnresolvedAttention` selects `ParkedResumable`. One root
    transaction advances post-turn to Applied, mutates authority at most once, and stores the
    completion/snapshot/result refs in intent state and `post_turn_application` journal. Exact
    duplicate joins; stale, reordered, mismatched, or conflicting completion/cut fails closed.
17. C1 owns the event-to-obligation materializer, canonical obligation revisions, and the complete
    per-session event cut required above. The current pre-C1 ledger cannot provide that semantic
    completeness proof: empty can mean either no unresolved obligation or not-yet-materialized
    events. C1 in turn requires B0 runtime identity/order, B1 accepted run identity, B2.1 durable
    observation/reconciliation, and B3.1 exact retained-event semantics. Therefore A1.2 can specify
    and persist the pending protocol but cannot close a ResumableClean post-turn or claim its full
    packet exit before that corridor lands. A1.2 does not implement any corridor owner or claim
    `RG-EVENT-01`, `RG-SUP-01`, `RG-SUP-02`, `RG-MSG-01`, `RG-OBL-01`, or `RG-OBL-02`.

### Crash reconciliation

On process restart or before retrying a nonterminal intent, `HostSessionAuthority` reconciles in this order:

1. Re-run the bootstrap classifier under the lock. Resume only an exact `InitializationPending`
   marker or a `ValidExisting` root; every invalid, partial, permission-invalid, unreadable, or
   unsupported root state fails closed and never initializes a replacement namespace.
2. Strictly decode the last verifiable root under lock and verify store ID and bound bootstrap home,
   the exactly-one-active key registry invariant, every required active/verification key envelope,
   the immutable greenfield certificate, empty pre-A1 authority collections, complete
   namespace/intent/index/journal invariants, and
   every reachable ref whose storage state requires bytes. A Released transport validates through
   its exact terminal handoff instead. Root uncertainty never falls back to external object or key
   presence.
3. `Issued` Start must have been created by A1.2 after verifying the greenfield certificate and
   empty pre-A1 authority collections and must own its exact reservation; Issued non-Start must
   retain its exact authority precondition. With no application marker it remains claimable until
   expiry. Missing/mismatched self reservation is corruption, not absence.
4. `Claimed` with unchanged precondition may resume under the exact claim or be reclaimed after
   lease expiry. No liveness observation substitutes.
5. Exact authority mutation plus journal/result ref commits Applied even if the caller saw failure;
   reconciliation recovers/joins the identical result. For Start, Authority origin must match the
   former reservation. A visible Authority without matching application proof fails closed.
6. Objects with no root parent are orphans and never prove issuance/application. A missing or
   mismatched object required by Retained/Present state is corruption. ReleaseEligible plus absent
   transport may advance only after exact terminal-handoff verification. Applied Start/Attach also
   verify their startup-ownership evidence/result ref. An AwaitingObligationCut Resume verifies its
   exact post-turn protocol-event ref, completion/event equality, acceptance/stream/transition
   correlation, and cut scope and remains pending until the semantic owner returns Complete;
   applied resumable post-turn Resume verifies the same event/completion chain, its exact Complete
   obligation snapshot ref, and current ledger revalidation.
7. Missing plan transport is reprojected only after revisioned input/application/startup-ownership/
   post-turn checks.
   Applied Resume with pending post-turn work accepts only the exact actor-bound protocol-event ref,
   matching completion/result, closed input state, and exact ledger cut when required for its
   committed run; ambiguous or incomplete evidence remains pending and diagnosable.
8. If root-directory fsync previously failed, report no inferred success. Reconcile whichever
   complete root revision and object set is verifiable, then use issuer index/application journal
   for exact retry; never choose the newest-looking orphan or temp file.
9. If neither prior precondition nor exact committed result can be proven—or any referenced HMAC
   key is lost—fail closed and retain diagnosable state without logging sensitive bytes or
   commitments.
10. Reconciliation, greenfield-certificate validation, key rotation/retirement, tombstoning, and
   payload cleanup are idempotent across repeated crashes and
   cannot increment initial or post-turn authority more than once per intent.

### Greenfield-only activation and unsupported pre-A1 state

A1 intentionally defines no legacy-authority migration or compatibility-adoption protocol. There
are no deployed users or valid pre-A1 authority records to preserve. Adding conversion,
quarantine, evidence, conflict arbitration, or dual-write semantics would create an unneeded
permanent authority surface. The generic StateStore migration/schema-evolution and
CompatibilityReadModel ownership in `01-target-architecture.md` remains available for other
schemas and read-only diagnostics; it does not authorize importing pre-A1 session/participant
records into A1 authority.

The closed rules are:

1. The bootstrap classifier enumerates the two exact pre-A1 authority collections defined above.
   A complete empty result is one prerequisite for `FreshAbsent`, `InitializationPending`,
   `ValidExisting`, and every semantic transaction.
2. Any safely observed artifact produces `UnsupportedLegacyState`. Multiple, contradictory,
   same-named, or differently shaped artifacts have the same single result; A1 does not parse them,
   create evidence, select a winner, or derive a session namespace.
3. Any unreadable, permission-invalid, symlinked, partially enumerable, or otherwise uncertain
   collection is `CorruptOrUnsupported`. A missing or unreadable individual record and an empty
   compatibility projection never imply namespace absence.
4. Once the greenfield certificate exists, every pre-A1 session/participant authority-write API is
   disabled for that store and must fail before touching either old or A1 state. Every A1 root
   transaction independently revalidates that the old collections remain empty, so an older or
   external writer cannot create state behind the certificate.
5. A1.1 routes every in-repository pre-A1 session/participant writer through the same root lock
   before activation. Such a writer may proceed only while neither `init-v1.json` nor
   `state-root-v1.json` exists; after marker publication it rejects without touching state. The
   initializer holds that lock from marker publication through the final empty-collection scan and
   root publication. Concurrent unmodified/mixed-version binaries are unsupported and must be
   quiesced; A1 never claims safety from process-liveness observation.
6. Pre-A1 helper plans without `intent_id` and `payload_commitment` cannot drive an A1 transition.
   Once a producer is switched, a mixed-version consumer fails closed and requires a newly issued
   valid intent; it never reconstructs authority from a plan or compatibility view.
7. A1 never deletes unsupported artifacts or offers a runtime reset flag. Because this is a
   greenfield contract, recovery is an explicit operator/developer cleanup outside the running A1
   authority protocol, followed by a new complete bootstrap classification. Diagnostics may report
   only bounded relative path and entry-type metadata; they never read or log artifact contents.

Helper endpoints/paths and auto-attach policy, eligibility, claim, and settlement remain unchanged.


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
5. PID, process/helper presence, active handles, readiness, and prompt-stream state are episode or
   transport observations only; their absence does not erase `ParkedResumable` authority.
6. Episode construction and launch follow durable transition application and cannot reset a parked
   session to `Allocating` or authorize a successor participant.

## 2A. Runtime event identity and ordering carrier

B0 extends the existing runtime stream family; it does not create a parallel transport. The
runtime producer assigns identity before emission. Host decoders, receipt code, supervisors,
messaging code, and the ledger consume those fields unchanged.

```rust
struct RuntimeFrameIdentityV1 {
    schema_version: u32,          // exactly 1
    stream_id: String,            // stable for one accepted runtime stream
    frame_sequence: u64,          // starts at 1; strictly monotonic and gap-free
}

struct RuntimeEventIdentityV1 {
    event_id: String,             // stable for one logical semantic event
    event_sequence: u64,          // starts at 1; strictly monotonic within stream
}

struct RuntimeTerminalIdentityV1 {
    terminal_event_id: String,
    terminal_event_sequence: u64,
}
```

Every `ExecuteStreamFrame` carries `RuntimeFrameIdentityV1`. Semantic `Event` frames additionally
carry `RuntimeEventIdentityV1`; the terminal `Exit` frame carries both a semantic event identity
and `RuntimeTerminalIdentityV1` with equal ID/sequence. `Start`, stdout, stderr, event, and terminal
frames participate in the same frame sequence. `Error` is either a typed semantic terminal frame
with exact terminal identity or an observation/transport error that cannot prove run completion.

B0 rules:

1. `stream_id`, frame sequence, event ID/sequence, and terminal identity are runtime-produced;
   the host never synthesizes them from arrival order, timestamps, payload hashes, span IDs, EOF,
   or process state.
2. Re-emitting the same logical frame/event after retry or reconnect preserves every identity.
   The producer never originates a second meaning at the same sequence position.
3. Frame and event sequences are positive and strictly monotonic. An exact identity with identical
   canonical bytes is valid replay and B2.1 consumes it as a no-op. A gap, reorder, conflicting
   duplicate, or frame after terminal is protocol-invalid and B2.1 rejects it; that consumer-side
   enforcement is not B0 ownership.
4. The terminal frame is the final semantic event and names its exact event ID/sequence. Stream
   exhaustion without it is not completion.
5. B0 owns only the carrier. B1 owns acceptance, B2.1 owns durable observation/dedupe/restart,
   B3.1 owns retained-event semantics, and C1 owns obligation materialization/completeness.

## 2B. Bounded retained worker event envelope

B3.1 supplies the minimum semantic envelope C1 needs without moving B3.2 early. It extends the
existing shared `AgentEvent` in `crates/common/src/agent_events.rs` with one optional, explicitly
typed top-level `worker_event` field. The same module defines the producer-construction-only
`NormalizedWorldWorkerEventFacetV1`. In `world-service/member_runtime.rs`, one named fail-closed
normalizer parses the existing provider `AgentWrapperEvent` kind/payload before `AgentEvent`
construction and produces this facet; it is the sole permitted provider-payload interpretation for
the C1 path. `ExecuteStreamFrame::Event { event: AgentEvent }` remains unchanged, so
`world-service` can produce the envelope and shell consumers can validate it without host-side
identity or semantic inference from `AgentEvent.data` or a second transport protocol.
Both the equality-only commitment representation and its host-transition correlation carrier live
in `crates/common/src/authority_commitment.rs` and are re-exported by
`crates/common/src/lib.rs`. They contain no verifier or semantic authority:

```rust
enum OpaqueAuthorityCommitmentV1 {
    CanonicalSha256 {
        digest_hex: String,
    },
    StoreHmacSha256 {
        key_id: String,
        domain: String,
        digest_hex: String,
    },
}

struct HostTransitionWorkCorrelationV1 {
    schema_version: u32,          // exactly 1
    authority_store_id: String,
    orchestration_session_id: String,
    authoritative_participant_id: String,
    transition_intent_id: String,
    transition_intent_revision_observed: u64,
    transition_run_id: String,
    transition_payload_commitment: OpaqueAuthorityCommitmentV1,
    authority_revision_observed: u64,
}

enum WorldWorkerEventClassV1 {
    Reply,
    ProgressUpdate,
    FollowUpQuestion,
    ApprovalRequest,
    Blocked,
    AttentionRequired,
    ForkRequest,
    ForkRecommendation,
    Result,
    Failure,
}

// Producer-construction type, not an independent wire envelope.
struct NormalizedWorldWorkerEventFacetV1 {
    schema_version: u32,          // exactly 1
    thread_id: String,
    event_class: WorldWorkerEventClassV1,
    attention_required: bool,
    causation_message_id: String,
    causation_request_id: String,
    payload: serde_json::Value,
}

struct WorldWorkerEventV1 {
    schema_version: u32,          // exactly 1
    acceptance_record_id: String,
    stream_id: String,
    frame_sequence: u64,
    event_id: String,
    event_sequence: u64,
    request_id: String,
    active_run_id: String,
    host_transition_correlation: Option<HostTransitionWorkCorrelationV1>,
    causation_message_id: String,
    causation_request_id: String,
    orchestration_session_id: String,
    source_participant_id: String,
    target_participant_id: String,
    source_backend_id: String,
    target_backend_id: String,
    world_id: String,
    world_generation: u64,
    thread_id: String,
    event_class: WorldWorkerEventClassV1,
    attention_required: bool,
    payload: serde_json::Value,
    emitted_at: Timestamp,
}

struct AgentEvent {
    // All existing legacy fields remain unchanged.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    worker_event: Option<WorldWorkerEventV1>,
}
```

The exact B0 event identity is copied unchanged. `acceptance_record_id` and `active_run_id` join the
B1 accepted retained turn. When B1 carries `host_transition_correlation`, B3.1 copies it unchanged;
B3.1 rejects absence or mismatch rather than inferring intent/run/authority scope from request ID,
active-run ID, foreground state, or payload. Before `AgentEvent` construction, the producer
normalizer maps every provider wrapper event into exact thread/class/attention/payload semantics
and joins request/message causation from the retained B1 context. Explicit supported non-attention
wrapper shapes receive a typed non-attention class. Unknown, ambiguous, deferred, or malformed
shapes fail the retained stream and C1 cut closed; they cannot be dropped, left untyped, or
downgraded to progress or no-attention. The final source/target, world, run, thread, class,
attention, and causation fields are explicit and validated before ledger delivery; the resulting
B2.1 generic journal ref commits the exact canonical shared-envelope bytes without the supervisor
interpreting them.

After emission, neither host code nor C1 may infer a missing field from free text,
`AgentEvent.data`, optional nested runtime payloads, the most recent worker, or foreground request
state. Raw provider payload may remain as compatibility/debug data, but it is not semantic truth
for the C1 path. B3.1 does not require an upstream provider-wrapper taxonomy change and does not
complete host-to-worker messaging, retained lifecycle, park, cancel, stop, fork, or model-facing
early return; those remain B3.2/B4.

`worker_event` may be absent only for streams outside the B3.1-adopted accepted-retained scope. In
that scope, every post-acknowledgement `ExecuteStreamFrame::Event` must carry it through the exact
terminal cut; absence or invalidity is a protocol failure and makes a Complete C1 snapshot
unavailable. Every identity/correlation/semantic field comes from this typed member rather than
host parsing of `AgentEvent.data`. The normalizer may read provider `AgentWrapperEvent.data` only
before emission, under its explicit exhaustive provider-shape tests, to construct
`NormalizedWorldWorkerEventFacetV1`.
`OpaqueAuthorityCommitmentV1` preserves the exact variant and fields of the HostSessionAuthority
commitment supplied later by A1.2, but B1/B3.1/C1 may only retain and compare it; they cannot mint,
verify, reinterpret, log as observability evidence, or use it as obligation semantics.

## 2C. `WorldWorkAcceptanceRecordV1`

B1 persists one minimal accepted-work anchor for either lifecycle family without pulling E2 or
B3.2 into the early corridor:

```rust
struct WorldWorkAcceptanceContextV1 {
    schema_version: u32,          // exactly 1
    proposed_acceptance_record_id: String,
    request_id: String,
    message_id: Option<String>,
    caller_backend_id: String,
    host_transition_correlation: Option<HostTransitionWorkCorrelationV1>,
}

enum AcceptedWorldWorkIdentityV1 {
    EphemeralTask {
        task_run_id: String,
    },
    RetainedTurn {
        active_run_id: String,
        message_id: String,
        target_participant_id: String,
    },
}

struct WorldWorkAcceptanceRecordV1 {
    schema_version: u32,          // exactly 1
    acceptance_record_id: String,
    request_id: String,
    authority_store_id: String,
    authority_revision_observed: u64,
    orchestration_session_id: String,
    caller_participant_id: String,
    caller_backend_id: String,
    target_backend_id: String,
    world_id: String,
    world_generation: u64,
    work_identity: AcceptedWorldWorkIdentityV1,
    host_transition_correlation: Option<HostTransitionWorkCorrelationV1>,
    current_policy_snapshot_ref: PolicySnapshotRefV1,
    current_policy_snapshot_hash: String,
    current_policy_revision: String,
    runtime_acceptance: RuntimeAcceptanceEvidenceV1,
    accepted_at: Timestamp,
    record_revision: u64,
}
```

B1 preallocates `proposed_acceptance_record_id` through ReceiptRegistry before submission, but that
candidate is not an accepted record and is not inspectable as accepted work until the runtime
acknowledgement arrives. `WorldWorkAcceptanceContextV1` lives in
`crates/transport-api-types/src/lib.rs` and imports the common correlation type. The existing
`ExecuteRequest` and `MemberTurnSubmitRequestV1` each gain one such typed context; `world-service`
retains the same request-scoped context in its stream/member context. On acknowledgement, B1
persists a record whose `acceptance_record_id`
exactly equals the proposal and whose `runtime_acceptance` joins the acknowledged B0 stream. An
exact submission retry reuses the same proposal; conflicting reuse fails closed. Subsequent B3.1
events copy the retained acceptance ID, request/message causation, caller backend, and optional
correlation into their typed `AgentEvent.worker_event` member; the existing member-turn request
supplies orchestration session, caller/worker participant, worker backend, active run, and world.
This is a typed
request-envelope extension only: it neither creates acceptance before acknowledgement nor changes
`ExecuteStreamFrame`.

The record owns accepted work identity, not final lifecycle, supervisor claim, host-transition
semantics, or model-facing receipt semantics. `host_transition_correlation` is absent for ordinary
world work and remains absent on every production path until A1.2 supplies it from an already
validated HostSessionAuthority transition. B1 defines and stores the optional carrier as part of
its canonical acceptance record, validates only exact equality with the record's
store/session/caller/authority fields, and retains it unchanged; it neither authenticates, issues,
applies, nor interprets the transition and never equates `transition_run_id` with the distinct
accepted task/active-run identity. The landed A1.1e facade is therefore sufficient for B1's current
authority resolution without a new hash domain or verifier. Production A1.2 later becomes the sole
source, first validating its own intent/revision/payload commitment and then supplying the exact
correlation through the HostSessionAuthority-owned call boundary. B1 validates the exact current
policy identity used for submission but does not invent the E2 retained-worker capability cap. E2
creates the immutable run/cap commitments and the final active receipt references this exact
acceptance record. A conflicting retry cannot create another record for the same runtime work
identity. B2.1 creates its separate claim and journal only through this record.

## 3. `ActiveEphemeralTaskReceiptV1`

```rust
struct ActiveEphemeralTaskReceiptV1 {
    schema_version: u32,
    acceptance_record_id: String,
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
    acceptance_record_id: String,
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

The B1 acceptance record is an internal durable anchor, not the returnable active receipt described
below. It captures exact current-policy and B0 acknowledgement truth. E2 must create the final
immutable policy/cap commitment and B2.1 must persist its separate observation claim before B2.2
may return the linked active receipt.

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
    acceptance_record_id: String,
    stream_id: String,
    frame_sequence: u64,
    runtime_submission_id: Option<String>,
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
    stream_id: String,
    last_durable_frame_sequence: u64,
    last_durable_event_sequence: Option<u64>,
    claim_revision: u64,
    lease_epoch: u64,
    resumable: bool,
}
```

V1 acceptance boundaries:

- `RuntimeAcceptanceEvidenceV1.acceptance_record_id` exactly equals the request's proposed ID, and
  the acknowledgement belongs to the runtime stream holding that same typed acceptance context;
  absence or mismatch fails before record persistence or B3.1 event emission.
- For `run_world_task`, acceptance may use the first non-terminal `Start` or `Registered` frame
  only when the protocol defines that frame as runtime acceptance and its B0 stream/frame identity
  includes or unambiguously joins to `task_run_id`.
- For `continue_world_worker`, acceptance requires an explicit retained-turn submission
  acknowledgement or first non-terminal B0 frame that includes or unambiguously joins to
  `active_run_id`, `message_id`, and the exact retained target.
- A socket write, HTTP request submission, process spawn attempt, or locally allocated ID is not runtime acceptance by itself.
- A terminal-only identity observation cannot be relabeled as pre-terminal acceptance.
- If the current runtime protocol cannot expose accepted identity before terminal exit, extend that protocol before changing the foreground tool to receipt-oriented early return.

B1 persists and inspects both task and retained-turn acceptance records while the existing
foreground call still waits. B2.1 then atomically hands observation to the supervisor while that
blocking compatibility behavior may remain. Foreground early return is a separate B2.2 gate.

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
5. `active_turn_ref` is a lifecycle reference updated atomically with B1 accepted-record creation
   and supervisor closeout; RetainedWorkerRuntime does not own accepted-turn identity.

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

1. **Accepted anchor first:** B1 persists the acceptance record, exact current-policy identity, B0
   stream identity, and acknowledgement sequence before B2.1 creates an observation claim.
2. **Single logical observer:** supervisors claim a lease with
   `(acceptance_record_id, record_revision, lease_epoch)`. A stale lease cannot write a newer
   revision.
3. **Durable observation journal:** at B1 acceptance, B2.1 installs the only post-acceptance
   observation path with no unjournaled handoff gap. It records canonical B0 frame bytes and the
   receipt-scoped frame/event cursor plus the exact B1 acceptance record ID/revision, accepted-work
   identity, and optional transition correlation before waiter delivery or derived receipt/terminal
   state.
4. **Restart discovery:** startup scans non-terminal accepted/running receipts and resumes
   observation from the durable cursor or performs exact runtime reconciliation.
5. **Frame dedupe/order:** each frame is keyed by exact `(acceptance_record_id, stream_id,
   frame_sequence)`. An identical duplicate is a no-op; a gap, reorder, conflicting duplicate, or
   post-terminal frame fails closed.
6. **Event dedupe/order:** durable worker events use exact `(acceptance_record_id, event_id,
   event_sequence)` joined to their frame. Reprocessing cannot duplicate the journal entry. C1,
   not the supervisor, decides whether and how that event materializes an obligation.
   Once B3.1 lands, its semantic validator requires the event's acceptance ID, active run, and
   optional transition correlation to exactly equal the B1 record before generic journal handoff;
   absence, substitution, or mismatch fails closed without inference. B2.1's packet exit itself
   requires only opaque canonical-event commitment plus B0/B1 identity and order.
7. **Monotonic states:** terminal states never revert; stale observers cannot overwrite newer
   state; equal-revision conflicting writes fail closed.
8. **Interrupted observation:** EOF, timeout, observer/process loss, or PID/helper/socket state
   without exact terminal event proof records an interruption and retry metadata. It does not
   fabricate terminal success or a complete obligation cut.
9. **Reconciliation:** only exact producer replay/reconciliation that returns the missing B0 frames
   and exact terminal event may advance the cursor or close the run. A runtime known to have exited
   without that terminal event records an interruption/protocol failure with diagnostics, remains
   incomplete, and cannot produce a C1 Complete cut; ambiguous truth likewise retries/fails closed.
10. **Terminal ordering:** only the exact B0 terminal event ID/sequence closes the observation
    journal. Receipt terminal state and worker active-turn clearing commit atomically or through
    replay-safe idempotent owner-approved steps; StateStore supplies persistence only.
11. **Ledger handoff:** the supervisor invokes C1 with durable exact B3.1 events and the terminal
    cut, including exact acceptance-record ID/revision, stream ID, accepted-work identity, and the
    scope-equal owner-supplied transition correlation when a transition-scoped snapshot will be queried. It does not
    classify/materialize obligations. C1 independently commits canonical ledger revisions and
    completeness; coordinated storage never transfers semantic ownership.
12. **Blocking compatibility:** B2.1 may leave the foreground waiting on the durable receipt after
    handoff. Only B2.2 enables model-visible early return.
13. **Cancellation:** one durable cancel request ID is reused across retries; repeated transport
    delivery is safe.
14. **Diagnostics:** non-zero exit, stream error, reconciliation failure, and cancel failure retain
    exact active-run/session/world/policy/stream/event joins.

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
9. no compatibility copy or `CompatibilityUnproven` evidence is used for contract promotion.
