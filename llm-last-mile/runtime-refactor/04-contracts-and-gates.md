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
authority root. On supported Unix hosts, a creator fixes the intended owner before
creation, creates only the final physical directory with mode `0700` independent of umask, opens
that directory no-follow, and accepts it only after post-open validation proves all of the
following:

1. the selected absolute UTF-8 physical path is the path represented by the opened handle;
2. the handle names a directory owned by the intended per-user owner;
3. permission and special bits are exactly `0700`;
4. the ACL has only the base owner/group/other access entries represented by mode `0700`, with no
   named user, named group, inherited, default, or other extended ACL entry, even when an ACL mask
   would make an unexpected entry ineffective;
5. no path component or final entry was followed through a symlink or substituted with another
   type, and the captured platform physical identity matches before acceptance; and
6. a final no-follow reopen/revalidation observes the same physical identity, owner, type, mode,
   and ACL as creation/opening, so replacement or validation uncertainty fails closed.

Creation must use an exclusive/no-follow operation or an equivalent directory-relative sequence
that distinguishes successful creation from `AlreadyExists`. A newly created root is never
accepted from its requested mode alone: it is reopened and validated. The creator must `fsync` the
new directory and its affected parent where the platform supports the A1 durable-filesystem
contract. Ambient umask may remove bits during creation, but the creator may set the new inode to
exact `0700` before acceptance; it may never broaden or otherwise repair a root that existed before
the attempt.

An existing root, including a custom `SUBSTRATE_HOME`, is accepted only if it already passes the
same rules. Wrong type, symlink, owner mismatch, `0755`, `0750`, any group/world permission,
setuid/setgid/sticky or other special bits, foreign or inherited ACL grants, changed identity, or
an indeterminate check returns exactly this diagnostic shape before any descendant write:

```text
substrate: unsupported SUBSTRATE_HOME '<path>': expected a private directory owned by intended uid <uid> with exact mode 0700 and no foreign ACL grants; found <reason>. Existing roots are never repaired; reset it manually and retry.
```

The `<reason>` token is one of `missing-parent`, `wrong-type`, `symlink`, `wrong-owner`,
`wrong-mode`, `foreign-acl`, `owner-ambiguous`, `replaced`, or `validation-unavailable`. For an
ordinary non-root process, the intended owner is its effective UID. An installer or provisioner
running as root resolves the intended account from its explicit supported user input first
(`SUBSTRATE_INSTALL_PRIMARY_USER` where applicable), then its verified invoking-user signal such
as `SUDO_USER`; macOS/Lima guest provisioning uses the discovered Lima VM user. It resolves that
account through the platform account database and uses the resulting UID. Root execution with no
unambiguous intended non-root user fails as `owner-ambiguous`; it does not silently create a
root-owned user state home. No product path chmods, chowns, removes ACLs, deletes contents,
migrates, adopts, converts, or falls back to a shared home. Failure occurs before config/runtime
scaffolding and before any authority marker, key, root, or legacy state mutation. Repeating
creation against an unchanged valid root is idempotent.

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

enum PostTurnCompletionOutcomeV1 {
    ResumableClean,
    TerminalClean,
    TerminalFailure,
}

struct PostTurnCompletionHashInputV1 {
    schema_version: u32,
    intent_id: String,
    run_id: String,
    authority_revision_observed: u64,
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
    post_turn_completion_ref: Option<AuthorityObjectRefV1>,
    post_turn_application_result_ref: Option<AuthorityObjectRefV1>,
    recorded_at: TimestampV1,
}
```

`CanonicalSha256` is exactly lowercase-hex
`SHA-256(CanonicalJsonV1(named_hash_input))`. The authority-record commitment uses
`DurableSessionAuthorityHashInputV1`; lineage uses `AuthoritativeLineageHashInputV1`; immutable
intent payload uses `HostSessionTransitionPayloadHashInputV1`; descriptor and attach-contract
objects use their named wrappers; resume handles, policy objects, and retained-worker reference
objects use `ResumeHandleHashInputV1`, `PolicyObjectHashInputV1`, and
`RetainedWorkerObjectHashInputV1`; both initial and post-turn application objects use
`ApplicationResultHashInputV1`; acceptance, completion, and terminal-handoff objects use their
corresponding named wrappers. `updated_at`, intent/claim/root revisions not explicitly present in a
wrapper, mutable state, presentation-only fields, plan/socket paths, and adjacent object metadata
are not accidentally swept into a hash. Authority-record, lineage, payload, descriptor,
attach-contract, resume-handle, policy, retained-worker, application, acceptance, completion, and
terminal-handoff commitments must use `CanonicalSha256`; a `StoreHmacSha256` variant in those
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
terminal-handoff, descriptor, resume-handle, policy, and retained-worker fixtures use their named
wrappers; the typed-ref fixture uses `AuthorityObjectRefV1` itself. Raw input, lease-token, and
transport fixtures commit both exact bytes and fixed-key HMAC outputs. Each sensitive-domain
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
  handoff and no remaining transport bytes; an unexpected surviving copy is securely removed
  under the lock and never regains authority meaning.
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
        authority_revision_before: Option<u64>,
        authority_revision_after: u64,
        active_authoritative_participant_id: String,
        resulting_posture: HostSessionPostureV1,
        authority_record_commitment: AuthorityObjectCommitmentV1,
        application_result_ref: AuthorityObjectRefV1,
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
    Applied {
        completion_ref: AuthorityObjectRefV1,
        authority_revision_before: u64,
        authority_revision_after: u64,
        resulting_posture: HostSessionPostureV1,
        application_result_ref: AuthorityObjectRefV1,
        applied_at: TimestampV1,
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
6. `intent_revision` increments on every durable intent-state, claim, input-handoff, post-turn, or
   transport-payload-state change. A stale intent or claim revision cannot mutate the record; this
   does not block an exact read-only join.
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
claim/application, and initial Start-origin authority birth as one intent protocol. A1.3 adopts
that protocol on real CLI/REPL consumers; A1.1 primitive tests are not evidence that a production
Start path is adopted.

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
   `PostTurnCompletion` ref, claimed intent, exact unresolved-obligation read, and current authority
   revision—not queue delivery, helper liveness, or timeout—determine the revision-checked result.

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
   multi-root success model is permitted. Start/Attach record `post_turn=NotApplicable`; Resume
   records exact run/revision `Pending`.
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
    exact intent/run/input ref/participant. Exact duplicate joins. Exact terminal failure before
    acceptance uses the intent's `TerminalHandoff`; missing/stale/reordered/mismatched evidence
    cannot change the substate.
11. Load/remove-before-use retries reproject from verified lease/input/transport refs through the
    existing builders. Applied retry may reproject only remaining handoff and never reapplies
    authority. Accepted or terminalized input is never redelivered.
12. Transport stays `Retained`/object-index `Present` until one exact committed
    `TerminalHandoffHashInputV1` proves release. Input-bearing modes require Accepted or
    TerminalWithoutAcceptance; input-free Start/Attach require NotApplicable. Applied Start/Attach
    require exact application/startup ownership or terminal reconciliation. Resume requires
    post-turn Applied or exact terminal failure. Rejected/Expired require proof of no application.
    One root revision records `ReleaseEligible` in both parent and object index, deletion retries
    idempotently, and a later root records `Released`. A missing plan is never evidence.
13. Resume completion first publishes/verifies a `PostTurnCompletionHashInputV1` object. One root
    transaction advances post-turn Pending to Applied, mutates authority at most once, and stores
    the completion/result refs in both intent state and `post_turn_application` journal. Exact
    duplicate joins; stale, reordered, mismatched, or conflicting completion fails closed. Initial
    intent expiry never erases an already-applied pending handoff/post-turn reconciliation.

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
   transport may advance only after exact terminal-handoff verification.
7. Missing plan transport is reprojected only after revisioned input/application/post-turn checks.
   Applied Resume with pending post-turn work accepts only the exact completion/failure ref for its
   committed run; ambiguous evidence remains pending and diagnosable.
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
